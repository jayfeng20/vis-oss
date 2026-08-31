# The agent contract

This is the spec for filling in a vis-oss study. `vis-oss <issue>` creates the directory
and copies this file into it as `AGENTS.md`; everything after that is your job.

A study is a **lesson about one issue**, written for someone who has never seen the
codebase. Its shape is the shape of a textbook chapter: the reader starts from the
problem, watches it happen, follows it into the code one step at a time, writes a little
of the code themselves, and finishes knowing enough to hold their own opinion about the
fix. It is a head start, not an answer. You will get things wrong, especially on a
complex issue, and the reader cannot cheaply tell which parts. So the standard is not
"sound authoritative" — it is **be checkable**: cite what you read, and mark plainly what
you inferred.

## What you are filling in

```
CONTEXT.md     the prose — the lesson itself, sections below, all of them
AGENTS.md      this file
00_*, 01_*     the probes — small runnable files the walkthrough tells the reader to run
```

A **probe** is a numbered script that demonstrates what the code does today — `00_` sets
state up, `01_` onward ask the issue's questions; *The probes* below is the full spec.
They sit at the top level of the study, next to the prose: they are few and numbered, so
a directory for them would be one level to open for nothing. A language that needs a project to run at
all — Rust — puts that project one level *up*, shared by every issue in the same
repository; see *Making them runnable*.

`CONTEXT.md` arrives with its headings in place and the issue body already pasted in.
Fill in every section. If a section genuinely does not apply, say so in one line and
say why — an empty heading reads as unfinished work, not as a considered "none".

## Write a lesson, not a report

The failure mode of documents like this is the reference dump: accurate sections that
each summarise something, none of which teaches. The books working programmers actually
learn from — *The Rust Programming Language*, *Crafting Interpreters* — avoid it with a
small set of habits, all mechanical enough to demand:

- **Observation before explanation.** Show the behaviour first — a command and its
  output — and only then open the code that produced it. A reader who has just watched
  four timings print wants to know which branch chose the slow one; a reader handed the
  branch first has nothing to attach it to.
- **One idea per step, and every step ends with something the reader can see** — output
  on their terminal, or a named line of code on their screen. A step whose result cannot
  be seen cannot be checked, and this whole document runs on being checkable.
- **Never open a file the reader has not been given a reason to open.** The reason is
  whatever they observed in the step before.
- **Show the expected output, hedged as a draft.** "You should see roughly this" is a
  checkpoint: the reader knows they are still on the path, or knows exactly where they
  left it. Since nothing here is executed, say the output is what a reading of the code
  predicts, not what a run produced.
- **Recap at the pivots.** One sentence at each turn — "you have now seen the slow path
  and who chooses it; what remains is where a warning could live" — is the difference
  between a guided walk and a corridor of facts.
- **The reader writes some of the code.** Understanding you typed outlasts understanding
  you read; that is why `partial` is the default level, and why the stubs are called
  exercises. See *Exercises and the tutorial level*.

## The sections

**What this is about.** The problem in plain language, for someone who has not read the
issue — an explanation, not a summary. If the issue assumes context, supply it here. End
with two to four *by the end you should be able to…* goals. Goals are promises the
walkthrough must keep, so state them as concrete abilities — "say which plan an
unindexed query takes and where it is chosen", not "understand indexing".

**Walkthrough.** The spine of the study: numbered steps, each one action — a command to
run or a file to open — followed by what the reader should observe and why the code
behaves that way, with `path/file.rs:123` pointing at the declarations responsible. Step
1 is always getting an environment, from the project's own docs. Run steps introduce
each probe at the moment it is needed; read steps trace what was just observed into the
code. Order the steps so understanding accumulates: the behaviour, then the branch that
chose it, then the code that does the work, then the place a fix would touch. Read every
line you cite — a confident wrong pointer is worse than none. For an issue with nothing
to run, the steps alternate between a claim and the code that confirms or contradicts
it; the discipline is the same.

**Prior art.** Code in this project that already solves a structurally similar problem —
another index type, another backend, a sibling subsystem. Usually the most valuable
section, because it turns a design argument into a precedent. Search for the *phrasing*
of the feature, not only its nouns. At most two, closest first. If there is none, say so
and say where you looked.

