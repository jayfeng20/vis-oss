# The agent contract

This is the spec for creating a vis-oss study. `vis-oss <issue>` creates the directory
and copies this file into it as `AGENTS.md`; everything after that is your job.

A study exists so that someone who is unfamiliar with the codebase can understand one issue in
it well enough to start contributing. The study is like a textbook that educates the reader on
the necessary contexts around the issue with code or comments depending on tutorial level set
by reader.

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

**What this actually is.** The problem in plain language, for someone who has not read the
issue — an explanation, not a summary. If the issue assumes context, supply it here.

**What happens today.** The current behaviour, traced through real code, naming the
functions and files that produce it. Concrete enough to reproduce.

**Where the code is.** A reading order, not a file listing: three to six `path/file.rs:123`
locations, each with why a newcomer is looking at it. Trace the execution path rather than
guessing from file names. Read every line you cite and omit what you could not verify — a
confident wrong pointer is worse than none.

**Prior art.** Code in this project that already solves a structurally similar problem —
another index type, another backend, a sibling subsystem. Usually the most valuable section,
because it turns a design argument into a precedent. Search for the *phrasing* of the
feature, not only its nouns. At most two, closest first. If there is none, say so and say
where you looked.

**Open questions.** Decisions the issue leaves open that the implementer cannot avoid. Each
needs what turns on the answer and your recommendation, given as a reading with its reason
so it can be argued with. *"What should the threshold be?"* is not one. *"Time or bytes?
Time is what users feel but makes the test flaky; the sibling feature chose bytes"* is. Do
not manufacture them.

**Examples.** What each file demonstrates, and the exact command to run it.

**How to verify.** Build, test and lint commands for the surfaces this touches, from the
project's own docs.

**What I could not verify.** Claims above resting on inference rather than something you
read, code you looked for and could not find, and questions for the maintainers. This is
what makes the rest safe to trust, because it bounds it. Leave it empty only when that is
honestly true.

**After — what the issue asks for.** Last, because it is the only section describing code
that does not exist. What differs once this is fixed, seen from outside, with the same
`file:line` annotations as the examples. Precise enough to test: not "a warning is shown"
but "one warning per query naming the column, and none for a small dataset or an indexed
query". Your reading, not a decision — the maintainers own the real shape.

## Shapes of issue

Most issues are one of four kinds, and the kind decides what the study must show. Work out
which you have before writing anything — the tracker's own labels usually say.

**Feature request** — it does not exist yet. Show three things: *before*, the current
behaviour with the feature absent, run and measured; *prior art*, at most two places in
this project that already solved a structurally similar problem, closest first; and
*after*, described only. There is no probe for the target behaviour, because the code
producing it does not exist.

**Bug** — something is wrong. The study is a reproduction: the setup and the call that
produce the wrong result, and an assertion of what is wrong. At `full` that file fails
today and passes once the bug is fixed, which makes it the acceptance check. A similar past
fix in the history is worth more here than a similar feature.

**Performance** — something is too slow. As for a feature request, except the measurement
is the deliverable rather than the framing: time the slow path, time whatever the project
already offers that is faster, at sizes far enough apart to show the trend.

**Documentation** — something is unexplained or wrong on the page. Often there is nothing
to run; say so in *Examples* rather than inventing a probe, and spend the study on where
the documented behaviour actually lives, so the page can be checked against the code.

An issue can be two of these. Pick the one the reporter is asking for: #804 asks for a
warning that does not exist, so it is a feature request, even though what it warns about
is a performance problem.

## The examples are the deliverable

Everything else in `CONTEXT.md` frames them. An example is a **runnable probe of what the
code does today**, annotated so that reading it teaches the code path, not just the API.

Write each probe **twice, from two vantage points**:

| written in | what it gives the reader |
|---|---|
| the language the behaviour is **observed** in | what a user actually experiences, which is why the issue was filed |
| the language the behaviour is **implemented** in | a probe that runs against their own tree with nothing rebuilt in between, so re-running it after the change is an acceptance check |

For a Rust library with Python bindings, a user meets the slow query in Python — that
probe is Python, annotated downward into the Rust. The second is Rust, calling the same
path directly.

*Implemented*, not *where the fix will land*: the maintainers own where it lands, and this
study does not get to assume it. And the implementation language always reaches the
behaviour, because a binding is built on the core's public API and cannot expose what the
core does not. If you cannot reach it from there, you have misread where it lives — find
the entry point the binding itself calls rather than reaching into internals.

When both are the same language — a binding-layer bug, or a project with no bindings —
there is one file. Say so in one line in *Examples*, so it reads as decided.

Both share a number, separated by extension: `01_flat_search.py` and `01_flat_search.rs`
are one probe from two sides. One setup file serves both, in whichever language is cheaper
for the reader to run.

### You write the code; you do not execute it

