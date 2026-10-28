/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */
/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::{Duration, Instant, UNIX_EPOCH};

// Third Party Imports
use arrow_array::{Float64Array, UInt64Array};
use criterion::{BenchmarkId, Criterion, Throughput};
use harmos::storage::Query;
use harmos::storage::query::transform::{
    Transform,
    aggregate::{Aggregate, Measure},
    downsample::{Algorithm, Downsample},
    histogram::Histogram,
    project::{Project, Projection},
};
use harmos::storage::query::{QueryArrow, RowSlice, Source, StreamSelection};

// Local Imports
use super::{fixture::Fixture, waveforms};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn run(criterion: &mut Criterion, executor: &tokio::runtime::Runtime) {
    let mut group = criterion.benchmark_group("storage/query");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
    for periods in [1, 6] {
        let samples = waveforms::samples(periods);
        let inputs = waveforms::inputs(&samples);
        for concurrency in [1, 2, 4] {
            let fixture = executor.block_on(Fixture::new());
            executor.block_on(async {
                fixture.storage.write_schema(waveforms::schema(), waveforms::definition()).await.unwrap();
                for inputs in inputs.chunks(192) {
                    let batch = inputs.to_vec();
                    fixture.storage.write_stream(batch).await.unwrap();
                }
                fixture.storage.flush().await.unwrap();
            });
            for kind in ["project", "rms", "histogram", "downsample"] {
                let query = plan(&fixture, periods, kind);
                let rows = executor.block_on(verify(&fixture, &query, kind, samples.len()));
                let throughput = if kind == "project" { rows } else { samples.len() };
                group.throughput(Throughput::Elements((throughput * concurrency) as u64));
                for metric in ["all-first-batches", "completion"] {
                    let case = if kind == "project" { format!("{metric}/{concurrency}") } else { format!("{kind}/{metric}/{concurrency}") };
                    group.bench_function(BenchmarkId::new(case, samples.len()), |bencher| {
                        bencher.iter_custom(|iterations| {
                            executor.block_on(async {
                                let mut elapsed = Duration::ZERO;
                                let mut resources = super::metrics::Observed::default();
                                for _ in 0..iterations {
                                    let probe = super::metrics::Probe::start(fixture.storage.clone());
                                    let openings = (0..concurrency).map(|_| async {
                                        let mut reader = fixture.storage.query(query.clone()).await.unwrap();
                                        assert!(matches!(reader.next().await.unwrap(), Some(QueryArrow::Schema(_))));
                                        let Some(QueryArrow::Batch(first)) = reader.next().await.unwrap() else {
                                            panic!("nonempty query");
                                        };
                                        (reader, first)
                                    });

                                    let started = Instant::now();
                                    let readers = futures_util::future::join_all(openings).await;
                                    let first_elapsed = started.elapsed();
                                    let completions = readers.into_iter().map(|(mut reader, first)| async move {
                                        let mut received = first.as_record_batch().num_rows();
                                        drop(first);
                                        while let Some(output) = reader.next().await.unwrap() {
                                            if let QueryArrow::Batch(batch) = output {
                                                received += batch.as_record_batch().num_rows();
                                            }
                                        }
                                        (received, reader.completion().unwrap().rows)
                                    });
                                    let completions = futures_util::future::join_all(completions).await;
                                    let completion = started.elapsed();
                                    resources.merge(probe.finish());
                                    elapsed += if metric == "all-first-batches" { first_elapsed } else { completion };
                                    for (received, completed) in completions {
                                        assert_eq!(received, rows);
                                        assert_eq!(completed, rows as u64);
                                    }
                                }
                                let metric = if kind == "project" { format!("query/{metric}") } else { format!("query/{kind}/{metric}") };
                                resources.report(&metric, samples.len(), concurrency, Some(&fixture.settings));
                                elapsed
                            })
                        });
                    });
                }
            }
            executor.block_on(fixture.finish());
        }
    }
    group.finish();
}
/* === Private Functions === */

fn plan(fixture: &Fixture, periods: usize, kind: &str) -> Query {
    let transform = match kind {
        "project" => Transform::Project(Project { fields: vec![Projection { field: "payload.value".into(), name: "value".into() }] }),
        "rms" => Transform::Aggregate(Aggregate {
            groups: vec![Projection { field: "payload.phase".into(), name: "phase".into() }],
            measures: vec![Measure { field: "payload.value".into(), statistic: harmos::Aggregate::Rms, name: "rms".into() }],
        }),
        "histogram" => Transform::Histogram(Histogram { field: "payload.value".into(), edges: vec![-12.0, -6.0, 0.0, 6.0, 12.0] }),
        "downsample" => Transform::Downsample(Downsample {
            max_rows: 128,
            algorithm: Algorithm::Extrema,
            axis: None,
            value: Some("payload.value".into()),
        }),
        _ => unreachable!("declared query case"),
    };
    Query {
        at: fixture.storage.cursor_saved(),
        source: Source::Stream(StreamSelection {
            schema: waveforms::schema(),
            scope: None,
            item: None,
            stream: None,
            source_start: Some(UNIX_EPOCH),
            source_end: (kind == "project")
                .then_some(UNIX_EPOCH + Duration::from_millis((periods * waveforms::SAMPLES_PER_PERIOD / 2) as u64)),
            after: 0,
        }),
        transforms: vec![transform],
        slice: RowSlice { offset: if kind == "project" { 3 } else { 0 }, limit: None },
    }
}

async fn verify(fixture: &Fixture, query: &Query, kind: &str, input_rows: usize) -> usize {
    let mut reader = fixture.storage.query(query.clone()).await.unwrap();
    let mut rows = 0usize;
    let mut histogram_count = 0u64;
    while let Some(output) = reader.next().await.unwrap() {
        let QueryArrow::Batch(batch) = output else {
            continue;
        };
        let batch = batch.as_record_batch();
        rows += batch.num_rows();
        match kind {
            "rms" => {
                let values = batch.column_by_name("rms").unwrap().as_any().downcast_ref::<Float64Array>().unwrap();
                for value in values.values() {
                    assert!((value - waveforms::AMPLITUDE / 2.0_f64.sqrt()).abs() < 1e-12);
                }
            }
            "histogram" => {
                let counts = batch.column_by_name("count").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
                histogram_count += counts.values().iter().sum::<u64>();
            }
            "downsample" => {
                assert!(batch.column_by_name("payload").is_some());
            }
            "project" => {
                assert!(batch.column_by_name("value").is_some());
            }
            _ => unreachable!("declared query case"),
        }
    }
    assert_eq!(reader.completion().unwrap().rows, rows as u64);
    match kind {
        "project" => assert_eq!(rows, input_rows / 2 - 3),
        "rms" => assert_eq!(rows, 3),
        "histogram" => assert_eq!(histogram_count, input_rows as u64),
        "downsample" => assert!((1..=128).contains(&rows)),
        _ => unreachable!("declared query case"),
    }
    rows
}

/* === Tests === */
