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

**Examples.** What is in `examples/`, what each demonstrates, and the exact command to
run it.

**How to verify.** The build, test and lint commands for the surfaces this touches,
taken from the project's own docs rather than assumed.

**What I could not verify.** Every claim above that rests on inference rather than
something you read, plus code you looked for and could not find, plus questions worth
putting to the maintainers. This section is what makes the rest of the study safe to
trust, because it bounds it. Leaving it empty asserts that everything above is solid, so
leave it empty only when that is true.

**After — what the issue asks for.** Last, because it is the only section describing code
that does not exist yet. What is different once this is fixed, seen from outside: an API
that changes shape, or the same calls behaving differently — a warning appears, a write
gets faster, a scan stops reading a column. Use the same `file:line` annotations as the
examples. State it precisely enough to test: "a warning is shown" is not enough, "one
warning per query naming the column, and none for a small dataset or an indexed query" is.
This is your reading of the issue, not a decision — the maintainers own the real shape.

## The examples are the deliverable

Everything else in `CONTEXT.md` frames them. An example is a **runnable probe of what the
code does today**, annotated so that reading it teaches the code path — not just the API.

Write them in the language the behaviour is *observed* in, which is not always the
language the fix lands in. For a Rust library with Python bindings, a user meets the slow
query in Python; the example is Python, and its annotations point down into the Rust.

### Annotate the path into the code

Every call that matters carries a comment naming where the thing it reaches is
**declared** — the function, struct, or endpoint definition — so the reader can open the
file and read it. Not "this is slow", but "this is slow, and here is the function that
makes it slow".

```python
ds.to_table(nearest={"column": "vector", "q": q, "k": 10})
# -> Scanner::vector_search, rust/lance/src/dataset/scanner.rs:5075
#    loads the index list, finds nothing covering this column, so at :5305
#    it takes the flat branch
# -> KNNVectorDistanceExec, declared rust/lance/src/io/exec/knn.rs:150
#    scores every row; execute() at :904 is where the work happens
```

Point at declarations, not only call sites: a reader who opens `knn.rs:150` finds the
struct and can read outward. Verify every line you cite, and name the symbol alongside it
so the reference survives the file moving.

These are teaching material, so hold them to production standards: real error handling, no
bare `except`, no toy shortcuts, the project's own conventions and datasets. Check
`benchmarks/`, `test_data/` and any datagen scripts before writing a generator — using
what the maintainers use makes your numbers comparable to theirs.

Every example needs a `run` command that works from a stated directory, and must actually
run once the reader has filled in whatever the tutorial level left out.

Name them in the project's own vocabulary, numbered in the order they are run:
`00_build_datasets.py` before `01_flat_search.py`, because Lance calls them datasets and
`scanner.rs` calls it flat search. Avoid words the project does not use — `corpus` is an
NLP term that a study of an IO bug cannot reuse, and `fixture` reads as test scaffolding
when these are things the reader runs and watches.

### Tutorial level

`CONTEXT.md` records the level the study was generated at.

| level | the example contains |
|---|---|
| `full` | comments and `TODO`s. The reader writes every line. |
| `partial` | scaffolding, with the parts worth thinking about left as stubs — `todo!()`, `raise NotImplementedError`, a body that returns nothing yet. |
| `none` | complete, runnable code. |

At `full` and `partial`, being vague is not the same as being a tutorial. "Figure out how
to time this" helps nobody. "Run each query three times and take the median — the first
call pays for opening files and warming page cache" is a tutorial. Name the API, the file,
and the shape of the assertion; withhold only the typing.

### Every example ends with an AFTER block

A commented block showing what would differ when this same file runs once the issue is
fixed. A description, never a patch.

```python
# ---- AFTER ----
# Same script, unchanged. stderr gains one line:
#   WARN lance: brute-force vector search over 500000 rows on column "vector";
#   consider creating a vector index
# The small corpus above stays silent, which is the part worth testing.
```

If the fix changes an API rather than a behaviour, show the call site both ways.

## Never write the fix

You are explaining an issue, not resolving it. Do not modify the project's source, and do
not put a patch in the study. The reader decides what the change should be; your job is
that they can make that decision quickly and from real code.

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
