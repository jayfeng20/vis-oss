//! The vis-oss study — the JSON contract between an investigating agent and this tool.
//!
//! An agent reads an issue and a codebase and emits one [`Study`]. vis-oss owns every
//! decision about presentation and, more importantly, about *staleness*: a study is
//! pinned to the commit it was written against, and every code reference carries
//! enough context to detect that the code has moved out from under it.
//!
//! # Evolving this schema
//!
//! The prompts that produce studies change more often than vis-oss does, so the
//! contract bends rather than breaks. The same four rules caliper uses apply here:
//!
//! 1. **Parsing is permissive; `vis-oss validate` is strict.** Only what is needed to
//!    identify the issue at all is required. Everything else defaults, and a thin
//!    study renders with gaps instead of failing.
//! 2. **Unknown fields are preserved, not dropped**, via the `extra` catch-all, so a
//!    prompt can emit a new field before vis-oss knows about it.
//! 3. **Open sets where the taxonomy is domain-specific.** `role` on an anchor is a
//!    free string; `stage` and `mode` stay closed because rendering branches on them.
//! 4. **Renames ship as `#[serde(alias = "...")]`.**
//!
//! Bump [`SCHEMA_VERSION`] only when a field changes *meaning*.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bumped only when an existing field changes meaning — not for additions.
pub const SCHEMA_VERSION: u32 = 1;

/// Unknown JSON keys, kept so they survive a round trip.
pub type Extra = BTreeMap<String, serde_json::Value>;

/// One investigation of one issue.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Study {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub issue: Issue,
    /// The checkout the study was written against. Anchors are meaningless without it.
    #[serde(default)]
    pub pin: Pin,
    /// Whether examples are exercises for the reader or finished code.
    #[serde(default)]
    pub mode: Mode,
    /// Plain-language restatement of the problem, for someone who has not read the issue.
    #[serde(default)]
    pub summary: Option<String>,
    /// What the code does today.
    #[serde(default)]
    pub current_behavior: Option<String>,
    /// What it should do once the issue is resolved.
    #[serde(default)]
    pub desired_behavior: Option<String>,
    /// Where to start reading, in the order a newcomer should read it.
    #[serde(default)]
    pub entry_points: Vec<Anchor>,
    /// Existing code that already solves a similar problem — the strongest signal a
    /// study can carry, because it converts a design argument into a precedent.
    #[serde(default)]
    pub prior_art: Vec<Anchor>,
    /// Decisions the implementer must make that the issue does not settle.
    #[serde(default)]
    pub open_questions: Vec<OpenQuestion>,
    /// Runnable files under the study directory.
    #[serde(default)]
    pub examples: Vec<Example>,
    /// How to build and test the affected surface.
    #[serde(default)]
    pub verify: Vec<String>,

    #[serde(flatten, default)]
    pub extra: Extra,
}

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

/// Issue metadata, normally filled by `vis-oss init` from `gh`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Issue {
    /// `owner/name` of the repository the issue lives in.
    #[serde(default)]
    pub repo: String,
    #[serde(default)]
    pub number: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub created_at: String,
    /// The issue body, verbatim. Kept so the study is readable offline.
    #[serde(default)]
    pub body: Option<String>,
    /// People who have said they are working on this. An unfulfilled claim is not a
    /// blocker, but you should know about it before you open a PR.
    #[serde(default)]
    pub claims: Vec<Claim>,

    #[serde(flatten, default)]
    pub extra: Extra,
}

/// A prior "I'll take this" comment.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Claim {
    pub user: String,
    #[serde(default)]
    pub at: String,
    #[serde(default)]
    pub url: String,
    /// Whether the claimant ever opened a PR. A claim with no PR after months is stale.
    #[serde(default)]
    pub opened_pr: bool,
}

/// The checkout a study was written against.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Pin {
    /// Absolute or relative path to the local checkout the anchors refer to.
    #[serde(default)]
    pub root: String,
    /// Commit the anchors were resolved against.
    #[serde(default)]
    pub commit: String,

    #[serde(flatten, default)]
    pub extra: Extra,
}

/// Whether examples teach or solve.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Examples are comment-and-`TODO` scaffolds; the reader writes the code.
    #[default]
    Tutorial,
    /// Examples are finished, runnable code.
    Solution,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Tutorial => "tutorial",
            Mode::Solution => "solution",
        }
    }
}

/// A pointer into the codebase, resolvable and drift-checkable.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Anchor {
    /// Path relative to the pinned repo root.
    pub path: String,
    /// 1-indexed line. Optional: some anchors name a whole file.
    #[serde(default)]
    pub line: Option<u32>,
    /// A distinctive substring expected at `line` — a signature, a constant, a message.
    ///
    /// This is what makes drift detectable. Line numbers rot on every rebase; a symbol
    /// can be found again, so `vis-oss validate` reports "moved to line N" rather than
    /// silently pointing at unrelated code.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Why a reader should care about this location.
    #[serde(default)]
    pub role: Option<String>,

    #[serde(flatten, default)]
    pub extra: Extra,
}

/// A decision the issue leaves open.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenQuestion {
    pub question: String,
    /// What changes depending on the answer. A question with no consequence is noise.
    #[serde(default)]
    pub why_it_matters: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    /// What the investigating agent would do absent an answer.
    #[serde(default)]
    pub recommendation: Option<String>,

    #[serde(flatten, default)]
    pub extra: Extra,
}

/// A file in the study directory the reader can actually run.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Example {
    /// Path relative to the study directory.
    pub file: String,
    #[serde(default)]
    pub title: String,
    /// Whether this demonstrates today's behavior or the behavior after a fix.
    #[serde(default)]
    pub stage: Stage,
    /// What the reader should learn from running it.
    #[serde(default)]
    pub shows: Option<String>,
    /// The exact command to run it.
    #[serde(default)]
    pub run: Option<String>,

    #[serde(flatten, default)]
    pub extra: Extra,
}

/// Which side of the change an example illustrates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    /// Reproduces what the code does today.
    #[default]
    Current,
    /// Demonstrates the behavior the issue asks for.
    Proposed,
    /// Shows both sides side by side.
    Both,
}

impl Stage {
    pub fn label(self) -> &'static str {
        match self {
            Stage::Current => "current",
            Stage::Proposed => "proposed",
            Stage::Both => "before/after",
        }
    }
}
