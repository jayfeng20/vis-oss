//! `vis-oss init` — create the skeleton an investigating agent fills in.
//!
//! Everything here is mechanical: read the issue, read the checkout, write a
//! well-formed but empty [`Study`]. No judgement is applied, because judgement is the
//! agent's job and this tool must stay testable and deterministic.

use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::git::{self, Staleness};
use crate::study::{Extra, Issue, Mode, Pin, Study, SCHEMA_VERSION};

/// Directory, relative to the repository root, that study directories live in.
///
/// Inside the repo so a study sits next to the code it describes, and so paths in it
/// are short. Excluded via `.git/info/exclude` rather than `.gitignore`, because a
/// contributor's scratch work must not appear in the diff they send upstream.
pub const SCRATCH_DIR: &str = "vis-oss-scratch";

pub struct InitOptions {
    pub number: u64,
    /// `owner/name`. Inferred from the git remotes when absent.
    pub repo: Option<String>,
    /// Where to write. Defaults to `<repo root>/vis-oss-scratch/<number>/`.
    pub out: Option<PathBuf>,
    pub mode: Mode,
    /// Checkout the anchors will refer to. Defaults to the enclosing repository.
    pub source: Option<PathBuf>,
}

pub struct InitOutcome {
    pub dir: PathBuf,
    pub study: Study,
    pub notes: Vec<String>,
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
/// Split from [`init`] so the caller can put a decision in front of the user — a study
/// written against a checkout that is hundreds of commits behind describes code that
/// no longer exists, and no amount of care downstream repairs that.
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

    let dir = opts
        .out
        .clone()
        .unwrap_or_else(|| source.join(SCRATCH_DIR).join(opts.number.to_string()));

    Ok(Plan {
        source,
        repo,
        dir,
        staleness: git::staleness(&cwd),
        notes,
    })
}

pub fn init(opts: &InitOptions, plan: Plan) -> Result<InitOutcome> {
    let Plan {
        source,
        repo,
        dir,
        mut notes,
        ..
    } = plan;

    let commit = git::head_commit(&source).unwrap_or_default();
    if commit.is_empty() {
        notes.push("could not read HEAD — pin.commit left empty".to_string());
    }

    let issue = match fetch_issue(&repo, opts.number) {
        Ok(issue) => issue,
        Err(e) => {
            notes.push(format!("gh lookup failed ({e}); wrote a stub issue block"));
            Issue {
                repo: repo.clone(),
                number: opts.number,
                ..Issue::default()
            }
        }
    };
    let study = Study {
        schema_version: SCHEMA_VERSION,
        issue,
        pin: Pin {
            root: source.to_string_lossy().into_owned(),
            commit,
            extra: Extra::default(),
        },
        mode: opts.mode,
        summary: None,
        current_behavior: None,
        desired_behavior: None,
        entry_points: Vec::new(),
        prior_art: Vec::new(),
        open_questions: Vec::new(),
        examples: Vec::new(),
        verify: Vec::new(),
        extra: Extra::default(),
    };

    if dir.join("study.json").exists() {
        bail!(
            "{} already contains a study.json — delete it or pass a different directory",
            dir.display()
        );
    }
    std::fs::create_dir_all(dir.join("examples"))
        .with_context(|| format!("creating {}", dir.display()))?;
    std::fs::write(
        dir.join("study.json"),
        serde_json::to_string_pretty(&study)? + "\n",
    )?;
    std::fs::write(dir.join("README.md"), readme(&study))?;

    // Only meaningful when the study lives inside the repository it describes.
    if dir.starts_with(&source) {
        match git::ensure_ignored(&source, &dir.join("study.json")) {
            git::Excluded::Added(pattern) => notes.push(format!(
                "excluded {pattern} locally (info/exclude, not .gitignore) — verified ignored"
            )),
            git::Excluded::Already => {}
            git::Excluded::Failed(reason) => notes.push(format!(
                "WARNING: {} is NOT ignored by git ({reason}). \
                 Exclude it yourself before you commit.",
                dir.display()
            )),
        }
    }

    Ok(InitOutcome { dir, study, notes })
}

fn readme(study: &Study) -> String {
    let i = &study.issue;
    format!(
        r"# {repo} #{number}

{title}

{url}

A **vis-oss study**: a structured investigation of one open-source issue, so you can
understand it before you change anything.

## Layout

| Path | What it is |
|---|---|
| `study.json` | The study itself. Source of truth — everything else is derived. |
| `examples/` | Runnable files: what the code does today, and what it should do after. |
| `README.md` | This file. |

## Reading it

```sh
vis-oss render .        # the study, formatted, with drift warnings
vis-oss validate .      # is it complete, and has the code moved under it?
```

`render` re-resolves every code reference against the checkout, so a study that has
gone stale says so instead of quietly pointing at the wrong lines. To keep a copy in
version control: `vis-oss render . > context.md`.

## Filling it in

`study.json` starts mostly empty; an investigating agent fills it. See the
[agent contract](https://github.com/jayfeng20/vis-oss/blob/main/docs/agent-contract.md)
for what each field means and what makes a study good rather than merely valid.
",
        repo = i.repo,
        number = i.number,
        title = if i.title.is_empty() {
            "(title unavailable)"
        } else {
            &i.title
        },
        url = i.url,
    )
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
        .arg("number,title,url,state,labels,createdAt,body,comments")
        .output()
        .context("running `gh` (is the GitHub CLI installed and authenticated?)")?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    let v: Value = serde_json::from_slice(&out.stdout).context("parsing gh output")?;

    Ok(Issue {
        repo: repo.to_string(),
        number,
        title: v["title"].as_str().unwrap_or_default().to_string(),
        url: v["url"].as_str().unwrap_or_default().to_string(),
        state: v["state"].as_str().unwrap_or_default().to_string(),
        labels: v["labels"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|l| l["name"].as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        created_at: v["createdAt"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .take(10)
            .collect(),
        body: v["body"].as_str().map(ToString::to_string),
        extra: Extra::default(),
    })
}
