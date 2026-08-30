//! vis-oss — visualize open-source software, so an unfamiliar issue is legible before
//! you change anything.
//!
//! The split is the same one caliper draws for PR review: an agent investigates and
//! emits a structured [`Study`](study::Study); vis-oss owns the skeleton, the
//! validation and the presentation. The binary never invokes an agent, which keeps it
//! deterministic and testable.
//!
//! What vis-oss adds beyond a directory template is **drift detection**. A study is a
//! set of claims about where things live in a codebase, and those claims rot on every
//! rebase. Each anchor carries a symbol as well as a line, so [`anchor::check`] can
//! report "moved to line 5305" instead of leaving a reader staring at the wrong code.

pub mod anchor;
pub mod git;
pub mod markdown;
pub mod render;
pub mod repair;
pub mod scaffold;
pub mod study;
pub mod validate;
