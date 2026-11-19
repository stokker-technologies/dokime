/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Report Save Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    env,
    fmt::Debug,
    fs::{self, OpenOptions},
    io::{self, Error, ErrorKind},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{BenchmarkId, Criterion};
use harmos_protocol::model::{self, Declarations};
use harmos_runtime::filesystem;
use prost::Message;
use prost_types::Timestamp;
use sha2::{Digest, Sha256};
use tempfile::{Builder, NamedTempFile};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;
use tracing::{
    Event, Id, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Record},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

// Local Imports
use super::{WRITE_ROWS, aggregate, boundary, definition, publication, report, samples};
use dokime::client::ExecutionOutcome;
use dokime_protocol::builders::Plot;
use dokime_protocol::host::documents_client::DocumentsClient;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::files_client::FilesClient;
use dokime_protocol::host::plots::SnapshotsListRequest;
use dokime_protocol::host::plots::snapshots_client::SnapshotsClient;
use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{
    DocumentScope, DocumentsGetRequest, FilesGetRequest, FilesReadRequest, ReportSubject, ReportsCreateRequest, ReportsGetRequest,
    ReportsPublishRequest, document_binding_result, document_frame_frame, report_subject,
};
use dokime_protocol::model::{
    Aggregate, Card, Document, FieldReference, Metadata, Origin, PluginRun, RecordContract, RecordContracts, Section, Suite, Suites,
    Summary, SummaryTile, Termination, TestCase, TestCaseTemplate, card::Body, origin,
};
use dokime_runtime::services;
use dokime_storage::{Evidence, Sample, Store};

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
    let mut group = criterion.benchmark_group("reports/save");
    let plots = 116;
    boundary(
        &format!("Report Save, {plots} Distinct Streams, Eight Sections and Three Records"),
        "fixed 1,024 samples per stream; 306 native plot browsing views and their cut indexes prepared first; Create, Publish, Get, Files.Get, bounded Read, SHA-256 verification, adjacent staging, sync and two atomic publications; excludes interactive picker and WebDriver; not the current OBC recording",
    );
    group.bench_function(BenchmarkId::new("distinct_streams", plots), |b| {
        let (root, server, mut reports, mut files, frame_bytes) = executor.block_on(async {
            let root = tempfile::tempdir().unwrap();
            let store = Store::open(root.path()).unwrap();
            let mut cards = (0..plots)
                .map(|index| {
                    let metadata = Metadata {
                        identifier: format!("plot-{index}"),
                        ..Default::default()
                    };
                    let mut plot = Plot::time_series(FieldReference::new(format!("readings-{index}")).field("value")).message;
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
                                value: Some(aggregate("readings-0", kind)),
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
            let declaration = catalog.streams.remove(0);
            catalog.streams = (0..plots)
                .map(|index| model::StreamDeclaration {
                    id: format!("readings-{index}"),
                    ..declaration.clone()
                })
                .collect();
            let records = RecordContracts {
                records: vec![RecordContract {
                    metadata: Some(Metadata {
                        identifier: "reading".into(),
                        ..Default::default()
                    }),
                    message: "benchmark.Row".into(),
                }],
            };
            catalog.sections.push(model::Section {
                r#type: "dokime.model.v1.RecordContracts".into(),
                payload: records.encode_to_vec(),
            });
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
                        streams: (0..plots).map(|index| format!("readings-{index}")).collect(),
                        ..Default::default()
                    }),
                })),
            });
            store.create(&execution).unwrap();
            for index in 0..plots {
                let stream = format!("readings-{index}");
                store.open_stream("execution", &stream).unwrap();
                let rows = samples(0, WRITE_ROWS).into_iter().map(|row| Sample { stream: stream.clone(), ..row }).collect::<Vec<_>>();
                store.sample_batch("execution", &rows).unwrap();
            }
            for index in 0..3 {
                let record = RecordsAppendRequest {
                    execution: "execution".into(),
                    kind: "reading".into(),
                    payload: samples(index, 1).remove(0).row,
                    source_at: Some(Timestamp::default()),
                    request_id: format!("record-{index}"),
                };
                store
                    .append(
                        "execution",
                        Evidence {
                            kind: "record:reading".into(),
                            payload: record.encode_to_vec(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
            println!("Report Source | 8 sections | {plots} distinct-stream plots | {} samples | 3 records", plots * WRITE_ROWS);
            store.close_streams("execution").unwrap();
            store.close("execution", Termination::Completed, ExecutionOutcome::default()).unwrap();
            store.seal("execution", Default::default()).unwrap();
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
            let frame_bytes = operations.message_bytes as u64;
            let server = services::serve(([127, 0, 0, 1], 0).into(), root.path(), settings, operations).await.unwrap();
            let endpoint = format!("http://{}", server.endpoint());
            let reports = ReportsClient::connect(endpoint.clone()).await.unwrap();
            let files = FilesClient::connect(endpoint.clone()).await.unwrap();
            let mut documents = DocumentsClient::connect(endpoint.clone()).await.unwrap();
            let mut snapshots = SnapshotsClient::connect(endpoint).await.unwrap();
            // Prepare Native Browsing Views Before the Save, Like the Desktop
            let mut browsing = Vec::new();
            let mut at = None;
            let mut page_token = String::new();
            loop {
                let document = documents
                    .get(DocumentsGetRequest {
                        scope: Some(DocumentScope {
                            cycle_id: "cycle".into(),
                            execution_id: "execution".into(),
                        }),
                        document_id: "report".into(),
                        at,
                        page_token,
                        ..Default::default()
                    })
                    .await
                    .unwrap()
                    .into_inner();
                for frame in document.frames {
                    let Some(document_frame_frame::Kind::Binding(binding)) = frame.frame.and_then(|frame| frame.kind) else { continue };
                    let Some(document_binding_result::Kind::Plot(plot)) = binding.result.and_then(|result| result.kind) else {
                        continue;
                    };
                    let request = plot.request.unwrap();
                    browsing.push(request.clone());
                    snapshots
                        .list(SnapshotsListRequest {
                            requests: vec![request],
                            ..Default::default()
                        })
                        .await
                        .unwrap();
                }
                at = document.resolved_at;
                page_token = document.next_page_token;
                if page_token.is_empty() {
                    break;
                }
            }
            // Retain Distinct Viewport Selections as a Long-Lived Desktop Does
            for index in browsing.len()..306 {
                let mut request = browsing[index % browsing.len()].clone();
                request.panel_width = Some(640 + index as u32);
                snapshots
                    .list(SnapshotsListRequest {
                        requests: vec![request],
                        ..Default::default()
                    })
                    .await
                    .unwrap();
            }
            println!("Report Browsing | 306 retained plot views with native files and view-at indexes beside report objects");
            (root, server, reports, files, frame_bytes)
        });
        let mut sequence = 0;
        b.iter_custom(|iterations| {
            let result: io::Result<Duration> = executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    times.0.lock().unwrap().clear();
                    let subject = ReportSubject {
                        kind: Some(report_subject::Kind::Execution("execution".into())),
                    };
                    let request = ReportsCreateRequest {
                        subject: Some(subject.clone()),
                        document_id: "report".into(),
                        request_id: format!("report-{sequence}"),
                    };
                    sequence += 1;
                    let started = Instant::now();

                    // Create and Publish the Frozen Execution Report
                    let prepared = reports.create(request).await.unwrap().into_inner();
                    let creation = started.elapsed();
                    let reference = prepared.file.unwrap();
                    let publishing = Instant::now();
                    let published = reports
                        .publish(ReportsPublishRequest {
                            subject: Some(subject.clone()),
                            name: reference.metadata.as_ref().unwrap().id.clone(),
                            expected_revision: reference.sha256,
                            request_id: format!("publish-{sequence}"),
                        })
                        .await
                        .unwrap()
                        .into_inner();
                    let publication = publishing.elapsed();

                    // Resolve the Published File and Its Transfer Cut
                    let selecting = Instant::now();
                    let selected = reports
                        .get(ReportsGetRequest {
                            subject: Some(subject),
                            name: published.report.unwrap().identifier,
                            at: None,
                        })
                        .await
                        .unwrap()
                        .into_inner();
                    let reference = selected.file.unwrap();
                    let selected = files.get(FilesGetRequest { file: Some(reference.clone()), at: None }).await.unwrap().into_inner();
                    let selected_file = selected.file.as_ref().unwrap();
                    if selected_file.length != reference.length || selected_file.sha256 != reference.sha256 {
                        return Err(Error::new(ErrorKind::InvalidData, "Measure Report Save: Restore Its Unchanged Selected File"));
                    }
                    let maximum =
                        frame_bytes.saturating_sub(1024).min(selected.chunk_bytes).min(selected.message_bytes.saturating_sub(1024));
                    if maximum == 0 {
                        return Err(Error::new(ErrorKind::InvalidData, "Measure Report Save: Restore Its Positive Transfer Limits"));
                    }
                    let selection = selecting.elapsed();

                    // Stream, Verify and Durably Publish Beside the Destination
                    let staged = Builder::new().prefix(".report-").tempfile_in(root.path())?.into_temp_path();
                    fs::remove_file(&staged)?;
                    let output = NamedTempFile::new_in(root.path())?;
                    let mut file = File::from_std(output.reopen()?);
                    let mut digest = Sha256::new();
                    let mut offset = 0;
                    let mut at = selected.resolved_at;
                    let mut transfer = Duration::ZERO;
                    while offset < reference.length {
                        let reading = Instant::now();
                        let reply = files
                            .read(FilesReadRequest {
                                file: Some(reference.clone()),
                                offset,
                                max_bytes: maximum,
                                at: at.clone(),
                            })
                            .await
                            .unwrap()
                            .into_inner();
                        transfer += reading.elapsed();
                        if reply.total_length != reference.length
                            || reply.sha256 != reference.sha256
                            || reply.data.is_empty()
                            || reply.data.len() as u64 > maximum
                            || offset.saturating_add(reply.data.len() as u64) > reference.length
                            || reply.resolved_at.is_none()
                            || at.is_some() && at != reply.resolved_at
                        {
                            return Err(Error::new(
                                ErrorKind::InvalidData,
                                "Measure Report Save: Restore Its Complete File at the Returned Cut",
                            ));
                        }
                        at = reply.resolved_at;
                        offset += reply.data.len() as u64;
                        digest.update(&reply.data);
                        file.write_all(&reply.data).await?;
                    }
                    if digest.finalize().as_slice() != reference.sha256.as_slice() {
                        return Err(Error::new(ErrorKind::InvalidData, "Measure Report Save: Restore Bytes Matching Its File Digest"));
                    }
                    file.flush().await?;
                    file.sync_all().await?;
                    drop(file);
                    output.persist_noclobber(&staged).map_err(|source| source.error)?;
                    let input = OpenOptions::new().read(true).write(true).open(&staged)?;
                    input.sync_all()?;
                    drop(input);
                    filesystem::publication::install(&staged, &root.path().join(format!("report-{sequence}.html")))
                        .map_err(|source| Error::other(format!("Measure Report Save: Publish Its New Destination: {source}")))?;
                    let total = started.elapsed();
                    elapsed += total;
                    if env::var_os("DOKIME_BENCH_ONCE").is_some() {
                        println!(
                            "Report Save | create={:.6} publish={:.6} select={:.6} transfer={:.6} write={:.6} total={:.6} ms",
                            creation.as_secs_f64() * 1000.0,
                            publication.as_secs_f64() * 1000.0,
                            selection.as_secs_f64() * 1000.0,
                            transfer.as_secs_f64() * 1000.0,
                            total.saturating_sub(creation + publication + selection + transfer).as_secs_f64() * 1000.0,
                            total.as_secs_f64() * 1000.0
                        );
                        for (phase, values) in times.0.lock().unwrap().iter_mut() {
                            values.sort_by(f64::total_cmp);
                            let middle = values.len() / 2;
                            let median = if values.len() % 2 == 0 { (values[middle - 1] + values[middle]) / 2.0 } else { values[middle] };
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
                Ok(report(elapsed, iterations))
            });
            result.expect("Measure Report Save: Restore Its Verified Native Delivery")
        });
        executor.block_on(async {
            server.close().await.unwrap();
            services::remove_credentials(root.path(), &publication::credentials()).unwrap();
        });
    });
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
