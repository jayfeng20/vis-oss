# lance-format/lance #804 — Display warning if the index is not built for a vector column during query

https://github.com/lance-format/lance/issues/804  
open · opened 2023-04-24 · good first issue · help wanted · A-python · rust

> Studied against `324cedd9dc7add89f54f8ad14abb9c826698eab5` in `~/Coding/lance`.  
> Probes are at tutorial level **partial** — scaffolding is written; the parts worth thinking about are exercises. See `AGENTS.md`.

> **Written by an agent, and not reviewed.** It is a head start on understanding
> this issue, not an answer to it. Check any file reference before you rely on it,
> and treat the reasoning as a first draft to argue with — the maintainers own
> what the fix should be.

---

## The issue

When user does a ANN query, show warnings if the index was not built for the column, for potentially slow query speed.

However, doing so blindly makes it really annoying for smaller datasets that really don't need an ANN index. One alternative could be that during the KNN (flat search) execution, if the runtime exceed a certain threshold to print the warning.

---

## What this is about

A vector similarity search against a column with no vector index does not fail. Lance
falls back to brute force: read every vector, compute the distance to the query, sort,
take the top k. Correct, but O(rows × dimensions) per query.

On 1,000 rows that is instant. On 100M rows it is a multi-second query that the user may
not realise could be milliseconds with an index. The issue asks Lance to say so.

The whole difficulty is the author's own caveat. A warning on every unindexed vector
query fires constantly during development and testing, where brute force is genuinely the
right choice. So it has to be conditional on the query actually being expensive — which
means the interesting question is not "should we warn" but "how do we know it was
expensive".

By the end of this study you should be able to:

- run the same query on the flat and the indexed path, and put a number on each;
- point at the branch that chooses brute force and the operator that pays for it;
- name the precedent this project already has for a threshold-latched warning;
- argue for elapsed time, bytes, or rows as the quantity a threshold should measure.

## Walkthrough

Every file this walkthrough runs is a draft — written, never executed, and the Rust one
never compiled. Each carries a provenance line saying what was read instead; expect the
Rust file to want a fix or two on first `cargo check`.

**1. Get an environment.** Per `python/AGENTS.md`, lance's Python surface is a `uv`
project with no venv to activate — that file says in as many words not to rely on an
activated environment. Once, from the checkout:

```sh
cd ~/Coding/lance/python
make install        # runs uv sync, which builds the pylance extension; slow, do not interrupt
```

**2. Build the datasets.** Run the setup file:

```sh
uv run --frozen python <study>/00_build_datasets.py
```

It writes three datasets: a 2k one where brute force is the right call and any future
warning must stay silent, and two 20k ones identical but for an IVF_PQ index — same
seed, so any difference between them is attributable to the index alone. You should see
three lines, each naming the dataset, its row count, and its indices; the indexed write
dominates the runtime because it trains IVF centroids. The small dataset matters as much
as the large ones: it is the case the issue author had in mind when they warned against
warning blindly.

**3. Complete the probe, then watch the silence.** Open `01_flat_search.py`. It times
one query four ways — small, large without index, large with index, large with the index
refused via `use_index=False` — and two exercises gate it: write the timing helper
`median_ms` (Exercise 1) and predict the ANN plan marker (Exercise 3). Each gap's
message says exactly what goes there. Fill both, then:

```sh
LANCE_LOG=warn uv run --frozen python <study>/01_flat_search.py
```

You should see roughly — predicted from reading, not from a run —

```
  small (2k, no index)                 ?.? ms   plan=flat
  large (20k, no index)                ?.? ms   plan=flat
  large (20k, indexed)                 ?.? ms   plan=ANN
  large (20k, use_index=False)         ?.? ms   plan=flat
```

