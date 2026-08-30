//! Static, non-interactive rendering of a [`Study`] to a terminal.
//!
//! Deterministic output, no cursor control, colour only when the destination supports
//! it — the same pipe-safe contract caliper renders under.

use std::fmt::Write as _;

use owo_colors::{OwoColorize, Style};

use crate::anchor::{self, AnchorState};
use crate::study::{Anchor, Stage, Study};

const MIN_WIDTH: usize = 60;
const MAX_WIDTH: usize = 110;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Section {
    Header,
    Problem,
    Map,
    Questions,
    Examples,
    Verify,
}

impl Section {
    pub const ALL: &'static [Section] = &[
        Section::Header,
        Section::Problem,
        Section::Map,
        Section::Questions,
        Section::Examples,
        Section::Verify,
    ];
}

#[derive(Debug, Clone)]
pub struct RenderOpts {
    pub sections: Vec<Section>,
    pub width: usize,
    pub color: bool,
    /// Re-resolve anchors against the working tree and annotate drift inline.
    pub check_anchors: bool,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            sections: Section::ALL.to_vec(),
            width: 100,
            color: false,
            check_anchors: true,
        }
    }
}

/// Terminal width clamped to a readable range, or `MAX_WIDTH` when not a terminal.
pub fn detect_width() -> usize {
    terminal_size::terminal_size()
        .map_or(MAX_WIDTH, |(w, _)| usize::from(w.0))
        .clamp(MIN_WIDTH, MAX_WIDTH)
}

struct Ctx {
    width: usize,
    color: bool,
}

impl Ctx {
    fn paint(&self, text: &str, style: Style) -> String {
        if self.color {
            text.style(style).to_string()
        } else {
            text.to_string()
        }
    }
    fn dim(&self, text: &str) -> String {
        self.paint(text, Style::new().dimmed())
    }
    fn bold(&self, text: &str) -> String {
        self.paint(text, Style::new().bold())
    }
    fn wrap(&self, text: &str, indent: &str) -> String {
        textwrap::fill(
            text,
            textwrap::Options::new(self.width.saturating_sub(indent.len()))
                .initial_indent(indent)
                .subsequent_indent(indent),
        )
    }
}

pub fn render(study: &Study, opts: &RenderOpts) -> String {
    let ctx = Ctx {
        width: opts.width,
        color: opts.color,
    };
    let mut out = String::new();
    for (i, section) in opts.sections.iter().enumerate() {
        let before = out.len();
        match section {
            Section::Header => header(&mut out, study, &ctx),
            Section::Problem => problem(&mut out, study, &ctx),
            Section::Map => map(&mut out, study, &ctx, opts.check_anchors),
            Section::Questions => questions(&mut out, study, &ctx),
            Section::Examples => examples(&mut out, study, &ctx),
            Section::Verify => verify(&mut out, study, &ctx),
        }
        if out.len() > before && i + 1 < opts.sections.len() {
            out.push('\n');
        }
    }
    out
}

fn rule(out: &mut String, ctx: &Ctx, label: &str) {
    let dashes = ctx.width.saturating_sub(label.len() + 4);
    let _ = writeln!(
        out,
        "{}",
        ctx.dim(&format!("── {label} {}", "─".repeat(dashes)))
    );
    out.push('\n');
}

fn header(out: &mut String, study: &Study, ctx: &Ctx) {
    let i = &study.issue;
    let _ = writeln!(
        out,
        "{}",
        ctx.bold(&format!("{} #{} {}", i.repo, i.number, i.title))
    );
    let mut meta = Vec::new();
    if !i.state.is_empty() {
        meta.push(i.state.to_lowercase());
    }
    if !i.created_at.is_empty() {
        meta.push(format!("opened {}", i.created_at));
    }
    if !i.labels.is_empty() {
        meta.push(i.labels.join(" · "));
    }
    if !meta.is_empty() {
        let _ = writeln!(out, "{}", ctx.dim(&meta.join("  ·  ")));
    }
    if !i.url.is_empty() {
        let _ = writeln!(out, "{}", ctx.dim(&i.url));
    }
    if !study.pin.commit.is_empty() {
        let short: String = study.pin.commit.chars().take(9).collect();
        let drift = crate::git::head_commit(std::path::Path::new(&study.pin.root))
            .filter(|head| !head.starts_with(&short))
            .map(|head| {
                let h: String = head.chars().take(9).collect();
                format!("  ⚠ checkout now at {h}")
            })
            .unwrap_or_default();
        let _ = writeln!(out, "{}", ctx.dim(&format!("pinned to {short}{drift}")));
    }
    let _ = writeln!(
        out,
        "{}",
        ctx.dim(&format!("examples are in {} mode", study.mode.label()))
    );
    out.push('\n');
}

