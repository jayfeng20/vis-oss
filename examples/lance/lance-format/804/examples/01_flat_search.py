"""
What a vector query does today when the column has no index.

This is a probe, not a fix. Running it shows you two things: that Lance silently falls
back to brute force, and how far that fallback is from the indexed path. The comments
trace each call down to the function that actually does the work, so reading the file
teaches the code path.

    cd ~/Coding/lance/python
    LANCE_LOG=warn uv run python <this file>

Tutorial level: full — every step below is yours to write.

Not run: at level full there is nothing to execute yet. What was verified is that the
APIs named below exist at lance f603c551 — `nearest`, `explain_plan` and `use_index` on
python/python/lance/dataset.py, and every rust file:line cited, each opened and read.
The timings in TODO 4 are the point of the exercise and have not been measured.

LANCE_LOG is read by init_logging, declared python/src/lib.rs:169. It sets the
env_logger level for Rust log records, which go to stderr. Keep it on so that the
absence of a warning is something you have observed rather than assumed.
"""

# TODO 1. Load the three datasets built by 00_build_datasets.py: small, large_noindex,
#    large_indexed. Fail with "run 00_build_datasets.py first" rather than letting pyarrow
#    raise from somewhere deep.

# TODO 2. Take the query vector FROM the data, not from np.random:
#      q = ds.take([0], columns=["vector"]).column("vector")[0].values.to_numpy()
#    A random uniform vector in 1536-dim space is near-equidistant from everything, so
#    recall numbers become meaningless and the behaviour you are looking for hides.

# TODO 3. Time k=10 on each dataset. Run each 3+ times and take the MEDIAN — the first
#    call pays for opening files and warming page cache, and reporting that number is how
#    imaginary regressions get filed.
#
#      ds.to_table(nearest={"column": "vector", "q": q, "k": 10})
#      # -> LanceDataset.to_table, python/python/lance/dataset.py:1486
#      # -> Scanner::vector_search, rust/lance/src/dataset/scanner.rs:5075
#      #    loads the index list and looks for one covering this column. `q.use_index`
#      #    is read at :5086; with no index it falls through to :5305,
#      #    `// No index found. use flat search.`
#      # -> Scanner::flat_knn, rust/lance/src/dataset/scanner.rs:6269
#      #    builds the node that does the scoring
#      # -> KNNVectorDistanceExec, declared rust/lance/src/io/exec/knn.rs:150
#      #    execute() at :869 streams every batch through compute_distance; the
#      #    per-batch loop starts at :904. This is the O(rows x dim) work.
#
#    For the indexed dataset the same call diverges at :5305 into the ANN path:
#      # -> ANNIvfPartitionExec / ANNIvfSubIndexExec, declared knn.rs:1157 and :1374
#      #    probe a few IVF partitions instead of scanning everything

# TODO 4. Print the ratio between large-unindexed and large-indexed. That single number
#    is the argument for this issue and belongs in a PR description.

# TODO 5. Prove which path each query took rather than trusting the timings:
#      print(ds.scanner(nearest={...}).explain_plan(verbose=True))
#    Brute force shows a `KNNVectorDistance` node over a scan and no ANN node; the
#    indexed one shows `ANNSubIndex`. The Rust test at scanner.rs:9873 asserts exactly
#    this way, so you are matching an existing convention.

# TODO 6. Assert that stderr is EMPTY for the large unindexed query. Run it in a
#    subprocess and read stderr — Rust log records reach stderr through env_logger and
#    never touch Python's logging, so caplog cannot see them. The working pattern is
#    test_lance_log_file, python/python/tests/test_log.py:85.

# TODO 7. Repeat the large query with use_index=False against large_indexed. Confirm the
#    timing matches large_noindex. This is the deliberate-opt-out case, and open question
#    2 in CONTEXT.md is whether it should warn. Decide with the timing in front of you.

# ---- AFTER ----
#
# Every TODO above stays exactly the same. Two things change when you run it:
#
#   1. TODO 6 stops passing. stderr gains one line for the large unindexed query:
#
#        WARN lance::io::exec::knn: brute-force vector search scored 500000 rows on
#        column "vector"; consider creating a vector index on this column
#
#      Once. Not once per RecordBatch, and not once per DataFusion partition — the node
#      inherits its input's partitioning, so execute() at knn.rs:869 runs per partition
#      and a latch created inside it would fire repeatedly.
#
#   2. Nothing else moves. The small dataset stays silent. The indexed query stays
#      silent. Timings are unchanged; this issue adds a message, not a fast path.
#
# The API does not change. `to_table(nearest=...)` keeps its signature — this is a
# behaviour change, visible only on stderr.