with **nothing on stderr**, even though `LANCE_LOG=warn` is watching for it. That
silence is the issue: a 1k-row flat query and a 100M-row one are indistinguishable from
outside except by wall time. An earlier run of this study, at these sizes, reported
`flat / indexed: 0.6x` — the index *loses* at 20k rows. If you see the same, that is the
issue's own caveat showing up in a measurement, and it is why a threshold is needed at
all.

**4. Find who chose the slow path.** The `to_table(nearest=...)` call you just timed
lands in `Scanner::vector_search` — `rust/lance/src/dataset/scanner.rs:5075`, the whole
decision. It reads `q.use_index` around `:5086`, loads the dataset's indices, and looks
for one covering the queried column. Finding none, it falls through to `:5305`, marked
`// No index found. use flat search.` — the fork itself; everything below it is the
brute-force path. `Scanner::flat_knn` at `:6269` then builds the scoring node. Read the
stretch from `:5075` to `:5305` and notice what is absent: no logging, at any level, on
either side of the fork. `:5305` is the simplest place to warn and the wrong one — it
fires unconditionally, which is exactly the annoyance the issue names.

**5. Find who pays for it.** The node `flat_knn` builds is `KNNVectorDistanceExec`,
declared `rust/lance/src/io/exec/knn.rs:150`. Its `execute()` at `:869` streams every
batch through the distance computation — the per-batch loop starts at `:904`, just below
`// Empty batches don't have a vector column to score`. This is the O(rows × dims) work,
and the one place the cost is *measurable* rather than predicted, which is what the
issue's threshold idea needs. One partitioning caveat, visible at the declaration: in the
non-batch case the node inherits its input's partitioning, so `execute()` runs once per
DataFusion partition — a counter or a warned-already latch created inside `execute()`
would fire once per partition, so they belong on the struct as fields.

While in `scanner.rs`, note `:5200`: an existing `log::warn!` for the neighbouring case
where the requested metric is incompatible with the index and Lance falls back to brute
force. Whatever warning this issue adds should match its phrasing and level.

You have now seen the four cases, the branch that chooses the flat path, and the
operator that could measure it. What remains is the precedent for *when* to warn, and
the acceptance loop for a fix.

**6. The same probe from the implementation side.** `01_flat_search.rs` is the identical
probe in Rust, calling `Dataset::open` and `Scanner::nearest` directly — so it runs
against your own working tree with no Python extension to rebuild, and re-running it
after a change is the acceptance check for the `AFTER` block. It is a `[[bin]]` in a
Cargo project one level up, shared by every issue studied in this repository; that
manifest is not checked in — its `lance` path dependency must point at your checkout —
so write it from the template in `AGENTS.md` first. Its `median_ms` is Exercise 2, and
it shares the plan-marker prediction (Exercise 3).

```sh
cd <study root>/lance/lance-format
RUST_LOG=warn cargo run --bin 804_flat_search
```

The first build compiles lance's whole dependency graph; expect minutes, not a hang.

## Prior art

**Flat full-text search already implements this exact feature.**

`rust/lance-index/src/scalar/inverted/index/flat_search.rs:208`

```rust
/// If we accumulate this many bytes we warn the user they probably want to use an FTS index instead.
pub(super) const BYTES_ACCUMULATED_WARNING_THRESHOLD: u64 = 1024 * 1024 * 1024; // 1GB
```

and at line 448, inside the per-batch closure:

```rust
let bytes_accumulated = bytes_accumulated
    .fetch_add(result_batch.get_array_memory_size() as u64, Ordering::Relaxed);
if bytes_accumulated > BYTES_ACCUMULATED_WARNING_THRESHOLD
    && !bytes_warning_emitted.swap(true, Ordering::Relaxed)
{
    tracing::warn!("Flat full text search is accumulating a large number of bytes.  Consider using an FTS index instead.");
}
```

