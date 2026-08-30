"""
Builds the three datasets 01_flat_search.py queries.

This is the first file, so it carries the environment. Per python/AGENTS.md lance uses
uv, and there is no venv to activate — that file says in as many words not to rely on an
activated environment. Once, from the checkout:

    cd ~/Coding/lance/python
    make install        # runs uv sync, which builds the pylance extension; expect it
                        # to take a while, and do not interrupt it

Then this file, from that same directory:

    uv run --frozen python <this file>

Tutorial level: partial — this file is setup, so it is complete. The stub is in 01.

Not run — Python, so there is nothing to compile. APIs read against lance 324cedd9d:
write_dataset at python/python/lance/dataset.py:7719, LanceDataset.create_index at
:4124, list_indices at :1122. The row counts are the ones an earlier run of this study
confirmed will train an IVF_PQ index; expect the third write to take most of the time.

Issue #804 turns on one distinction: a brute-force vector search that is cheap versus one
that is expensive. So you need both. The small dataset matters as much as the large one —
it is the case that must stay silent, and the case the issue author had in mind when they
warned against warning blindly.
"""

import shutil
from pathlib import Path

import lance
import numpy as np
import pyarrow as pa

ROOT = Path.home() / "lance-study-data"
DIM = 128
SMALL_ROWS = 2_000
LARGE_ROWS = 20_000


def make_table(rows: int, seed: int) -> pa.Table:
    """Unit-norm random vectors. Normalised so L2 and cosine rank identically."""
    rng = np.random.default_rng(seed)
    vectors = rng.standard_normal((rows, DIM), dtype=np.float32)
    vectors /= np.linalg.norm(vectors, axis=1, keepdims=True)
    return pa.table(
        {
            "id": pa.array(range(rows), pa.int32()),
            # FixedSizeList<float32, DIM> is what the vector path requires: the column
            # type is checked in Scanner::vector_search, rust/lance/src/dataset/
            # scanner.rs:5075. A plain List column is rejected before the query ever
            # reaches the KNN path.
            "vector": pa.FixedSizeListArray.from_arrays(
                pa.array(vectors.reshape(-1)), DIM
            ),
        }
    )


def write(name: str, rows: int, seed: int, *, index: bool) -> lance.LanceDataset:
    path = ROOT / f"{name}.lance"
    if path.exists():
        shutil.rmtree(path)
    dataset = lance.write_dataset(make_table(rows, seed), path, mode="overwrite")
    if index:
        # num_sub_vectors must divide DIM: 128 / 16 = 8 bytes per compressed vector.
        # This call is the slow one — training the IVF centroids dominates the script.
        dataset = dataset.create_index(
            "vector", index_type="IVF_PQ", num_partitions=16, num_sub_vectors=16
        )
    names = [i["name"] for i in dataset.list_indices()]
    print(f"  {name:16} {dataset.count_rows():>6} rows  indices={names or '[]'}")
    return dataset


def main() -> None:
    ROOT.mkdir(exist_ok=True)
    print(f"writing to {ROOT}")
    write("small", SMALL_ROWS, seed=0, index=False)
    # Same seed for both large datasets: the only difference must be the index.
    write("large_noindex", LARGE_ROWS, seed=1, index=False)
    write("large_indexed", LARGE_ROWS, seed=1, index=True)


if __name__ == "__main__":
    main()
