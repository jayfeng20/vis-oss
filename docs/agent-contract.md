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
AGENTS.md      this file
00_*, 01_*     the examples, alongside the prose
```

Examples sit at the top level of the study. They are few and numbered, so a directory for
them would be one level to open for nothing. A language that needs a project to run at
all — Rust — puts that project one level *up*, shared by every issue in the same
repository; see *Making them runnable*.

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

### Small and fast, first time

An example earns its place by landing the reader in the right code path, not by
reproducing the issue's headline number. Write the **smallest input that still reaches
that path**, and hold the study to a budget: no single file over about thirty seconds,
and all of them together under two minutes on the machine you are on. The budget is run
time — a first `cargo build` against a large workspace is minutes and there is nothing to
be done about that except say so, as below.

This is cheap to obey, because scale is almost never what the reader needs. A query over
a thousand rows takes the same branch as one over a million, and the branch is what is
being taught.

The failure mode the budget exists to prevent is a hunt for a number. You write a probe,
the measurement comes out unconvincing, so you grow the data, retrain the index, add
repeats, and try again — each attempt costing minutes, none of it changing what the reader
learns, and all of it spent before the study has a single section written. Concretely:

- **Do not write throwaway scripts to find a size or a parameter that makes a result come
  out.** If your first honest run does not show what you expected, that *is* a finding —
  write it down and move on. An issue whose premise does not reproduce at small scale is
  worth knowing about.
- **Do not build anything the project would itself call a benchmark** — a trained index, a
  large generated dataset, a full load of real data. Those cost minutes, and they belong
  to the user.
- If a behaviour genuinely only appears at a scale you cannot reach inside the budget,
  keep the small file, say plainly where the claim is made that it is unmeasured, and
  offer the heavy run.

### The heavy run is a conversation, not a file

Some questions really do need scale: a query that is only slow at a million rows, a build
that only regresses on a big workspace. Finish the study without them. Then, in your reply
to the user — not in the study — say in one line what a longer run would add, roughly what
it would cost, and that you will do it if asked.

Never start one unprompted. A study that arrives in two minutes and offers a ten-minute
measurement is worth more than one that takes twelve minutes and never asked.

### Which files to create

Two roles. Names are yours to choose — pick what the project would call the thing — but
every file must be recognisably one of these, numbered in the order it is run.

| role | prefix | how many |
|---|---|---|
| **setup** — whatever must exist before anything can be observed: data written, a server started, a state reached | `00_` | at most one, and none if the project already ships what you need |
| **probe** — one behaviour worth watching on its own, running against today's code | `01_`, `02_`, … | as many as there are genuinely separate behaviours |

Check `test_data/`, `benchmarks/` and any datagen scripts before writing setup: reusing
what the maintainers already ship makes your numbers comparable to theirs, and costs you
nothing to build. Split probes when the
behaviours differ — a slow read and a wrong result are two files; timing and plan
inspection of one query are one file.

**There is no file for the target behaviour.** It cannot run, because the code producing
it does not exist. A file like `02_proposed.py` is a patch wearing an example's clothes,
and writing it is writing the fix. What changes belongs in the `AFTER` block of the probe
it affects, and in full in the study's last section.

### What every file contains

In order:

1. **A docstring**, stating in this order: what this file demonstrates, in one sentence;
   the exact command to run it and the directory to run it from; and the tutorial level.
2. **A provenance line** — whether *you* ran it, and what came out. This is the single
   most valuable line in the file, because it is what separates a checked example from a
   plausible one.
3. **The body**, annotated as below.
4. **An `AFTER` block** — probes only. Setup files do not need one; nothing about
   building the inputs changes.

```python
"""
Times the same vector query with and without an index, and shows that the slow path
says nothing.

    cd ~/Coding/lance/python
    LANCE_LOG=warn uv run python <this file>

Tutorial level: none — this file is complete.

Verified: run 2026-08-30 against lance f603c551, 20k rows x 128 dims, 3.1s total.
    flat 14.6ms · indexed 24.7ms · plan flat vs ANN · stderr empty

At this size the index is *slower* than scanning everything, so the ratio the issue
assumes is not visible here and this file does not claim it. The plan shapes are the
part that holds at any size, and they are what the probe asserts on.
"""
```

That last paragraph is not an apology. Writing down the measurement you actually got,
including when it undercuts the issue, is the whole value of having run the thing.

If you could not run it, say that instead, and say why: `Not run: needs a 6GB download.
The numbers below are predicted, not measured.` Never leave provenance out — an example
with no provenance line is indistinguishable from one that was never checked.

**Assert what is structural; print what is not.** A claim the file makes about the
project should fail loudly when it stops being true — but only a claim that holds on any
machine at any size. Which plan is chosen, which operator appears in it, whether a warning
reached stderr, what a call returns: assert those, and they defend themselves. Durations,
ratios and recall move with the hardware and the row count, so print them next to the
numbers you measured and let the reader compare. `assert flat_ms > indexed_ms * 10` is not
a check, it is a flake you handed to someone else — and tuning the input until it passes
is how an afternoon disappears.

### Making them runnable

**Python, or any interpreted binding:** nothing to set up. Give the command and the
directory, and use the project's own environment tooling — `uv run`, `poetry run`,
whatever its docs prescribe.

**Rust:** one Cargo project per *project studied*, not per issue, at the repository level
of the study root — one directory above the issue:

```
~/vis-oss/lance/lance-format/          <- the Cargo project lives here
  Cargo.toml
  target/                              <- built once, reused by every issue
  804/
    CONTEXT.md   AGENTS.md   01_flat_search.rs
  8245/
    CONTEXT.md   AGENTS.md   01_stringview.rs
