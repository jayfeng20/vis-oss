//! The files `init` writes into a new study directory.
//!
//! `CONTEXT.md` arrives with its headings already in place and the issue body pasted
//! in, so the agent filling it in is completing a document rather than inventing a
//! structure — and so a human reading a half-finished study can see what is missing.

/// The agent contract, compiled in so a study directory is self-contained.
pub const AGENT_CONTRACT: &str = include_str!("../docs/agent-contract.md");

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
    pub tutorial: bool,
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
    let mode = if c.tutorial { "tutorial" } else { "solution" };
    let commit = if c.commit.is_empty() {
        "(unknown)"
    } else {
        c.commit
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
         > Examples are in **{mode}** mode. See `AGENTS.md` for what that means.\n\
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
         ## What this actually is\n\
         \n\
         <!-- Plain language, for someone who has not read the issue. -->\n\
         \n\
         ## What happens today\n\
         \n\
         <!-- The current behaviour, traced through real code. Name functions and files. -->\n\
         \n\
         ## What should happen\n\
         \n\
         <!-- Precise enough to test. -->\n\
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
              claim that everything above is solid. -->\n",
        repo = c.repo,
        number = c.number,
        title = c.title,
        url = c.url,
        meta = meta,
        labels = labels,
        commit = commit,
        source = c.source,
        mode = mode,
        body = body,
    )
}
