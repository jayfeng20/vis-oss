//! Where studies are filed, remembered between runs.
//!
//! One plain-text file holding one path. A format with a parser would be a dependency
//! and a schema for a single value.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Resolution order: the path given on the command line, then the saved root, then
/// `~/vis-oss`.
pub fn root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Some(saved) = saved()? {
        return Ok(saved);
    }
    Ok(home()?.join("vis-oss"))
}

/// The saved root, if one has been set and still exists as a readable path.
pub fn saved() -> Result<Option<PathBuf>> {
    let path = config_file()?;
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(None);
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(expand_tilde(trimmed)?))
}

/// Remember `path` as the root for future runs. Returns where the setting was written.
pub fn save(path: &Path) -> Result<PathBuf> {
    let file = config_file()?;
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let absolute = expand_tilde(&path.to_string_lossy())?;
    std::fs::write(&file, format!("{}\n", absolute.display()))
        .with_context(|| format!("writing {}", file.display()))?;
    Ok(file)
}

fn config_file() -> Result<PathBuf> {
    Ok(home()?.join(".config").join("vis-oss").join("root"))
}

/// Accept `~/…` in the saved file and on the command line, since both are typed by hand.
fn expand_tilde(path: &str) -> Result<PathBuf> {
    match path.strip_prefix("~/") {
        Some(rest) => Ok(home()?.join(rest)),
        None if path == "~" => home(),
        None => Ok(PathBuf::from(path)),
    }
}

fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .context("could not determine the home directory")
}
