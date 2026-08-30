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

Write the probe in the language the behaviour is *observed* in, which is not always the
language the fix lands in. For a Rust library with Python bindings, a user meets the slow
query in Python; that probe is Python, and its annotations point down into the Rust.

**Then add the second language when the fix lands there and its API allows it.** The two
files do different jobs, so neither replaces the other:

| written in | what it gives the reader |
|---|---|
| the language it is observed in | what a user actually experiences, which is why the issue was filed at all |
| the language the fix lands in | a probe that compiles against their own working tree, so re-running it after the change is a working acceptance check |

Give both the same number and let the extension separate them: `01_flat_search.py` and
`01_flat_search.rs` are one probe seen from two sides, and numbering them `01` and `02`
would claim they are two behaviours.

Skip the second file when the API does not permit it — if what you need is not reachable
from outside the crate, or exists only in the binding layer. Say which in one line in the
study's *Examples* section, and do not reach into internals to manufacture it.

One setup file serves both where the data is the same on disk, which it usually is. Write
it in whichever language is cheaper for the reader to get running, and say in the other
probe that it depends on it.

### You write the code; you do not execute it

**Nothing here is run, and nothing is compiled.** Executing a file means paying whatever
the project charges for a first execution, and compiling a Rust one against a large
workspace means building its whole dependency graph — for a project pulling in Arrow and
DataFusion that is around nine hundred crates before the compiler reaches your thirty
lines. Neither cost has anything to do with how small your example is, and both are work
the reader does anyway on their own tree.

So the check is reading: open the file, find the symbol, confirm the signature. Name the
paths you read in the provenance line, so the reader can see what was confirmed and what
was taken on trust.

What you owe instead is a file that matches its **tutorial level** — complete at `none`,
scaffolded with stubs at `partial`, `TODO`s at `full`. The level is the contract with the
reader about how finished this is, and it is the honest lever you do have. It is also what
bounds the risk: `partial`, the default, ships the part you can stand behind from reading
and leaves the rest marked, so an uncompiled file is not pretending to be a working one.

Be straight about it in the provenance line. A Rust example nobody compiled will sometimes
not compile — a trait bound, a generic parameter, an async runtime detail — and the
reader's first `cargo check` is where that surfaces. Saying so costs a sentence and saves
them the assumption that it built clean.

Do not go further. No running an example to see whether it works, no throwaway scripts
hunting for a data size or a parameter that makes a measurement come out, and nothing the
project would itself call a benchmark — a trained index, a large generated dataset, a full
load of real data.

Size still matters, because someone else pays for it: write the **smallest input that
reaches the code path**. A query over a thousand rows takes the same branch as one over a
million, and the branch is what is being taught.

### A long run is a conversation, not a file

Some questions do need executing — does this reproduce, and how slow is slow. Finish the
study first. Then, in your reply to the user and not in the study, say in one line what
running it would settle, roughly what that would cost, and that you will do it if asked.

That offer is the whole mechanism. The user knows what their machine and their afternoon
are worth; you do not. A study that arrives in minutes and offers to go further beats one
that spent an hour deciding for them.

### Which files to create

Two roles. Names are yours to choose — pick what the project would call the thing — but
every file must be recognisably one of these, numbered in the order it is run.

| role | prefix | how many |
|---|---|---|
| **setup** — whatever must exist before anything can be observed: data written, a server started, a state reached | `00_` | at most one, and none if the project already ships what you need |
| **probe** — one behaviour worth watching on its own, running against today's code | `01_`, `02_`, … | as many as there are genuinely separate behaviours — the same behaviour in two languages shares one number |

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

**Python, or any interpreted binding:** nothing for *you* to create — the project already
has an environment, and the reader's job is to enter it. See *Getting an environment* below.

**Rust:** one Cargo project per *project studied*, not per issue, at the repository level
of the study root — one directory above the issue:

```
~/vis-oss/lance/lance-format/          <- the Cargo project lives here
  Cargo.toml
  target/                              <- built once, reused by every issue
  804/
    CONTEXT.md   AGENTS.md   00_build_datasets.py
    01_flat_search.py        01_flat_search.rs      <- one probe, both languages
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

You write that `Cargo.toml` and the `[[bin]]` entry; you do not build them. Say in the
docstring that the reader's first build is slow — it is compiling the project's whole
dependency graph — so a long compile is not read as a hang, and say that the file is
uncompiled so a first error is expected rather than alarming.
Do not reach for `cargo -Zscript`: it is nightly-only. If what you need to observe is not
reachable from outside the crate, that is a signal the example belongs in the observation
language instead, not that you should reach into internals.

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

`CONTEXT.md` records the level the study was generated at. It applies to every file, and
defaults to `partial`. Since nobody has executed these, `none` would promise complete
working code on no evidence, and that promise is what the reader would act on. Write the
scaffolding you can stand behind from reading; leave the rest as stubs.

| level | the file contains |
|---|---|
| `full` | comments and `TODO`s. The reader writes every line. |
| `partial` | scaffolding, with the parts worth thinking about left as stubs. |
| `none` | complete code, written to run — but still never executed, so the claim rests on reading alone. |

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
