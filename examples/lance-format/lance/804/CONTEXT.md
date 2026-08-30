# lance-format/lance #804 — Display warning if the index is not built for a vector column during query

https://github.com/lance-format/lance/issues/804  
open · opened 2023-04-24 · good first issue · help wanted · A-python · rust

> Studied against `f603c5516b41c3aae4cb0569b4c96a5253078d81` in `~/Coding/lance`.  
> Examples are in **tutorial** mode. See `AGENT.md` for what that means.

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

## What should happen

A vector query that actually performs an expensive brute-force scan emits **one** warning
naming the column and suggesting an index.

Silent in all of these:

- the dataset is small enough that brute force was the right call
- an index served the query
- the caller passed `use_index=False` (see open question 2)

"One" means once per query — not once per `RecordBatch`, and not once per DataFusion
partition. Both are easy to get wrong; see the note under *Where the code is*.

## Where the code is

Read in this order.

**`rust/lance/src/dataset/scanner.rs:5075`** — `async fn vector_search(`. The whole
decision. Loads indices, decides whether any covers the queried column, and branches.
`q.use_index` is read at the top, around line 5086.

**`rust/lance/src/dataset/scanner.rs:5305`** — `// No index found. use flat search.` The
fork itself; everything below it is the brute-force path. This is the simplest place to
warn and the wrong one: it fires unconditionally, which is exactly the annoyance the
issue names.

**`rust/lance/src/dataset/scanner.rs:6269`** — `fn flat_knn(&self`. Builds the
`KNNVectorDistanceExec` node that does the work.

**`rust/lance/src/io/exec/knn.rs:150`** — `pub struct KNNVectorDistanceExec`. The
execution node. A counter and a warning latch belong here as *fields*: in the non-batch
case the node inherits its input's partitioning, so `execute()` runs once per partition
and anything created inside it would warn once per partition.

**`rust/lance/src/io/exec/knn.rs:904`** — `// Empty batches don't have a vector column to
score`, inside `execute()` just above the per-batch distance stream. The proposed warning
site: the work is measurable here, unlike at plan time.

**`rust/lance/src/dataset/scanner.rs:5201`** — an existing `log::warn!` for the
neighbouring case where a requested metric is incompatible with the index and Lance falls
back to brute force. Match its phrasing and level.

## Prior art

**Flat full-text search already implements this exact feature.**

`rust/lance-index/src/scalar/inverted/index/flat_search.rs:207`

```rust
/// If we accumulate this many bytes we warn the user they probably want to use an FTS index instead.
pub(super) const BYTES_ACCUMULATED_WARNING_THRESHOLD: u64 = 1024 * 1024 * 1024; // 1GB
```

and at line 449, inside the per-batch closure:

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

Run from `~/Coding/lance/python` in every case.

**`examples/00_corpus.py`** — builds the corpora the other two need: one small enough that
brute force is correct and must stay silent, one large enough that it is not. Offers real
1536-dim OpenAI embeddings as well as a synthetic fallback.

```sh
uv run python <study>/examples/00_corpus.py --scale small
uv run python <study>/examples/00_corpus.py --scale large --real
```

**`examples/01_current.py`** — *before*. Times the same query against indexed and
unindexed copies of the same data, proves which path ran with `explain_plan`, and shows
nothing is emitted on stderr at any size.

```sh
LANCE_LOG=warn uv run python <study>/examples/01_current.py
```

**`examples/02_proposed.py`** — *after*. The acceptance criteria as executable checks:
large-unindexed warns exactly once; small, indexed, and `use_index=False` stay silent.
Fails today, which is the point.

```sh
uv run python <study>/examples/02_proposed.py
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

**No timings were measured.** The premise that brute force is dramatically slower at scale
is standard and near-certain, but the ratio that belongs in a PR description is not in this
study, because no large corpus was ever built. `examples/01_current.py` is written to
produce it; nobody has run it.

**None of the three example files has been executed.** They are written against APIs I
read in the source (`nearest`, `explain_plan`, `create_index`, `list_indices`) and against
lance's own `benchmarks/dbpedia-openai/datagen.py`, but the scripts themselves are
unrun — expect to fix small things.

**The threshold recommendation is a judgement with no data behind it.** Rows × dimensions
is argued from the cost model, not from measurement, and I did not check whether Lance
already tracks a suitable quantity at that point in the stream. If it does not, the
counter may cost more than the warning is worth, which would change the answer.

**Whether `use_index=False` should stay silent is a guess about intent.** It is worth
asking on the issue rather than deciding in a PR.

**Not checked:** whether anyone has attempted this before in a closed PR, and whether the
maintainers have opinions recorded somewhere other than the issue — Discord, for instance,
which `CONTRIBUTING.md` points contributors to.
