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
    QueryOutput, RowSlice,
    transform::{
        FieldPath, Transform,
        aggregate::{Aggregate, Measure},
        downsample::{Algorithm, Downsample},
        histogram::Histogram,
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
    let mut group = criterion.benchmark_group("storage_transforms");
    group.sample_size(10);
    for rows in [1_024usize, 16_384] {
        let fixture = executor.block_on(Fixture::new(4));
        let payloads = samples(rows);
        for (index, batch) in payloads.chunks(1024).enumerate() {
            executor.block_on(fixture.append((index * 1024) as u64, batch));
        }
        executor.block_on(fixture.storage.flush()).unwrap();
        let rms = Transform::Aggregate(Aggregate {
            groups: vec![],
            measures: vec![Measure {
                field: "payload.value".into(),
                statistic: harmos::Aggregate::Rms,
                name: "rms".into(),
            }],
        });
        executor.block_on(async {
            let mut reader = fixture
                .storage
                .query(fixture.plan(vec![rms.clone()], RowSlice::default()))
                .await
                .unwrap();
            let mut actual = None;
            while let Some(output) = reader.next().await.unwrap() {
                if let QueryOutput::Batch(batch) = output {
                    actual = Some(
                        FieldPath::from("rms")
                            .value(batch.as_record_batch(), 0)
                            .unwrap(),
                    );
                }
            }
            match actual.unwrap() {
                harmos::storage::reduce::ScalarValue::Float(value) => {
                    assert!((value - 325.0 / 2.0f64.sqrt()).abs() < 1e-9)
                }
                _ => panic!("unexpected RMS representation"),
            }
        });
        group.throughput(Throughput::Elements(rows as u64));
        for (name, transform) in [
            ("rms", rms),
            (
                "histogram",
                Transform::Histogram(Histogram {
                    field: "payload.value".into(),
                    edges: vec![-325.0, -162.5, 0.0, 162.5, 325.0],
                }),
            ),
            (
                "downsample",
                Transform::Downsample(Downsample {
                    max_rows: 256,
                    axis: None,
                    algorithm: Algorithm::Extrema,
                    value: Some("payload.value".into()),
                }),
            ),
        ] {
            group.bench_function(BenchmarkId::new(name, rows), |bencher| {
                bencher.iter(|| {
                    std::hint::black_box(executor.block_on(
                        fixture.collect(fixture.plan(vec![transform.clone()], RowSlice::default())),
                    ))
                });
            });
        }
        executor.block_on(fixture.runtime.stop()).unwrap();
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
