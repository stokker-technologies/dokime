/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story: Concurrent Aggregate Reads Beside A Write

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use criterion::{BenchmarkId, SamplingMode, Throughput};
use harmos::storage::query::{
    RowSlice,
    transform::{
        Transform,
        aggregate::{Aggregate, Measure},
    },
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn benchmark(criterion: &mut Criterion) {
    let executor = executor();
    let mut group = criterion.benchmark_group("storage_mixes_traffic");
    // Every iteration opens a fresh store: equal iteration counts per sample.
    group.sampling_mode(SamplingMode::Flat);
    let payloads = samples(4096);
    let update = samples(256);
    let update_items = (4096..4352).map(|index| id(&format!("sample-{index}"))).collect::<Vec<_>>();
    group.throughput(Throughput::Elements(4096));
    for concurrency in [1usize, 2, 4] {
        group.bench_function(BenchmarkId::from_parameter(concurrency), |bencher| {
            bencher.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = executor.block_on(Fixture::new(concurrency));
                    for (index, batch) in payloads.chunks(1024).enumerate() {
                        executor.block_on(fixture.append((index * 1024) as u64, batch));
                    }
                    executor.block_on(fixture.storage.flush()).unwrap();
                    let plans = (0..4)
                        .map(|_| {
                            fixture.plan(
                                vec![Transform::Aggregate(Aggregate {
                                    groups: vec![],
                                    measures: vec![Measure {
                                        field: "payload.value".into(),
                                        statistic: harmos::Aggregate::Rms,
                                        name: "rms".into(),
                                    }],
                                })],
                                RowSlice::default(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let started = std::time::Instant::now();
                    executor.block_on(async {
                        let reads = futures_util::future::join_all(plans.into_iter().map(|plan| fixture.collect(plan)));
                        let write = async {
                            fixture.append_with_ids(4096, &update, &update_items).await;
                            fixture.storage.flush().await.unwrap();
                        };
                        let (results, ()) = tokio::join!(reads, write);
                        assert!(results.iter().all(|result| result.0 == 1));
                    });
                    elapsed += started.elapsed();
                    executor.block_on(fixture.runtime.stop()).unwrap();
                }
                elapsed
            });
        });
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
