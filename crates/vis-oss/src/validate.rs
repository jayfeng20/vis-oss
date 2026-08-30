//! Quality checks on a study.
//!
//! Parsing a study is deliberately permissive so a thin one still renders. This is
//! where strictness lives instead. The checks split into two kinds, and the split is
//! the point:
//!
//! - **Errors** mean the study would actively mislead a reader — an example file that
//!   is not on disk, an anchor pointing into a file that no longer exists.
//! - **Warnings** mean it is thin, or has drifted in a way vis-oss can describe
//!   precisely enough to repair.

use std::path::Path;

use crate::anchor::{self, AnchorState};
use crate::study::{Anchor, Study};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
}

impl Diagnostic {
    fn error(message: impl Into<String>) -> Self {
        Self {
            level: Level::Error,
            message: message.into(),
        }
    }
    fn warn(message: impl Into<String>) -> Self {
        Self {
            level: Level::Warning,
            message: message.into(),
        }
    }
}

/// Check a study for completeness and drift. `study_dir` is where `study.json` lives.
pub fn validate(study: &Study, study_dir: &Path) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    if study.issue.number == 0 {
        out.push(Diagnostic::error("issue.number is missing or zero"));
    }
    if study.issue.repo.is_empty() {
        out.push(Diagnostic::error(
            "issue.repo is empty (expected `owner/name`)",
        ));
    }
    if study.issue.title.trim().is_empty() {
        out.push(Diagnostic::warn("issue.title is empty"));
    }

    check_prose(study, &mut out);
    check_pin(study, &mut out);
    check_anchors(study, &mut out);
    check_examples(study, study_dir, &mut out);
    check_questions(study, &mut out);

    if study.verify.is_empty() {
        out.push(Diagnostic::warn(
            "verify is empty — a reader cannot confirm they reproduced anything",
        ));
    }

    out
}

fn check_prose(study: &Study, out: &mut Vec<Diagnostic>) {
    let blank = |s: &Option<String>| s.as_deref().is_none_or(|v| v.trim().is_empty());
    if blank(&study.summary) {
        out.push(Diagnostic::warn(
            "summary is empty — the study does not say what the issue is",
        ));
    }
    if blank(&study.current_behavior) {
        out.push(Diagnostic::warn(
            "current_behavior is empty — no 'before' to compare against",
        ));
    }
    if blank(&study.desired_behavior) {
        out.push(Diagnostic::warn(
            "desired_behavior is empty — no 'after' to compare against",
        ));
    }
}

fn check_pin(study: &Study, out: &mut Vec<Diagnostic>) {
    if study.pin.commit.trim().is_empty() {
        out.push(Diagnostic::warn(
            "pin.commit is empty — anchors cannot be trusted without knowing what they were written against",
        ));
        return;
    }
    if study.pin.root.trim().is_empty() {
        return;
    }
    let root = Path::new(&study.pin.root);
    if let Some(head) = anchor::head_commit(root) {
        let pinned = study.pin.commit.trim();
        // Compare on the shorter of the two so an abbreviated pin still matches.
        let n = pinned.len().min(head.len());
        if !pinned.is_empty() && pinned[..n] != head[..n] {
            out.push(Diagnostic::warn(format!(
                "checkout has moved: study pinned to {} but {} is at {}",
                &pinned[..n.min(9)],
                study.pin.root,
                &head[..n.min(9)]
            )));
        }
    }
}

fn check_anchors(study: &Study, out: &mut Vec<Diagnostic>) {
    if study.entry_points.is_empty() {
        out.push(Diagnostic::warn(
            "entry_points is empty — no reading order for a newcomer",
        ));
    }
    if study.pin.root.trim().is_empty() {
        if !study.entry_points.is_empty() || !study.prior_art.is_empty() {
            out.push(Diagnostic::warn(
                "pin.root is empty — anchors cannot be checked against a working tree",
            ));
        }
        return;
    }
    let root = Path::new(&study.pin.root);
    let groups: [(&str, &Vec<Anchor>); 2] = [
        ("entry_points", &study.entry_points),
        ("prior_art", &study.prior_art),
    ];

    for (group, anchors) in groups {
        for a in anchors {
            let state = anchor::check(root, a);
            if !state.is_problem() {
                continue;
            }
            let at = a
                .line
                .map_or_else(|| a.path.clone(), |l| format!("{}:{l}", a.path));
            let msg = format!("{group} {at}: {}", state.describe());
            if state.is_misleading() {
                // A moved symbol is repairable and we just said where it went; the rest
                // means the anchor now points at something the author never saw.
                if matches!(state, AnchorState::Moved { .. }) {
                    out.push(Diagnostic::warn(msg));
                } else {
                    out.push(Diagnostic::error(msg));
                }
            } else {
                out.push(Diagnostic::warn(msg));
            }
        }
    }
}

fn check_examples(study: &Study, study_dir: &Path, out: &mut Vec<Diagnostic>) {
    if study.examples.is_empty() {
        out.push(Diagnostic::warn(
            "examples is empty — nothing for the reader to run",
        ));
        return;
    }
    for ex in &study.examples {
        if !study_dir.join(&ex.file).exists() {
            out.push(Diagnostic::error(format!(
                "example file not on disk: {}",
                ex.file
            )));
        }
        if ex.run.as_deref().is_none_or(|r| r.trim().is_empty()) {
            out.push(Diagnostic::warn(format!(
                "example {} has no `run` command",
                ex.file
            )));
        }
        if ex.title.trim().is_empty() {
            out.push(Diagnostic::warn(format!(
                "example {} has no title",
                ex.file
            )));
        }
    }
    let has_current = study.examples.iter().any(|e| {
        matches!(
            e.stage,
            crate::study::Stage::Current | crate::study::Stage::Both
        )
    });
    let has_proposed = study.examples.iter().any(|e| {
        matches!(
            e.stage,
            crate::study::Stage::Proposed | crate::study::Stage::Both
        )
    });
    if !has_current {
        out.push(Diagnostic::warn("no example shows current behavior"));
    }
    if !has_proposed {
        out.push(Diagnostic::warn("no example shows the proposed behavior"));
    }
}

fn check_questions(study: &Study, out: &mut Vec<Diagnostic>) {
    for q in &study.open_questions {
        if q.why_it_matters
            .as_deref()
            .is_none_or(|v| v.trim().is_empty())
        {
            out.push(Diagnostic::warn(format!(
                "open question has no why_it_matters: {}",
                truncate(&q.question, 60)
            )));
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    format!("{cut}…")
}
