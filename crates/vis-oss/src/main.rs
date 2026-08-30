use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

use vis_oss::git::Staleness;
use vis_oss::scaffold::{self, InitOptions};

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
    number: u64,
    /// Base directory to file the study under. Defaults to `~/vis-oss`.
    ///
    /// The study lands at `<base>/<owner>/<name>/<number>/` either way.
    base: Option<PathBuf>,
    /// `owner/name`. Inferred from the git remotes when omitted.
    #[arg(long)]
    repo: Option<String>,
    /// Checkout to study. Defaults to the enclosing repository.
    #[arg(long)]
    source: Option<PathBuf>,
    /// Write finished code in examples instead of exercises for the reader.
    #[arg(long)]
    solution: bool,
    /// Proceed without asking, even if the checkout is behind upstream.
    #[arg(long, short = 'y')]
    yes: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let opts = InitOptions {
        number: cli.number,
        repo: cli.repo,
        base: cli.base,
        solution: cli.solution,
        source: cli.source,
    };

    let plan = scaffold::plan(&opts)?;
    if let Some(stale) = plan.staleness.as_ref().filter(|s| s.behind > 0) {
        if !confirm_stale(stale, &plan.source, cli.yes)? {
            return Ok(());
        }
    }

    let (dir, notes) = scaffold::init(&opts, plan)?;
    for note in &notes {
        eprintln!("note: {note}");
    }
    let dir = dir.display();
    println!("created {dir}");
    println!("  CONTEXT.md   the study — an agent fills in the empty sections");
    println!("  AGENT.md     what a good study contains");
    println!("  examples/    today's behaviour, and the target");
    println!();
    println!("next: point an agent at {dir} and tell it to follow AGENT.md");
    Ok(())
}

/// Warn that the checkout is behind, and let the user go sync instead.
///
/// A study written against a stale checkout documents code upstream has already
/// changed, and every file reference in it is wrong in a way that is invisible later.
/// Syncing stays the user's decision. Returns whether to proceed.
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
    eprintln!("stopped. Sync, then run vis-oss again.");
    Ok(false)
}
