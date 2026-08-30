# The agent contract

This is the spec for filling in a vis-oss study. `vis-oss <issue>` creates the directory
and copies this file into it as `AGENTS.md`; everything after that is your job.

A study exists so that someone who has never seen a codebase can understand one issue in
it well enough to start contributing. It is a head start, not an answer. You will get
things wrong, especially on a complex issue, and the reader cannot cheaply tell which
parts. So the standard is not "sound authoritative" — it is **be checkable**: cite what
you read, and mark plainly what you inferred.

## What you are filling in

```
CONTEXT.md     the study — sections below, all of them
examples/      runnable files: what happens today, what should happen after
AGENTS.md       this file
```

`CONTEXT.md` arrives with its headings in place and the issue body already pasted in.
Fill in every section. If a section genuinely does not apply, say so in one line and
say why — an empty heading reads as unfinished work, not as a considered "none".

## The sections

**What this actually is.** Restate the issue in plain language for someone who has not
read it. Not a summary of the text — an explanation of the problem. If the issue is
badly worded or assumes context, this is where you supply what it assumes.

**What happens today.** The current behaviour, traced through real code. Name the
functions and files that produce it. This is half of the before/after pair and it must
be concrete enough to reproduce.

**What should happen.** Your reading of the behaviour the issue is asking for, stated
precisely enough to test. "A warning is shown" is not enough; "one warning per query
naming the column, and none for a small dataset or an indexed query" is. This is a
reading, not a decision — the maintainers own what the fix should be, and where the issue
is ambiguous say so here rather than resolving it silently.

**Where the code is.** A reading order, not a file listing. Three to six locations,
each as `path/to/file.rs:123` followed by why a newcomer is looking at it. Trace the
actual execution path — do not guess from file names. Read every line you cite, and
omit anything you could not verify: a confident wrong pointer is worse than no pointer.

**Prior art.** Existing code in this same project that already solves a structurally
similar problem — another index type, another backend, a sibling subsystem. This is the
highest-value section in the document, because it turns a design argument into a
precedent: "here is how this codebase already answered this question" beats any amount
of reasoning. Search for the *phrasing* of the feature, not only its nouns. If there
genuinely is none, write "none found" and say where you looked.

**Open questions.** Decisions the issue does not settle and the implementer cannot
avoid. Each needs what changes depending on the answer, and your recommendation, so the
reader can proceed without waiting. Write the recommendation as your reading and give the
reason, so it can be argued with: "the sibling feature chose bytes, so bytes" invites a
check, while "use bytes" asks to be obeyed. *"What should the threshold be?"* is not an open
question. *"Time or bytes? Time is what users feel, but makes behaviour
machine-dependent and the test flaky; the sibling feature chose bytes"* is. Do not
manufacture these — if the issue is genuinely unambiguous, say so.

**Examples.** What is in `examples/`, and the exact command to run each.

**How to verify.** The build, test and lint commands for the surfaces this touches,
taken from the project's own docs rather than assumed.

**What I could not verify.** Every claim above that rests on inference rather than
something you read, plus code you looked for and could not find, plus questions worth
putting to the maintainers. This section is what makes the rest of the study safe to
trust, because it bounds it. Leaving it empty asserts that everything above is solid, so
leave it empty only when that is true.

## Examples must show both sides

At least one file showing today's behaviour and one showing the target. That pair is
the deliverable — a reader cannot judge whether a change is worth making from one side
alone.

Every example needs a `run` command that works from a stated directory, and it must be
one you have reason to believe works. An example nobody can execute is prose with a
`.py` extension.

Prefer the project's own data over synthetic data. Check `benchmarks/`, `test_data/`,
and any datagen scripts before writing a generator — using what the maintainers already
use makes results comparable to theirs and costs you nothing.

## Tutorial mode versus solution mode

`init` records which one at the top of `CONTEXT.md`.

**Tutorial** is the default. Examples are comments and `TODO`s that direct the reader
precisely — name the API, the file, the shape of the assertion — without writing the
code for them. The goal is that they come away able to work in the codebase, which does
not happen if they are handed a script. Being vague is not the same as being a tutorial:
"figure out how to time this" helps nobody. "Run each query three times and take the
median; the first call pays for opening files and warming page cache" is a tutorial.

**Solution** mode writes finished, runnable code.

## Ground rules

- **Never invent a line number, an API, or a flag.** Read the file. Verify the symbol
  exists. Everything you write will be trusted by someone who cannot check it cheaply.
- **Record the commit you worked against** — it is already at the top of `CONTEXT.md`.
  If you pull mid-investigation, re-verify your references before finishing.
- **Do not post anything to GitHub.** You investigate and write local files. Issue
  comments and pull requests are the user's to send.
- **Say what you could not determine**, in the section that exists for it. A study that
  admits an unknown is more useful than one that papers over it.
- **Do not smooth over a confusing codebase.** If something took you three reads to
  follow, say so — that is exactly what the next person needs warning about.
