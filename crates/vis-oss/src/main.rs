use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use vis_oss::git::Staleness;
use vis_oss::render::{self, RenderOpts, Section};
use vis_oss::scaffold::{self, InitOptions};
use vis_oss::study::{Mode, Study};
use vis_oss::validate::{self, Level};
use vis_oss::{markdown, repair};

/// Output form for `render`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Format {
    /// Formatted for a terminal, with drift warnings inline.
    Term,
    /// Markdown, for committing next to the study or pasting into an issue.
    Markdown,
}

#[derive(Parser)]
#[command(
    name = "vis-oss",
    version,
    about = "Scaffold, validate and render agent-generated studies of open-source issues",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a study skeleton for an issue, for an agent to fill in.
    ///
    /// Run from inside a checkout of the project: the repository slug, root and commit
    /// are read from git. On a fork the `upstream` remote wins over `origin`, because
    /// that is where the issues live.
    Init {
        /// Issue number.
        number: u64,
        /// Directory to create. Defaults to `<repo root>/vis-oss-scratch/<number>/`.
        dir: Option<PathBuf>,
        /// `owner/name`. Inferred from the git remotes when omitted.
        #[arg(long)]
        repo: Option<String>,
        /// Checkout the anchors refer to. Defaults to the enclosing repository.
        #[arg(long)]
        source: Option<PathBuf>,
        /// Emit finished code in examples instead of exercises for the reader.
        #[arg(long, conflicts_with = "tutorial")]
        solution: bool,
        /// Emit comment-and-TODO exercises in examples. The default.
        #[arg(long)]
        tutorial: bool,
        /// Proceed without asking, even if the checkout is behind upstream.
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Check a study for completeness, and for drift against the working tree.
    Validate {
        /// Study directory, or the path to a study.json.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Print a study.
    Render {
        /// Study directory, or the path to a study.json.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output form.
        #[arg(long, value_enum, default_value = "term")]
        format: Format,
        /// Which sections to print, in the order given. Defaults to all.
        #[arg(long, value_enum, value_delimiter = ',')]
        section: Vec<Section>,
        /// Do not re-resolve anchors against the working tree.
        #[arg(long)]
        no_check: bool,
        /// Wrap width. Defaults to the terminal width, clamped to a readable range.
        #[arg(long)]
        width: Option<usize>,
        #[arg(long, conflicts_with = "color", help = "Disable colour")]
        no_color: bool,
        #[arg(long, help = "Force colour even when stdout is not a terminal")]
        color: bool,
    },

    /// Rewrite anchors whose symbols moved, and re-pin to the current commit.
    Repair {
        /// Study directory, or the path to a study.json.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Report what would change without writing.
        #[arg(long)]
        dry_run: bool,
    },

    /// Print the JSON Schema for a study, for agents to emit against.
    Schema,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Init {
            number,
            dir,
            repo,
            source,
            solution,
            tutorial: _,
            yes,
        } => cmd_init(number, dir, repo, source, solution, yes),
        Command::Validate { path } => cmd_validate(&path),
        Command::Render {
            path,
            format,
            section,
            no_check,
            width,
            no_color,
            color,
        } => cmd_render(&path, format, section, no_check, width, no_color, color),
        Command::Repair { path, dry_run } => cmd_repair(&path, dry_run),
        Command::Schema => {
            let schema = schemars::schema_for!(Study);
            println!("{}", serde_json::to_string_pretty(&schema)?);
            Ok(())
        }
    }
}

fn cmd_init(
    number: u64,
    dir: Option<PathBuf>,
    repo: Option<String>,
    source: Option<PathBuf>,
    solution: bool,
    yes: bool,
) -> Result<()> {
    let opts = InitOptions {
        number,
        repo,
        out: dir,
        mode: if solution {
            Mode::Solution
        } else {
            Mode::Tutorial
        },
        source,
    };
    let plan = scaffold::plan(&opts)?;

    if let Some(stale) = plan.staleness.as_ref().filter(|s| s.behind > 0) {
        if !confirm_stale(stale, &plan.source, yes)? {
            return Ok(());
        }
    }

    let outcome = scaffold::init(&opts, plan)?;
    for note in &outcome.notes {
        eprintln!("note: {note}");
    }
    let dir = outcome.dir.display();
    println!("created {dir}");
    println!("  study.json   the contract — an agent fills this in");
    println!("  examples/    runnable before/after files go here");
    println!("  README.md    what this directory is");
    println!();
    println!("next: point an agent at it, then `vis-oss render {dir}`");
    Ok(())
}

