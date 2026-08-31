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
    /// Complete, runnable code — the whole thing worked through.
    Full,
    /// Scaffolding is present; the parts worth thinking about are left as stubs.
    Partial,
    /// Comments and `TODO`s only. The reader writes every line.
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
            Tutorial::Full => "complete code, nothing left to write",
            Tutorial::Partial => {
                "scaffolding is written; the parts worth thinking about are exercises"
            }
            Tutorial::None => "every line is yours to write, from `TODO`s",
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
/// Section order is the order a lesson runs in: what the problem is, then a walkthrough
/// that observes it and traces it into the code, then the precedent, the exercises, the
/// decisions, and last the target and the trust boundary.
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
         > Probes are at tutorial level **{mode}** — {mode_describes}. See `AGENTS.md`.\n\
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
         {notes}## What this is about\n\
         \n\
         <!-- Plain language, for someone who has not read the issue. End with 2-4\n\
              \"by the end you should be able to...\" goals — concrete abilities. -->\n\
         \n\
         ## Walkthrough\n\
         \n\
         <!-- Numbered steps. Each is one action — a command to run or a file to open —\n\
              then what the reader should observe and why the code does that, with\n\
              file:line to the declarations. Step 1 is getting an environment.\n\
              Observation before explanation; recap at the pivots. -->\n\
         \n\
         ## Prior art\n\
         \n\
         <!-- Code in this project that already solves a similar problem, or \"none found\"\n\
              and where you looked. Usually the most valuable section. -->\n\
         \n\
         ## Exercises\n\
         \n\
         <!-- One entry per stub — an instrument to write or a prediction to fill: file\n\
              and symbol, what goes there, what completing it teaches, and which\n\
              walkthrough step supplies what is needed. \"None — <why>\" is a fine answer\n\
              when there is nothing to run. -->\n\
         \n\
         ## Open questions\n\
         \n\
         <!-- What the issue leaves undecided, now that the reader knows enough to argue:\n\
              what turns on each answer, and your recommendation with its reason. Empty\n\
              is a fine answer if the issue is unambiguous. -->\n\
         \n\
         ## After — what the issue asks for\n\
         \n\
         <!-- What is different once this is fixed, seen from outside: an API that changes\n\
              shape, or the same calls behaving differently — a warning appears, a write\n\
              gets faster. Use the same file:line annotations as the probes. End with how\n\
              a fix would be checked: the probe to rerun, and the project's own build,\n\
              test and lint commands. Describe the target; do not write the patch. -->\n\
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
        mode_describes = mode_describes,
        body = body,
        notes = notes,
    )
}
