"""
BEFORE — what Lance does today.

Goal: see with your own eyes that a brute-force vector search is (a) dramatically
slower than the indexed one and (b) completely silent about it, at every size. That
silence is the issue.

    cd ~/Coding/lance/python
    LANCE_LOG=warn uv run python <this file>

LANCE_LOG is read by python/src/lib.rs::init_logging and sets the env_logger level for
Rust log records, which go to STDERR. Keep it on so you can prove nothing is emitted
rather than assuming it.
"""

# TODO 1. Load the three datasets from example 00: small, large_noindex, large_indexed.
#    Assert each exists with a useful message; a missing corpus should say "run 00
#    first", not throw FileNotFoundError from deep inside pyarrow.

# TODO 2. Take a query vector FROM the data, not from np.random.
#    q = ds.take([0], columns=["vector"]).column("vector")[0].values.to_numpy()
#    A random uniform vector in 1536-dim space is roughly equidistant from everything
#    (curse of dimensionality), which makes recall numbers meaningless and hides the
#    behaviour you are trying to observe.

# TODO 3. Time the same k=10 query three ways. Run each 3+ times and take the MEDIAN —
#    the first call pays for opening files and warming page cache, and reporting that
#    number is how people talk themselves into imaginary regressions:
#      a. small,         no index          -> expect single-digit ms
#      b. large_noindex, no index          -> expect hundreds of ms to seconds
#      c. large_indexed, index             -> expect low ms
#    ds.to_table(nearest={"column": "vector", "q": q, "k": 10})

# TODO 4. Print the ratio b/c. That single number is the argument for the warning and
#    belongs in your PR description.

# TODO 5. Prove which path each query actually took, rather than trusting the timings:
#    print(ds.scanner(nearest={...}).explain_plan(verbose=True))
#    - brute force  -> a `KNNVectorDistance` node over a LanceRead/scan, no ANN node
#    - indexed      -> an `ANNSubIndex` node appears
#    Assert on this. The Rust test at rust/lance/src/dataset/scanner.rs:9873 asserts
#    exactly this way ("Expected flat search, but got ANN index in plan"), so you are
#    matching an existing convention.

# TODO 6. Now the actual point: capture stderr around case (b) and assert it is EMPTY.
#    Simplest honest way is to run the query in a subprocess and read its stderr:
#      subprocess.run([sys.executable, "-c", "<query script>"],
#                     capture_output=True, env={**os.environ, "LANCE_LOG": "warn"})
#    Today that assertion passes. After your change it must fail for the large case and
#    still pass for the small one — which is what example 02 checks.

# TODO 7. Also run the large query with use_index=False on the INDEXED dataset:
#      ds.to_table(nearest={..., "use_index": False})
#    Confirm the timing matches large_noindex. This is the "deliberate opt-out" case,
#    and open question 2 in the study is whether it should warn. Form your own opinion
#    here, with the timing in front of you.