/// Warn that the checkout is behind, and let the user go sync instead.
///
/// A study written against a stale checkout documents code that upstream has already
/// changed, and every line number in it is wrong in a way that is invisible later. That
/// is worth interrupting for. Returns whether to proceed.
fn confirm_stale(stale: &Staleness, source: &Path, yes: bool) -> Result<bool> {
    let reference = stale.reference();
    eprintln!(
        "warning: this checkout is {} commit(s) behind {reference}.",
        stale.behind
    );
    eprintln!("         A study written now will describe code that has already moved.");
    eprintln!();
    eprintln!("  to sync first:");
    eprintln!("    git -C {} fetch {}", source.display(), stale.remote);
    eprintln!(
        "    git -C {} merge --ff-only {reference}",
        source.display()
    );
    eprintln!();

    if yes {
        eprintln!("proceeding anyway (--yes)");
        return Ok(true);
    }
    if !std::io::stdin().is_terminal() {
        eprintln!("proceeding anyway (not a terminal; pass --yes to silence this)");
        return Ok(true);
    }
    eprint!("Continue with the stale checkout? [y/N] ");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    if matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
        return Ok(true);
    }
    eprintln!("stopped. Sync, then run vis-oss init again.");
    Ok(false)
}

fn cmd_validate(path: &Path) -> Result<()> {
    let (study, dir) = load(path)?;
    let diags = validate::validate(&study, &dir);
    let errors = diags.iter().filter(|d| d.level == Level::Error).count();
    let warnings = diags.len() - errors;

    let mut stdout = std::io::stdout().lock();
    for d in &diags {
        let tag = match d.level {
            Level::Error => "error",
            Level::Warning => "warn",
        };
        writeln!(stdout, "{tag}: {}", d.message)?;
    }
    if diags.is_empty() {
        writeln!(stdout, "ok — study is complete and anchors resolve")?;
    } else {
        writeln!(stdout, "\n{errors} error(s), {warnings} warning(s)")?;
    }
    if errors > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_render(
    path: &Path,
    format: Format,
    section: Vec<Section>,
    no_check: bool,
    width: Option<usize>,
    no_color: bool,
    color: bool,
) -> Result<()> {
    let (study, _) = load(path)?;
    if format == Format::Markdown {
        print!("{}", markdown::render(&study));
        return Ok(());
    }
    let stdout = std::io::stdout();
    let opts = RenderOpts {
        sections: if section.is_empty() {
            Section::ALL.to_vec()
        } else {
            section
        },
        width: width.unwrap_or_else(render::detect_width),
        color: color || (!no_color && stdout.is_terminal()),
        check_anchors: !no_check,
    };
    let mut lock = stdout.lock();
    write!(lock, "{}", render::render(&study, &opts))?;
    Ok(())
}

fn cmd_repair(path: &Path, dry_run: bool) -> Result<()> {
    let (mut study, dir) = load(path)?;
    let result = repair::repair(&mut study);
    for c in &result.changes {
        match c {
            repair::Change::Relined {
                group,
                path,
                from,
                to,
            } => {
                let was = from.map_or_else(|| "?".to_string(), |f| f.to_string());
                println!("{group} {path}: {was} -> {to}");
            }
            repair::Change::Repinned { from, to } => println!("re-pinned {from} -> {to}"),
        }
    }
    for u in &result.unfixable {
        println!("needs a human: {} {} — {}", u.group, u.path, u.reason);
    }
    if result.is_empty() {
        println!("nothing to repair");
        return Ok(());
    }
    if dry_run {
        println!("\n(dry run, nothing written)");
        return Ok(());
    }
    if !result.changes.is_empty() {
        let file = dir.join("study.json");
        std::fs::write(&file, serde_json::to_string_pretty(&study)? + "\n")
            .with_context(|| format!("writing {}", file.display()))?;
        println!("\nwrote {}", file.display());
    }
    if !result.unfixable.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

/// Accept either a study directory or a path straight to `study.json`.
fn load(path: &Path) -> Result<(Study, PathBuf)> {
    let (file, dir) = if path.is_dir() {
        (path.join("study.json"), path.to_path_buf())
    } else {
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        (path.to_path_buf(), dir)
    };
    let text =
        std::fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
    let study =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", file.display()))?;
    Ok((study, dir))
}
