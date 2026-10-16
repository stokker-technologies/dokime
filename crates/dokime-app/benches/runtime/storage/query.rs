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
        project::{Project, Projection},
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
    let mut group = criterion.benchmark_group("storage_query");
    group.sample_size(10);
    for rows in [1_024usize, 16_384] {
        let fixture = executor.block_on(Fixture::new(4));
        let payloads = samples(rows);
        for (index, batch) in payloads.chunks(1024).enumerate() {
            executor.block_on(fixture.append((index * 1024) as u64, batch));
        }
        executor.block_on(fixture.storage.flush()).unwrap();
        let transforms = vec![Transform::Project(Project {
            fields: vec![Projection {
                field: "payload.value".into(),
                name: "voltage".into(),
            }],
        })];
        let slice = RowSlice {
            offset: (rows / 4) as u64,
            limit: Some((rows / 2) as u64),
        };
        assert_eq!(
            executor
                .block_on(fixture.collect(fixture.plan(transforms.clone(), slice)))
                .0,
            (rows / 2) as u64
        );
        group.throughput(Throughput::Elements((rows / 2) as u64));
        group.bench_function(BenchmarkId::new("completion", rows), |bencher| {
            bencher.iter(|| {
                std::hint::black_box(
                    executor.block_on(fixture.collect(fixture.plan(transforms.clone(), slice))),
                )
            });
        });
        group.bench_function(BenchmarkId::new("first_batch", rows), |bencher| {
            bencher.iter_custom(|iterations| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    elapsed += executor
                        .block_on(fixture.collect(fixture.plan(transforms.clone(), slice)))
                        .1;
                }
                elapsed
            });
        });
        executor.block_on(fixture.runtime.stop()).unwrap();
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