Same feature, different data type: threshold-triggered rather than blind, latched to fire
once per query via `AtomicBool::swap`, phrased as "consider using an index". It sidesteps
the small-dataset annoyance exactly as this issue asks, and it means the project has
effectively already chosen threshold-over-blind. Note it measures **accumulated bytes**,
not the wall-clock time the issue suggests.

Two further pieces that decide implementation details:

**`python/src/tracing.rs:250`** — `fn on_event` forwards tracing events into `log` at the
global `LANCE_TRACING` level rather than the event's own, so `tracing::warn!` reaches a
Python user at INFO with a reformatted message, while `log::warn!` arrives at WARN. The
tree runs roughly 92 `log::warn!` to 6 `tracing::warn!`.

**`python/python/tests/test_log.py:85`** — `def test_lance_log_file(` shows how Rust log
output is actually tested from Python: subprocess, `LANCE_LOG`, read stderr. `caplog`
cannot see it, because `env_logger` writes to stderr and bypasses Python's `logging`.

I found no vector-side equivalent of the FTS warning, and no "consider creating an index"
message anywhere else in `rust/` — that phrasing has exactly one hit in the tree.

## Exercises

Three, shared across the two probes: two instruments to write (one per language) and one
prediction to fill. Each gap names the API to call and the shape of the result; steps
3–5 of the walkthrough supply everything else.

**Exercise 1 — `median_ms` in `01_flat_search.py`.** Time `REPEATS` runs of
`to_table(nearest=...)` around `time.perf_counter()` and return the median in
milliseconds. The quantity the issue argues about is the one you measure yourself — and
choosing the *median* is the lesson: the first run pays for opening files and warming the
page cache, and reporting that number is how imaginary regressions get filed.

**Exercise 2 — `median_ms` in `01_flat_search.rs`.** The same measurement, driving
`Scanner::nearest` and `try_into_batch` directly. What it adds over Exercise 1 is the
acceptance loop: once you can run this binary, a change to `KNNVectorDistanceExec` in
your checkout is one `cargo run` away from a verdict, with nothing rebuilt in between.

**Exercise 3 — the ANN plan marker, in both files (a prediction).** `uses_ann` needs the
substring that marks the ANN path in an `explain_plan` output, and both files leave it
blank. It can only be filled by reading: the operators are declared in
`rust/lance/src/io/exec/knn.rs` (walkthrough step 5), and the project's own tests assert
on the same string — `scanner.rs:9363` for the ANN path, `:9872` for the flat one. The
three plan assertions in `main()` grade your answer on the first run.

## Open questions

**1. Should the threshold measure elapsed time, bytes scanned, or rows scanned?**
Elapsed time is what a user feels, but it makes behaviour machine-dependent and the test
flaky under CI load. Bytes and rows are deterministic. *My reading:* follow the FTS
precedent and use a deterministic size measure — rows × dimensions is the honest cost model
for vector distance and, unlike bytes, does not vary with column encoding.

**2. Should an explicit `use_index=False` stay silent?** It is a deliberate opt-out rather
than a missing index, so warning is arguably noise — but it is also the flag used to
measure recall against brute force, where the reminder may be welcome. *My reading:*
silent, since someone who typed `use_index=False` knows what they asked for — but this is a
guess about intent and worth asking on the issue.

**3. `log::warn!` or `tracing::warn!`?** Per `python/src/tracing.rs` above, the choice
decides whether a Python user sees a WARN or a reformatted INFO. *My reading:*
`log::warn!`, matching the neighbouring fallback warning in `scanner.rs` — but see *What I
could not verify*, because the premise here is unobserved. Worth raising on
the issue that the FTS warning may be less visible from Python than intended.

## After — what the issue asks for

No API changes. `to_table(nearest=...)` and `LanceDataset.nearest` keep their signatures;
this is a behaviour change, visible only on stderr.

Running `01_flat_search.py`, once its exercises are filled, a sufficiently large
unindexed query gains one line:

```
WARN lance::io::exec::knn: brute-force vector search scored 500000 rows on column
"vector"; consider creating a vector index on this column
```

