/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story: Projected Query Paging

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
    let mut completes = criterion.benchmark_group("query_completes_slice");
    for rows in [1_024usize, 16_384] {
        let fixture = loaded(&executor, rows);
        let transforms = projection();
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
        completes.throughput(Throughput::Elements((rows / 2) as u64));
        completes.bench_function(BenchmarkId::from_parameter(rows), |bencher| {
            bencher.iter(|| {
                std::hint::black_box(
                    executor.block_on(fixture.collect(fixture.plan(transforms.clone(), slice))),
                )
            });
        });
        executor.block_on(fixture.runtime.stop()).unwrap();
    }
    completes.finish();

    // Time to the first batch: the reader drains outside the measured interval.
    let mut first = criterion.benchmark_group("query_delivers_batch");
    for rows in [1_024usize, 16_384] {
        let fixture = loaded(&executor, rows);
        let transforms = projection();
        let slice = RowSlice {
            offset: (rows / 4) as u64,
            limit: Some((rows / 2) as u64),
        };
        first.bench_function(BenchmarkId::from_parameter(rows), |bencher| {
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
    first.finish();
}

/* === Private Functions === */
fn projection() -> Vec<Transform> {
    vec![Transform::Project(Project {
        fields: vec![Projection {
            field: "payload.value".into(),
            name: "voltage".into(),
        }],
    })]
}

/* === Tests === */
