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
    let mut group = criterion.benchmark_group("storage_ingestion");
    group.sample_size(10);
    for rows in [1_024usize, 16_384] {
        let payloads = samples(rows);
        let items = (0..rows)
            .map(|index| id(&format!("sample-{index}")))
            .collect::<Vec<_>>();
        group.throughput(Throughput::Elements(rows as u64));
        for durable in [false, true] {
            let name = if durable { "durable" } else { "accepted" };
            group.bench_function(BenchmarkId::new(name, rows), |bencher| {
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
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
