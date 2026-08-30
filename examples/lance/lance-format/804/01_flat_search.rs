//! Times the same vector query with and without an index, and shows that the slow path
//! says nothing — the same probe as `01_flat_search.py`, from the other side.
//!
//! The Python file shows what a user meets. This one calls the same path directly, so
//! once you change `KNNVectorDistanceExec` and re-run it, the `AFTER` block below is a
//! working acceptance check with no extension to rebuild in between.
//!
//! First, an environment. This file is a `[[bin]]` in a Cargo project one level up, at the
//! repository level of the study root — not in the issue directory, and not shipped here,
//! because its `lance` path dependency has to point at *your* checkout. `AGENTS.md` has the
//! manifest to write; it is a dozen lines and one `[[bin]]` per example. Once it exists:
//!
//!     cd <study root>/lance/lance-format     # the directory holding Cargo.toml
//!     RUST_LOG=warn cargo run --bin 804_flat_search
//!
//! The first build compiles lance's whole dependency graph — around nine hundred crates,
//! Arrow and DataFusion among them — so expect minutes and do not read it as a hang. Run
//! `00_build_datasets.py` first; this reads the datasets that writes.
//!
//! Tutorial level: partial — `median_ms` is yours to write.
//!
//! Draft: neither run nor compiled. APIs read against lance 324cedd9d — Dataset::open at
//! rust/lance/src/dataset.rs:514, Dataset::scan at :1674, Scanner::nearest at
//! rust/lance/src/dataset/scanner.rs:1813, use_index at :2089, limit at :1779,
//! explain_plan at :6756, try_into_batch at :2495. Expect a fix or two on first
//! `cargo check`; nothing here was verified by a compiler.

use std::path::PathBuf;

use lance::dataset::Dataset;
use lance::deps::arrow_array::{Array, FixedSizeListArray, Float32Array};

const K: usize = 10;
const REPEATS: usize = 5;

fn dataset_uri(name: &str) -> PathBuf {
    let home = std::env::var("HOME").expect("HOME is not set");
    PathBuf::from(home).join("lance-study-data").join(format!("{name}.lance"))
}

async fn open(name: &str) -> Dataset {
    let uri = dataset_uri(name);
    Dataset::open(uri.to_str().expect("non-UTF-8 path"))
        .await
        .unwrap_or_else(|e| panic!("{} — run 00_build_datasets.py first ({e})", uri.display()))
}

/// Median wall time of one nearest-neighbour query, in milliseconds.
///
/// Build a scan with `dataset.scan()`, call `nearest("vector", query, K)?` and
/// `use_index(use_index)` on it, then run `try_into_batch().await?` REPEATS times around
/// `std::time::Instant::now()`, and return the median of the millisecond timings. Median
/// rather than min or mean: the first call pays for opening files and warming the page
/// cache, and reporting that number is how imaginary regressions get filed.
///
/// That `try_into_batch` is the whole point of this file, so it is worth knowing where it
/// goes:
///   -> Scanner::vector_search, rust/lance/src/dataset/scanner.rs:5075
///      reads `q.use_index` at :5086 and looks for an index covering the column. Finding
///      none, it falls through to :5305, `// No index found. use flat search.`, then
///      Scanner::flat_knn at :6269 builds the scoring node.
///   -> KNNVectorDistanceExec, declared rust/lance/src/io/exec/knn.rs:150. execute() at
///      :869 streams every batch through the distance computation; the per-batch loop
///      starts at :904. This is the O(rows x dim) work, and where the warning would go.
///
///      With an index the same call diverges at :5305 into ANNIvfPartitionExec
///      (knn.rs:1164) and ANNIvfSubIndexExec (knn.rs:1381), which probe a few IVF
///      partitions instead of scanning everything.
async fn median_ms(_dataset: &Dataset, _query: &Float32Array, _use_index: bool) -> f64 {
    todo!("time REPEATS runs of the scan above and return the median in ms; the first run pays for page-cache warmup")
}

/// Read the plan rather than inferring the path from timings.
async fn uses_ann(dataset: &Dataset, query: &Float32Array, use_index: bool) -> bool {
    let mut scan = dataset.scan();
    scan.nearest("vector", query as &dyn Array, K)
        .expect("nearest rejected the column or the query type");
    scan.use_index(use_index);
    let plan = scan.explain_plan(true).await.expect("explain_plan failed");
    // The tests in the same file assert on this string: scanner.rs:9363 for the ANN path,
    // :9872 for the flat one.
    plan.contains("ANNSubIndex")
}

/// Take the query FROM the data. A random uniform vector in high-dimensional space is
/// near-equidistant from everything, which makes recall meaningless.
async fn first_vector(dataset: &Dataset) -> Float32Array {
    let mut scan = dataset.scan();
    scan.limit(Some(1), None).expect("limit rejected");
    let batch = scan.try_into_batch().await.expect("scan failed");
    let column = batch
        .column_by_name("vector")
        .expect("no vector column")
        .as_any()
        .downcast_ref::<FixedSizeListArray>()
        .expect("vector column is not a FixedSizeList");
    column
        .value(0)
        .as_any()
        .downcast_ref::<Float32Array>()
        .expect("vector values are not f32")
        .clone()
}

#[tokio::main]
async fn main() {
    let small = open("small").await;
    let noindex = open("large_noindex").await;
    let indexed = open("large_indexed").await;

    let query = first_vector(&noindex).await;

    let cases: [(&str, &Dataset, bool); 4] = [
        ("small (2k, no index)", &small, true),
        ("large (20k, no index)", &noindex, true),
        ("large (20k, indexed)", &indexed, true),
        ("large (20k, use_index=false)", &indexed, false),
    ];
    for (label, dataset, use_index) in cases {
        let elapsed = median_ms(dataset, &query, use_index).await;
        let path = if uses_ann(dataset, &query, use_index).await { "ANN" } else { "flat" };
        println!("  {label:32} {elapsed:7.1} ms   plan={path}");
    }

    // Which plan was chosen is structural: it holds on any machine, at any size.
    assert!(!uses_ann(&noindex, &query, true).await, "expected the flat path");
    assert!(uses_ann(&indexed, &query, true).await, "expected the ANN path");
    assert!(!uses_ann(&indexed, &query, false).await, "use_index=false must be flat");

    println!("  nothing was written to stderr: the slow path is silent");
}

// ---- AFTER ----
//
// This file does not change. One thing does: a sufficiently large unindexed query gains a
// line on stderr, once per query.
//
//   WARN lance::io::exec::knn: brute-force vector search scored 20000 rows on column
//   "vector"; consider creating a vector index on this column
//
// Once — not per RecordBatch, and not per DataFusion partition. KNNVectorDistanceExec
// inherits its input's partitioning in the non-batch case, so execute() at knn.rs:869
// runs once per partition; a latch created inside it would fire repeatedly. The FTS
// precedent latches with AtomicBool::swap at
// rust/lance-index/src/scalar/inverted/index/flat_search.rs:448.
//
// Running this binary is how you check that, without touching the project's own tests:
// make the change in your checkout, `cargo run --bin 804_flat_search` again, and the line
// either appears once or it does not.