fn problem(out: &mut String, study: &Study, ctx: &Ctx) {
    let fields = [
        ("What it is", study.summary.as_deref()),
        ("Today", study.current_behavior.as_deref()),
        ("Wanted", study.desired_behavior.as_deref()),
    ];
    if fields
        .iter()
        .all(|(_, v)| v.is_none_or(|s| s.trim().is_empty()))
    {
        return;
    }
    rule(out, ctx, "PROBLEM");
    for (label, value) in fields {
        let Some(value) = value.map(str::trim).filter(|s| !s.is_empty()) else {
            continue;
        };
        let _ = writeln!(out, "  {}", ctx.bold(label));
        let _ = writeln!(out, "{}", ctx.wrap(value, "    "));
        out.push('\n');
    }
}

fn map(out: &mut String, study: &Study, ctx: &Ctx, check: bool) {
    if study.entry_points.is_empty() && study.prior_art.is_empty() {
        return;
    }
    rule(out, ctx, "CODE MAP");
    let root = std::path::Path::new(&study.pin.root);
    let section = |title: &str, anchors: &[Anchor], out: &mut String| {
        if anchors.is_empty() {
            return;
        }
        let _ = writeln!(out, "  {}", ctx.bold(title));
        for a in anchors {
            let at = a
                .line
                .map_or_else(|| a.path.clone(), |l| format!("{}:{l}", a.path));
            let state = if check && !study.pin.root.is_empty() {
                anchor::check(root, a)
            } else {
                AnchorState::Unverifiable
            };
            let flag = match &state {
                AnchorState::Ok | AnchorState::Unverifiable => String::new(),
                other => ctx.paint(&format!("  ⚠ {}", other.describe()), Style::new().yellow()),
            };
            let _ = writeln!(out, "    {}{}", ctx.bold(&at), flag);
            if let Some(role) = a.role.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                let _ = writeln!(out, "{}", ctx.wrap(role, "      "));
            }
        }
        out.push('\n');
    };
    section("Start here", &study.entry_points, out);
    section("Prior art", &study.prior_art, out);
}

fn questions(out: &mut String, study: &Study, ctx: &Ctx) {
    if study.open_questions.is_empty() {
        return;
    }
    rule(out, ctx, "OPEN QUESTIONS");
    for (n, q) in study.open_questions.iter().enumerate() {
        let _ = writeln!(
            out,
            "{}",
            ctx.wrap(&format!("{}. {}", n + 1, q.question), "  ")
        );
        if let Some(why) = q
            .why_it_matters
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let _ = writeln!(out, "{}", ctx.dim(&ctx.wrap(why, "     ")));
        }
        for opt in &q.options {
            let _ = writeln!(out, "{}", ctx.wrap(&format!("- {opt}"), "     "));
        }
        if let Some(rec) = q
            .recommendation
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let _ = writeln!(out, "{}", ctx.wrap(&format!("→ {rec}"), "     "));
        }
        out.push('\n');
    }
}

fn examples(out: &mut String, study: &Study, ctx: &Ctx) {
    if study.examples.is_empty() {
        return;
    }
    rule(out, ctx, "EXAMPLES");
    for ex in &study.examples {
        let tag = match ex.stage {
            Stage::Current => ctx.paint(Stage::Current.label(), Style::new().yellow()),
            Stage::Proposed => ctx.paint(Stage::Proposed.label(), Style::new().green()),
            Stage::Both => ctx.paint(Stage::Both.label(), Style::new().cyan()),
        };
        let _ = writeln!(out, "  [{}] {}", tag, ctx.bold(&ex.file));
        if !ex.title.trim().is_empty() {
            let _ = writeln!(out, "{}", ctx.wrap(&ex.title, "      "));
        }
        if let Some(shows) = ex.shows.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let _ = writeln!(out, "{}", ctx.dim(&ctx.wrap(shows, "      ")));
        }
        if let Some(run) = ex.run.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            let _ = writeln!(out, "{}", ctx.dim(&format!("      $ {run}")));
        }
        out.push('\n');
    }
}

fn verify(out: &mut String, study: &Study, ctx: &Ctx) {
    if study.verify.is_empty() {
        return;
    }
    rule(out, ctx, "VERIFY");
    for step in &study.verify {
        let _ = writeln!(out, "{}", ctx.dim(&format!("  $ {step}")));
    }
    out.push('\n');
}
