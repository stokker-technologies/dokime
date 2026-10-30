/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Deterministic Long-Execution Evidence Through The Real Runtime And SQLite.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    io::{BufWriter, Write},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, SamplingMode};
use sha2::{Digest, Sha256};

// Local Imports
use crate::service_support::{self as support, prelude::*};
use dokime_domain::model::reports::wire::ReportSubject;
use dokime_runtime::telemetry::TelemetryContract;
use dokime_runtime::{
    document::{Document, DocumentSection, Plot},
    operations::reports::MaterializeReport,
};

/* === Constants === */
const PAGE: u64 = 512;
const START: u64 = 1_788_220_800;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Fixture {
    _root: tempfile::TempDir,
    owner: dokime_runtime::Runtime,
    runtime: RuntimeHandle,
    execution: TestExecutionId,
    pending: TestExecutionId,
    rows: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    async fn new(hours: u64) -> Self {
        let root = tempfile::tempdir().unwrap();
        let executor = Arc::new(BlockingPublishingExecutor::new());
        let owner =
            dokime_runtime::Runtime::with_execution_fixture(support::config(root.path()), "testdb", executor.clone()).await.unwrap();
        let runtime = owner.handle();
        let mut suite = fixture_suite(&["passed", "passed"]);
        let document = document();
        for template in &mut suite.test_case_templates {
            template.attributes.clear();
            template.document = Some(document.clone());
        }
        register_custom_suite(&runtime, suite.clone()).await;
        let registration = runtime.test_suites().await.into_iter().find(|entry| entry.suite == suite).unwrap();
        let cases = materialize_cases(&registration.suite_id, &suite);
        let cycle = runtime
            .create_cycle(
                CycleDefinition {
                    metadata: metadata("bench.finalization", "Long Execution"),
                    continue_automatically: false,
                },
                CommandContext::default(),
            )
            .await
            .unwrap();
        runtime.append_fixture_cases(cycle.clone(), cases, CommandContext::default()).await.unwrap();
        let executions = runtime.watch_state().latest().state().cycles[&cycle].execution_ids.clone();
        let execution = executions[0].clone();
        let pending = executions[1].clone();
        let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        executor.wait_until_entered().await;
        let scope = runtime.watch_state().latest().state().executions[&execution].capture_scope();
        let context = dokime_runtime::domains::execution::authoring::SidecarStepContext::new(runtime.clone(), scope.clone());
        context.set_traceability_parameter_async("device.serial", "deterministic-obc-001").await.unwrap();

        let mut rows = 0;
        for contract in support::contracts::suite_telemetry_contracts() {
            // Synthetic Evidence Density Belongs To This Workload, Not The Plugin's Runtime Settings.
            let cadence = match contract.stream.as_str() {
                "host.system" | "host.network" | "host.wifi.link" => 1_000,
                "host.bluetooth" => 5_000,
                "host.wifi.latency" => 10_000,
                "host.wifi.scan" => 60_000,
                _ => contract.interval.as_ref().map_or(100, |interval| interval.0),
            };
            let count = hours * 3_600_000 / cadence;
            for start in (0..count).step_by(PAGE as usize) {
                let samples = (start..(start + PAGE).min(count))
                    .map(|index| {
                        let stamp = Timestamp::from_system_time(
                            SystemTime::UNIX_EPOCH + Duration::from_secs(START) + Duration::from_millis(index * cadence),
                        );
                        let id = dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("{}:{index}", contract.stream));
                        let mut sample = TelemetrySample::new(id, contract.stream.clone(), index, stamp, "testdb");
                        sample.scope = Some(scope.clone());
                        sample.values = values(&contract, index);
                        support::contracts::typed_fixture_sample(sample)
                    })
                    .collect();
                let batch = TelemetryBatch::new(dokime_runtime::model::identity::new_id(), contract.stream.clone(), samples);
                let receipt = runtime.publish_telemetry_batch(batch).await.unwrap();
                assert_eq!(receipt.dropped, 0);
            }
            rows += count;
        }
        for index in 0..hours * 60 {
            let mut record = support::evidence::custom_record();
            record.scope = Some(scope.clone());
            record.timestamp = Some(Timestamp::from_system_time(SystemTime::UNIX_EPOCH + Duration::from_secs(START + index * 60)));
            record.message = Some(format!("OBC status at minute {index}"));
            runtime.publish_record(record).await.unwrap();
        }
        let source = root.path().join("device.log");
        {
            let mut log = BufWriter::new(std::fs::File::create(&source).unwrap());
            for index in 0..hours * 3_600 {
                writeln!(log, "{index}: device connected; voltage=230; current=2.3").unwrap();
            }
            log.flush().unwrap();
        }
        let log_length = std::fs::metadata(&source).unwrap().len();
        let tail = runtime.start_execution_log_tail(execution.clone(), "logs/device.log", source).await.unwrap();
        // Persist The Historical Prefix During The Run, Leaving An Empty Tail.
        loop {
            let files = runtime.execution_files(&execution).await.unwrap();
            if files.iter().any(|file| file.relative_path == "logs/device.log" && file.size_bytes == log_length) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        runtime.stop_execution_log_tail(&tail).await.unwrap();
        runtime.storage().flush().await.unwrap();
        Self {
            _root: root,
            owner,
            runtime,
            execution,
            pending,
            rows,
        }
    }

    async fn report(&self) -> Duration {
        let at = self.runtime.storage().cursor_saved();
        let started = Instant::now();
        let report = self
            .runtime
            .operations()
            .call::<MaterializeReport>(MaterializeReport {
                subject: ReportSubject::Execution(self.execution.to_string()),
                at,
            })
            .await
            .unwrap();
        let mut source = self
            .runtime
            .storage()
            .read_file(harmos::storage::FileRead {
                source: harmos::storage::FileSource::Stored(report.report.clone()),
                range: harmos::storage::ByteRange { offset: 0, limit: None },
            })
            .await
            .unwrap();
        let mut destination = BufWriter::new(tempfile::tempfile().unwrap());
        let mut digest = Sha256::new();
        let mut length = 0;
        while let Some(batch) = source.next().await.unwrap() {
            assert_eq!(batch.offset(), length);
            destination.write_all(batch.bytes()).unwrap();
            digest.update(batch.bytes());
            length += batch.bytes().len() as u64;
        }
        source.completion().unwrap();
        destination.flush().unwrap();
        destination.get_ref().sync_all().unwrap();
        let elapsed = started.elapsed();
        assert_eq!(length, report.report.length);
        assert_eq!(digest.finalize().as_slice(), report.report.sha256.as_slice());
        eprintln!("report_bytes={length}");
        elapsed
    }

    async fn measure(mut self, hours: u64) -> (Duration, Duration, Duration) {
        let report = self.report().await;
        eprintln!("hours={hours} report_save_ms={}", report.as_secs_f64() * 1000.0);
        let started = Instant::now();
        let stopped = self.runtime.stop_execution(self.execution.clone(), CommandContext::default()).await;
        let stop = started.elapsed();
        eprintln!("hours={hours} stop_ms={} result={stopped:?}", stop.as_secs_f64() * 1000.0);
        stopped.unwrap();
        let started = Instant::now();
        let cancelled = self.runtime.cancel_execution(self.pending.clone(), CommandContext::default()).await;
        let cancel = started.elapsed();
        eprintln!("hours={hours} cancel_ms={} result={cancelled:?}", cancel.as_secs_f64() * 1000.0);
        cancelled.unwrap();
        println!(
            "{}",
            serde_json::json!({"measurement":"long_execution", "hours":hours, "rows":self.rows,
            "stop_ms":stop.as_secs_f64()*1000.0, "cancel_ms":cancel.as_secs_f64()*1000.0, "report_save_ms":report.as_secs_f64()*1000.0})
        );
        drop(self.runtime);
        self.owner.shutdown().await.unwrap();
        (stop, cancel, report)
    }
}

