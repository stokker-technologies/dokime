/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plot Benchmarks: Live Telemetry Delta Delivery To A Plot Watch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::time::{Duration, Instant};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, Throughput};
use futures_util::StreamExt;
use harmos::{Message, schema::Described};

// Local Imports
use dokime_runtime::telemetry::{TelemetryBatch, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetrySample};
use dokime_runtime::{Identifier, Metadata, Timestamp, ValueType};
use dokime_runtime::{InstalledPlugin, Runtime, RuntimeConfig};

/* === Constants === */
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
/// Times publication of one telemetry batch until the plot watch has
/// delivered its last row; batch construction stays outside the interval.
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let root = tempfile::tempdir().unwrap();
    let mut owner = executor.block_on(Runtime::start(config(root.path()))).unwrap();
    let runtime = owner.handle();
    let mut watch = executor.block_on(runtime.watch_plot_telemetry(&[])).unwrap().pop().expect("the telemetry stream is watchable");

    let mut group = criterion.benchmark_group("plot_pushes_deltas");
    for rows in [1_usize, 64] {
        group.throughput(Throughput::Elements(rows as u64));
        group.bench_function(BenchmarkId::from_parameter(rows), |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let batch = batch(rows);
                        let started = Instant::now();
                        let receipt = runtime.publish_telemetry_batch(batch).await.unwrap();
                        let last = receipt.stream_cursors.values().copied().max().unwrap();
                        while watch.next().await.expect("the watch closes only with the runtime") < last {}
                        elapsed += started.elapsed();
                    }
                    elapsed
                })
            });
        });
    }
    group.finish();

    drop(watch);
    drop(runtime);
    executor.block_on(owner.shutdown()).unwrap();
}

/* === Private Functions === */
fn config(root: &std::path::Path) -> RuntimeConfig {
    let schema = harmos::schema::graph::Schema::from_declaration(PlotRow::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap();
    let plugin = InstalledPlugin {
        source: STREAM.into(),
        version: "bench".into(),
        description: "Plot delta benchmark".into(),
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
            plot_window: None,
            allow_extra_fields: false,
            scalar: false,
            validation_mode: None,
            foreign: false,
        }],
        ..Default::default()
    };
    RuntimeConfig::new(root, metadata("bench.plots", "Plot Bench")).with_installed_plugins([plugin])
}

fn batch(rows: usize) -> TelemetryBatch {
    let stream = identifier(STREAM);
    let samples = (0..rows)
        .map(|index| {
            let timestamp = dokime_runtime::model::time::now_utc();
            let mut sample = TelemetrySample::new(dokime_runtime::model::identity::new_id(), stream.clone(), 0, timestamp.clone(), STREAM);
            let value = index as f64;
            sample.values.insert("value".to_owned(), value.into());
            sample.storage_payload = Some(PlotRow { timestamp: timestamp.as_str().to_owned(), value: Some(value) }.encode_to_vec());
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