Precisely:

- **exactly once per query.** Not per `RecordBatch`, and not per DataFusion partition —
  `KNNVectorDistanceExec` (declared `rust/lance/src/io/exec/knn.rs:150`) inherits its
  input's partitioning in the non-batch case, so `execute()` at `:869` runs once per
  partition. The counter and latch belong on the struct, not inside `execute()`. The FTS
  precedent does this with `AtomicBool::swap` at
  `rust/lance-index/src/scalar/inverted/index/flat_search.rs:448`.
- **once for a batch query too.** `nearest` with a 2-D `q` goes through the
  `is_batch_nearest` path (the flag is set from `is_batch_nearest_query`, `scanner.rs:1802`); eight query vectors must not produce eight warnings.
- **silent below the threshold.** The small dataset produces nothing. This is the whole
  difficulty of the issue, and the only part reviewers will scrutinise.
- **silent when an index served the query**, and — see open question 2 — silent when the
  caller passed `use_index=False`.
- **emitted with `log::warn!`,** so it arrives at a Python user as WARN rather than being
  re-levelled through `on_event` (`python/src/tracing.rs:250`). See *What I could not
  verify*.

Timings do not change. This issue adds a message, not a fast path.

**Checking a fix.** The acceptance test is walkthrough step 6: change your checkout,
`cargo run --bin 804_flat_search` again, and the line appears exactly once — or does not.
`01_flat_search.py` checks the same thing from the user's side after `make build`
rebuilds the extension. The project's own gates, from its docs:

```sh
cargo check --workspace --tests --benches
cargo test -p lance knn
cargo clippy --all --tests --benches -- -D warnings

cd python && make build && uv run pytest python/tests/test_log.py -x
```

Python commands must go through `uv` — see `python/AGENTS.md`. `make build` is required
after any Rust change, or you are running new Python against an old extension.

## What I could not verify

**The Python visibility claim in *Prior art* is inference, not observation.** I read
`on_event` in `python/src/tracing.rs` and concluded that a `tracing::warn!` reaches a
Python user at the `LANCE_TRACING` level rather than WARN. I never ran a flat FTS query
over 1GB with `LANCE_LOG=warn` to watch what actually lands on stderr. If that inference
is wrong, open question 3 dissolves and the FTS precedent should simply be copied. This is
the single claim most worth checking before citing it on the issue.

**Nothing here was executed.** The probes were written and their APIs checked by
reading the source at the commit above; no query was run and no dataset was built while
writing this. Every file reference below and above is a read reference. Treat the numbers
quoted from an earlier run of this study as history, not as a measurement of the code you
have.

**The crossover point is not known.** The earlier run only covered 20k rows x 128 dims,
where the index came out *slower* than brute force — 0.6x. That supports the issue's
premise about small datasets and says nothing about where a threshold belongs. Somebody
needs to run this at 100k, 1M and 10M to find where the index starts winning; that number
is the one a threshold has to sit above, and it is missing.

**The dimension is unrepresentative.** 128 dims was chosen so the probe is cheap for
whoever runs it. Real embedding workloads are 768-1536, where flat search costs 6-12x more
per row and the crossover arrives much earlier. Rerun at a realistic dimension before
trusting any threshold derived from these numbers.

**The threshold recommendation is a judgement with no data behind it.** Rows × dimensions
is argued from the cost model, not from measurement, and I did not check whether Lance
already tracks a suitable quantity at that point in the stream. If it does not, the
counter may cost more than the warning is worth, which would change the answer.

**Whether `use_index=False` should stay silent is a guess about intent.** It is worth
asking on the issue rather than deciding in a PR.

**Not checked:** whether anyone has attempted this before in a closed PR, and whether the
maintainers have opinions recorded somewhere other than the issue — Discord, for instance,
which `CONTRIBUTING.md` points contributors to.
