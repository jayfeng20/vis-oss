---
description: Investigate an open-source issue and fill in a vis-oss study
---

Investigate an issue and produce a **vis-oss study** — a durable, drift-checkable
explanation of what the issue is, where the relevant code lives, and what a reader must
decide before implementing.

Argument: a study directory created by `vis-oss init`, or an issue number to init first.

## Procedure

1. **Read the contract.** `docs/agent-contract.md` in the vis-oss repo, or
   `vis-oss schema`. It defines every field and, more importantly, what makes a study
   good rather than merely valid.

2. **Init if needed.** From inside a checkout of the target project:
   `vis-oss init <number>`. It infers the repo from git remotes (preferring `upstream`),
   and pins the commit.

3. **Read the issue completely**, including every comment. Issues often contain the
   maintainer's own doubts about the obvious solution — that material belongs in
   `open_questions`, and it is usually the most valuable thing on the page.

4. **Find the code.** Trace the actual execution path, do not guess from file names.
   Fill `entry_points` as a *reading order*: where a newcomer starts, and why each file
   matters.

5. **Hunt for prior art before designing anything.** Has this codebase already solved a
   structurally identical problem elsewhere — a different index type, a different
   backend, a sibling subsystem? Search for the *phrasing* of the feature, not only its
   nouns. Prior art converts a design argument into a precedent and is the single
   highest-value thing a study can carry. If there genuinely is none, say so explicitly
   rather than leaving the list empty and ambiguous.

6. **Verify every anchor symbol is distinctive.** Before writing an anchor, grep its
   symbol and confirm it appears once. `fn execute(` in a file with six execute methods
   will resolve to the wrong one and silently mislead. Prefer a signature with its
   parameters, a constant name, or a log message.

7. **Write the examples.** At least one `current` and one `proposed`. Honour `mode`:
   in `tutorial` mode write comments and `TODO`s that direct the reader precisely
   without writing the implementation for them. Every example needs a `run` command
   that works from a stated directory. Prefer a real dataset over synthetic data when
   the project ships one — check `benchmarks/` and any datagen scripts first.

8. **Validate, and fix what it names.** `vis-oss validate <dir>` must pass with zero
   errors before you report done. Then `vis-oss render <dir>` and read it as a
   newcomer would.

## Rules

- **Never invent a line number.** Read the file. If an anchor cannot be verified, omit
  it rather than guessing — a confident wrong pointer is worse than no pointer.
- **Do not post anything to GitHub.** This command investigates and writes local files.
  Issue comments and PRs are the user's to send.
- **Report what you could not determine.** A study that admits an unknown is more useful
  than one that papers over it.
- **Correct earlier files if the code moved mid-investigation.** If you pull during the
  work, re-verify anchors and re-pin before finishing.