**Exercises.** One entry per stub: the file and symbol, what goes there, what completing
it teaches, and which walkthrough step supplies what the reader needs to write it. If
nothing is stubbed — the level is `full`, or the issue has nothing to run — say so in
one line, so it reads as decided.

**Open questions.** Decisions the issue leaves open that the implementer cannot avoid.
This section comes late because only a reader who has walked the path can argue back;
write each question so they can. Each needs what turns on the answer and your
recommendation, given as a reading with its reason. *"What should the threshold be?"* is
not one. *"Time or bytes? Time is what users feel but makes the test flaky; the sibling
feature chose bytes"* is. Do not manufacture them.

**After — what the issue asks for.** The only section describing code that does not
exist. What differs once this is fixed, seen from outside, with the same `file:line`
annotations as the probes. Precise enough to test: not "a warning is shown" but "one
warning per query naming the column, and none for a small dataset or an indexed query".
End with how a fix would be checked: which probe to rerun as the acceptance test, and
the project's own build, test and lint commands, from its docs. Your reading, not a
decision — the maintainers own the real shape.

**What I could not verify.** Claims above resting on inference rather than something you
read, code you looked for and could not find, and questions for the maintainers. This is
what makes the rest safe to trust, because it bounds it. Leave it empty only when that
is honestly true.

## Shapes of issue

Most issues are one of four kinds, and the kind decides what the lesson must show. Work
out which you have before writing anything — the tracker's own labels usually say. Each
shape below ends with a worked skeleton from a real lance issue; the `file:line`
references in them were verified by reading lance at `4a54e5dde`, except where marked as
the issue's own report.

**Feature request** — it does not exist yet. Show three things: *before*, the current
behaviour with the feature absent, run and measured; *prior art*, at most two places in
this project that already solved a structurally similar problem, closest first; and
*after*, described only. There is no probe for the target behaviour, because the code
producing it does not exist.

*Worked skeleton — lance #804, "display warning if the index is not built for a vector
column during query". The finished study ships in the vis-oss repository under
`examples/lance/lance-format/804/`.*

1. *Observe.* Setup writes three datasets — 2k rows, 20k rows, 20k rows plus an IVF_PQ
   index — because the issue's own caveat ("annoying for smaller datasets") makes the
   small, must-stay-silent case as load-bearing as the slow one. The probe times one
   query four ways, prints which plan each took, and shows stderr staying empty: the
   complaint, reproduced on the reader's own terminal.
2. *Explain.* Open `Scanner::vector_search` — it loads the indices, finds none covering
   the column, and takes the flat branch; then `KNNVectorDistanceExec::execute`, where
   the O(rows × dims) work happens and where anything that counts work would have to
   live.
3. *Prior art.* Flat full-text search already warns past a byte threshold, latched with
   `AtomicBool::swap` to fire once — the project has already chosen threshold-over-blind
   once, which converts the issue's open design question into a precedent.
4. *Exercise.* The timing helper is the stub: the quantity the issue argues about is the
   one the reader measures themselves.
5. *After.* One WARN line, quoted exactly, once per query — and the small dataset stays
   silent, which is the entire difficulty. Rerunning the probe is the acceptance check.

**Bug** — something is wrong. The probe is a reproduction: the setup and the call that
produce the wrong result, and an assertion of what is wrong. At `full` that file fails
today and passes once the bug is fixed, which makes it the acceptance check. A similar
past *fix* in the history is worth more here than a similar feature.

*Worked skeleton — lance #8846, "filtering a Float16 column with a numeric literal
always fails".*

1. *Observe.* The probe writes a three-row `Float16` dataset and filters `value < 0.0`;
   the expected output is the exact error, quoted. A `Float32` column passing the same
   predicate sits in the same file, because "the neighbouring type works" is a
   distinction the issue itself draws.
2. *Explain.* `safe_coerce_scalar` (`rust/lance-datafusion/src/expr.rs:19`) has
   `Float32` and `Float64` arms and no `Float16` in either direction;
   `rust/lance-datafusion/src/logical_expr.rs:24` turns the resulting `None` into the
   error just observed.
3. *The part that hides.* The index path calls the same helper, in `maybe_scalar`
   (`rust/lance-index/src/scalar/expression.rs:2081`), so fixing only one direction
   leaves the scalar index silently bypassed — correct rows, wrong plan. The probe
   prints the query plan for exactly this reason.
