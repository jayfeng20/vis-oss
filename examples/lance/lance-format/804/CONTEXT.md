# lance-format/lance #804 — Display warning if the index is not built for a vector column during query

https://github.com/lance-format/lance/issues/804  
open · opened 2023-04-24 · good first issue · help wanted · A-python · rust

> Studied against `324cedd9dc7add89f54f8ad14abb9c826698eab5` in `~/Coding/lance`.  
> Examples are at tutorial level **partial** — scaffolding is written; the interesting parts are stubs. See `AGENTS.md`.

> **Written by an agent, and not reviewed.** It is a head start on understanding
> this issue, not an answer to it. Check any file reference before you rely on it,
> and treat the reasoning as a first draft to argue with — the maintainers own
> what the fix should be.

---

## The issue

When user does a ANN query, show warnings if the index was not built for the column, for potentially slow query speed.

However, doing so blindly makes it really annoying for smaller datasets that really don't need an ANN index. One alternative could be that during the KNN (flat search) execution, if the runtime exceed a certain threshold to print the warning.

---

## What this actually is

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

## What happens today

`Scanner::vector_search` loads the dataset's indices, finds none covering the queried
column, and takes the flat branch — building a `KNNVectorDistanceExec` over a full scan.

Nothing is logged, at any level. A 1k-row query and a 100M-row query are
indistinguishable from outside; the only signal is wall time. The same is true when the
user passes `use_index=False` against a column that *does* have an index.

## Where the code is

Read in this order.

**`rust/lance/src/dataset/scanner.rs:5075`** — `async fn vector_search(`. The whole
decision. Loads indices, decides whether any covers the queried column, and branches.
`q.use_index` is read at the top, around line 5086.

**`rust/lance/src/dataset/scanner.rs:5305`** — `// No index found. use flat search.` The
fork itself; everything below it is the brute-force path. This is the simplest place to
warn and the wrong one: it fires unconditionally, which is exactly the annoyance the
issue names.

**`rust/lance/src/dataset/scanner.rs:6269`** — `fn flat_knn(&self,`. Builds the
`KNNVectorDistanceExec` node that does the work.

**`rust/lance/src/io/exec/knn.rs:150`** — `pub struct KNNVectorDistanceExec`. The
execution node. A counter and a warning latch belong here as *fields*: in the non-batch
case the node inherits its input's partitioning, so `execute()` runs once per partition
and anything created inside it would warn once per partition.

**`rust/lance/src/io/exec/knn.rs:904`** — `// Empty batches don't have a vector column to
score`, inside `execute()` (`:869`) just above the per-batch distance stream. The proposed warning
site: the work is measurable here, unlike at plan time.

**`rust/lance/src/dataset/scanner.rs:5200`** — an existing `log::warn!` for the
neighbouring case where a requested metric is incompatible with the index and Lance falls
back to brute force. Match its phrasing and level.

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

## Examples

**All three are drafts** — none was run, and the Rust one was never compiled. See the
provenance line in each for what was checked by reading instead, and expect the Rust file
to want a fix or two on first `cargo check`. Each probe has one stub, `median_ms`, which
you write before it will run.

The probe exists twice: `01_flat_search.py` is what a user meets, `01_flat_search.rs` is
the same probe in the language the behaviour is implemented in. The Rust one runs against
your own working tree with no extension to rebuild, so re-running it after your change is
the acceptance check for the `AFTER` block.

**`00_build_datasets.py`** — writes three datasets: a 2k one where brute force is the
right call and must stay silent, and two 20k ones identical but for an IVF_PQ index, so
any difference is attributable to the index alone.

```sh
uv run --frozen python <study>/00_build_datasets.py
```

**`01_flat_search.py`** — the probe. Times the same query four ways, asserts which path
the plan actually took, and shows that nothing reaches stderr. Its comments trace each
call down to the declaration that does the work, and its `AFTER` block says what changes.
The timing helper is the stub; the plan-inspection helper and every annotation are
complete.

```sh
LANCE_LOG=warn uv run --frozen python <study>/01_flat_search.py
```

An earlier run of this study, at these sizes, reported `flat / indexed: 0.6x` — the index
loses. If you see the same, that is the issue's own caveat showing up in a measurement.

**`01_flat_search.rs`** — the same probe from Rust. It is a `[[bin]]` in the shared study
project one level up (`../Cargo.toml`), whose `lance` path dependency you point at your
own checkout. The first build compiles lance's whole dependency graph, so expect minutes.

```sh
cd <study root>/lance/lance-format
RUST_LOG=warn cargo run --bin 804_flat_search
```

## How to verify

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

**Nothing here was executed.** The examples were written and their APIs checked by
reading the source at the commit above; no query was run and no dataset was built while
writing this. Every file reference below and above is a read reference. Treat the numbers
quoted from an earlier run of this study as history, not as a measurement of the code you
have.

**The crossover point is not known.** The earlier run only covered 20k rows x 128 dims,
where the index came out *slower* than brute force — 0.6x. That supports the issue's
premise about small datasets and says nothing about where a threshold belongs. Somebody
needs to run this at 100k, 1M and 10M to find where the index starts winning; that number
is the one a threshold has to sit above, and it is missing.

**The dimension is unrepresentative.** 128 dims was chosen so the example is cheap for
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

## After — what the issue asks for

No API changes. `to_table(nearest=...)` and `LanceDataset.nearest` keep their signatures;
this is a behaviour change, visible only on stderr.

Running `01_flat_search.py`, once its stub is filled, a sufficiently large unindexed
query gains one line:

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
