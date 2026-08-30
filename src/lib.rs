//! vis-oss — understand an open-source issue before you try to fix it.
//!
//! The program is deliberately small. It reads an issue and a checkout, creates a
//! directory, and writes a `CONTEXT.md` with its headings in place and the agent
//! contract beside it. Everything that makes a study *good* lives in that contract,
//! which is prose, because the study is prose.
//!
//! What vis-oss will not do is invoke an agent or parse a study back in. Both were
//! tried and removed: orchestration makes the binary non-deterministic, and a schema in
//! the middle meant an agent writing markdown into JSON strings so a renderer could
//! turn it back into markdown.

pub mod git;
pub mod scaffold;
pub mod template;