**Nothing here is run, and nothing is compiled.** A first execution costs whatever the
project charges to build — for a Rust workspace, its whole dependency graph — which has
nothing to do with how small your example is, and is work the reader does anyway on their
own tree.

So the check is reading: open the file, find the symbol, confirm the signature, and name
the paths you read in the provenance line.

Say there that the file is a draft. An uncompiled Rust example will sometimes not compile,
and the reader's first `cargo check` is where that surfaces — a minute if they were told,
an afternoon of doubt if they were not.

Do not go further: no running an example to see whether it works, no throwaway scripts
hunting for a size or a parameter that makes a measurement come out, nothing the project
would call a benchmark.

**Those limits are on what you execute, not on what you write.** The reader typed the
command, so their file may build indexes, write a large dataset and time queries when the
issue calls for it. Reading "do not train an index" as "the example must not train one"
turns a budget for you into a hole in the deliverable.

Size still matters, because someone else pays for it: write the **smallest input that
answers the question**. Usually that is tiny — a thousand rows takes the same branch as a
million. When the issue is *about* the difference between those two, it is not.

### A long run is a conversation, not a file

Some questions need executing: does this reproduce, how slow is slow. Finish the study,
then say in your reply — not in the study — what running it would settle, roughly what it
would cost, and that you will do it if asked. Never start one unasked. The user knows what
their afternoon is worth; you do not.

### Derive the probe from the issue's own words

The issue names the conditions it cares about, and those conditions *are* the probe. This
is mechanical enough to do deliberately rather than by feel:

1. **List what the issue distinguishes.** Every clause saying behaviour should differ
   between two situations is an axis.
2. **Cross the axes.** The cases that fall out are rows of one probe, not a file each.
3. **Measure what the issue measures.** If it names a quantity — a runtime, a row count, a
   byte total — that quantity is the probe's output.
4. **Stop there.** Everything else you learned belongs in the prose.

Worked, on the issue these examples come from:

> *"show warnings if the index was not built for the column"* — index present or absent is
> one axis.
> *"doing so blindly makes it really annoying for smaller datasets that really don't need
> an ANN index"* — small or large is the second.
> *"if the runtime exceed a certain threshold to print the warning"* — runtime is what to
> measure.

Two axes crossed is four cases — small indexed, small flat, large indexed, large flat —
each timed, printed as four rows of one table. In one place the reader sees both the case
that ought to warn and the case that must stay silent, and that pair is the entire
difficulty the issue is about. Note what follows: the probe builds indexes and writes a
genuinely large dataset, because the axes demand it. Sizes are chosen so the difference is
visible, not so the file is cheap — the reader is the one running it.

What does not belong: that the same query also takes the flat path when the requested
metric mismatches. True, found while reading, and worth one sentence in *What happens
today*. Not a second probe — that turns one question into three and leaves the reader
working out which one the issue was.

The reverse holds too. If the issue is about a wrong result, do not time anything; timing
would be the tour. Match the probe to the claim being made.

An example that needs a hundred lines of harness before it reaches the project's API is
teaching the harness. If capturing output or driving the API honestly takes that much,
that is worth a sentence in *What I could not verify* — the next contributor will hit it
too.

### Which files to create

Two roles. Names are yours to choose — pick what the project would call the thing — but
every file must be recognisably one of these, numbered in the order it is run.

| role | prefix | how many |
|---|---|---|
| **setup** — whatever must exist before anything can be observed: data written, a server started, a state reached | `00_` | at most one, and none if the project already ships what you need |
| **probe** — the issue's question made runnable against today's code | `01_`, `02_`, … | as many as the issue asks separate questions, which is usually one. The cases it distinguishes are rows inside a probe, not files beside it; one probe in two languages shares one number |

Check `test_data/`, `benchmarks/` and any datagen scripts before writing setup: reusing
what the maintainers already ship makes your numbers comparable to theirs, and costs you
nothing to build. Split probes when the behaviours differ — a slow read and a wrong result are two files; timing and plan
inspection of one query are one file.

**There is no file for the target behaviour.** It cannot run, because the code producing
it does not exist. A file like `02_proposed.py` is a patch wearing an example's clothes,
and writing it is writing the fix. What changes belongs in the `AFTER` block of the probe
it affects, and in full in the study's last section.

### What every file contains

In order:

1. **A docstring**, stating in this order: what this file demonstrates, in one sentence;
   the exact command to run it and the directory to run it from; and the tutorial level.
   The lowest-numbered file in each language also says how to get an environment where
   that command works — see below.
2. **A provenance line** — that it was neither run nor compiled, and the paths you read to
   confirm the APIs it calls, with the commit you read them at. Keep it short. It exists so
   the reader knows which parts to distrust.
3. **The body**, annotated as below.
4. **An `AFTER` block** — probes only. Setup files do not need one; nothing about
   building the inputs changes.

