/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Capture Benchmarks: Record Admission And Saved Replay Through The Runtime

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, SamplingMode, Throughput};
use tempfile::TempDir;

// Local Imports
use dokime_runtime::core::{Identifier, Metadata};
use dokime_runtime::{
    Runtime, RuntimeConfig, RuntimeHandle,
    records::{
        ProductCalibrationDetails, ProductCalibrationKind, ProductCalibrationPhase,
        ProductRecordDraft,
    },
};

/* === Constants === */
const COUNTS: [u64; 3] = [500, 1_000, 2_000];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// A started runtime on a fresh storage root; setup and teardown stay untimed.
struct Fixture {
    _root: TempDir,
    owner: Runtime,
    handle: RuntimeHandle,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Fixture {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let config = RuntimeConfig::new(
            root.path(),
            Metadata::new(Identifier::new("runtime.hub").unwrap(), "Runtime Hub", None),
        );
        let owner = Runtime::start(config).await.unwrap();
        let handle = owner.handle();
        handle.storage().flush().await.unwrap();
        Self {
            _root: root,
            owner,
            handle,
        }
    }

    async fn publish(&self, count: u64) {
        for draft in drafts(count) {
            self.handle.publish_product_record(draft).await.unwrap();
        }
    }

    async fn stop(self) {
        drop(self.handle);
        let mut owner = self.owner;
        owner.shutdown().await.unwrap();
    }
}

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    let mut group = criterion.benchmark_group("runtime_admits_records");
    // Every iteration starts a runtime: equal iteration counts per sample.
    group.sampling_mode(SamplingMode::Flat);
    for count in COUNTS {
        group.throughput(Throughput::Elements(count));
        group.bench_function(BenchmarkId::from_parameter(count), |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let fixture = Fixture::new().await;
                        let started = Instant::now();
                        fixture.publish(count).await;
                        elapsed += started.elapsed();
                        fixture.stop().await;
                    }
                    elapsed
                })
            });
        });
    }
    group.finish();

    let mut group = criterion.benchmark_group("runtime_replays_records");
    group.sampling_mode(SamplingMode::Flat);
    for count in COUNTS {
        group.throughput(Throughput::Elements(count));
        group.bench_function(BenchmarkId::from_parameter(count), |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let fixture = Fixture::new().await;
                        fixture.publish(count).await;
                        let captured = fixture.handle.captured_records();
                        let frontier = captured.frontier();
                        assert!(
                            captured.wait_after(frontier - 1).await.unwrap().unwrap() >= frontier
                        );
                        drop(captured);
                        fixture.handle.storage().flush().await.unwrap();
                        // A fresh reader after the save barrier replays saved
                        // storage rather than the manager's unsaved buffer.
                        let reader = fixture.handle.captured_records();
                        let started = Instant::now();
                        let mut after = 0;
                        let mut received = 0;
                        while let Some(record) = reader.next(after, frontier).await.unwrap() {
                            after = record.sequence;
                            received += 1;
                        }
                        elapsed += started.elapsed();
                        assert_eq!(received, count);
                        drop(reader);
                        fixture.stop().await;
                    }
                    elapsed
                })
            });
        });
    }
    group.finish();
}

/* === Private Functions === */
fn drafts(count: u64) -> Vec<ProductRecordDraft> {
    (0..count)
        .map(|_| {
            ProductCalibrationDetails::new(
                "benchmark-charger",
                ProductCalibrationKind::Field,
                ProductCalibrationPhase::Completed,
            )
            .into_draft("benchmark.charger")
        })
        .collect()
}

/* === Tests === */
