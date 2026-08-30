# The agent contract

This is the normative spec for anything that produces a vis-oss study — today the
`/vis-oss` command, tomorrow whatever replaces it. `vis-oss schema` emits the
machine-readable version; this document explains the parts a schema cannot express.

## The shape

One directory per issue, filed outside the project at
`~/vis-oss/<owner>/<name>/<issue>/`. `vis-oss init` creates it; the agent fills it in.

```
~/vis-oss/lance-format/lance/804/
  study.json     the contract — everything else is derived from it
  examples/      runnable files: what happens today, what should happen after
  README.md      what this directory is
```

Studies live outside the repository on purpose, so one can never be committed into the
pull request the contributor is preparing. Anchor paths are therefore always relative
to `pin.root`, never to the study directory.

Write files. Never pass a study as a shell argument — agent-generated shell with a
multi-kilobyte quoted JSON payload is a reliability problem, not an interface.

## What is required

Only what is needed to identify the issue at all: `issue.repo` and `issue.number`.
Everything else has a default, and a thin study renders with gaps rather than failing.
Losing a whole investigation to one malformed field is the worst possible outcome.

Quality is enforced separately by `vis-oss validate`, which is where strictness lives.

## Anchors: how a study points at code

Most of a study's value is in telling the reader *where to look*. An anchor is one such
pointer, and it carries a `symbol` as well as a `line`:

```json
{
  "path": "rust/lance/src/dataset/scanner.rs",
  "line": 5305,
  "symbol": "// No index found. use flat search.",
  "role": "The fork between the ANN path and brute force."
}
```

`vis-oss validate` re-resolves each one and reports `moved to line 5412` instead of
leaving the reader staring at the wrong code. **An anchor without a symbol is
unverifiable and vis-oss will say so.** Pick a symbol that is distinctive and unlikely
to be reformatted: a signature, a constant name, a log message. Not `}`.

## entry_points versus prior_art

This split is the one thing to get right.

**`entry_points`** is a reading order. Where should someone who has never seen this
codebase start, and in what sequence, to understand the issue? Three to six anchors.
Each `role` answers "why am I looking at this file".

**`prior_art`** is existing code that already solves a similar problem. This is the
highest-value thing a study can contain, because it converts a design argument into a
precedent — "here is how this codebase already answered this question" beats any
amount of reasoning from first principles. Look hard for it before concluding there is
none, and say so explicitly when there genuinely is none.

## Examples must show both sides

Every study needs at least one `current` example and one `proposed` example. That pair
*is* the deliverable: "here is what happens today, here is what should happen after".
A study with only one side does not let a reader judge whether the change is worth
making.

`mode` decides what those files contain:

| mode | examples contain |
|---|---|
| `tutorial` | comments and `TODO`s directing the reader to write the code |
| `solution` | finished, runnable code |

In `tutorial` mode, do not write the implementation. Direct the reader precisely —
name the API, the file, the shape of the assertion — and let them type it. The point is
that they learn the codebase, not that they obtain a script.

Every example needs a `run` command that works from a stated directory. An example
nobody can execute is prose in a file with a `.py` extension.

## Open questions must have consequences

An `open_question` is a decision the issue does not settle and the implementer cannot
avoid. It needs `why_it_matters` — what actually changes depending on the answer — and
a `recommendation`, so the reader can proceed without waiting.

*"What should the threshold be?"* is not an open question. *"Should the threshold
measure elapsed time or bytes scanned? Time is what users feel, but it makes behavior
machine-dependent and tests flaky; the FTS precedent chose bytes."* is.

Do not manufacture these. A study with no genuine open questions should have an empty
list, and that is a useful signal in itself.

## Pin honestly

`pin.commit` must be the commit the anchors were actually resolved against. If you
investigated, then pulled, re-verify the anchors before writing the pin. A pin that
claims freshness the anchors do not have is worse than no pin.
