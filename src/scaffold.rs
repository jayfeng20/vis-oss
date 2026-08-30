//! Creating the directory an agent fills in.
//!
//! Everything here is mechanical: read the issue, read the checkout, write the
//! skeleton. No judgement is applied, because judgement is the agent's job and this
//! part must stay predictable.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context as _, Result};
use serde_json::Value;

use crate::git::{self, Staleness};
use crate::template::{self, Tutorial, AGENT_CONTRACT};

pub struct InitOptions {
    pub number: u64,
    /// `owner/name`. Inferred from the git remotes when absent.
    pub repo: Option<String>,
    /// Root to file the study under. Overrides the saved root for this run only.
    pub base: Option<PathBuf>,
    /// How much of the examples the reader writes themselves.
    pub tutorial: Tutorial,
    /// Checkout to study. Defaults to the enclosing repository.
    pub source: Option<PathBuf>,
    /// Steering for the agent: prior art to look at, an angle to take.
    pub notes: Vec<String>,
    /// Archive an existing study and write a fresh skeleton.
    pub redo: bool,
}

/// Everything `init` needs to know before it is willing to write anything.
pub struct Plan {
    pub source: PathBuf,
    pub repo: String,
    pub dir: PathBuf,
    pub staleness: Option<Staleness>,
    pub notes: Vec<String>,
}

/// Work out where the study goes and whether the checkout is worth studying yet.
///
/// Split from [`init`] so the caller can put a decision in front of the user: a study
/// written against a checkout that is hundreds of commits behind describes code that no
/// longer exists, and nothing downstream repairs that.
pub fn plan(opts: &InitOptions) -> Result<Plan> {
    let mut notes = Vec::new();
    let cwd = std::env::current_dir()?;
    let source = match &opts.source {
        Some(p) => p.clone(),
        None => git::repo_root(&cwd).context(
            "not inside a git repository — cd into your clone of the project, or pass --source",
        )?,
    };

    let repo = if let Some(r) = &opts.repo {
        r.clone()
    } else {
        let slug = git::repo_slug(&source)
            .context("could not infer the repository from git remotes — pass --repo owner/name")?;
        notes.push(format!(
            "repo {slug}, from the git remotes (upstream preferred)"
        ));
        slug
    };

    let base = crate::config::root(opts.base.clone())?;
    let dir = study_path(&base, &repo, opts.number);
    if git::is_inside_repo(&dir) {
        notes.push(format!(
            "{} is inside a git repository — it will show up in git status",
            dir.display()
        ));
    }

    Ok(Plan {
        source,
        repo,
        dir,
        staleness: git::staleness(&cwd),
        notes,
    })
}

/// What `init` did.
pub enum Created {
    /// A new study was written.
    Study { dir: PathBuf, notes: Vec<String> },
    /// One was already there, and was left alone.
    Existing {
        dir: PathBuf,
        pinned: Option<String>,
        head: Option<String>,
    },
}

/// The commit an existing study records, read back out of its header.
fn pinned_commit(context: &str) -> Option<String> {
    let line = context.lines().find(|l| l.contains("Studied against"))?;
    let mut parts = line.split('`');
    parts.next()?;
    parts.next().map(str::to_string).filter(|s| !s.is_empty())
}

/// Write the study skeleton. Returns the directory created.
pub fn init(opts: &InitOptions, plan: Plan) -> Result<Created> {
    let Plan {
        source,
        repo,
        dir,
        mut notes,
        ..
    } = plan;

    let commit = git::head_commit(&source).unwrap_or_default();
    if commit.is_empty() {
        notes.push("could not read HEAD — the study will not record a commit".to_string());
    }

    let issue = match fetch_issue(&repo, opts.number) {
        Ok(issue) => issue,
        Err(e) => {
            notes.push(format!("gh lookup failed ({e}); wrote a stub header"));
            Issue::stub(opts.number)
        }
    };

    // Re-running for an issue already studied is a normal thing to do — usually to find
    // the study again, or to check whether it has gone stale. It is not an error, and
    // the existing work is never overwritten.
    if let Ok(existing) = std::fs::read_to_string(dir.join("CONTEXT.md")) {
        if opts.redo {
            // Move it aside rather than deleting: a study is hours of reading, and the
            // commit it was pinned to names it usefully.
            let stamp = pinned_commit(&existing).map_or_else(
                || "previous".to_string(),
                |c| c.chars().take(9).collect::<String>(),
            );
            let archive = dir.with_file_name(format!(
                "{}.{stamp}",
                dir.file_name().unwrap_or_default().to_string_lossy()
            ));
            if archive.exists() {
                bail!(
                    "{} already exists — move or delete it first",
                    archive.display()
                );
            }
            std::fs::rename(&dir, &archive)
                .with_context(|| format!("archiving to {}", archive.display()))?;
            notes.push(format!(
                "archived the previous study to {}",
                archive.display()
            ));
        } else {
            return Ok(Created::Existing {
                pinned: pinned_commit(&existing),
                head: git::head_commit(&source),
                dir,
            });
        }
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

    let context = template::context_md(&template::Context {
        repo: &repo,
        number: opts.number,
        title: &issue.title,
        url: &issue.url,
        state: &issue.state,
        created_at: &issue.created_at,
        labels: &issue.labels,
        body: &issue.body,
        source: &source.to_string_lossy(),
        commit: &commit,
        tutorial: opts.tutorial,
        notes: &opts.notes,
    });
    std::fs::write(dir.join("CONTEXT.md"), context)?;
    std::fs::write(dir.join("AGENTS.md"), AGENT_CONTRACT)?;

    Ok(Created::Study { dir, notes })
}

/// `<root>/<name>/<owner>/<issue>/` — the layout studies are filed under.
///
/// Repository name first, because that is what you recognise when browsing the root;
/// the owner disambiguates two projects that share a name.
fn study_path(root: &Path, repo: &str, number: u64) -> PathBuf {
    let mut parts = repo.split('/').filter(|p| !p.is_empty());
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or(owner);
    root.join(name).join(owner).join(number.to_string())
}

struct Issue {
    title: String,
    url: String,
    state: String,
    created_at: String,
    labels: Vec<String>,
    body: String,
}

impl Issue {
    fn stub(number: u64) -> Self {
        Self {
            title: format!("(could not read issue #{number})"),
            url: String::new(),
            state: String::new(),
            created_at: String::new(),
            labels: Vec::new(),
            body: String::new(),
        }
    }
}

fn fetch_issue(repo: &str, number: u64) -> Result<Issue> {
    let out = Command::new("gh")
        .args([
            "issue",
            "view",
            &number.to_string(),
            "--repo",
            repo,
            "--json",
        ])
        .arg("title,url,state,labels,createdAt,body")
        .output()
        .context("running `gh` (is the GitHub CLI installed and authenticated?)")?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    let v: Value = serde_json::from_slice(&out.stdout).context("parsing gh output")?;
    let text = |key: &str| v[key].as_str().unwrap_or_default().to_string();

    Ok(Issue {
        title: text("title"),
        url: text("url"),
        state: text("state"),
        created_at: text("createdAt").chars().take(10).collect(),
        labels: v["labels"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|l| l["name"].as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        body: text("body"),
    })
}
