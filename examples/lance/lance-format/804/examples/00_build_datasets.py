"""
Build the datasets that 01_flat_search.py queries.

Issue #804 turns on ONE distinction: a brute-force vector search that is cheap versus
one that is expensive. So you need both, and the small one matters as much as the large
one — it is the case that must stay silent, and it is the case the issue author was
worried about when they warned against warning blindly.

    --scale small     ~5k rows      brute force is correct here; must NEVER warn
    --scale large   ~500k rows      brute force is a mistake here; must warn once

    --real            use real OpenAI embeddings (1536-dim) instead of random floats

Why bother with --real: dimensionality drives the cost. Flat KNN is O(rows x dim), so
1536-dim OpenAI vectors are ~12x more expensive per row than 128-dim SIFT, and random
uniform vectors have unrealistically uniform distances. The real dataset is what makes
the timing gap in example 01 obvious rather than arguable.

    cd ~/Coding/lance/python
    uv run python <this file> --scale small
    uv run python <this file> --scale large
"""

# TODO 1. imports: argparse, pathlib, time, numpy as np, pyarrow as pa, lance

# TODO 2. Decide where datasets live. Put them OUTSIDE the lance checkout —
#    `*.lance` is gitignored there, but a 6GB dataset inside a repo you are about to
#    rebase is still a bad time. Suggest: ~/lance-study-data/

# TODO 3. Synthetic path (default; no download, works offline).
#    - DIM = 1536 to match the real dataset, so timings are comparable
#    - rows = 5_000 for small, 500_000 for large
#    - build the vector column as:
#        pa.FixedSizeListArray.from_arrays(
#            pa.array(np.random.rand(rows * DIM).astype(np.float32)), DIM)
#      FixedSizeList<float32, DIM> is what `get_vector_type` in scanner.rs requires;
#      a plain List column will be rejected before you reach the KNN path at all.
#    - add an "id" column so you can tell rows apart in output
#    - generate in chunks (say 50k rows) and pass an iterator of RecordBatch to
#      lance.write_dataset — building 500k x 1536 float32 in one numpy array is ~3GB
#      resident and will thrash before it writes.

# TODO 4. Real path (--real): KShivendu/dbpedia-entities-openai-1M, 1M x 1536 OpenAI
#    embeddings. Lance already ships a converter for exactly this dataset:
#
#        ~/Coding/lance/benchmarks/dbpedia-openai/datagen.py
#
#    Read it before writing anything — it shows the FixedSizeList conversion
#    (`to_fixed_size_array`) and the schema Lance wants. Two options:
#      a. shell out to it:  uv run ./datagen.py -o <path>   (downloads all 1M, ~6GB)
#      b. adapt its `convert_dataset()` with `load_dataset(..., split="train[:500000]")`
#         to take a subset — faster, and 500k is already far past any sane threshold.
#    Requires `datasets` (see benchmarks/dbpedia-openai/requirements.txt).

# TODO 5. Write TWO copies of the large dataset. This is the part people skip and then
#    cannot explain their own results:
#      - large_noindex.lance   left with no vector index
#      - large_indexed.lance   same rows, then create_index(...)
#    Comparing an indexed and unindexed query on the SAME data is the only way to
#    attribute a timing difference to the index rather than to the data.
#
#    ds.create_index("vector", index_type="IVF_PQ",
#                    num_partitions=256, num_sub_vectors=96)
#    (num_sub_vectors must divide DIM: 1536 / 96 = 16.)
#    Expect index construction to take minutes on 500k rows. That is k-means running.

# TODO 6. Print what you built — path, row count, dim, on-disk size, and whether an
#    index exists (ds.list_indices()). Example 01 should be able to assume the datasets
#    are there and correct.
