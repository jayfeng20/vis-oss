//! Markdown rendering — the form a study takes when it outlives the terminal.
//!
//! A study is a reference document you come back to while implementing, so it needs a
//! form that can be committed, diffed, read on a forge, or pasted into an issue. That
//! is a different job from the terminal view, which optimises for one skim, so this is
//! a separate emitter rather than the same one with the colour turned off.

use std::fmt::Write as _;

use crate::study::{Stage, Study};

pub fn render(study: &Study) -> String {
    let mut out = String::new();
    header(&mut out, study);
    problem(&mut out, study);
    code_map(&mut out, study);
    questions(&mut out, study);
    examples(&mut out, study);
    verify(&mut out, study);
    out
}

fn header(out: &mut String, study: &Study) {
    let out = &mut *out;
    let i = &study.issue;

    let _ = writeln!(out, "# {} #{} — {}", i.repo, i.number, i.title);
    out.push('\n');

    let mut meta: Vec<String> = Vec::new();
    if !i.state.is_empty() {
        meta.push(format!("**{}**", i.state.to_lowercase()));
    }
    if !i.created_at.is_empty() {
        meta.push(format!("opened {}", i.created_at));
    }
    if !i.labels.is_empty() {
        meta.push(
            i.labels
                .iter()
                .map(|l| format!("`{l}`"))
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    if !meta.is_empty() {
        let _ = writeln!(out, "{}\n", meta.join(" · "));
    }
    if !i.url.is_empty() {
        let _ = writeln!(out, "<{}>\n", i.url);
    }
    if !study.pin.commit.is_empty() {
        let s: String = study.pin.commit.chars().take(9).collect();
        let _ = writeln!(
            out,
            "> Anchors resolved against `{s}`. Run `vis-oss validate` to check they still hold.\n"
        );
    }
}

fn problem(out: &mut String, study: &Study) {
    section(out, "The problem", |o| {
        for (label, value) in [
            ("What it is", study.summary.as_deref()),
            ("Today", study.current_behavior.as_deref()),
            ("Wanted", study.desired_behavior.as_deref()),
        ] {
            if let Some(v) = value.map(str::trim).filter(|s| !s.is_empty()) {
                let _ = writeln!(o, "**{label}.** {v}\n");
            }
        }
    });
}

fn code_map(out: &mut String, study: &Study) {
    if !study.entry_points.is_empty() || !study.prior_art.is_empty() {
        let _ = writeln!(out, "## Code map\n");
        for (title, anchors) in [
            ("Start here", &study.entry_points),
            ("Prior art", &study.prior_art),
        ] {
            if anchors.is_empty() {
                continue;
            }
            let _ = writeln!(out, "### {title}\n");
            let _ = writeln!(out, "| Location | Why it matters |");
            let _ = writeln!(out, "|---|---|");
            for a in anchors {
                let at = a
                    .line
                    .map_or_else(|| a.path.clone(), |l| format!("{}:{l}", a.path));
                let role = a.role.as_deref().unwrap_or("").replace('|', "\\|");
                let _ = writeln!(out, "| `{at}` | {role} |");
            }
            out.push('\n');
        }
    }
}

fn questions(out: &mut String, study: &Study) {
    if !study.open_questions.is_empty() {
        let _ = writeln!(out, "## Open questions\n");
        for (n, q) in study.open_questions.iter().enumerate() {
            let _ = writeln!(out, "**{}. {}**\n", n + 1, q.question);
            if let Some(w) = q
                .why_it_matters
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let _ = writeln!(out, "{w}\n");
            }
            for opt in &q.options {
                let _ = writeln!(out, "- {opt}");
            }
            if !q.options.is_empty() {
                out.push('\n');
            }
            if let Some(r) = q
                .recommendation
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let _ = writeln!(out, "→ *{r}*\n");
            }
        }
    }
}

fn examples(out: &mut String, study: &Study) {
    if !study.examples.is_empty() {
        let _ = writeln!(out, "## Examples\n");
        let _ = writeln!(out, "Written in **{}** mode.\n", study.mode.label());
        for ex in &study.examples {
            let _ = writeln!(out, "### `{}` — {}\n", ex.file, Stage::label(ex.stage));
            if !ex.title.trim().is_empty() {
                let _ = writeln!(out, "{}\n", ex.title);
            }
            if let Some(s) = ex.shows.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                let _ = writeln!(out, "{s}\n");
            }
            if let Some(r) = ex.run.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                let _ = writeln!(out, "```sh\n{r}\n```\n");
            }
        }
    }
}

fn verify(out: &mut String, study: &Study) {
    if !study.verify.is_empty() {
        let _ = writeln!(out, "## Verify\n");
        let _ = writeln!(out, "```sh");
        for step in &study.verify {
            let _ = writeln!(out, "{step}");
        }
        let _ = writeln!(out, "```\n");
    }
}

fn section(out: &mut String, title: &str, body: impl FnOnce(&mut String)) {
    let mut inner = String::new();
    body(&mut inner);
    if inner.trim().is_empty() {
        return;
    }
    let _ = writeln!(out, "## {title}\n");
    out.push_str(&inner);
}