4. *Exercise.* The plan assertion is the stub: telling `refine_filter=` from
   `ScalarIndexQuery` in explain output is the skill the reader keeps.
5. *After.* The unchanged probe passes and the plan names the index. Prior art is a past
   fix that added a type arm to the same match, found by searching the history, not the
   tree.

**Performance** — something is too slow. As for a feature request, except the
measurement is the deliverable rather than the framing: measure the slow path, measure
whatever the project already offers that is faster, at sizes far enough apart to show
the trend. Prefer a structural quantity — a request count, a byte total, rows scanned —
over wall time wherever the issue allows: a count is the same on every machine, and
milliseconds are not.

*Worked skeleton — lance #8831, "BlobFile file-protocol consumers issue one object-store
GET per 8 KiB".*

1. *Observe.* The probe writes one 2 MiB blob, wraps the returned `BlobFile` in a
   ten-line counting reader, and reads it the way `zipfile` would — 8 KiB at a time.
   Expected output: 257 reads. The same loop through `io.BufferedReader`: one. The
   faster thing the project already offers *is* the comparison, and one blob shows the
   trend — no second dataset needed.
2. *Explain.* `BlobFile` (`python/python/lance/blob.py:333`) subclasses `io.RawIOBase`,
   which is unbuffered by contract: every `read(n)` is exactly one `readinto` (`:386`),
   which is one storage request. The probe counts requests instead of timing them
   because 257 is structural — the same on a laptop and in CI — where a duration is
   weather.
3. *Exercise.* The counting wrapper is the stub: it is the measuring instrument, and a
   reader who built the instrument trusts the number.
4. *After.* The same consumer loop, and the count falls from 257 to a handful. How large
   the buffer should be is the open question, not part of the assertion.

**Documentation** — something is unexplained or wrong on the page. Often there is
nothing to run; say so in *Exercises* rather than inventing a probe, and spend the
walkthrough alternating between what the page claims and where the claimed behaviour
actually lives, one claim per step, so the page can be checked against the code.

*Worked skeleton — lance #8851, "stable row id spec describes migration and
inline/external storage inaccurately".*

1. *Claim one.* The spec says stable row ids "cannot be turned on later"
   (`docs/src/format/table/row_id_lineage.md:76`); the issue reports a shipped
   `Dataset::migrate_to_stable_row_ids` whose error messages direct users to it. One
   step, both sides cited, a verdict per claim.
2. *Claim two.* The proto comments promise sequences over 200KB move to external files
   (`protos/table.proto:339`); a search shows `RowIdMeta::External` is matched in
   several places and constructed in none, and reading one is
   `todo!("External file loading not yet implemented")`
   (`rust/lance-table/src/rowids/version.rs:282`). The threshold is specification
   fiction, and the study's job is to make that checkable.
3. *After.* States what the corrected page must say — migration exists and under what
   constraints; `External` is a planned encoding no writer emits — not the corrected
   prose itself, which would be the fix.

An issue can be two of these. Pick the one the reporter is asking for: #804 asks for a
warning that does not exist, so it is a feature request, even though what it warns about
is a performance problem.

## The probes

The walkthrough is the spine; the probes are where it touches ground. A study without
runnable probes is a blog post. A probe is a **runnable check of what the code does
today**, annotated so that reading it teaches the code path, not just the API.

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
there is one file. Say so in one line in the walkthrough step that runs it, so it reads
as decided.

Both share a number, separated by extension: `01_flat_search.py` and `01_flat_search.rs`
are one probe from two sides. One setup file serves both, in whichever language is cheaper
for the reader to run.

### You write the code; you do not execute it

**Nothing here is run, and nothing is compiled.** A first execution costs whatever the
project charges to build — for a Rust workspace, its whole dependency graph — which has
nothing to do with how small your probe is, and is work the reader does anyway on their
own tree.

So the check is reading: open the file, find the symbol, confirm the signature, and name
the paths you read in the provenance line.

Say there that the file is a draft. An uncompiled Rust probe will sometimes not compile,
and the reader's first `cargo check` is where that surfaces — a minute if they were told,
an afternoon of doubt if they were not.