```python
"""
Times the same vector query with and without an index, and shows that the slow path
says nothing.

    cd ~/Coding/lance/python
    LANCE_LOG=warn uv run python <this file>

Tutorial level: partial — the timing helper is yours to write.

Not run, not compiled. APIs read against lance f603c551:
write_dataset and create_index in python/python/lance/dataset.py, LANCE_LOG handling
in python/src/lib.rs:169. The branch it takes comes from reading Scanner::vector_search;
the timings are whatever your machine does.
"""
```

Real paths, not a claim that you were careful: "APIs verified" is worth nothing to a
reader who cannot see what you opened. Never leave provenance out — an example without one
is indistinguishable from one written from memory.

**If you assert, assert what is structural.** Assertions are not required — nobody has
run them, and a file that teaches the code path has done its job. But one that is there
runs on a machine you have never seen, so it must hold on any of them. Which plan is
chosen, which operator appears in it, whether a warning reached stderr: those hold.
Durations, ratios and recall move with the hardware and the row count, so print them and
let the reader judge. `assert flat_ms > indexed_ms * 10` is not a check, it is a flake
posted to a stranger.

### Making them runnable

**Python, or any interpreted binding:** nothing for *you* to create. See *Getting an
environment* below.

**Rust:** one Cargo project per *project studied*, not per issue, one directory above the
issue. A per-issue project means a separate `target/`, so the whole dependency graph is
rebuilt for every study; sharing one builds it once.

```
~/vis-oss/lance/lance-format/          <- the Cargo project lives here
  Cargo.toml
  target/                              <- built once, reused by every issue
  804/
    CONTEXT.md   AGENTS.md   00_build_datasets.py
    01_flat_search.py        01_flat_search.rs      <- one probe, both languages
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

If that `Cargo.toml` already exists, **add a `[[bin]]`** rather than creating another.
Name it `<issue>_<file>` so issues cannot collide, and give the run command as `cargo run
--bin 804_flat_search` from the directory holding the manifest.

You write the manifest; you do not build it. Say in the docstring that the reader's first
build is slow — it compiles the project's whole dependency graph — and that the file is
uncompiled, so a first error is expected rather than alarming.

Do not reach for `cargo -Zscript`: it is nightly-only.

#### Getting an environment

A run command only works inside an environment, and the reader does not have one yet. So
the lowest-numbered file **in each language** opens with what it takes to get there — once,
at the top, not repeated in every file.

Take it from the project's own docs. The answer differs per project, and guessing it wrong
strands the reader on step one:

- **Python:** whatever that project prescribes. A `uv` project usually wants `uv sync` or
  `make install` once and then `uv run python <file>` with **no activation at all** —
  telling that reader to `source .venv/bin/activate` is simply wrong, and some projects say
  in as many words not to rely on an activated environment. A plain venv or poetry project
  does want the activation line. Read `python/AGENTS.md`, `CONTRIBUTING.md` or the README
  and copy what is actually there.
- **Rust:** the directory holding the shared study `Cargo.toml` — one level up from the
  study directory, and not the project's own checkout — plus the `cargo run --bin` line.

If a native extension has to be compiled before anything can import it, that is part of the
environment and belongs in this block. It is usually the longest step, and discovering it
after a failed import is how a reader loses an hour.

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

These are teaching material, not production code: skip the defensive error handling and
the configurability, and spend the space on the annotations instead. Follow the project's
own conventions, and avoid words it does not use — `corpus` is an NLP term a study of an IO bug cannot reuse, and `fixture`
reads as test scaffolding when these are files the reader runs and watches.

### Tutorial level

`CONTEXT.md` records the level, and it applies to every file. It defaults to `partial`,
because nothing here is executed and `full` would promise working code on no evidence.

| level | the file contains |
|---|---|
| `full` | complete code, written to run — but never executed, so the claim rests on reading. |
| `partial` | scaffolding, with the parts worth thinking about left as stubs. |
| `none` | comments and `TODO`s. The reader writes every line. |

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

At `partial`, finish everything that reaches the code path — the imports, the setup, the
call itself, the annotations — and stub only what the reader gains from working out. The
file should be one or two stubs away from running, so filling them is a short task with a
visible result rather than a rewrite.

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
  exists. This is the only check standing between you and the reader — no run will catch
  what you got wrong — and everything you write will be trusted by someone who cannot
  check it cheaply.
- **Record the commit you worked against** — it is already at the top of `CONTEXT.md`.
  If you pull mid-investigation, re-verify your references before finishing.
- **Do not post anything to GitHub.** You investigate and write local files. Issue
  comments and pull requests are the user's to send.
- **Say what you could not determine**, in the section that exists for it. A study that
  admits an unknown is more useful than one that papers over it.
- **Do not smooth over a confusing codebase.** If something took you three reads to
  follow, say so — that is exactly what the next person needs warning about.
