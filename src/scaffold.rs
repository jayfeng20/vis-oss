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
use crate::template::{self, AGENT_CONTRACT};

/// Default base directory for studies, under the user's home.
///
/// Outside any repository on purpose. A study written inside the project it describes
/// shows up in `git status` and can be committed by accident into the very pull request
/// the contributor is preparing.
pub const DEFAULT_BASE: &str = "vis-oss";

pub struct InitOptions {
    pub number: u64,
    /// `owner/name`. Inferred from the git remotes when absent.
    pub repo: Option<String>,
    /// Base directory to file the study under. Defaults to `~/vis-oss`.
    ///
    /// The study lands at `<base>/<owner>/<name>/<number>/` either way, so pointing
    /// several issues at one base keeps them organised rather than colliding.
    pub base: Option<PathBuf>,
    /// Write finished code in examples rather than exercises.
    pub solution: bool,
    /// Checkout to study. Defaults to the enclosing repository.
    pub source: Option<PathBuf>,
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

    let base = match opts.base.clone() {
        Some(b) => b,
        None => home()
            .context("could not determine the home directory — pass a base directory")?
            .join(DEFAULT_BASE),
    };
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

/// Write the study skeleton. Returns the directory created.
pub fn init(opts: &InitOptions, plan: Plan) -> Result<(PathBuf, Vec<String>)> {
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

    if dir.join("CONTEXT.md").exists() {
        bail!(
            "{} already contains a CONTEXT.md — delete it or pass a different base",
            dir.display()
        );
    }
    std::fs::create_dir_all(dir.join("examples"))
        .with_context(|| format!("creating {}", dir.display()))?;

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
        tutorial: !opts.solution,
    });
    std::fs::write(dir.join("CONTEXT.md"), context)?;
    std::fs::write(dir.join("AGENTS.md"), AGENT_CONTRACT)?;

    Ok((dir, notes))
}

/// `<base>/<owner>/<name>/<issue>/` — the layout studies are filed under.
///
/// Keyed by the full `owner/name` rather than the bare repository name so that two
/// projects that happen to share a name do not collide.
fn study_path(base: &Path, repo: &str, number: u64) -> PathBuf {
    let mut path = base.to_path_buf();
    for part in repo.split('/').filter(|p| !p.is_empty()) {
        path.push(part);
    }
    path.push(number.to_string());
    path
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
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
