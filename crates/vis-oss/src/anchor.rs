//! Resolving code anchors, and detecting when they have rotted.
//!
//! A study is a set of claims about where things are in a codebase, and those claims
//! decay every time upstream moves. The failure mode is quiet and expensive: a line
//! number that once pointed at the function you cared about now points at unrelated
//! code, and the reader believes it.
//!
//! So an [`Anchor`](crate::study::Anchor) carries a `symbol` — a distinctive substring
//! expected at that line — and [`check`] re-resolves it against the working tree. A
//! moved symbol is reported with its new line rather than treated as a failure, which
//! makes a stale study repairable instead of merely wrong.

use std::path::Path;

use crate::study::Anchor;

/// How far from the recorded line a symbol may be found and still count as "here".
///
/// Non-zero because a study is written by reading a file, and an edit a few lines up
/// shifts every anchor below it without changing what they point at.
const NEARBY_LINES: u32 = 2;

/// The outcome of re-resolving one anchor against the working tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorState {
    /// The symbol is at, or within [`NEARBY_LINES`] of, the recorded line.
    Ok,
    /// The symbol is still in the file, but somewhere else.
    Moved { to: u32 },
    /// The file exists but the symbol is gone.
    SymbolNotFound,
    /// No `symbol` was recorded, so nothing can be verified beyond the file existing.
    Unverifiable,
    /// The recorded line is past the end of the file.
    LineOutOfRange { file_lines: u32 },
    /// The path does not resolve under the pinned root.
    FileMissing,
}

impl AnchorState {
    /// Whether this state should be reported to the user.
    pub fn is_problem(&self) -> bool {
        !matches!(self, AnchorState::Ok)
    }

    /// Whether this state means the anchor is actively misleading, rather than merely
    /// unverified. Drives the error/warning split in `validate`.
    pub fn is_misleading(&self) -> bool {
        matches!(
            self,
            AnchorState::Moved { .. }
                | AnchorState::SymbolNotFound
                | AnchorState::LineOutOfRange { .. }
                | AnchorState::FileMissing
        )
    }

    pub fn describe(&self) -> String {
        match self {
            AnchorState::Ok => "ok".to_string(),
            AnchorState::Moved { to } => format!("moved to line {to}"),
            AnchorState::SymbolNotFound => "symbol no longer in file".to_string(),
            AnchorState::Unverifiable => "no symbol recorded, not verifiable".to_string(),
            AnchorState::LineOutOfRange { file_lines } => {
                format!("line past end of file ({file_lines} lines)")
            }
            AnchorState::FileMissing => "file not found".to_string(),
        }
    }
}

/// Re-resolve one anchor against `root`.
pub fn check(root: &Path, anchor: &Anchor) -> AnchorState {
    let path = root.join(&anchor.path);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return AnchorState::FileMissing;
    };
    let lines: Vec<&str> = text.lines().collect();
    let total = u32::try_from(lines.len()).unwrap_or(u32::MAX);

    let Some(symbol) = anchor
        .symbol
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return match anchor.line {
            Some(line) if line > total => AnchorState::LineOutOfRange { file_lines: total },
            _ => AnchorState::Unverifiable,
        };
    };

    let found_at = |idx: usize| u32::try_from(idx).unwrap_or(u32::MAX) + 1;

    match anchor.line {
        Some(line) => {
            if line > total {
                return AnchorState::LineOutOfRange { file_lines: total };
            }
            let lo = line.saturating_sub(NEARBY_LINES).max(1);
            let hi = (line + NEARBY_LINES).min(total);
            // `line` is 1-indexed; the slice below is [lo-1, hi).
            let near = lines[(lo - 1) as usize..hi as usize]
                .iter()
                .any(|l| l.contains(symbol));
            if near {
                return AnchorState::Ok;
            }
            lines
                .iter()
                .position(|l| l.contains(symbol))
                .map_or(AnchorState::SymbolNotFound, |idx| AnchorState::Moved {
                    to: found_at(idx),
                })
        }
        None => {
            if lines.iter().any(|l| l.contains(symbol)) {
                AnchorState::Ok
            } else {
                AnchorState::SymbolNotFound
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, body).unwrap();
        p
    }

    fn anchor(path: &str, line: Option<u32>, symbol: Option<&str>) -> Anchor {
        Anchor {
            path: path.to_string(),
            line,
            symbol: symbol.map(ToString::to_string),
            role: None,
            extra: crate::study::Extra::default(),
        }
    }

    #[test]
    fn detects_exact_moved_and_missing() {
        let dir = std::env::temp_dir().join(format!("vis-oss-anchor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        write(&dir, "a.rs", "one\ntwo\nfn target() {}\nfour\n");

        // Exact hit.
        assert_eq!(
            check(&dir, &anchor("a.rs", Some(3), Some("fn target"))),
            AnchorState::Ok
        );
        // Within the nearby window.
        assert_eq!(
            check(&dir, &anchor("a.rs", Some(4), Some("fn target"))),
            AnchorState::Ok
        );
        // Outside it — reported with the real line so the study can be repaired.
        assert_eq!(
            check(&dir, &anchor("a.rs", Some(1), Some("four"))),
            AnchorState::Moved { to: 4 }
        );
        assert_eq!(
            check(&dir, &anchor("a.rs", Some(1), Some("absent"))),
            AnchorState::SymbolNotFound
        );
        assert_eq!(
            check(&dir, &anchor("a.rs", Some(99), Some("one"))),
            AnchorState::LineOutOfRange { file_lines: 4 }
        );
        assert_eq!(
            check(&dir, &anchor("nope.rs", Some(1), Some("x"))),
            AnchorState::FileMissing
        );
        assert_eq!(
            check(&dir, &anchor("a.rs", Some(1), None)),
            AnchorState::Unverifiable
        );
        // A file-level anchor still verifies the symbol exists somewhere.
        assert_eq!(
            check(&dir, &anchor("a.rs", None, Some("fn target"))),
            AnchorState::Ok
        );

        std::fs::remove_dir_all(&dir).ok();
    }
}
