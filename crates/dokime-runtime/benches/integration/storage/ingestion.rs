/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story: Stream Batch Admission And Durability

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use criterion::{BenchmarkId, SamplingMode, Throughput};

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
    for (name, durable) in [
        ("storage_accepts_batches", false),
        ("storage_persists_batches", true),
    ] {
        let mut group = criterion.benchmark_group(name);
        // Every iteration opens a fresh store: equal iteration counts per sample.
        group.sampling_mode(SamplingMode::Flat);
        for rows in [1_024usize, 16_384] {
            let payloads = samples(rows);
            let items = (0..rows)
                .map(|index| id(&format!("sample-{index}")))
                .collect::<Vec<_>>();
            group.throughput(Throughput::Elements(rows as u64));
            group.bench_function(BenchmarkId::from_parameter(rows), |bencher| {
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let fixture = executor.block_on(Fixture::new(4));
                        let started = std::time::Instant::now();
                        let accepted =
                            executor.block_on(fixture.append_with_ids(0, &payloads, &items));
                        if durable {
                            executor.block_on(fixture.storage.flush()).unwrap();
                        }
                        elapsed += started.elapsed();
                        executor.block_on(fixture.storage.flush()).unwrap();
                        assert!(fixture.storage.cursor_saved().covers(&accepted).unwrap());
                        executor.block_on(fixture.runtime.stop()).unwrap();
                    }
                    elapsed
                });
            });
        }
        group.finish();
    }
}

/* === Private Functions === */

/* === Tests === */
