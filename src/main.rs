//! vis-oss — understand an open-source issue before you try to fix it.
//!
//! The program is deliberately small. It reads an issue and a checkout, creates a
//! directory, and writes a `CONTEXT.md` with its headings in place and the agent
//! contract beside it. Everything that makes a study *good* lives in that contract,
//! which is prose, because the study is prose.
//!
//! What vis-oss will not do is invoke an agent or parse a study back in. Both were
//! tried and removed: orchestration makes the binary non-deterministic, and a schema in
//! the middle meant an agent writing markdown into JSON strings so a renderer could
//! turn it back into markdown.

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use clap::Parser;

mod command;
mod config;
mod git;
mod scaffold;
mod template;

use git::Staleness;
use scaffold::{Created, InitOptions};
use template::Tutorial;

/// Create a study directory for an open-source issue, for an agent to fill in.
///
/// Run from inside your clone of the project. The repository, root and commit are read
/// from git; the `upstream` remote wins over `origin`, because on a fork that is where
/// the issue lives.
#[derive(Parser)]
#[command(
    name = "vis-oss",
    version,
    about = "Understand an open-source issue before you try to fix it",
    long_about = None
)]
struct Cli {
    /// Issue number.
    #[arg(required_unless_present_any = [
        "install_command", "set_root", "update", "sync_upstream"
    ])]
    number: Option<u64>,
    /// Root to file this study under, overriding the saved root for this run.
    ///
    /// The study lands at `<root>/<name>/<owner>/<number>/` either way.
    base: Option<PathBuf>,
    /// `owner/name`. Inferred from the git remotes when omitted.
    #[arg(long)]
    repo: Option<String>,
    /// Checkout to study. Defaults to the enclosing repository.
    #[arg(long)]
    source: Option<PathBuf>,
    /// How much of the examples the reader writes themselves.
    ///
    /// Defaults to `none` because a file of `TODO`s cannot be executed, so nobody —
    /// including the agent that wrote it — can check that it works.
    #[arg(long, value_enum, default_value = "none")]
    tutorial: Tutorial,
    /// Install the `/vis-oss` command for any agent CLI found under $HOME, then exit.
    #[arg(long)]
    install_command: bool,
    /// Reinstall the latest vis-oss and refresh the agent command, then exit.
    #[arg(long)]
    update: bool,
    /// Remember where studies are filed, for every later run, then exit.
    #[arg(long, value_name = "PATH")]
    set_root: Option<PathBuf>,
    /// Fast-forward the checkout onto the canonical remote.
    ///
    /// With an issue number, it runs before the study is written. On its own, it syncs
    /// and exits.
    #[arg(long)]
    sync_upstream: bool,
    /// Steer the agent: prior art to look at, an angle to take. Repeatable.
    #[arg(long, value_name = "TEXT")]
    note: Vec<String>,
    /// Archive an existing study and write a fresh skeleton.
    #[arg(long)]
    redo: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(path) = cli.set_root {
        let file = config::save(&path)?;
        println!(
            "studies will be filed under {}",
            config::root(None)?.display()
        );
        println!("remembered in {}", file.display());
        return Ok(());
    }
    if cli.update {
        return command::update();
    }
    if cli.install_command {
        return install_command();
    }
    let Some(number) = cli.number else {
        // Only reachable with --sync-upstream, which clap allows without a number.
        return sync_only(cli.source.as_deref());
    };
    let opts = InitOptions {
        number,
        repo: cli.repo,
        base: cli.base,
        tutorial: cli.tutorial,
        source: cli.source,
        notes: cli.note,
        redo: cli.redo,
    };

    let plan = scaffold::plan(&opts)?;
    if let Some(stale) = plan.staleness.as_ref().filter(|s| s.behind > 0) {
        let synced = cli.sync_upstream && sync(stale, &plan.source);
        if !synced && !confirm_stale(stale, &plan.source)? {
            return Ok(());
        }
    }

    let (dir, mut notes) = match scaffold::init(&opts, plan)? {
        Created::Study { dir, notes } => (dir, notes),
        Created::Existing { dir, pinned, head } => return report_existing(&dir, &pinned, &head),
    };
    for path in command::outdated() {
        notes.push(format!(
            "{} was written by an older vis-oss; run --install-command to refresh it",
            path.display()
        ));
    }
    for note in &notes {
        eprintln!("note: {note}");
    }
    let dir = dir.display();
    println!("created {dir}");
    println!("  CONTEXT.md   the study — an agent fills in the empty sections");
    println!("  AGENTS.md    what a good study contains");
    println!("  00_*, 01_*   runnable probes of today, each ending in what changes");
    println!();
    println!("next: /vis-oss in your agent, or tell it to follow {dir}/AGENTS.md");
    Ok(())
}