```

```toml
[package]
name = "lance-studies"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
lance = { path = "/Users/you/Coding/lance/rust/lance" }   # the pinned checkout
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }

[[bin]]
name = "804_flat_search"
path = "804/01_flat_search.rs"
```

Per-issue would mean a separate `target/`, so the project's whole dependency graph gets
rebuilt for every issue you study — for a workspace the size of Lance that is hundreds of
crates and minutes each time. Sharing one project builds them once.

So: if `Cargo.toml` already exists one level up, **add a `[[bin]]` to it** rather than
creating another. Name the binary `<issue>_<file>` so two issues cannot collide, and give
the run command in the docstring as
`cargo run --bin 804_flat_search` from the directory holding `Cargo.toml`.

The contributor gets a real binary compiled against their own working tree — so once they
make the change, re-running the probe shows the `AFTER` block coming true. That is a
working acceptance check that never touches the project's source.

Say in the docstring that the first build is slow, and give the real number if you ran it.
Do not reach for `cargo -Zscript`: it is nightly-only. If what you need to observe is not
reachable from outside the crate, that is a signal the example belongs in the observation
language instead, not that you should reach into internals.

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
#    scores every row; execute() at :869 is where the work happens
```

Point at declarations, not only call sites: a reader who opens `knn.rs:150` finds the
struct and can read outward. Verify every line you cite, and name the symbol alongside it
so the reference survives the file moving.

These are teaching material, so hold them to production standards: real error handling, no
bare `except`, no toy shortcuts, the project's own conventions. Avoid words the project
does not use — `corpus` is an NLP term a study of an IO bug cannot reuse, and `fixture`
reads as test scaffolding when these are files the reader runs and watches.

### Tutorial level

`CONTEXT.md` records the level the study was generated at. It applies to every file, and defaults to `none` — a file of `TODO`s cannot be executed, so writing one forfeits the only check available to you.

| level | the file contains |
|---|---|
| `full` | comments and `TODO`s. The reader writes every line. |
| `partial` | scaffolding, with the parts worth thinking about left as stubs. |
| `none` | complete, runnable code. |

Mark a stub the way the language does, and say what goes there — never leave a silent
gap:

```rust
fn median_query_ms(ds: &Dataset, q: &[f32]) -> f64 {
    todo!("time three runs, return the median; the first pays for page-cache warmup")
}
```

```python
def median_query_ms(ds, q):
    raise NotImplementedError("time three runs, return the median; the first pays for "
                              "page-cache warmup")
```

Being vague is not the same as being a tutorial. "Figure out how to time this" helps
nobody. Name the API, the file, and the shape of the assertion; withhold only the typing.

At `full` and `partial` a file will not run as shipped, so its provenance line says what
you verified instead — that the APIs it names exist, and where you checked.

### Every behaviour probe ends with an AFTER block

A commented block showing what would differ when this same file runs once the issue is
fixed. A description, never a patch. Quote exact output rather than paraphrasing it:
"stderr gains `WARN lance: brute-force search over 500000 rows`" is testable, "a warning
is displayed" is not.

```python
# ---- AFTER ----
# Same script, unchanged. stderr gains one line:
#   WARN lance: brute-force vector search over 500000 rows on column "vector";
#   consider creating a vector index
# Once — not per batch, not per partition. The small dataset above stays silent,
# which is the part worth testing.
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