Do not go further: no running a probe to see whether it works, no throwaway scripts
hunting for a size or a parameter that makes a measurement come out, nothing the project
would call a benchmark.

**Those limits are on what you execute, not on what you write.** The reader typed the
command, so their file may build indexes, write a large dataset and time queries when the
issue calls for it. Reading "do not train an index" as "the probe must not train one"
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

The #804 skeleton above is this method run on that issue's three clauses: "show warnings
if the index was not built" makes index present-or-absent one axis, "annoying for smaller
datasets" makes small-or-large the second, "if the runtime exceed a certain threshold"
makes runtime the measurement. Two axes crossed is four cases — four timed rows of one
table, in which the reader sees both the case that ought to warn and the case that must
stay silent. That pair is the entire difficulty the issue is about. Note what follows:
the probe builds an index and writes a genuinely large dataset, because the axes demand
it. Sizes are chosen so the difference is visible, not so the file is cheap — the reader
is the one running it.

What does not belong: that the same query also takes the flat path when the requested
metric mismatches. True, found while reading, and worth one sentence in the walkthrough.
Not a second probe — that turns one question into three and leaves the reader working out
which one the issue was.

The reverse holds too. If the issue is about a wrong result, do not time anything; timing
would be the tour. Match the probe to the claim being made.

A probe that needs a hundred lines of harness before it reaches the project's API is
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
nothing to build. Split probes when the behaviours differ — a slow read and a wrong
result are two files; timing and plan inspection of one query are one file.

**There is no file for the target behaviour.** It cannot run, because the code producing
it does not exist. A file like `02_proposed.py` is a patch wearing a probe's clothes,
and writing it is writing the fix. What changes belongs in the `AFTER` block of the probe
it affects, and in full in the study's *After* section.

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
reader who cannot see what you opened. Never leave provenance out — a probe without one
is indistinguishable from one written from memory.

**If you assert, assert what is structural.** Assertions are not required — nobody has
run them, and a file that teaches the code path has done its job. But one that is there
runs on a machine you have never seen, so it must hold on any of them. Which plan is
chosen, which operator appears in it, whether a warning reached stderr, how many requests
were issued: those hold. Durations, ratios and recall move with the hardware and the row
count, so print them and let the reader judge. `assert flat_ms > indexed_ms * 10` is not
a check, it is a flake posted to a stranger.

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
step 1 of the walkthrough is getting one, and the lowest-numbered file **in each
language** opens with the same instructions — once, at the top, not repeated in every
file.

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
own conventions, and avoid words it does not use — `corpus` is an NLP term a study of an
IO bug cannot reuse, and `fixture` reads as test scaffolding when these are files the
reader runs and watches.

### Exercises and the tutorial level

`CONTEXT.md` records the level, and it applies to every file. It defaults to `partial`,
for two reasons. The honest one: nothing here is executed, so `full` would promise
working code on no evidence. The pedagogical one: a worked example is read once and
nodded at, while a gap the reader fills is understanding they own — textbooks put
exercises at the end of the chapter because completing something teaches more than
reading it, and `partial` is that principle applied to a probe.

| level | the file contains |
|---|---|
| `full` | complete code, written to run — a worked example for a reader who wants the artifact, not the lesson. Never executed, so the claim still rests on reading. |
| `partial` | scaffolding complete; the parts worth thinking about are exercises, left as stubs. |
| `none` | comments and `TODO`s. The reader writes every line, from your specification. |

At `partial`, choosing what to stub is choosing what the lesson examines:

- **Stub the load-bearing concept** — the quantity the issue argues about, the assertion
  that distinguishes its cases: the timing helper in a study of slowness, the plan check
  in a study of a bypassed index, the request counter in a study of amplification. Never
  the plumbing — opening datasets, parsing paths — which you finish so the exercise is
  reachable.
- **Make it solvable from the study alone.** The stub's docstring names the API, the
  file, and the shape of the result; the walkthrough has already taught the concept.
  Withhold only the typing. If solving it needs something the study never taught, teach
  it or do not stub it.
- **One or two stubs per probe**, so the file is one short, visible-result task away from
  running — a rewrite is not an exercise.

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

Being vague is not the same as being a tutorial: "figure out how to time this" helps
nobody.

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
