/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Query Evaluation Benchmarks

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    sync::Arc,
    time::{Duration, Instant, UNIX_EPOCH},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion, SamplingMode};
use harmos::Message;

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::document::{
    Document, DocumentCard, DocumentSection, Plot, TelemetrySummaryDefinition, TelemetrySummaryGroupDefinition,
    TelemetrySummaryTileDefinition,
};
use dokime_runtime::runtime::query::plot_payload_request_for_descriptor;
use dokime_runtime::{CurrentDocumentEvent, CurrentDocumentQuery};

/* === Constants === */

const COUNTS: [usize; 3] = [307_200, 1_536_000, 3_072_000];
const DAYS: u64 = 3;
const PLOTS: usize = 30;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Fixture {
    owner: dokime_runtime::Runtime,
    runtime: RuntimeHandle,
    executor: Arc<BlockingPublishingExecutor>,
    cycle: TestCycleId,
    execution: TestExecutionId,
    rows: usize,
    plots: usize,
    _root: tempfile::TempDir,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    async fn new(rows: usize, plots: usize) -> Self {
        let root = tempfile::tempdir().unwrap();
        let executor = Arc::new(BlockingPublishingExecutor::new());
        let mut config = super::plots::config(root.path());
        let contract = config.installed_plugins[0].telemetry_contracts[0].clone();
        config.installed_plugins[0].telemetry_contracts = (0..plots)
            .map(|index| {
                let mut contract = contract.clone();
                contract.stream = identifier(&format!("bench.plot.{index}"));
                contract.metadata.identifier = contract.stream.clone();
                contract
            })
            .collect();
        let owner = dokime_runtime::Runtime::with_execution_fixture(config, "testdb", executor.clone()).await.unwrap();
        let runtime = owner.handle();
        let mut suite = fixture_suite(&["passed"]);
        suite.test_case_templates[0].document = Some(document(plots));
        register_custom_suite(&runtime, suite.clone()).await;
        let registration = runtime.test_suites().await.into_iter().find(|entry| entry.suite == suite).unwrap();
        let cases = materialize_cases(&registration.suite_id, &suite);
        let cycle = runtime
            .create_cycle(
                CycleDefinition {
                    metadata: metadata("bench.documents", "Document Queries"),
                    continue_automatically: false,
                },
                CommandContext::default(),
            )
            .await
            .unwrap();
        runtime.append_fixture_cases(cycle.clone(), cases, CommandContext::default()).await.unwrap();
        let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
        assert_eq!(
            runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await.status,
            CommandStatus::Accepted
        );
        executor.wait_until_entered().await;
        let scope = runtime.watch_state().latest().state().executions[&execution].capture_scope();
        let waveform = super::waveforms::samples(1);
        let per_plot = rows / plots;
        for plot in 0..plots {
            let stream = identifier(&format!("bench.plot.{plot}"));
            for first in (0..per_plot).step_by(2_000) {
                let samples = (first..(first + 2_000).min(per_plot))
                    .map(|index| {
                        let nanos = (index as u128 * u128::from(DAYS) * 86_400_000_000_000 / (per_plot - 1) as u128) as u64;
                        let timestamp =
                            Timestamp::from_system_time(UNIX_EPOCH + Duration::from_secs(1_788_220_800) + Duration::from_nanos(nanos));
                        let mut sample = TelemetrySample::new(
                            dokime_runtime::model::identity::new_id(),
                            stream.clone(),
                            index as u64,
                            timestamp,
                            "bench_plot",
                        );
                        sample.scope = Some(scope.clone());
                        let value = waveform[(index + plot) % waveform.len()].value;
                        sample.values.insert("value".into(), Value::Number(value));
                        sample.storage_payload = Some(
                            super::plots::PlotRow {
                                timestamp: sample.timestamp.as_str().to_owned(),
                                value: Some(value),
                            }
                            .encode(),
                        );
                        sample
                    })
                    .collect();
                let receipt = runtime
                    .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream.clone(), samples))
                    .await
                    .unwrap();
                assert_eq!(receipt.dropped, 0);
            }
        }
        runtime.storage().flush().await.unwrap();
        Self {
            owner,
            runtime,
            executor,
            cycle,
            execution,
            rows,
            plots,
            _root: root,
        }
    }

    async fn structure(&self) -> Duration {
        let document = dokime_runtime::DocumentId::from_identifier("default").unwrap();
        let prepared = self.runtime.prepare_current_document(&self.cycle, &self.execution, &document).unwrap();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(128);
        let started = Instant::now();
        let (result, elapsed) = tokio::join!(
            prepared.query_stream(
                CurrentDocumentQuery {
                    structure_only: true,
                    ..Default::default()
                },
                sender
            ),
            async {
                assert!(matches!(receiver.recv().await, Some(CurrentDocumentEvent::Structure { .. })));
                let elapsed = started.elapsed();
                while receiver.recv().await.is_some() {}
                elapsed
            }
        );
        result.unwrap();
        elapsed
    }

    async fn cards(&self) -> Duration {
        let document = dokime_runtime::DocumentId::from_identifier("default").unwrap();
        let started = Instant::now();
        let captured = self.runtime.prepare_current_document(&self.cycle, &self.execution, &document).unwrap().capture().await.unwrap();
        assert!(
            captured
                .capture
                .cards
                .iter()
                .all(|card| !matches!(card.result, dokime_runtime::document::capture::CardBindingResult::Failed { .. })),
            "{}",
            serde_json::to_string(&captured.capture.cards).unwrap()
        );
        let scope = self.runtime.watch_state().latest().state().executions[&self.execution].capture_scope();
        let requests = captured
            .capture
            .cards
            .iter()
            .filter_map(|card| {
                let dokime_runtime::document::capture::CardBindingResult::Plot { descriptor } = &card.result else { return None };
                Some(plot_payload_request_for_descriptor(descriptor, Some(scope.clone()), Some(1_500), None))
            })
            .collect();
        let before = self.runtime.storage_read_counters();
        let payloads = self.runtime.query_plot_payloads_at(requests, captured.evidence_cut.as_ref().unwrap()).await.unwrap();
        assert_eq!(self.runtime.storage_read_counters().data_scan_rows, before.data_scan_rows);
        assert_eq!(payloads.len(), self.plots);
        for payload in payloads {
            assert_eq!(payload.payload.series[0].raw_points, self.rows / self.plots);
        }
        started.elapsed()
    }

    async fn close(mut self) {
        self.executor.release();
        drop(self.runtime);
        self.owner.shutdown().await.unwrap();
    }
}

