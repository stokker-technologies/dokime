/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plot Benchmarks: Durable Telemetry Visibility Through Native Storage

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, Throughput};
use futures_util::FutureExt;
use harmos::{Message, schema::Described};

// Local Imports
use dokime_runtime::runtime::query::{PlotPayloadRequest, RuntimePlotQueryMode};
use dokime_runtime::telemetry::{TelemetryBatch, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetrySample};
use dokime_runtime::{Identifier, Metadata, ValueType};
use dokime_runtime::{InstalledPlugin, Runtime, RuntimeConfig};

/* === Constants === */
const OWNER: &str = "bench_plot";
const STREAM: &str = "bench.plot";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
struct PlotRow {
    timestamp: String,
    value: Option<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    if std::env::var_os("DOKIME_PLOT_MEASURE").is_some() {
        executor.block_on(measure_queries());
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let mut owner = executor.block_on(Runtime::start(config(root.path()))).unwrap();
    let runtime = owner.handle();

    let mut group = criterion.benchmark_group("plot_saved_visibility");
    for rows in [1_usize, 64] {
        group.throughput(Throughput::Elements(rows as u64));
        group.bench_function(BenchmarkId::from_parameter(rows), |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let batch = batch(rows);
                        let prior = runtime.storage().cursor_saved();
                        let started = Instant::now();
                        runtime.publish_telemetry_batch(batch).await.unwrap();
                        let accepted = runtime.storage().cursor_accepted();
                        runtime.storage().await_durable(&accepted).await.unwrap();
                        let saved = runtime.storage().wait_saved_after(&prior).await.unwrap();
                        elapsed += started.elapsed();
                        assert!(saved.covers(&accepted).unwrap());
                    }
                    elapsed
                })
            });
        });
    }
    group.finish();

    drop(runtime);
    executor.block_on(owner.shutdown()).unwrap();
}

/* === Private Functions === */

async fn measure_queries() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = Runtime::start(config(root.path())).boxed_local().await.unwrap();
    let runtime = owner.handle();
    let mut saved = 0;
    for rows in [40_000, 1_000_000] {
        while saved < rows {
            let count = (rows - saved).min(2_000);
            runtime.publish_telemetry_batch(batch(count)).await.unwrap();
            saved += count;
        }
        runtime.storage().flush().await.unwrap();
        let request = PlotPayloadRequest {
            stream: STREAM.into(),
            x_key: Some("timestamp".into()),
            y_keys: vec!["value".into()],
            max_points: Some(1_500),
            ..Default::default()
        };
        let started = Instant::now();
        let result = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).boxed_local().await;
        let elapsed = started.elapsed();
        match result {
            Ok(envelopes) => {
                let series = &envelopes[0].payload.series[0];
                assert_eq!(series.raw_points, rows);
                assert!(series.points.len() <= 1_500);
                println!(
                    "{}",
                    serde_json::json!({"measurement":"plot_query", "rows":rows, "points":series.points.len(), "elapsed_ns":elapsed.as_nanos()})
                );
            }
            Err(error) => println!(
                "{}",
                serde_json::json!({"measurement":"plot_query", "rows":rows, "elapsed_ns":elapsed.as_nanos(), "error":error.to_string()})
            ),
        }
    }
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(super) fn config(root: &std::path::Path) -> RuntimeConfig {
    let schema = harmos::schema::graph::Schema::from_declaration(PlotRow::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap();
    let plugin = InstalledPlugin {
        source: OWNER.into(),
        version: "bench".into(),
        description: "Stored plot visibility benchmark".into(),
        enabled: true,
        telemetry_contracts: vec![TelemetryContract {
            stream: identifier(STREAM),
            schema_version: 1,
            metadata: metadata(STREAM, "Plot Bench"),
            template_id: None,
            owner: None,
            storage_schema: Some(schema),
            columns: vec![TelemetryColumn::new(metadata("value", "Value"), ValueType::Number, TelemetryRole::Value)],
            default_x: Some("timestamp".to_owned()),
            interval: None,
            allow_extra_fields: false,
            scalar: false,
            validation_mode: None,
            foreign: false,
        }],
        ..Default::default()
    };
    RuntimeConfig::new(root, metadata("bench.plots", "Plot Bench")).with_installed_plugins([plugin])
}

pub(super) fn batch(rows: usize) -> TelemetryBatch {
    let stream = identifier(STREAM);
    let samples = (0..rows)
        .map(|index| {
            let timestamp = dokime_runtime::model::time::now_utc();
            let mut sample = TelemetrySample::new(dokime_runtime::model::identity::new_id(), stream.clone(), 0, timestamp.clone(), OWNER);
            let value = index as f64;
            sample.values.insert("value".to_owned(), value.into());
            sample.storage_payload = Some(
                PlotRow {
                    timestamp: timestamp.as_str().to_owned(),
                    value: Some(value),
                }
                .encode(),
            );
            sample
        })
        .collect();
    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream, samples)
}

fn metadata(identifier: &str, label: &str) -> Metadata {
    Metadata::new(self::identifier(identifier), label, None)
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

/* === Tests === */
