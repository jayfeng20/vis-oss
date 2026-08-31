"""
Times the same vector query with and without an index, and shows that the slow path says
nothing.

    cd ~/Coding/lance/python
    LANCE_LOG=warn uv run --frozen python <this file>

Tutorial level: partial — median_ms is yours to write, and the ANN plan marker yours to
predict. Run 00_build_datasets.py first.

Not run, not compiled. APIs read against lance 324cedd9d:
to_table at python/python/lance/dataset.py:1559, scanner at :1182, explain_plan at :640,
take at :2228. The branches named below come from reading Scanner::vector_search,
rust/lance/src/dataset/scanner.rs:5075. Nothing here asserts on a duration, because the
numbers are whatever your machine does.

An earlier run of this study, at these sizes, found the index *slower* than the flat scan
— roughly 0.6x. If you see that too it is not a bug: IVF probing costs more than 20k
distance computations. That is the issue's premise, and it is why the issue asks for a
threshold rather than a flag.

LANCE_LOG is read by init_logging, declared python/src/lib.rs:177. It sets the env_logger
level for Rust log records, which go to stderr. Keep it on, so that the absence of a
warning is something you observed rather than assumed.
"""

import statistics
import sys
import time
from pathlib import Path

import lance
import numpy as np

ROOT = Path.home() / "lance-study-data"
K = 10
REPEATS = 5


def open_dataset(name: str) -> lance.LanceDataset:
    path = ROOT / f"{name}.lance"
    if not path.exists():
        sys.exit(f"{path} is missing — run 00_build_datasets.py first")
    return lance.dataset(path)


def median_ms(dataset: lance.LanceDataset, query: np.ndarray, *, use_index: bool) -> float:
    """Median wall time of one nearest-neighbour query, in milliseconds.

    Call dataset.to_table(nearest={"column": "vector", "q": query, "k": K,
    "use_index": use_index}) REPEATS times around time.perf_counter(), and return
    statistics.median of the millisecond timings. Median rather than min or mean: the
    first call pays for opening files and warming the page cache, and reporting that
    number is how imaginary regressions get filed.

    That to_table call is the whole point of this file, so it is worth knowing where it
    goes:
      -> LanceDataset.to_table, python/python/lance/dataset.py:1559
      -> Scanner::vector_search, rust/lance/src/dataset/scanner.rs:5075
         reads q.use_index at :5086 and looks for an index covering the column.
         Finding none, it falls through to :5305, `// No index found. use flat
         search.`, then Scanner::flat_knn at :6269 builds the scoring node.
      -> KNNVectorDistanceExec, declared rust/lance/src/io/exec/knn.rs:150.
         execute() at :869 streams every batch through the distance computation; the
         per-batch loop starts at :904. This is the O(rows x dim) work.

         With an index the same call diverges at :5305 into ANNIvfPartitionExec
         (knn.rs:1164) and ANNIvfSubIndexExec (knn.rs:1381), which probe a few IVF
         partitions instead of scanning everything.
    """
    raise NotImplementedError(
        "time REPEATS runs of to_table(nearest=...) and return the median in ms; "
        "the first run pays for page-cache warmup"
    )


def TODO(hint: str):
    raise NotImplementedError(hint)


# Which substring marks the ANN path in a plan? A prediction to fill by reading, not
# running: the operators are declared in rust/lance/src/io/exec/knn.rs, and the project's
# own tests assert on the same string — rust/lance/src/dataset/scanner.rs:9363 for the
# ANN path, :9872 for the flat one. The asserts in main() grade your answer.
ANN_PLAN_MARKER = TODO("the operator name an ANN plan contains")


def uses_ann(dataset: lance.LanceDataset, query: np.ndarray, *, use_index: bool) -> bool:
    """Read the plan rather than inferring the path from timings."""
    plan = dataset.scanner(
        nearest={"column": "vector", "q": query, "k": K, "use_index": use_index}
    ).explain_plan(verbose=True)
    return ANN_PLAN_MARKER in plan


def main() -> None:
    small = open_dataset("small")
    noindex = open_dataset("large_noindex")
    indexed = open_dataset("large_indexed")

    # Take the query FROM the data. A random uniform vector in high-dimensional space is
    # near-equidistant from everything, which makes recall meaningless.
    query = noindex.take([0], columns=["vector"]).column("vector")[0].values.to_numpy()

    cases = [
        ("small (2k, no index)", small, True),
        ("large (20k, no index)", noindex, True),
        ("large (20k, indexed)", indexed, True),
        ("large (20k, use_index=False)", indexed, False),
    ]
    for label, dataset, use_index in cases:
        elapsed = median_ms(dataset, query, use_index=use_index)
        path = "ANN" if uses_ann(dataset, query, use_index=use_index) else "flat"
        print(f"  {label:32} {elapsed:7.1f} ms   plan={path}")

    # Which plan was chosen is structural: it holds on any machine, at any size.
    assert not uses_ann(noindex, query, use_index=True), "expected the flat path"
    assert uses_ann(indexed, query, use_index=True), "expected the ANN path"
    assert not uses_ann(indexed, query, use_index=False), "use_index=False must be flat"

    flat = median_ms(noindex, query, use_index=True)
    ann = median_ms(indexed, query, use_index=True)
    print(f"\n  flat / indexed at {noindex.count_rows()} rows: {flat / ann:.1f}x")
    print("  nothing was written to stderr: the slow path is silent")


if __name__ == "__main__":
    main()

# ---- AFTER ----
#
# This file does not change. One thing does: a sufficiently large unindexed query gains a
# line on stderr, once per query.
#
#   WARN lance::io::exec::knn: brute-force vector search scored 20000 rows on column
#   "vector"; consider creating a vector index on this column
#
# Once — not per RecordBatch, and not per DataFusion partition. KNNVectorDistanceExec
# inherits its input's partitioning in the non-batch case, so execute() at knn.rs:869
# runs once per partition; a latch created inside it would fire repeatedly. The FTS
# precedent latches with AtomicBool::swap at
# rust/lance-index/src/scalar/inverted/index/flat_search.rs:448.
#
# At the sizes above, the large dataset should probably ALSO stay silent: brute force is
# winning there, and warning would be wrong. Whatever threshold is chosen has to sit above
# the crossover, which this file does not find — see CONTEXT.md, What I could not verify.
# Timings do not change; this issue adds a message, not a fast path.
