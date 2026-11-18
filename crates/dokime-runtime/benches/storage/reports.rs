/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Creation Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, HashMap},
    env::{self, consts::EXE_SUFFIX},
    fmt::Debug,
    fs,
    hint::black_box,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion};
use harmos_protocol::model::{self, Declarations};
use prost::Message;
use prost_types::Timestamp;
use tokio::runtime::Runtime;
use tracing::{
    Event, Id, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Record},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

// Local Imports
use super::{EXPORT_ROWS, WRITE_ROWS, aggregate, boundary, definition, publication, report, samples};
use dokime::client::ExecutionOutcome;
use dokime_protocol::builders::Plot;
use dokime_protocol::host::files_client::FilesClient;
use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{FilesReadRequest, PromptsCreateResponse, ReportSubject, ReportsCreateRequest, report_subject};
use dokime_protocol::model::{
    Aggregate, Card, Document, FieldReference, Metadata, Origin, PluginRun, Section, Suite, Suites, Summary, SummaryTile, Termination,
    TestCase, TestCaseTemplate, card::Body, origin, parameter_expression, value,
};
use dokime_runtime::{Host, services};
use dokime_storage::Store;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
pub(super) struct Times(Arc<Mutex<BTreeMap<String, Vec<f64>>>>);

#[derive(Default)]
struct ReportEvent {
    phase: Option<String>,
    elapsed_ms: Option<f64>,
    samples: Option<u64>,
}