fn install_command() -> Result<()> {
    let written = command::install()?;
    if written.is_empty() {
        let home = std::env::var("HOME").unwrap_or_default();
        eprintln!("no agent command directory found. Looked for:");
        for dir in command::candidates(Path::new(&home)) {
            eprintln!("  {}", dir.display());
        }
        eprintln!("Create one and run this again.");
        std::process::exit(1);
    }
    for path in &written {
        println!("installed {}", path.display());
    }
    println!();
    println!("now: /vis-oss 804 in your agent, from inside the project's clone");
    Ok(())
}

/// `--sync-upstream` with no issue number: bring the checkout up to date and stop.
fn sync_only(source: Option<&Path>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let source = match source {
        Some(p) => p.to_path_buf(),
        None => git::repo_root(&cwd)
            .context("not inside a git repository — cd into your clone, or pass --source")?,
    };
    let Some(stale) = git::staleness(&cwd) else {
        anyhow::bail!("no usable remote to sync from");
    };
    if stale.behind == 0 {
        println!("already up to date with {}", stale.reference());
        return Ok(());
    }
    if sync(&stale, &source) {
        return Ok(());
    }
    anyhow::bail!("sync it yourself, then run vis-oss again");
}

/// Point at a study that already exists, and say whether the code has moved under it.
fn report_existing(dir: &Path, pinned: &Option<String>, head: &Option<String>) -> Result<()> {
    println!("already studied: {}", dir.display());
    let short = |c: &String| c.chars().take(9).collect::<String>();
    match (pinned, head) {
        (Some(p), Some(h)) if !h.starts_with(p.as_str()) && !p.starts_with(h.as_str()) => {
            println!(
                "  written against {}, checkout is now at {}",
                short(p),
                short(h)
            );
            println!("  its file references may have moved; re-read before relying on them");
        }
        (Some(p), _) => println!("  written against {}, which is what you have", short(p)),
        _ => {}
    }
    println!();
    println!("nothing was overwritten. Delete the directory to start over.");
    Ok(())
}

/// Try to fast-forward, reporting either outcome. Returns whether the checkout is current.
fn sync(stale: &Staleness, source: &Path) -> bool {
    match git::fast_forward(source, stale) {
        Ok(()) => {
            eprintln!(
                "synced: fast-forwarded {} commit(s) to {}",
                stale.behind,
                stale.reference()
            );
            true
        }
        Err(reason) => {
            eprintln!("could not sync: {reason}");
            false
        }
    }
}

/// Warn that the checkout is behind, and let the user go sync instead.
///
/// A study written against a stale checkout documents code upstream has already
/// changed, and every file reference in it is wrong in a way that is invisible later.
/// Syncing stays the user's decision. Returns whether to proceed.
fn confirm_stale(stale: &Staleness, source: &Path) -> Result<bool> {
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

    if !std::io::stdin().is_terminal() {
        eprintln!("proceeding anyway (not a terminal)");
        return Ok(true);
    }
    eprint!("Continue with the stale checkout? [y/N] ");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    if matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
        return Ok(true);
    }
    eprintln!("stopped. Sync, then run vis-oss again.");
    Ok(false)
}
