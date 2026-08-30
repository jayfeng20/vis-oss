---
description: Investigate an open-source issue and fill in a vis-oss study
---

Fill in a vis-oss study — a `CONTEXT.md` explaining one open-source issue well enough
that someone who has never seen the codebase could work on it.

Argument: a study directory created by `vis-oss <issue>`, or an issue number to create
one first.

## Procedure

1. **Read `AGENT.md` in the study directory.** It is the spec: what each section must
   contain, what makes a study good rather than merely complete, and what tutorial mode
   changes. Follow it.

2. **Create the directory if needed.** From inside a checkout of the target project:
   `vis-oss <number>`. It infers the repository from the git remotes, pins the commit,
   and warns if the checkout is behind upstream.

3. **Read the issue completely, including every comment.** Issues often contain the
   maintainer's own doubts about the obvious solution. That material is usually the most
   valuable thing on the page and belongs under *Open questions*.

4. **Trace the real execution path.** Do not guess from file names. Read every line you
   cite and verify each symbol exists at the line you claim.

5. **Hunt for prior art before designing anything.** Has this project already solved a
   structurally similar problem elsewhere? Search for the *phrasing* of the feature, not
   only its nouns. This is the highest-value section in the study.

6. **Write the examples.** At least one showing today's behaviour and one showing the
   target. Check `benchmarks/`, `test_data/` and any datagen scripts for real data
   before writing a generator.

7. **Read the finished `CONTEXT.md` as a newcomer would**, and fix what does not land.

## Rules

- **Never invent a line number, an API, or a flag.** Everything you write will be
  trusted by someone who cannot check it cheaply.
- **Do not post anything to GitHub.** Issue comments and pull requests are the user's to
  send.
- **Say what you could not determine** rather than papering over it.
