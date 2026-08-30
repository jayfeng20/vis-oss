//! Reading the state of the checkout a study is being written against.
//!
//! vis-oss assumes the workflow a contributor actually has: a fork cloned locally, an
//! `upstream` remote pointing at the real project, and a branch that may or may not
//! still resemble either. Everything here answers questions about that arrangement.

use std::path::{Path, PathBuf};
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The root of the git repository containing `from`.
pub fn repo_root(from: &Path) -> Option<PathBuf> {
    git(from, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

/// The commit currently checked out.
pub fn head_commit(root: &Path) -> Option<String> {
    git(root, &["rev-parse", "HEAD"]).filter(|s| !s.is_empty())
}

/// The name of the remote the issues live on.
///
/// Prefers `upstream` over `origin`: on a fork, `origin` is your copy, and both the
/// issues and the commits you are behind belong to upstream.
pub fn canonical_remote(root: &Path) -> Option<&'static str> {
    ["upstream", "origin"]
        .into_iter()
        .find(|r| git(root, &["remote", "get-url", r]).is_some())
}

/// The `owner/name` slug for the repository the issues live in.
pub fn repo_slug(root: &Path) -> Option<String> {
    let remote = canonical_remote(root)?;
    parse_slug(&git(root, &["remote", "get-url", remote])?)
}

/// Extract `owner/name` from an HTTPS or SSH git remote URL.
fn parse_slug(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("git@")
        .and_then(|s| s.split_once(':').map(|(_, p)| p))
        .or_else(|| {
            url.split_once("://")
                .map(|(_, r)| r)
                .and_then(|r| r.split_once('/').map(|(_, p)| p))
        })?;
    let rest = rest.strip_suffix(".git").unwrap_or(rest);
    let mut parts = rest.split('/').filter(|p| !p.is_empty());
    Some(format!("{}/{}", parts.next()?, parts.next()?))
}

/// The remote's default branch, e.g. `main`. Falls back to `main` when it cannot be read.
pub fn default_branch(root: &Path, remote: &str) -> String {
    git(
        root,
        &["symbolic-ref", &format!("refs/remotes/{remote}/HEAD")],
    )
    .and_then(|r| r.rsplit('/').next().map(ToString::to_string))
    .unwrap_or_else(|| "main".to_string())
}

/// How far the checkout has fallen behind the canonical remote's default branch.
#[derive(Debug, Clone)]
pub struct Staleness {
    pub remote: String,
    pub branch: String,
    /// Commits on `<remote>/<branch>` that are not in HEAD.
    pub behind: u32,
}

impl Staleness {
    pub fn reference(&self) -> String {
        format!("{}/{}", self.remote, self.branch)
    }
}

/// Update the canonical remote's refs so a staleness check means anything.
///
/// This touches only remote-tracking refs — no local branch, no working tree, no
/// merge. Syncing remains the user's decision; without it, though, the comparison is
/// against whatever was true the last time they happened to fetch.
pub fn fetch(root: &Path, remote: &str) -> bool {
    git(root, &["fetch", "--quiet", remote]).is_some()
}

/// Commits the checkout is missing relative to the canonical remote's default branch.
///
/// `None` when there is no usable remote or the comparison cannot be made.
pub fn staleness(root: &Path) -> Option<Staleness> {
    let remote = canonical_remote(root)?;
    fetch(root, remote);
    let branch = default_branch(root, remote);
    let reference = format!("{remote}/{branch}");
    let behind = git(
        root,
        &["rev-list", "--count", &format!("HEAD..{reference}")],
    )?
    .parse::<u32>()
    .ok()?;
    Some(Staleness {
        remote: remote.to_string(),
        branch,
        behind,
    })
}

/// Fast-forward the checkout onto the canonical remote's default branch.
///
/// Deliberately narrow. It refuses rather than resolving anything: no merge commit, no
/// rebase, no stash, no touching a branch other than the one checked out. A contributor
/// runs this before starting work, so the only case worth automating is the boring one —
/// clean tree, sitting on the default branch, strictly behind.
///
/// Returns the reason it declined, so the caller can say why.
pub fn fast_forward(root: &Path, stale: &Staleness) -> Result<(), String> {
    let branch = current_branch(root).ok_or_else(|| "HEAD is detached".to_string())?;
    if branch != stale.branch {
        return Err(format!(
            "on branch {branch}, not {}. Sync it yourself, or switch first",
            stale.branch
        ));
    }
    if !is_clean(root) {
        return Err("the working tree has uncommitted changes".to_string());
    }
    let reference = stale.reference();
    git(root, &["merge", "--ff-only", &reference])
        .map(|_| ())
        .ok_or_else(|| format!("git merge --ff-only {reference} failed; local commits?"))
}

/// The branch currently checked out, or `None` on a detached HEAD.
fn current_branch(root: &Path) -> Option<String> {
    git(root, &["rev-parse", "--abbrev-ref", "HEAD"]).filter(|b| b != "HEAD" && !b.is_empty())
}

/// Whether the working tree and index are free of changes.
fn is_clean(root: &Path) -> bool {
    git(root, &["status", "--porcelain"]).is_some_and(|s| s.is_empty())
}

/// Whether `path` sits inside a git working tree.
///
/// Used only to warn: a study written inside the repository it describes will show up
/// in `git status` and can be committed by accident.
pub fn is_inside_repo(path: &Path) -> bool {
    let probe = if path.exists() {
        path.to_path_buf()
    } else {
        // The study directory does not exist yet; ask about the nearest parent that does.
        match path.ancestors().find(|p| p.exists()) {
            Some(p) => p.to_path_buf(),
            None => return false,
        }
    };
    git(&probe, &["rev-parse", "--is-inside-work-tree"]).as_deref() == Some("true")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_and_ssh_remotes() {
        assert_eq!(
            parse_slug("https://github.com/lance-format/lance.git").as_deref(),
            Some("lance-format/lance")
        );
        assert_eq!(
            parse_slug("https://github.com/lance-format/lance").as_deref(),
            Some("lance-format/lance")
        );
        assert_eq!(
            parse_slug("git@github.com:jayfeng20/lance.git").as_deref(),
            Some("jayfeng20/lance")
        );
        assert_eq!(parse_slug("not-a-url"), None);
    }
}
