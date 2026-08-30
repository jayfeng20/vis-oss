"""
AFTER — the behaviour issue #804 asks for, written as executable acceptance criteria.

This file FAILS today. That is its job: run it before you touch any Rust and watch the
large-dataset case fail, then implement until every case passes. It is the difference
between "I added a warning" and "I added a warning that is not annoying", which is the
entire substance of the issue.

    cd ~/Coding/lance/python
    uv run python <this file>

Read the study's prior_art first — flat FTS already implements this exact feature
(rust/lance-index/src/scalar/inverted/index/flat_search.rs:449). You are writing the
vector twin of code that already exists and has already been reviewed.
"""

# TODO 1. Write ONE helper that runs a query in a subprocess and returns its stderr.
#    This must be a subprocess: Rust log records go to stderr via env_logger and never
#    reach Python's logging module, so pytest's caplog cannot see them. The existing
#    pattern is python/python/tests/test_log.py:85 (test_lance_log_file) —
#    subprocess.run([sys.executable, "-c", script], capture_output=True,
#                   env={"LANCE_LOG": "warn", ...}).
#    Getting this wrong is the single most likely way to waste an afternoon on #804.

# TODO 2. Pick the substring you will assert on, ONCE, as a module constant. Suggested
#    shape, mirroring the FTS wording so the two read as siblings:
#      "brute-force" ... "Consider creating a vector index"
#    Keep it short enough to survive rewording in review.

# TODO 3. The four acceptance cases. Each is one assertion on the captured stderr:
#
#      case                                    expected
#      ------------------------------------    ------------------------------
#      large, no index                         warns EXACTLY once
#      small, no index                         does NOT warn        <- the hard one
#      large, indexed                          does NOT warn
#      large, indexed, use_index=False         per study question 2
#
#    "Exactly once" means count occurrences, not `in`. Two bugs produce a duplicate
#    warning and both are easy to ship:
#      - a latch created inside execute() rather than on the node: KNNVectorDistanceExec
#        inherits its input's partitioning in the non-batch case (knn.rs, the
#        `properties` built in try_new_batch), so execute() runs once PER PARTITION.
#      - no latch at all: fires once per RecordBatch.
#    Both are what AtomicBool::swap(true, Ordering::Relaxed) prevents in the FTS code.

# TODO 4. The batch-query case, which is easy to forget:
#      ds.to_table(nearest={"column": "vector", "q": <2-D array of 8 vectors>, "k": 10})
#    A batch query goes through the is_batch_nearest path. Assert it warns ONCE, not
#    once per query vector — eight warnings for one call is worse than none.

# TODO 5. The threshold boundary. Whatever measure you chose (study question 1), build
#    a corpus just under it and one just over, and assert silence then noise. A
#    threshold with no test around it is a magic number, and a reviewer will say so.
#    Give the constant a doc comment saying what it measures and why that value —
#    rust/AGENTS.md requires this, and the FTS constant at flat_search.rs:207 is the
#    model to copy.

# TODO 6. Print a small pass/fail table and exit non-zero on any failure, so you can
#    run this from a shell loop while iterating on the Rust. Use `assert` rather than
#    print for the checks themselves — root AGENTS.md: "Replace print() in tests with
#    assert — prints don't catch regressions."

# TODO 7. When all cases pass, port cases 1-3 into the real suites and delete nothing
#    here — this file is your bench, the suites are the deliverable:
#      - Rust:   rust/lance/src/io/exec/knn.rs, in the existing `mod tests`
#                (assert on observable node state; there is no log-capture helper
#                 anywhere in rust/, so do not try to assert on log text there)
#      - Python: python/python/tests/test_log.py, following test_lance_log_file
