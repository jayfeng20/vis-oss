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

/// The branch currently checked out, or `None` on a detached HEAD.
pub fn current_branch(root: &Path) -> Option<String> {
    git(root, &["rev-parse", "--abbrev-ref", "HEAD"]).filter(|b| b != "HEAD" && !b.is_empty())
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

/// Where `info/exclude` lives for this checkout.
///
/// Not always `<root>/.git/info/`: in a linked worktree `.git` is a *file* pointing at
/// the real gitdir, and `info/exclude` belongs to the common dir shared by every
/// worktree. Guessing the path here would silently write an exclude file that git
/// never reads.
fn exclude_file(root: &Path) -> Option<PathBuf> {
    let common = git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .or_else(|| git(root, &["rev-parse", "--git-common-dir"]))?;
    let common = PathBuf::from(&common);
    let common = if common.is_absolute() {
        common
    } else {
        root.join(common)
    };
    Some(common.join("info").join("exclude"))
}

/// Whether git currently ignores `target`.
pub fn is_ignored(root: &Path, target: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["check-ignore", "--quiet"])
        .arg(target)
        .status()
        .is_ok_and(|s| s.success())
}

/// The result of trying to keep a study out of the user's commits.
#[derive(Debug, Clone)]
pub enum Excluded {
    /// The pattern was added and git now confirms the path is ignored.
    Added(String),
    /// Git already ignored it; nothing was written.
    Already,
    /// The path is *not* ignored and vis-oss could not make it so.
    Failed(String),
}

/// Make `target` ignored locally, and verify that it worked.
///
/// Writes to `info/exclude` rather than `.gitignore` on purpose: `.gitignore` is
/// tracked, so ignoring a scratch directory there would itself become a commit in the
/// contributor's PR. `info/exclude` is per-clone and never leaves the machine.
///
/// The write is verified with `git check-ignore` rather than assumed, because an
/// exclude pattern that does not match is indistinguishable from one that does until
/// something is accidentally committed.
pub fn ensure_ignored(root: &Path, target: &Path) -> Excluded {
    use std::io::Write as _;

    if is_ignored(root, target) {
        return Excluded::Already;
    }

    // Anchor to the repository root and restrict to directories, so the pattern cannot
    // match an unrelated file of the same name deeper in the tree.
    let rel = target.strip_prefix(root).unwrap_or(target);
    let first = rel
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().into_owned());
    let Some(first) = first else {
        return Excluded::Failed("study directory is the repository root".to_string());
    };
    let pattern = format!("/{first}/");

    let Some(path) = exclude_file(root) else {
        return Excluded::Failed("could not locate info/exclude".to_string());
    };
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return Excluded::Failed(format!("could not create {}", parent.display()));
        }
    }
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if !existing.lines().any(|l| l.trim() == pattern) {
        let write = (|| -> std::io::Result<()> {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            if !existing.is_empty() && !existing.ends_with('\n') {
                writeln!(f)?;
            }
            writeln!(
                f,
                "\n# vis-oss study directories (local only, never committed)\n{pattern}"
            )
        })();
        if let Err(e) = write {
            return Excluded::Failed(format!("writing {}: {e}", path.display()));
        }
    }

    if is_ignored(root, target) {
        Excluded::Added(pattern)
    } else {
        Excluded::Failed(format!(
            "{pattern} did not take effect; check {}",
            path.display()
        ))
    }
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