/* === Public Functions === */

pub fn benchmark(criterion: &mut Criterion) {
    let Some(mode) = std::env::var_os("DOKIME_FINALIZE_MEASURE") else {
        return;
    };
    let executor = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    if mode != "criterion" {
        tracing_subscriber::fmt()
            .with_env_filter("off,dokime_runtime::report=trace,dokime_runtime::domains::execution=debug,dokime_runtime::runtime::operations::provider=debug,dokime_runtime::runtime::storage=debug,dokime_runtime::storage::reports=debug")
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .try_init().unwrap();
        for hours in [1, 4] {
            executor.block_on(async { Fixture::new(hours).await.measure(hours).await });
        }
        return;
    }
    let mut group = criterion.benchmark_group("long_execution");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    for hours in [1, 4] {
        for measurement in ["stop", "cancel", "report_save"] {
            group.bench_function(BenchmarkId::new(measurement, hours), |bencher| {
                bencher.iter_custom(|iterations| {
                    executor.block_on(async {
                        let mut elapsed = Duration::ZERO;
                        for _ in 0..iterations {
                            let (stop, cancel, report) = Fixture::new(hours).await.measure(hours).await;
                            elapsed += match measurement {
                                "stop" => stop,
                                "cancel" => cancel,
                                _ => report,
                            };
                        }
                        elapsed
                    })
                });
            });
        }
    }
    group.finish();
}

/* === Private Functions === */

fn values(contract: &TelemetryContract, index: u64) -> BTreeMap<String, Value> {
    contract
        .columns
        .iter()
        .filter_map(|column| {
            let field = column.metadata.identifier.to_string();
            let mixed = index.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            let fraction = (mixed >> 11) as f64 / (1_u64 << 53) as f64;
            let value = match column.value_type {
                dokime_runtime::ValueType::Integer => {
                    Value::Integer(if field.ends_with("_bytes") { 8 * 1024 * 1024 * 1024 + index as i64 * 4096 } else { index as i64 })
                }
                dokime_runtime::ValueType::Number => Value::Number(match field.as_str() {
                    "elapsed_s" => index as f64 * 0.1,
                    "distance_m" => index as f64 * 0.02,
                    "speed_m_per_s" | "network_recv_mb_per_second" => 2.0 + fraction,
                    "wifi_signal_percent" => 70.0 + fraction * 10.0,
                    _ => 20.0 + fraction * 40.0,
                }),
                dokime_runtime::ValueType::Boolean => Value::Bool(true),
                dokime_runtime::ValueType::Array => Value::Array(Vec::new()),
                _ => return None,
            };
            Some((field, value))
        })
        .collect()
}

fn document() -> Document {
    let mut document = Document::new("default", "Long Execution Evidence");
    let mut section = DocumentSection::new("Measurements");
    for (stream, field) in [
        ("tracking", "distance_m"),
        ("tracking", "speed_m_per_s"),
        ("host.system", "cpu_load_percent"),
        ("host.network", "network_recv_mb_per_second"),
        ("host.wifi.link", "wifi_signal_percent"),
    ] {
        let mut plot = Plot::time_series(field);
        plot.id(format!("{stream}.{field}"));
        plot.y(dokime_runtime::expressions::telemetry(stream, field));
        section.plot(plot);
    }
    document.section(section);
    document
}

/* === Tests === */
