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

Investigate from the repository you are in, not from the study directory — you need to
search and read the project's source. Write into the study directory by absolute path.

## Make the examples actually run

They default to complete, runnable code, and running them is the point: an example you
executed carries real output in its provenance line, and one you only wrote is a guess
about an API. So, for each example:

1. Set up whatever it needs to run. For an interpreted binding, use the project's own
   tooling (`uv run`, `poetry run`) — nothing to create. For Rust, the study root holds
   one Cargo project per repository, one level above the issue directory: create
   `Cargo.toml` there with a path dependency on the checkout if it does not exist, and
   add a `[[bin]]` for your example if it does. `AGENTS.md` shows the layout.
2. Run it. Fix what breaks.
3. Put the real command and the real output in the file's provenance line.

If you genuinely cannot run one — it needs a large download, credentials, or hardware you
do not have — say so in that line and say why. Never leave provenance out.

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
