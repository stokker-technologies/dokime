/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// DataFusion Spike: Identical Fixed-Cut Sample Workloads Before And After

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports
use criterion::{Criterion, SamplingMode, Throughput};
use harmos::Aggregate;

// Local Imports
use crate::query_telemetry::Fixture;
use dokime_domain::model::query::{
    predicate::{Operator, Predicate, Scalar},
    reduction::{BucketRequest, Output},
    selection::Window,
};
use dokime_operations::query::AggregateSamples;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn benchmark(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("spike_samples");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    let executor = tokio::runtime::Runtime::new().unwrap();
    let mut fixture = None;
    for kind in ["aggregate_1000000", "bucket_1000000", "percentiles_50000"] {
        group.throughput(Throughput::Elements(if kind.starts_with("percentiles") { 50_000 } else { 1_000_000 }));
        group.bench_function(kind, |bencher| {
            let fixture = fixture.get_or_insert_with(|| {
                executor.block_on(async {
                    let fixture = Fixture::new(|config| {
                        config.local_storage.telemetry_rows = std::num::NonZeroUsize::new(4096).unwrap();
                        config.query.scanned_rows = 32_000_000;
                    })
                    .await;
                    for start in (0..1_000_000).step_by(1024) {
                        let rows = (start..(start + 1024).min(1_000_000))
                            .map(|index| {
                                (index as u64, index as i64 * 1000, Some((index % 1000) as f64), if index % 2 == 0 { "a" } else { "b" })
                            })
                            .collect::<Vec<_>>();
                        fixture.publish(&rows).await;
                    }
                    fixture.runtime.storage().flush().await.unwrap();
                    fixture
                })
            });
            let mut request = fixture.request();
            request.fields = vec!["value".into()];
            request.page.rows = Some(1024);
            request.r#where = vec![Predicate {
                field: "counter".into(),
                operator: Operator::Gte,
                values: vec![Scalar::Unsigned(100_000)],
            }];
            request.between = Some(Window {
                start: Some(dokime_domain::model::identity::wire::Timestamp { seconds: 0, nanos: 100_000_000 }),
                end: Some(dokime_domain::model::identity::wire::Timestamp { seconds: 1, nanos: 0 }),
            });
            request.bucket = Some(BucketRequest {
                every_nanos: 100_000_000,
                aggregate: Aggregate::Mean,
            });
            let percentiles = kind.starts_with("percentiles");
            if percentiles {
                request.r#where.clear();
                request.between = Some(Window {
                    start: None,
                    end: Some(dokime_domain::model::identity::wire::Timestamp { seconds: 0, nanos: 50_000_000 }),
                });
            }
            let statistics = AggregateSamples {
                execution_id: request.execution_id.clone(),
                stream: request.stream.clone(),
                versions: Vec::new(),
                r#where: request.r#where.clone(),
                between: request.between.clone(),
                at: None,
                outputs: if percentiles {
                    vec![Aggregate::Median, Aggregate::Percentile(95)]
                } else {
                    vec![Aggregate::Count, Aggregate::Sum, Aggregate::Mean, Aggregate::Std, Aggregate::Rms, Aggregate::Min, Aggregate::Max]
                }
                .into_iter()
                .enumerate()
                .map(|(index, aggregate)| Output {
                    alias: format!("s{index}"),
                    field: "value".into(),
                    aggregate,
                })
                .collect(),
            };
            let mut verified = false;
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        eprintln!("SPIKE_QUERY_BEGIN {kind}");
                        let start = Instant::now();
                        if !kind.starts_with("bucket") {
                            let result = fixture.runtime.query_aggregate_samples(statistics.clone(), usize::MAX).await.unwrap();
                            elapsed += start.elapsed();
                            assert_eq!(result.selected_rows, if percentiles { 50_000 } else { 900_000 });
                            assert!(result.complete);
                            if !verified {
                                eprintln!(
                                    "SPIKE_RESULT {kind} {}",
                                    serde_json::to_string(&(result.selected_rows, &result.values, &result.quality_counts)).unwrap()
                                );
                                verified = true;
                            }
                        } else {
                            let result = fixture.runtime.query_list_samples(request.clone(), usize::MAX).await.unwrap();
                            elapsed += start.elapsed();
                            assert_eq!(result.rows.len(), 18);
                            assert!(result.complete);
                            if !verified {
                                let stable = result
                                    .rows
                                    .iter()
                                    .map(|row| {
                                        let dokime_domain::model::query::page::SampleRow::Bucket(bucket) = row else {
                                            panic!("bucket result")
                                        };
                                        serde_json::to_string(&(
                                            &bucket.start,
                                            &bucket.end,
                                            &bucket.source,
                                            &bucket.versions,
                                            bucket.sample_count,
                                            &bucket.fields,
                                        ))
                                        .unwrap()
                                    })
                                    .collect::<Vec<_>>();
                                eprintln!("SPIKE_RESULT {kind} {}", serde_json::to_string(&stable).unwrap());
                                verified = true;
                            }
                        }
                        eprintln!("SPIKE_QUERY_END {kind}");
                    }
                    elapsed
                })
            });
        });
    }
    group.finish();
    if let Some(mut fixture) = fixture {
        executor.block_on(fixture.owner.shutdown()).unwrap();
    }
}

/* === Private Functions === */

/* === Tests === */
