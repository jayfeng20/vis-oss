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
use scaffold::InitOptions;
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
    /// Issue number, as it appears on the tracker.
    #[arg(value_name = "ISSUE_NUMBER", required_unless_present_any = [
        "install_command", "set_root", "update", "sync_upstream"
    ])]
    number: Option<u64>,
    /// Root to file this study under, overriding the saved root for this run.
    ///
    /// The study lands at `<root>/<name>/<owner>/<number>/` either way.
    #[arg(value_name = "STUDY_ROOT")]
    base: Option<PathBuf>,
    /// The repository the issue lives in. Inferred from the git remotes when omitted.
    #[arg(long, value_name = "OWNER/NAME", help_heading = STUDY)]
    repo: Option<String>,
    /// Checkout to study. Defaults to the enclosing repository.
    #[arg(long, value_name = "REPO_PATH", help_heading = STUDY)]
    source: Option<PathBuf>,
    /// How much of the example is written for you.
    ///
    /// Defaults to `partial`: nothing here is executed, so `full` would promise complete
    /// working code nobody has run. Scaffolding the agent stands behind from reading, with
    /// the interesting parts left to you, is what it can honestly deliver.
    #[arg(long, value_enum, value_name = "LEVEL", default_value = "partial", help_heading = STUDY)]
    tutorial: Tutorial,
    /// Steer the agent: prior art to look at, an angle to take. Repeatable.
    #[arg(long, value_name = "TEXT", help_heading = STUDY)]
    note: Vec<String>,
    /// Delete an existing study and write a fresh skeleton. The old one is not kept.
    #[arg(long, help_heading = STUDY)]
    redo: bool,
    /// Fast-forward the checkout onto the canonical remote.
    ///
    /// With an issue number, it runs before the study is written. On its own, it syncs
    /// and exits.
    #[arg(long, help_heading = STUDY)]
    sync_upstream: bool,
    /// Install the `/vis-oss` command for any agent CLI found under $HOME, then exit.
    #[arg(long, group = "setup", conflicts_with_all = STUDY_ARGS, help_heading = SETUP)]
    install_command: bool,
    /// Reinstall the latest vis-oss and refresh the agent command, then exit.
    #[arg(long, group = "setup", conflicts_with_all = STUDY_ARGS, help_heading = SETUP)]
    update: bool,
    /// Remember where studies are filed, for every later run, then exit.
    #[arg(long, value_name = "PATH", group = "setup", conflicts_with_all = STUDY_ARGS, help_heading = SETUP)]
    set_root: Option<PathBuf>,
}

/// The two halves of the CLI: flags that ride along with an issue number, and flags that
/// run alone. A setup flag next to a study is a contradiction clap rejects up front,
/// rather than one mode silently winning.
const STUDY: &str = "Creating a study (usually typed after /vis-oss in your agent)";
const SETUP: &str = "Setup (each runs alone, then exits)";
const STUDY_ARGS: [&str; 8] = [
    "number",
    "base",
    "repo",
    "source",
    "tutorial",
    "note",
    "redo",
    "sync_upstream",
];

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

    // Before the staleness prompt and before the issue lookup: an issue already studied
    // needs neither, and asking about a stale checkout is a question about work that is
    // not going to happen.
    if !opts.redo {
        if let Some(found) = scaffold::existing(&plan.dir, &plan.source) {
            return report_existing(&found.dir, &found.pinned, &found.head, &opts);
        }
    }

    if let Some(stale) = plan.staleness.as_ref().filter(|s| s.behind > 0) {
        let synced = cli.sync_upstream && sync(stale, &plan.source);
        if !synced && !confirm_stale(stale, &plan.source)? {
            return Ok(());
        }
    }

    let scaffold::Created { dir, mut notes } = scaffold::init(&opts, plan)?;
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
    println!("now: /vis-oss <issue-number> in your agent, from inside the project's clone");
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
fn report_existing(
    dir: &Path,
    pinned: &Option<String>,
    head: &Option<String>,
    opts: &InitOptions,
) -> Result<()> {
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
    if opts.notes.is_empty() {
        println!("nothing was overwritten. --redo deletes it and starts again.");
    } else {
        println!("nothing was overwritten — including your --note, which was NOT recorded.");
        println!("To redo the study with it (the existing one is deleted, not kept):");
        println!("  {}", redo_hint(opts.number, &opts.notes));
    }
    Ok(())
}

/// The rerun that carries dropped notes into a fresh study, ready to paste.
fn redo_hint(number: u64, notes: &[String]) -> String {
    use std::fmt::Write as _;
    let mut hint = format!("vis-oss {number} --redo");
    for note in notes {
        // Writing to a String cannot fail.
        let _ = write!(hint, " --note \"{}\"", note.replace('"', "\\\""));
    }
    hint
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }

    #[test]
    fn setup_flags_run_alone() {
        assert!(Cli::try_parse_from(["vis-oss", "--update"]).is_ok());
        assert!(Cli::try_parse_from(["vis-oss", "--install-command"]).is_ok());
        assert!(Cli::try_parse_from(["vis-oss", "--set-root", "/tmp/studies"]).is_ok());
        assert!(Cli::try_parse_from(["vis-oss", "--sync-upstream"]).is_ok());
    }

    #[test]
    fn setup_flags_reject_a_study() {
        // Without the conflict, --set-root would win and the study would silently
        // never be created.
        for study in [
            ["804", ""],
            ["--note", "a lead"],
            ["--redo", ""],
            ["--sync-upstream", ""],
            ["--tutorial", "full"],
        ] {
            let args = std::iter::once("vis-oss")
                .chain(["--set-root", "/tmp/studies"])
                .chain(study.into_iter().filter(|a| !a.is_empty()));
            assert!(Cli::try_parse_from(args).is_err(), "{study:?} was accepted");
        }
        assert!(Cli::try_parse_from(["vis-oss", "--update", "--install-command"]).is_err());
        assert!(Cli::try_parse_from(["vis-oss", "804", "--update"]).is_err());
    }

    #[test]
    fn redo_hint_carries_every_note_and_survives_quotes() {
        let notes = vec![
            "look at flat_search.rs".to_string(),
            "the plan says \"flat\"".to_string(),
        ];
        assert_eq!(
            redo_hint(804, &notes),
            "vis-oss 804 --redo --note \"look at flat_search.rs\" --note \"the plan says \\\"flat\\\"\""
        );
    }

    #[test]
    fn study_flags_travel_with_the_number() {
        let cli = Cli::try_parse_from([
            "vis-oss",
            "804",
            "/tmp/notes",
            "--tutorial",
            "none",
            "--note",
            "a lead",
            "--redo",
            "--sync-upstream",
        ])
        .expect("a fully specified study must parse");
        assert_eq!(cli.number, Some(804));
        assert!(
            Cli::try_parse_from(["vis-oss"]).is_err(),
            "a bare call has nothing to do"
        );
    }
}