struct QuerySpan {
    phase: &'static str,
    started: Instant,
    value: ReportEvent,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Visit for ReportEvent {
    fn record_debug(&mut self, _: &Field, _: &dyn Debug) {}

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "phase" {
            self.phase = Some(value.into());
        }
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if field.name() == "elapsed_ms" {
            self.elapsed_ms = Some(value);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "samples" {
            self.samples = Some(value);
        }
    }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Times {
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        let phase = match attributes.metadata().name() {
            "source.read" => "source_read",
            "source.block" => "source_block",
            "document.reduce" => "statistics",
            "plot.query" => "plot",
            _ => return,
        };
        let mut value = ReportEvent::default();
        attributes.record(&mut value);
        context.span(id).unwrap().extensions_mut().insert(QuerySpan { phase, started: Instant::now(), value });
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        let span = context.span(id).unwrap();
        if let Some(query) = span.extensions_mut().get_mut::<QuerySpan>() {
            values.record(&mut query.value);
        }
    }

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        let span = context.span(&id).unwrap();
        if let Some(query) = span.extensions().get::<QuerySpan>() {
            let elapsed_ms = query.started.elapsed().as_secs_f64() * 1000.0;
            let mut stages = self.0.lock().unwrap();
            stages.entry(query.phase.into()).or_default().push(elapsed_ms);
            if let Some(samples) = query.value.samples {
                stages.entry(format!("{}_samples", query.phase)).or_default().push(samples as f64);
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut value = ReportEvent::default();
        event.record(&mut value);
        if let (Some(phase), Some(elapsed_ms)) = (value.phase, value.elapsed_ms) {
            let mut stages = self.0.lock().unwrap();
            stages.entry(phase.clone()).or_default().push(elapsed_ms);
            if let Some(samples) = value.samples {
                stages.entry(format!("{phase}_samples")).or_default().push(samples as f64);
            }
        }
    }
}

/* === Public Functions === */

pub(super) fn benchmark(criterion: &mut Criterion, executor: &Runtime, times: &Times) {
    let mut group = criterion.benchmark_group("reports");
    for plots in [1, 116] {
        boundary(
            &format!("Report Create, {plots} Plots, 16,384 Unique Samples"),
            "fixed synthetic sealed stream; shared-window plots, summary values, complete native capture, serialization, template and durable upload; excludes fixture preparation; not the OBC or thermal workstation recording",
        );
        group.bench_function(BenchmarkId::new("shared_window", plots), |b| {
            let (root, server, mut reports) = executor.block_on(async {
                let root = tempfile::tempdir().unwrap();
                let store = Store::open(root.path()).unwrap();
                let mut cards = (0..plots)
                    .map(|index| {
                        let metadata = Metadata {
                            identifier: format!("plot-{index}"),
                            ..Default::default()
                        };
                        let mut plot = Plot::time_series(FieldReference::new("readings").field("value")).message;
                        plot.metadata = Some(metadata.clone());
                        Card {
                            metadata: Some(metadata),
                            body: Some(Body::Plot(plot)),
                            ..Default::default()
                        }
                    })
                    .collect::<Vec<_>>();
                cards.insert(
                    0,
                    Card {
                        metadata: Some(Metadata {
                            identifier: "statistics".into(),
                            ..Default::default()
                        }),
                        body: Some(Body::Summary(Summary {
                            tiles: [Aggregate::Count, Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Rms]
                                .into_iter()
                                .map(|kind| SummaryTile {
                                    value: Some(aggregate("readings", kind)),
                                    ..Default::default()
                                })
                                .collect(),
                        })),
                        ..Default::default()
                    },
                );
                let document = Document {
                    metadata: Some(Metadata {
                        identifier: "report".into(),
                        ..Default::default()
                    }),
                    sections: cards
                        .chunks(15)
                        .enumerate()
                        .map(|(index, cards)| Section {
                            metadata: Some(Metadata {
                                identifier: format!("section-{index}"),
                                ..Default::default()
                            }),
                            cards: cards.to_vec(),
                            ..Default::default()
                        })
                        .collect(),
                };
                let suites = Suites {
                    suites: vec![Suite {
                        metadata: Some(Metadata {
                            identifier: "report".into(),
                            ..Default::default()
                        }),
                        templates: vec![TestCaseTemplate {
                            metadata: Some(Metadata {
                                identifier: "report.capture".into(),
                                ..Default::default()
                            }),
                            documents: vec![document],
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                };
                let mut execution = definition();
                let mut catalog = Declarations::decode(execution.catalog.as_slice()).unwrap();
                catalog.sections.push(model::Section {
                    r#type: "dokime.model.v1.Suites".into(),
                    payload: suites.encode_to_vec(),
                });
                execution.catalog = catalog.encode_to_vec();
                execution.origin = Some(Origin {
                    kind: Some(origin::Kind::PluginRun(PluginRun {
                        plugin: "report".into(),
                        template: "report.capture".into(),
                        digest: vec![1],
                        started_at: Some(Timestamp::default()),
                        test_case: Some(TestCase {
                            streams: vec!["readings".into()],
                            ..Default::default()
                        }),
                    })),
                });
                store.create(&execution).unwrap();
                store.open_stream("execution", "readings").unwrap();
                for first in (0..EXPORT_ROWS).step_by(WRITE_ROWS) {
                    store.sample_batch("execution", &samples(first, WRITE_ROWS)).unwrap();
                }
                store.close_streams("execution").unwrap();
                store.close("execution", Termination::Completed, ExecutionOutcome::default()).unwrap();
                store.seal("execution").unwrap();
                store.shutdown_samples().await.unwrap();
                drop(store);
                let settings = publication::settings();
                let operations = services::ClientSettings {
                    read_timeout: Duration::from_secs(30),
                    mutation_timeout: Duration::from_secs(30),
                    watch_timeout: Duration::from_secs(30),
                    max_cases: 1000,
                    page_items: 1000,
                    message_bytes: settings.placement.message_bytes.get(),
                    stream_buffer: settings.placement.stream_window.get(),
                };
                let server = services::serve(([127, 0, 0, 1], 0).into(), root.path(), settings, operations).await.unwrap();
                let reports = ReportsClient::connect(format!("http://{}", server.endpoint())).await.unwrap();
                (root, server, reports)
            });
            let mut sequence = 0;
            b.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        times.0.lock().unwrap().clear();
                        let request = ReportsCreateRequest {
                            subject: Some(ReportSubject {
                                kind: Some(report_subject::Kind::Execution("execution".into())),
                            }),
                            document_id: "report".into(),
                            request_id: format!("report-{sequence}"),
                        };
                        sequence += 1;
                        let started = Instant::now();
                        black_box(reports.create(request).await.unwrap());
                        elapsed += started.elapsed();
                        if env::var_os("DOKIME_BENCH_ONCE").is_some() {
                            for (phase, values) in times.0.lock().unwrap().iter_mut() {
                                values.sort_by(f64::total_cmp);
                                let middle = values.len() / 2;
                                let median =
                                    if values.len() % 2 == 0 { (values[middle - 1] + values[middle]) / 2.0 } else { values[middle] };
                                println!(
                                    "Report Stage | {phase} | total={:.6} min={:.6} median={:.6} max={:.6} | {} calls",
                                    values.iter().sum::<f64>(),
                                    values[0],
                                    median,
                                    values[values.len() - 1],
                                    values.len()
                                );
                            }
                        }
                    }
                    report(elapsed, iterations)
                })
            });
            executor.block_on(async {
                server.close().await.unwrap();
                services::remove_credentials(root.path(), &publication::credentials()).unwrap();
            });
        });
    }
    for (name, executable, template, action) in
        [("thermal", "dokime-rust-thermal", "thermal.cycle", "0"), ("obc", "plugin-obc", "obc.benchmark.plots", "yes")]
    {
        group.bench_function(format!("examples/{name}"), |b| {
            let (root, execution) = executor.block_on(async {
                let root = PathBuf::from(
                    env::var_os("DOKIME_REPORT_EXAMPLE_ROOT")
                        .expect("Measure Example Reports: Select a Fresh Benchmark Root with DOKIME_REPORT_EXAMPLE_ROOT"),
                )
                .join(name);
                fs::create_dir_all(&root).unwrap();
                if root.join("execution.sqlite3").exists() {
                    let store = Store::open(&root).unwrap();
                    let execution = store.identifiers().unwrap().remove(0);
                    return (root, execution);
                }
                let settings = publication::settings();
                let operations = services::ClientSettings {
                    read_timeout: Duration::from_secs(30),
                    mutation_timeout: Duration::from_secs(30),
                    watch_timeout: Duration::from_secs(30),
                    max_cases: 1000,
                    page_items: 1000,
                    message_bytes: settings.placement.message_bytes.get(),
                    stream_buffer: settings.placement.stream_window.get(),
                };
                let host = Host::open(&root, settings, operations).await.unwrap();
                let executable = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/debug/{executable}{EXE_SUFFIX}"));
                let entry = host.add_managed(name, &executable, &[]).await.unwrap();
                let catalog = host.catalog(&entry).unwrap();
                let definition = catalog
                    .suites
                    .iter()
                    .flat_map(|suite| &suite.templates)
                    .find(|definition| definition.metadata.as_ref().unwrap().identifier == template)
                    .unwrap();
                let mut parameters = definition
                    .parameters
                    .iter()
                    .map(|parameter| {
                        let Some(parameter_expression::Kind::Literal(value)) =
                            parameter.expression.as_ref().and_then(|expression| expression.kind.as_ref())
                        else {
                            panic!("Measure Example Reports: Resolve the Template's Nonliteral Parameter");
                        };
                        (parameter.metadata.as_ref().unwrap().identifier.clone(), value.clone())
                    })
                    .collect::<HashMap<_, _>>();
                if name == "obc" {
                    parameters.get_mut("duration_s").unwrap().kind = Some(value::Kind::Number(1.0));
                    parameters.get_mut("sample_interval_s").unwrap().kind = Some(value::Kind::Number(0.001));
                }
                let origin = host.materialize(&entry, template, parameters).await.unwrap();
                let cycle = host.start_cycle(vec![origin]).await.unwrap();
                let execution = cycle.executions[0].clone();
                let prompt = host.next_prompt(&execution).await.unwrap();
                host.answer(
                    &execution,
                    PromptsCreateResponse {
                        prompt: prompt.prompt,
                        action: action.into(),
                        accepted: true,
                        ..Default::default()
                    },
                )
                .unwrap();
                let snapshot = cycle.wait().await.unwrap();
                println!(
                    "Example Source | {name} | {} samples | {} records",
                    snapshot.executions[0].samples.len(),
                    snapshot.executions[0].evidence.iter().filter(|entry| entry.kind.starts_with("record:")).count()
                );
                host.shutdown().await.unwrap();
                services::remove_credentials(&root, &publication::credentials()).unwrap();
                (root, execution)
            });
            let (server, mut reports, mut files) = executor.block_on(async {
                let settings = publication::settings();
                let operations = services::ClientSettings {
                    read_timeout: Duration::from_secs(30),
                    mutation_timeout: Duration::from_secs(30),
                    watch_timeout: Duration::from_secs(30),
                    max_cases: 1000,
                    page_items: 1000,
                    message_bytes: settings.placement.message_bytes.get(),
                    stream_buffer: settings.placement.stream_window.get(),
                };
                let server = services::serve(([127, 0, 0, 1], 0).into(), &root, settings, operations).await.unwrap();
                let endpoint = format!("http://{}", server.endpoint());
                let reports = ReportsClient::connect(endpoint.clone()).await.unwrap();
                let files = FilesClient::connect(endpoint).await.unwrap();
                (server, reports, files)
            });
            b.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        times.0.lock().unwrap().clear();
                        let request = ReportsCreateRequest {
                            subject: Some(ReportSubject {
                                kind: Some(report_subject::Kind::Execution(execution.clone())),
                            }),
                            request_id: uuid::Uuid::now_v7().to_string(),
                            ..Default::default()
                        };
                        let started = Instant::now();
                        let response = reports.create(request).await.unwrap().into_inner();
                        let creation = started.elapsed();
                        elapsed += creation;
                        let reference = response.file.unwrap();
                        let started = Instant::now();
                        let mut bytes = Vec::new();
                        let mut at = None;
                        while (bytes.len() as u64) < reference.length {
                            let reply = files
                                .read(FilesReadRequest {
                                    file: Some(reference.clone()),
                                    offset: bytes.len() as u64,
                                    max_bytes: 0,
                                    at,
                                })
                                .await
                                .unwrap()
                                .into_inner();
                            assert!(!reply.data.is_empty());
                            bytes.extend(reply.data);
                            at = reply.resolved_at;
                        }
                        let transfer = started.elapsed();
                        let started = Instant::now();
                        let mut output = tempfile::tempfile().unwrap();
                        output.write_all(&bytes).unwrap();
                        output.sync_all().unwrap();
                        let write = started.elapsed();
                        if env::var_os("DOKIME_BENCH_ONCE").is_some() {
                            println!(
                                "Example | {name} | create={:.6} transfer={:.6} write={:.6} total={:.6} ms",
                                creation.as_secs_f64() * 1000.0,
                                transfer.as_secs_f64() * 1000.0,
                                write.as_secs_f64() * 1000.0,
                                (creation + transfer + write).as_secs_f64() * 1000.0
                            );
                            for (phase, values) in times.0.lock().unwrap().iter_mut() {
                                values.sort_by(f64::total_cmp);
                                let middle = values.len() / 2;
                                let median =
                                    if values.len() % 2 == 0 { (values[middle - 1] + values[middle]) / 2.0 } else { values[middle] };
                                println!(
                                    "Report Stage | {phase} | total={:.6} min={:.6} median={:.6} max={:.6} | {} calls",
                                    values.iter().sum::<f64>(),
                                    values[0],
                                    median,
                                    values[values.len() - 1],
                                    values.len()
                                );
                            }
                        }
                    }
                    report(elapsed, iterations)
                })
            });
            executor.block_on(async {
                server.close().await.unwrap();
                services::remove_credentials(&root, &publication::credentials()).unwrap();
            });
        });
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
