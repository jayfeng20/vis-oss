//! The files `init` writes into a new study directory.
//!
//! `CONTEXT.md` arrives with its headings already in place and the issue body pasted
//! in, so the agent filling it in is completing a document rather than inventing a
//! structure — and so a human reading a half-finished study can see what is missing.

/// The agent contract, compiled in so a study directory is self-contained.
pub const AGENT_CONTRACT: &str = include_str!("../docs/agent-contract.md");

/// How much of an example the reader is expected to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Tutorial {
    /// Comments and `TODO`s only. The reader writes every line.
    Full,
    /// Scaffolding is present; the parts worth thinking about are left as stubs.
    Partial,
    /// Complete, runnable code.
    None,
}

impl Tutorial {
    pub fn label(self) -> &'static str {
        match self {
            Tutorial::Full => "full",
            Tutorial::Partial => "partial",
            Tutorial::None => "none",
        }
    }

    fn describes(self) -> &'static str {
        match self {
            Tutorial::Full => "every line is yours to write, from `TODO`s",
            Tutorial::Partial => "scaffolding is written; the interesting parts are stubs",
            Tutorial::None => "complete and runnable as shipped",
        }
    }
}

pub struct Context<'a> {
    pub repo: &'a str,
    pub number: u64,
    pub title: &'a str,
    pub url: &'a str,
    pub state: &'a str,
    pub created_at: &'a str,
    pub labels: &'a [String],
    pub body: &'a str,
    pub source: &'a str,
    pub commit: &'a str,
    pub tutorial: Tutorial,
    /// Steering from whoever asked for the study.
    pub notes: &'a [String],
}

/// The skeleton of a study.
///
/// Section order is the order a newcomer needs them in: what the problem is, then what
/// the code does now, then where to look, then what to decide.
pub fn context_md(c: &Context) -> String {
    let labels = if c.labels.is_empty() {
        String::new()
    } else {
        format!(" · {}", c.labels.join(" · "))
    };
    let meta = [
        (!c.state.is_empty()).then(|| c.state.to_lowercase()),
        (!c.created_at.is_empty()).then(|| format!("opened {}", c.created_at)),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");
    let mode = c.tutorial.label();
    let mode_describes = c.tutorial.describes();
    let commit = if c.commit.is_empty() {
        "(unknown)"
    } else {
        c.commit
    };
    let notes = if c.notes.is_empty() {
        String::new()
    } else {
        let items = c
            .notes
            .iter()
            .map(|n| format!("- {}\n", n.trim()))
            .collect::<String>();
        format!("## Start from\n\n{items}\n<!-- Supplied by whoever asked for this study. Treat as a lead, not a conclusion:\n     verify it before repeating it, and say in the study if it did not hold. -->\n\n")
    };
    let body = if c.body.trim().is_empty() {
        "_(the issue has no body)_".to_string()
    } else {
        c.body.trim().to_string()
    };

    format!(
        "# {repo} #{number} — {title}\n\
         \n\
         {url}  \n\
         {meta}{labels}\n\
         \n\
         > Studied against `{commit}` in `{source}`.  \n\
         > Examples are at tutorial level **{mode}** — {mode_describes}. See `AGENTS.md`.\n\
         \n\
         > **Written by an agent, and not reviewed.** It is a head start on understanding\n\
         > this issue, not an answer to it. Check any file reference before you rely on it,\n\
         > and treat the reasoning as a first draft to argue with — the maintainers own\n\
         > what the fix should be.\n\
         \n\
         ---\n\
         \n\
         ## The issue\n\
         \n\
         {body}\n\
         \n\
         ---\n\
         \n\
         {notes}## What this actually is\n\
         \n\
         <!-- Plain language, for someone who has not read the issue. -->\n\
         \n\
         ## What happens today\n\
         \n\
         <!-- The current behaviour, traced through real code. Name functions and files. -->\n\
         \n\
         ## Where the code is\n\
         \n\
         <!-- A reading order. `path/to/file.rs:123` then why a newcomer is looking at it. -->\n\
         \n\
         ## Prior art\n\
         \n\
         <!-- Code in this project that already solves a similar problem, or \"none found\"\n\
              and where you looked. Usually the most valuable section. -->\n\
         \n\
         ## Open questions\n\
         \n\
         <!-- What the issue leaves undecided, what turns on each answer, and your\n\
              recommendation. Empty is a fine answer if the issue is unambiguous. -->\n\
         \n\
         ## Examples\n\
         \n\
         <!-- What is in examples/, and the exact command to run each. -->\n\
         \n\
         ## How to verify\n\
         \n\
         <!-- Build, test and lint commands, from the project's own docs. -->\n\
         \n\
         ## What I could not verify\n\
         \n\
         <!-- Claims above that rest on inference rather than something you read, code you\n\
              could not find, and questions to put to the maintainers. An empty section is a\n\
              claim that everything above is solid. -->\n\
         \n\
         ## After — what the issue asks for\n\
         \n\
         <!-- What is different once this is fixed, seen from outside: an API that changes\n\
              shape, or the same calls behaving differently — a warning appears, a write\n\
              gets faster. Use the same file:line annotations as the examples. Describe\n\
              the target; do not write the patch. -->\n",
        repo = c.repo,
        number = c.number,
        title = c.title,
        url = c.url,
        meta = meta,
        labels = labels,
        commit = commit,
        source = c.source,
        mode = mode,
        mode_describes = mode_describes,
        body = body,
        notes = notes,
    )
}
