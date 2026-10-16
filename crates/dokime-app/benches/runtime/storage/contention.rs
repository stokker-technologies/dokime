/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use criterion::{BenchmarkId, Throughput};
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
    let mut group = criterion.benchmark_group("storage_contention");
    group.sample_size(10);
    let payloads = samples(4096);
    let update = samples(256);
    let update_items = (4096..4352)
        .map(|index| id(&format!("sample-{index}")))
        .collect::<Vec<_>>();
    group.throughput(Throughput::Elements(4096));
    for concurrency in [1usize, 2, 4] {
        group.bench_function(
            BenchmarkId::new("mixed_read_write", concurrency),
            |bencher| {
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
                            let reads = futures_util::future::join_all(
                                plans.into_iter().map(|plan| fixture.collect(plan)),
                            );
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
            },
        );
    }
    group.finish();
    // Sample Memory In A Separate Diagnostic Pass, Outside Criterion Timings.
    for concurrency in [1usize, 2, 4] {
        executor.block_on(diagnostic(concurrency, &payloads));
    }
}

/* === Private Functions === */
async fn diagnostic(concurrency: usize, payloads: &[Vec<u8>]) {
    use harmos::storage::query::QueryOutput;
    let fixture = Fixture::new(concurrency).await;
    for (index, batch) in payloads.chunks(1024).enumerate() {
        fixture.append((index * 1024) as u64, batch).await;
    }
    fixture.storage.flush().await.unwrap();
    let mut reader = fixture
        .storage
        .query(fixture.plan(vec![], RowSlice::default()))
        .await
        .unwrap();
    let following = fixture.plan(vec![], RowSlice::default());
    let mut peak_read = 0usize;
    let mut peak_pending = 0usize;
    let mut peak_rss = 0u64;
    let mut system = sysinfo::System::new();
    let pid = sysinfo::get_current_pid().unwrap();
    let slow_consumer = async {
        while let Some(output) = reader.next().await.unwrap() {
            if let QueryOutput::Batch(batch) = output {
                let status = fixture.storage.status();
                peak_read = peak_read.max(status.read_bytes);
                peak_pending = peak_pending.max(status.pending_bytes);
                system.refresh_processes_specifics(
                    sysinfo::ProcessesToUpdate::Some(&[pid]),
                    false,
                    sysinfo::ProcessRefreshKind::nothing().with_memory(),
                );
                peak_rss = peak_rss.max(system.process(pid).map_or(0, |process| process.memory()));
                // Retain One Batch Across A Scheduler Turn To Exercise Consumer Backpressure.
                tokio::task::yield_now().await;
                std::hint::black_box(batch);
            }
        }
        drop(reader);
    };
    let (_, (_, admission_to_first_batch)) =
        tokio::join!(slow_consumer, fixture.collect(following));
    eprintln!(
        "storage contention config active={concurrency} read_budget=134217728; sampled_read_peak={peak_read}; sampled_pending_peak={peak_pending}; sampled_process_rss_peak={peak_rss}; queued_query_first_batch_us={}",
        admission_to_first_batch.as_micros()
    );
    fixture.runtime.stop().await.unwrap();
}

/* === Tests === */