/* === Public Functions === */

pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let mut group = criterion.benchmark_group("document_queries");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    for rows in COUNTS {
        let mut fixture = None;
        for measurement in ["structure", "cards_and_plots"] {
            group.bench_function(BenchmarkId::new(measurement, rows), |bencher| {
                let fixture = fixture.get_or_insert_with(|| executor.block_on(Fixture::new(rows, PLOTS)));
                bencher.iter_custom(|iterations| {
                    executor.block_on(async {
                        let probe = super::metrics::Probe::start(fixture.runtime.storage().clone());
                        let mut elapsed = Duration::ZERO;
                        for _ in 0..iterations {
                            elapsed += if measurement == "structure" { fixture.structure().await } else { fixture.cards().await };
                        }
                        probe.finish().report(
                            &format!("document_queries/{measurement}"),
                            rows,
                            PLOTS,
                            Some(&dokime_runtime::config::storage_defaults()),
                        );
                        elapsed
                    })
                });
            });
        }
        if let Some(fixture) = fixture {
            executor.block_on(fixture.close());
        }
    }
    let rows = 1_000_000;
    let mut fixture = None;
    group.bench_function(BenchmarkId::new("median", rows), |bencher| {
        let fixture = fixture.get_or_insert_with(|| executor.block_on(Fixture::new(rows, 1)));
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    elapsed += fixture.cards().await;
                }
                elapsed
            })
        });
    });
    if let Some(fixture) = fixture {
        executor.block_on(fixture.close());
    }
    group.finish();
}

/* === Private Functions === */

fn document(plots: usize) -> Document {
    let mut document = Document::new("Document Queries");
    let mut section = DocumentSection::new("Measurements");
    let field = dokime_runtime::expressions::telemetry("bench.plot.0", "value");
    let mut group = TelemetrySummaryGroupDefinition::new("signal", "Signal");
    for (label, expression) in [
        ("Count", field.clone().count()),
        ("Min", field.clone().min()),
        ("Max", field.clone().max()),
        ("Mean", field.clone().mean()),
        ("RMS", field.clone().rms()),
    ] {
        group.tile(TelemetrySummaryTileDefinition::new(label, expression));
    }
    if plots == 1 {
        group.tile(TelemetrySummaryTileDefinition::new("Median", field.median()));
    }
    let mut summary = TelemetrySummaryDefinition::new();
    summary.group(group);
    section.card(DocumentCard::telemetry_summary(summary));
    for index in 0..plots {
        let mut plot = Plot::time_series(format!("Signal {index}"));
        plot.id(format!("bench.signal.{index}"));
        plot.y(dokime_runtime::expressions::telemetry(format!("bench.plot.{index}"), "value"));
        section.plot(plot);
    }
    document.section(section);
    document
}

/* === Tests === */
