//! `vis-oss init` — create the skeleton an investigating agent fills in.
//!
//! Everything here is mechanical: read the issue, read the checkout, write a
//! well-formed but empty [`Study`]. No judgement is applied, because judgement is the
//! agent's job and this tool must stay testable and deterministic.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::anchor;
use crate::study::{Claim, Extra, Issue, Mode, Pin, Study, SCHEMA_VERSION};

/// Phrases that, in an issue comment, mean "I intend to do this".
///
/// Deliberately loose: a false positive costs one line of output, while a missed claim
/// means duplicating someone's work.
const CLAIM_PHRASES: &[&str] = &[
    "working on",
    "work on this",
    "i'll take",
    "ill take",
    "assign it to me",
    "assign this to me",
    "assign me",
    "pick this up",
    "picking this up",
    "like to try",
    "let me try",
    "i can try",
    "submit a pr",
    "working towards a pr",
];

pub struct InitOptions {
    pub number: u64,
    /// `owner/name`. Inferred from the git remotes when absent.
    pub repo: Option<String>,
    pub out: PathBuf,
    pub mode: Mode,
    /// Checkout the anchors will refer to. Defaults to the enclosing repository.
    pub source: Option<PathBuf>,
}

pub struct InitOutcome {
    pub dir: PathBuf,
    pub study: Study,
    pub notes: Vec<String>,
}

pub fn init(opts: &InitOptions) -> Result<InitOutcome> {
    let mut notes = Vec::new();

    let cwd = std::env::current_dir()?;
    let source = match &opts.source {
        Some(p) => p.clone(),
        None => anchor::repo_root(&cwd).context(
            "not inside a git repository — run vis-oss from a checkout, or pass --source",
        )?,
    };

    let repo = match &opts.repo {
        Some(r) => r.clone(),
        None => anchor::repo_slug(&source)
            .context("could not infer the repository from git remotes — pass --repo owner/name")?,
    };
    if opts.repo.is_none() {
        notes.push(format!(
            "inferred repo {repo} from git remotes (upstream preferred)"
        ));
    }

    let commit = anchor::head_commit(&source).unwrap_or_default();
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
    for c in &issue.claims {
        let tail = if c.opened_pr {
            "and opened a PR"
        } else {
            "but never opened a PR"
        };
        notes.push(format!("@{} claimed this {} {}", c.user, c.at, tail));
    }

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

    let dir = &opts.out;
    if dir.join("study.json").exists() {
        bail!(
            "{} already contains a study.json — refusing to overwrite",
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

    Ok(InitOutcome {
        dir: dir.clone(),
        study,
        notes,
    })
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

    let claims = extract_claims(repo, &v);

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
        claims,
        extra: Extra::default(),
    })
}

/// Find comments that read as "I'm taking this", and check whether the author ever
/// followed through with a PR.
fn extract_claims(repo: &str, v: &Value) -> Vec<Claim> {
    let Some(comments) = v["comments"].as_array() else {
        return Vec::new();
    };
    let mut claims = Vec::new();
    for c in comments {
        let body = c["body"].as_str().unwrap_or_default().to_lowercase();
        if !CLAIM_PHRASES.iter().any(|p| body.contains(p)) {
            continue;
        }
        let Some(user) = c["author"]["login"].as_str() else {
            continue;
        };
        if claims.iter().any(|e: &Claim| e.user == user) {
            continue;
        }
        claims.push(Claim {
            user: user.to_string(),
            at: c["createdAt"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .take(10)
                .collect(),
            url: c["url"].as_str().unwrap_or_default().to_string(),
            opened_pr: author_has_pr(repo, user),
        });
    }
    claims
}

/// Best-effort: whether `user` has ever opened a PR against `repo`.
///
/// A claim with no PR behind it after months is the signal that an issue is actually
/// free to pick up, so this is worth one extra API call per claimant.
fn author_has_pr(repo: &str, user: &str) -> bool {
    Command::new("gh")
        .args([
            "pr", "list", "--repo", repo, "--author", user, "--state", "all", "--limit", "1",
            "--json", "number",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| serde_json::from_slice::<Value>(&o.stdout).ok())
        .and_then(|v| v.as_array().map(|a| !a.is_empty()))
        .unwrap_or(false)
}

/// Where `init` should write when the user does not say.
pub fn default_out(number: u64) -> PathBuf {
    Path::new(&format!("study-{number}")).to_path_buf()
}
