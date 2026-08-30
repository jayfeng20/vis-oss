//! Installing the slash command that runs the whole flow.
//!
//! Without it the agent step is a sentence the user retypes every time. The command is
//! plain markdown in a directory the agent already reads, so installing it is writing
//! one file — vis-oss still never invokes an agent.

use std::path::{Path, PathBuf};

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
const BODY: &str = r"Run `vis-oss $ARGUMENTS` from the current repository, then fill in the study it creates.

The command prints the directory it made. In that directory:

- `AGENTS.md` is the contract. Read it first and follow it — it defines what each
  section of `CONTEXT.md` must contain and what makes a study good rather than merely
  complete.
- `CONTEXT.md` is the study. It arrives with its headings in place and the issue body
  already pasted in. Fill in every empty section.
- `examples/` is where the runnable before/after files go.

Investigate from the repository you are in now, not from the study directory — you need
to search and read the project's source. Write your findings into the study directory
using its absolute path.

If `vis-oss` reports the checkout is behind upstream, stop and tell the user rather than
writing a study against code that has already moved.

Never invent a line number, an API, or a flag. Read the file and verify the symbol
exists at the line you cite. Do not post anything to GitHub.";

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

/// Directories checked, for reporting when none were found.
pub fn candidates(home: &Path) -> Vec<PathBuf> {
    TARGETS.iter().map(|(_, d, _)| home.join(d)).collect()
}
