//! `vis-oss repair` — write drift corrections back into the study.
//!
//! [`crate::anchor::check`] already computes the new location of a symbol that moved.
//! Reporting that and then making a human hand-edit JSON wastes the most useful thing
//! the tool knows. Repair closes the loop: re-resolve every anchor, rewrite the lines
//! that moved, and re-pin to the commit the anchors now describe.
//!
//! What repair deliberately does *not* do is invent. A symbol that has vanished may
//! have been renamed, deleted, or moved to another file — those are judgement calls,
//! so they are reported for a human and left untouched.

use std::path::Path;

use crate::anchor::{self, AnchorState};
use crate::study::{Anchor, Study};

#[derive(Debug, Clone)]
pub enum Change {
    /// A line number was corrected.
    Relined {
        group: &'static str,
        path: String,
        from: Option<u32>,
        to: u32,
    },
    /// The study was re-pinned to the current checkout.
    Repinned { from: String, to: String },
}

#[derive(Debug, Clone)]
pub struct Unfixable {
    pub group: &'static str,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct Repair {
    pub changes: Vec<Change>,
    pub unfixable: Vec<Unfixable>,
}

impl Repair {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.unfixable.is_empty()
    }
}

/// Re-resolve every anchor and correct the ones that merely moved.
///
/// Returns the changes made; the caller decides whether to persist `study`.
pub fn repair(study: &mut Study) -> Repair {
    let mut out = Repair::default();
    if study.pin.root.trim().is_empty() {
        return out;
    }
    let root = Path::new(&study.pin.root).to_path_buf();

    let fix = |group: &'static str, anchors: &mut Vec<Anchor>, out: &mut Repair| {
        for a in anchors.iter_mut() {
            match anchor::check(&root, a) {
                AnchorState::Moved { to } => {
                    out.changes.push(Change::Relined {
                        group,
                        path: a.path.clone(),
                        from: a.line,
                        to,
                    });
                    a.line = Some(to);
                }
                state @ (AnchorState::SymbolNotFound
                | AnchorState::FileMissing
                | AnchorState::LineOutOfRange { .. }) => {
                    out.unfixable.push(Unfixable {
                        group,
                        path: a.path.clone(),
                        reason: state.describe(),
                    });
                }
                AnchorState::Ok | AnchorState::Unverifiable => {}
            }
        }
    };
    fix("entry_points", &mut study.entry_points, &mut out);
    fix("prior_art", &mut study.prior_art, &mut out);

    // Re-pin only once the anchors describe the new commit — otherwise the pin would
    // claim a freshness the unfixable anchors do not have.
    if out.unfixable.is_empty() {
        if let Some(head) = anchor::head_commit(&root) {
            if head != study.pin.commit {
                out.changes.push(Change::Repinned {
                    from: short(&study.pin.commit),
                    to: short(&head),
                });
                study.pin.commit = head;
            }
        }
    }
    out
}

fn short(commit: &str) -> String {
    let s: String = commit.chars().take(9).collect();
    if s.is_empty() {
        "(unpinned)".to_string()
    } else {
        s
    }
}
