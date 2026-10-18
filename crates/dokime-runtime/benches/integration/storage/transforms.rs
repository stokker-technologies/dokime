/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Benchmark Story: Aggregate, Histogram And Downsample Queries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use criterion::{BenchmarkId, Throughput};
use harmos::storage::query::{
    QueryArrow, RowSlice,
    transform::{
        FieldPath, FieldPathExecution, Transform,
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
    let fixtures = [1_024usize, 16_384].map(|rows| (rows, loaded(&executor, rows)));
    for (_, fixture) in &fixtures {
        executor.block_on(assert_rms(fixture));
    }
    for (name, transform) in [
        ("transform_computes_rms", rms()),
        (
            "transform_bins_histogram",
            Transform::Histogram(Histogram { field: "payload.value".into(), edges: vec![-325.0, -162.5, 0.0, 162.5, 325.0] }),
        ),
        (
            "transform_downsamples_extrema",
            Transform::Downsample(Downsample {
                max_rows: 256,
                axis: None,
                algorithm: Algorithm::Extrema,
                value: Some("payload.value".into()),
            }),
        ),
    ] {
        let mut group = criterion.benchmark_group(name);
        for (rows, fixture) in &fixtures {
            group.throughput(Throughput::Elements(*rows as u64));
            group.bench_function(BenchmarkId::from_parameter(rows), |bencher| {
                bencher.iter(|| {
                    std::hint::black_box(executor.block_on(fixture.collect(fixture.plan(vec![transform.clone()], RowSlice::default()))))
                });
            });
        }
        group.finish();
    }
    for (_, fixture) in fixtures {
        executor.block_on(fixture.runtime.stop()).unwrap();
    }
}

/* === Private Functions === */
fn rms() -> Transform {
    Transform::Aggregate(Aggregate {
        groups: vec![],
        measures: vec![Measure { field: "payload.value".into(), statistic: harmos::Aggregate::Rms, name: "rms".into() }],
    })
}

async fn assert_rms(fixture: &Fixture) {
    let mut reader = fixture.storage.query(fixture.plan(vec![rms()], RowSlice::default())).await.unwrap();
    let mut actual = None;
    while let Some(output) = reader.next().await.unwrap() {
        if let QueryArrow::Batch(batch) = output {
            actual = Some(FieldPath::from("rms").value(batch.as_record_batch(), 0).unwrap());
        }
    }
    match actual.unwrap() {
        harmos::storage::query::scalar::ScalarValue::Float(value) => {
            assert!((value - 325.0 / 2.0f64.sqrt()).abs() < 1e-9)
        }
        _ => panic!("unexpected RMS representation"),
    }
}

/* === Tests === */
