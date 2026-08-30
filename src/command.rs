//! Installing the slash command that runs the whole flow.
//!
//! Without it the agent step is a sentence the user retypes every time. The command is
//! plain markdown in a directory the agent already reads, so installing it is writing
//! one file — vis-oss still never invokes an agent.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

/// Agent CLIs that read prompt files out of a directory under `$HOME`.
///
/// Each entry is (name, path under home, whether the format wants frontmatter). Only
/// directories that already exist are written to, so installing does not create config
/// trees for tools the user does not have.
const TARGETS: &[(&str, &str, bool)] = &[
    ("Claude Code", ".claude/commands", true),
    ("Codex", ".codex/prompts", false),
];

/// The command body. `$ARGUMENTS` is the issue number the user typed.
const BODY: &str = r"Run `vis-oss $ARGUMENTS` from the repository you are in, then fill in the study it creates.

The command prints the directory it made. In it:

- `AGENTS.md` is the contract. **Read it fully before writing anything** — it defines
  what each section must contain, how examples are structured and annotated, and what
  makes a study useful rather than merely complete. Follow it.
- `CONTEXT.md` is the study, with its headings in place and the issue body pasted in.
  Fill in every empty section.
- The examples go alongside them, numbered: `00_` for setup if it is needed, then
  `01_`, `02_` for the behaviours worth watching.

Write each probe in the language the behaviour is observed in, and add a second file in
the language the fix lands in when its API reaches what you need — the first shows what a
user hits, the second compiles against the contributor's own tree and so doubles as an
acceptance check. Same behaviour, same number, different extension: `01_flat_search.py`
alongside `01_flat_search.rs`. Skip the second when the thing is not reachable from
outside the crate, and say so in one line.

Investigate from the repository you are in, not from the study directory — you need to
search and read the project's source. Write into the study directory by absolute path.

## Do not run the examples

You write them; the reader runs them. Getting one to execute means first paying whatever
the project charges for a first execution — a binding compiled from source, a dataset
written before anything can be queried — and that cost is unpredictable and unrelated to
how small your example is. So:

- **Do not run an example**, or a cut-down version of one, to check that it works.
- **Do not write throwaway scripts** hunting for a data size or parameter that makes a
  measurement come out the way you expected.
- **Do not build a trained index, a large generated dataset, or anything else the project
  would call a benchmark.**

There is one mechanical check, and only for Rust: `cargo check` the shared study project
once your file is in it, and fix what it reports. For Python and anything else interpreted
there is nothing worth running — a syntax check catches what you would not get wrong, and
anything stronger means importing the project.

Otherwise the check is reading. Open the file, find the symbol, confirm the signature — an
API you read is as real as one you called. Say what you checked in each file's provenance
line: for Rust that it compiles, otherwise the paths you read. Real paths, never just a
claim that you were careful.

Give each example what it needs to be run by someone else: the exact command and directory,
the project's own tooling (`uv run`, `poetry run`), and for Rust the shared Cargo project
one level above the issue directory, with a `[[bin]]` for your file. `AGENTS.md` has the
layout. Write the smallest input that reaches the code path — a thousand rows takes the
same branch as a million.

The lowest-numbered file in each language also opens with how to get an environment where
its command works, taken from the project's own docs — not guessed. A `uv` project usually
needs `uv sync` once and then `uv run`, with no venv to activate at all; a plain venv or
poetry project needs the activation line. If a native extension must be compiled before
anything can import it, that belongs in the block too — it is normally the longest step.

When the study is done, if actually running something would settle a question, say in one
line what it would settle and what it would cost, and offer to do it. Never start unasked.

## If the checkout is behind

`vis-oss` will say so and stop. Tell the user, and offer `vis-oss <issue> --sync-upstream`,
which fast-forwards first. Do not write a study against code that has already moved.

Never invent a line number, an API, or a flag: read the file and verify the symbol is
there. Do not modify the project's source, and do not post anything to GitHub.";

/// Install the command for every supported agent whose directory already exists.
pub fn install() -> Result<Vec<PathBuf>> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .context("could not determine the home directory")?;

    let mut written = Vec::new();
    for (name, subdir, frontmatter) in TARGETS {
        let dir = home.join(subdir);
        if !dir.is_dir() {
            continue;
        }
        let path = dir.join("vis-oss.md");
        std::fs::write(&path, contents(*frontmatter))
            .with_context(|| format!("writing the {name} command to {}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

fn contents(frontmatter: bool) -> String {
    if frontmatter {
        format!(
            "---\n\
             description: Create a vis-oss study for an issue and fill it in\n\
             argument-hint: <issue-number>\n\
             ---\n\n\
             {BODY}\n"
        )
    } else {
        format!("{BODY}\n")
    }
}

/// Installed command files whose contents no longer match this binary.
///
/// The command is a copy, so it goes stale the moment vis-oss is upgraded — and a stale
/// one quietly instructs the agent to do the wrong thing, which is invisible until a
/// study comes out wrong. Cheap to check on every run.
pub fn outdated() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
    else {
        return Vec::new();
    };
    TARGETS
        .iter()
        .filter_map(|(_, subdir, frontmatter)| {
            let path = home.join(subdir).join("vis-oss.md");
            let current = std::fs::read_to_string(&path).ok()?;
            (current != contents(*frontmatter)).then_some(path)
        })
        .collect()
}

/// Directories checked, for reporting when none were found.
pub fn candidates(home: &Path) -> Vec<PathBuf> {
    TARGETS.iter().map(|(_, d, _)| home.join(d)).collect()
}

/// Reinstall from source, then refresh the agent command with the *new* binary.
///
/// The two copies — the binary and the prompt file it writes — go stale independently,
/// and a stale prompt quietly instructs the agent wrongly. Doing both in one step is the
/// only way they cannot drift apart.
///
/// Replacing a running executable is fine on Unix, where the old inode survives until the
/// process exits. On Windows the file is locked and cargo will refuse.
pub fn update() -> Result<()> {
    let repository = env!("CARGO_PKG_REPOSITORY");
    if repository.is_empty() {
        anyhow::bail!("this build records no repository to update from");
    }

    println!("installing the latest {repository}");
    let status = Command::new("cargo")
        .args(["install", "--git", repository, "--force"])
        .status()
        .context("running `cargo` (is it on your PATH?)")?;
    if !status.success() {
        anyhow::bail!("cargo install failed; nothing was changed");
    }

    // Deliberately the freshly written binary, not this process: running our own
    // `install()` here would write the command text this build was compiled with, which
    // is exactly the version being replaced.
    let installed = installed_path()?;
    let status = Command::new(&installed)
        .arg("--install-command")
        .status()
        .with_context(|| format!("running {}", installed.display()))?;
    if !status.success() {
        anyhow::bail!("{} --install-command failed", installed.display());
    }
    Ok(())
}

/// Where `cargo install` puts binaries.
fn installed_path() -> Result<PathBuf> {
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|h| PathBuf::from(h).join(".cargo"))
        })
        .context("could not locate CARGO_HOME")?;
    let path = home.join("bin").join(env!("CARGO_PKG_NAME"));
    if path.exists() {
        return Ok(path);
    }
    anyhow::bail!(
        "{} is not there — was this copy installed with cargo?",
        path.display()
    )
}
