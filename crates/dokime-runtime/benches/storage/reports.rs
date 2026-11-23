/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report and Evidence Delivery Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    env,
    fmt::Debug,
    ops::Range,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::Criterion;
use harmos_protocol::model::{self, Declarations, FileReference};
use prost::Message;
use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet, Timestamp};
use sha2::{Digest, Sha256};
use tempfile::{NamedTempFile, TempDir};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;
use tonic::transport::Channel;
use tracing::{
    Event, Id, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Record},
};
use tracing_subscriber::{Layer, layer::Context, registry::LookupSpan};

// Local Imports
use super::{aggregate, boundary, publication, report};
use dokime_client_internal::ExecutionOutcome;
use dokime_protocol::builders::Plot;
use dokime_protocol::host::documents_client::DocumentsClient;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence_client::EvidenceClient;
use dokime_protocol::host::files_client::FilesClient;
use dokime_protocol::host::plots::snapshots_client::SnapshotsClient;
use dokime_protocol::host::plots::{PlotRequest, SnapshotsListRequest};
use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{
    DocumentScope, DocumentsGetRequest, EvidenceArchiveRequest, EvidenceSubject, FilesGetRequest, FilesReadRequest, ReportSubject,
    ReportsCreateRequest, ReportsGetRequest, document_binding_result, document_frame_frame, evidence_subject, report_subject,
};
use dokime_protocol::model::{
    Aggregate, Card, Document, EvidenceCard, FieldReference, Metadata, Origin, PluginRun, RecordContract, RecordContracts, Section, Suite,
    Suites, Summary, SummaryTile, Termination, TestCase, TestCaseTemplate, card::Body, origin,
};
use dokime_runtime::services;
use dokime_storage::{Evidence, Execution, Sample, Store};

/* === Constants === */

/// Five Minutes of Live Capture.
const SECONDS: usize = 300;
/// Telemetry Streams at 1 kHz.
const TELEMETRY: usize = 3;
/// Status Streams at 10 Hz.
const STATUS: usize = 8;
/// One Wi-Fi Scan Every Three Seconds Reports Forty Networks.
const SCAN_SECONDS: usize = 3;
const SCAN_ROWS: usize = 40;
/// One Record Every 25 Seconds.
const RECORD_SECONDS: usize = 25;
const BATCH_ROWS: usize = 4096;
const BASE_NS: i64 = 1_788_220_800_000_000_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
pub(super) struct Times(pub(super) Arc<Mutex<BTreeMap<String, Vec<f64>>>>);

#[derive(Default)]
struct ReportEvent {
    phase: Option<String>,
    elapsed_ms: Option<f64>,
}

#[derive(Clone, PartialEq, Message)]
pub(super) struct Reading {
    #[prost(double, tag = "1")]
    pub(super) value: f64,
}

#[derive(Clone, PartialEq, Message)]
struct Scan {
    #[prost(uint32, tag = "1")]
    channel: u32,

    #[prost(double, tag = "2")]
    rssi_dbm: f64,

    #[prost(string, tag = "3")]
    ssid: String,
}

pub(super) struct Delivery {
    pub(super) root: TempDir,
    pub(super) server: services::Server,
    pub(super) reports: ReportsClient<Channel>,
    files: FilesClient<Channel>,
    evidence: EvidenceClient<Channel>,
    snapshots: SnapshotsClient<Channel>,
    plots: Vec<PlotRequest>,
    frame_bytes: u64,
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
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Times {
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        if ["plot.", "source.", "document.", "storage.deletion."].iter().any(|prefix| attributes.metadata().name().starts_with(prefix)) {
            context.span(id).unwrap().extensions_mut().insert(Instant::now());
        }
    }

    fn on_record(&self, _: &Id, _: &Record<'_>, _: Context<'_, S>) {}

    fn on_close(&self, id: Id, context: Context<'_, S>) {
        let span = context.span(&id).unwrap();
        if let Some(started) = span.extensions().get::<Instant>() {
            self.0.lock().unwrap().entry(span.name().into()).or_default().push(started.elapsed().as_secs_f64() * 1000.0);
        }
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut value = ReportEvent::default();
        event.record(&mut value);
        if let (Some(phase), Some(elapsed_ms)) = (value.phase, value.elapsed_ms) {
            self.0.lock().unwrap().entry(phase).or_default().push(elapsed_ms);
        }
    }
}

impl Delivery {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        seal(root.path()).await;
        let mut delivery = Self::serve(root, Vec::new()).await;

        // Bind the Document's Plots the Way the Desktop Opens a Sealed Execution
        let mut documents = DocumentsClient::connect(format!("http://{}", delivery.server.endpoint())).await.unwrap();
        let (mut at, mut page_token) = (None, String::new());
        loop {
            let document = documents
                .get(DocumentsGetRequest {
                    scope: Some(DocumentScope {
                        cycle_id: "cycle".into(),
                        execution_id: "execution".into(),
                    }),
                    at,
                    page_token,
                    ..Default::default()
                })
                .await
                .unwrap()
                .into_inner();
            for frame in document.frames {
                let Some(document_frame_frame::Kind::Binding(binding)) = frame.frame.and_then(|frame| frame.kind) else { continue };
                if let Some(document_binding_result::Kind::Plot(plot)) = binding.result.and_then(|result| result.kind) {
                    delivery.plots.push(PlotRequest {
                        panel_width: Some(640),
                        ..plot.request.unwrap()
                    });
                }
            }
            (at, page_token) = (document.resolved_at, document.next_page_token);
            if page_token.is_empty() {
                break;
            }
        }

        // Executions Sealed Before Freezing Existed Compute Their Plots Once per Host
        let started = Instant::now();
        let slowest = delivery.plots().await;
        println!(
            "Plots Before Freeze | {:.6} ms | slowest={:.6} ms",
            started.elapsed().as_secs_f64() * 1000.0,
            slowest.as_secs_f64() * 1000.0
        );

        // Freeze the Report and Then the Evidence Archive Once, as Sealing Does
        let started = Instant::now();
        delivery
            .reports
            .create(ReportsCreateRequest {
                subject: Some(subject()),
                request_id: "sealed".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        println!("Report Materialization | {:.6} ms", started.elapsed().as_secs_f64() * 1000.0);
        let started = Instant::now();
        delivery.archive().await;
        println!("Evidence Materialization and First Delivery | {:.6} ms", started.elapsed().as_secs_f64() * 1000.0);

        // Restart, So Every Measurement Reads Only What Sealing Retained
        let Self { root, server, plots, .. } = delivery;
        server.close().await.unwrap();
        Self::serve(root, plots).await
    }

    pub(super) async fn serve(root: TempDir, plots: Vec<PlotRequest>) -> Self {
        let server = host(root.path()).await;
        let endpoint = format!("http://{}", server.endpoint());
        Self {
            reports: ReportsClient::connect(endpoint.clone()).await.unwrap(),
            files: FilesClient::connect(endpoint.clone()).await.unwrap(),
            evidence: EvidenceClient::connect(endpoint.clone()).await.unwrap(),
            snapshots: SnapshotsClient::connect(endpoint).await.unwrap(),
            root,
            server,
            plots,
            frame_bytes: publication::settings().placement.message_bytes.get() as u64,
        }
    }

    /// Load Every Plot of the Sealed Document, One Snapshot List per Plot as the Desktop Watches Them.
    async fn plots(&mut self) -> Duration {
        let mut slowest = Duration::ZERO;
        for request in self.plots.clone() {
            let started = Instant::now();
            let reply = self
                .snapshots
                .list(SnapshotsListRequest {
                    requests: vec![request],
                    ..Default::default()
                })
                .await
                .unwrap()
                .into_inner();
            assert!(reply.envelopes.iter().all(|envelope| envelope.payload.is_some()), "List Snapshots: Return Every Plot Payload");
            slowest = slowest.max(started.elapsed());
        }
        slowest
    }

    /// Download Report the Way the Desktop Does: Select the Report, Then Stream and Verify Its File.
    async fn report(&mut self, sequence: usize) -> FileReference {
        let prepared = self
            .reports
            .create(ReportsCreateRequest {
                subject: Some(subject()),
                request_id: format!("download-{sequence}"),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_inner();
        let selected = self
            .reports
            .get(ReportsGetRequest {
                subject: Some(subject()),
                name: prepared.report.unwrap().identifier,
                at: None,
            })
            .await
            .unwrap()
            .into_inner();
        let reference = selected.file.unwrap();
        self.deliver(&reference).await;
        reference
    }

    /// Download Evidence the Way the Desktop Does: Resolve the Archive, Then Stream and Verify It.
    pub(super) async fn archive(&mut self) -> FileReference {
        let started = Instant::now();
        let response = self
            .evidence
            .archive(EvidenceArchiveRequest {
                subject: Some(EvidenceSubject {
                    kind: Some(evidence_subject::Kind::Execution("execution".into())),
                }),
                at: None,
            })
            .await
            .unwrap()
            .into_inner();
        let reference = response.file.unwrap();
        if env::var_os("DOKIME_BENCH_ONCE").is_some() {
            println!("Archive Request | {:.6} ms", started.elapsed().as_secs_f64() * 1000.0);
        }
        self.deliver(&reference).await;
        reference
    }

    async fn deliver(&mut self, reference: &FileReference) {
        let selected = self.files.get(FilesGetRequest { file: Some(reference.clone()), at: None }).await.unwrap().into_inner();
        let file = selected.file.as_ref().unwrap();
        assert!(file.length == reference.length && file.sha256 == reference.sha256, "Deliver: Select the Unchanged File");
        let maximum = self.frame_bytes.saturating_sub(1024).min(selected.chunk_bytes).min(selected.message_bytes.saturating_sub(1024));
        let output = NamedTempFile::new_in(self.root.path()).unwrap();
        let mut written = File::from_std(output.reopen().unwrap());
        let mut digest = Sha256::new();
        let mut at = selected.resolved_at;
        let mut offset = 0;
        let mut reading = Duration::ZERO;
        while offset < reference.length {
            let started = Instant::now();
            let reply = self
                .files
                .read(FilesReadRequest {
                    file: Some(reference.clone()),
                    offset,
                    max_bytes: maximum,
                    at: at.clone(),
                })
                .await
                .unwrap()
                .into_inner();
            reading += started.elapsed();
            assert!(!reply.data.is_empty() && reply.sha256 == reference.sha256, "Deliver: Read the Complete File at Its Cut");
            at = reply.resolved_at;
            offset += reply.data.len() as u64;
            digest.update(&reply.data);
            written.write_all(&reply.data).await.unwrap();
        }
        assert_eq!(digest.finalize().as_slice(), reference.sha256.as_slice(), "Deliver: Verify the File Digest");
        let syncing = Instant::now();
        written.sync_all().await.unwrap();
        if env::var_os("DOKIME_BENCH_ONCE").is_some() {
            println!(
                "Delivery Stages | read={:.6} ms | sync={:.6} ms | {} bytes",
                reading.as_secs_f64() * 1000.0,
                syncing.elapsed().as_secs_f64() * 1000.0,
                reference.length
            );
        }
    }
}

/* === Public Functions === */

pub(super) fn benchmark(criterion: &mut Criterion, executor: &Runtime, times: &Times) {
    let mut group = criterion.benchmark_group("reports/delivery");
    group.sample_size(10);
    let mut delivery = None;
    boundary(
        "Report Download, Typical Sealed Execution",
        "five minutes: three 1 kHz telemetry streams, eight 10 Hz status streams, a Wi-Fi stream of 40 rows every 3 s, 14 plots (time series, histograms, heat strip), 12 records; report and archive frozen once as sealing does, then the host restarts; Create, Get, Files.Get, bounded Read, SHA-256 and a synced file",
    );
    group.bench_function("report", |b| {
        let delivery = delivery.get_or_insert_with(|| executor.block_on(Delivery::new()));
        let mut sequence = 0;
        b.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    times.0.lock().unwrap().clear();
                    sequence += 1;
                    let started = Instant::now();
                    let file = delivery.report(sequence).await;
                    elapsed += started.elapsed();
                    stages(times, "Report", file.length);
                }
                report(elapsed, iterations)
            })
        });
    });
    boundary(
        "Evidence Archive, Typical Sealed Execution",
        "same restarted host; Evidence.Archive of the archive frozen at seal, Files.Get, bounded Read, SHA-256 and a synced file",
    );
    group.bench_function("archive", |b| {
        let delivery = delivery.get_or_insert_with(|| executor.block_on(Delivery::new()));
        b.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    times.0.lock().unwrap().clear();
                    let started = Instant::now();
                    let file = delivery.archive().await;
                    elapsed += started.elapsed();
                    stages(times, "Archive", file.length);
                }
                report(elapsed, iterations)
            })
        });
    });
    boundary(
        "Plot Snapshots, Typical Sealed Execution",
        "same restarted host; every document plot bound through Documents.Get, then one Snapshots.List per plot at a 640 px panel; total over all plots",
    );
    group.bench_function("plots", |b| {
        let delivery = delivery.get_or_insert_with(|| executor.block_on(Delivery::new()));
        b.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    times.0.lock().unwrap().clear();
                    let started = Instant::now();
                    let slowest = delivery.plots().await;
                    elapsed += started.elapsed();
                    if env::var_os("DOKIME_BENCH_ONCE").is_some() {
                        println!("Plots | {} plots | slowest={:.6} ms", delivery.plots.len(), slowest.as_secs_f64() * 1000.0);
                    }
                    stages(times, "Plots", 0);
                }
                report(elapsed, iterations)
            })
        });
    });
    if let Some(delivery) = delivery {
        executor.block_on(delivery.server.close()).unwrap();
        services::remove_credentials(delivery.root.path(), &publication::credentials()).unwrap();
    }
    group.finish();
}

/// Serve a Storage Root on a Local Port With Generous Bench Deadlines.
pub(super) async fn host(root: &Path) -> services::Server {
    let settings = publication::settings();
    // Generous Bench Deadlines Let the Measurement Report Slow Paths Instead of Refusing Them.
    let operations = services::ClientSettings {
        read_timeout: Duration::from_secs(600),
        mutation_timeout: Duration::from_secs(600),
        watch_timeout: Duration::from_secs(600),
        max_cases: 1000,
        page_items: 1000,
        message_bytes: settings.placement.message_bytes.get(),
        stream_buffer: settings.placement.stream_window.get(),
    };
    services::serve(([127, 0, 0, 1], 0).into(), root, settings, operations).await.unwrap()
}

pub(super) fn subject() -> ReportSubject {
    ReportSubject {
        kind: Some(report_subject::Kind::Execution("execution".into())),
    }
}

pub(super) fn stages(times: &Times, case: &str, length: u64) {
    if env::var_os("DOKIME_BENCH_ONCE").is_none() {
        return;
    }
    println!("{case} Delivery | {length} bytes");
    for (phase, values) in times.0.lock().unwrap().iter() {
        println!("{case} Stage | {phase} | total={:.6} ms | {} calls", values.iter().sum::<f64>(), values.len());
    }
}

/// Create the Typical Execution With Its Document and Open Its Streams.
pub(super) fn open(store: &Store, document: Document) -> Vec<String> {
    let telemetry = (0..TELEMETRY).map(|index| format!("telemetry.{index}"));
    let status = (0..STATUS).map(|index| format!("status.{index}"));
    let streams = telemetry.chain(status).chain(["wifi".to_string()]).collect::<Vec<_>>();
    store.create(&execution(&streams, document)).unwrap();
    store.open_streams("execution", &streams).unwrap();
    streams
}

/// Write the Typical Capture's Rows and Records Within a Span of Whole Seconds, Returning Its Row Count.
pub(super) fn capture(store: &Store, seconds: Range<usize>) -> usize {
    let mut rows = 0;
    let telemetry = (0..TELEMETRY).map(|index| (format!("telemetry.{index}"), 1000));
    for (stream, rate) in telemetry.chain((0..STATUS).map(|index| (format!("status.{index}"), 10))) {
        let indices = seconds.start * rate..seconds.end * rate;
        let period_ns = 1_000_000_000 / rate as i64;
        for first in indices.clone().step_by(BATCH_ROWS) {
            let batch = (first..(first + BATCH_ROWS).min(indices.end))
                .map(|index| {
                    let value = Reading {
                        value: 10.0 * (index as f64 * 0.01).sin() + (index % 7) as f64 * 0.1,
                    };
                    sample(&stream, index, BASE_NS + index as i64 * period_ns, value.encode_to_vec())
                })
                .collect::<Vec<_>>();
            store.sample_batch("execution", &batch).unwrap();
        }
        rows += indices.len();
    }
    let scans = seconds.start.div_ceil(SCAN_SECONDS) * SCAN_ROWS..seconds.end.div_ceil(SCAN_SECONDS) * SCAN_ROWS;
    let batch = scans
        .map(|index| {
            let (scan, network) = (index / SCAN_ROWS, index % SCAN_ROWS);
            let value = Scan {
                channel: [1, 6, 11, 36, 40, 44, 48, 52, 100, 149][network % 10],
                rssi_dbm: -35.0 - network as f64 * 1.2 - (scan as f64 * 0.3).sin() * 4.0,
                ssid: format!("network-{network}"),
            };
            sample("wifi", index, BASE_NS + (scan * SCAN_SECONDS) as i64 * 1_000_000_000, value.encode_to_vec())
        })
        .collect::<Vec<_>>();
    rows += batch.len();
    for chunk in batch.chunks(BATCH_ROWS) {
        store.sample_batch("execution", chunk).unwrap();
    }
    for index in seconds.start.div_ceil(RECORD_SECONDS)..seconds.end.div_ceil(RECORD_SECONDS) {
        let record = RecordsAppendRequest {
            execution: "execution".into(),
            kind: "reading".into(),
            payload: Reading { value: index as f64 }.encode_to_vec(),
            source_at: Some(Timestamp {
                seconds: BASE_NS / 1_000_000_000 + (index * RECORD_SECONDS) as i64,
                nanos: 0,
            }),
            request_id: format!("record-{index}"),
        };
        let evidence = Evidence {
            kind: "record:reading".into(),
            payload: record.encode_to_vec(),
            source_ns: BASE_NS + (index * RECORD_SECONDS) as i64 * 1_000_000_000,
            ..Default::default()
        };
        store.append("execution", evidence).unwrap();
    }
    rows
}

/// The Typical Execution's Document: Summary, Records, Telemetry Series and Histogram, Wi-Fi and Status Plots.
pub(super) fn document() -> Document {
    let series = |stream: &str| plot(format!("{stream}.series"), Plot::time_series(FieldReference::new(stream).field("value")));
    let tiles = [Aggregate::Count, Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Rms]
        .into_iter()
        .map(|kind| SummaryTile {
            value: Some(aggregate("telemetry.0", kind)),
            ..Default::default()
        })
        .collect();
    Document {
        metadata: Some(Metadata {
            identifier: "delivery.report".into(),
            ..Default::default()
        }),
        sections: vec![
            section(
                "summary",
                vec![
                    card("statistics".into(), Body::Summary(Summary { tiles })),
                    card("records".into(), Body::Evidence(EvidenceCard::Records as i32)),
                ],
            ),
            section(
                "telemetry",
                (0..TELEMETRY)
                    .map(|index| series(&format!("telemetry.{index}")))
                    .chain([plot("telemetry.histogram".into(), Plot::histogram(FieldReference::new("telemetry.0").field("value")))])
                    .collect(),
            ),
            section(
                "wifi",
                vec![
                    plot(
                        "wifi.channels".into(),
                        Plot::heat_strip(FieldReference::new("wifi").field("rssi_dbm")).cells(FieldReference::new("wifi").field("channel")),
                    ),
                    plot("wifi.histogram".into(), Plot::histogram(FieldReference::new("wifi").field("rssi_dbm"))),
                    plot("wifi.series".into(), Plot::time_series(FieldReference::new("wifi").field("rssi_dbm"))),
                ],
            ),
            section("status", (0..STATUS).map(|index| series(&format!("status.{index}"))).collect()),
        ],
    }
}

/* === Private Functions === */

/// Write and Seal the Typical Execution Directly Into a Fresh Storage Root.
async fn seal(root: &Path) {
    let store = Store::open(root).unwrap();
    let streams = open(&store, document());
    let rows = capture(&store, 0..SECONDS);
    println!(
        "Delivery Source | {} streams | {rows} samples | {} plots | {} records",
        streams.len(),
        TELEMETRY + STATUS + 3,
        SECONDS.div_ceil(RECORD_SECONDS)
    );
    store.close_streams("execution").unwrap();
    store.close("execution", Termination::Completed, ExecutionOutcome::default()).unwrap();
    store.seal("execution", Default::default()).unwrap();
    store.shutdown_samples().await.unwrap();
}

pub(super) fn sample(stream: &str, index: usize, source_ns: i64, row: Vec<u8>) -> Sample {
    Sample {
        sequence: index as u64,
        source_ns,
        receipt_ns: source_ns,
        stream: stream.into(),
        row,
        ..Default::default()
    }
}

pub(super) fn execution(streams: &[String], document: Document) -> Execution {
    let streams = streams.iter().map(|stream| (stream.clone(), if stream == "wifi" { "Scan" } else { "Reading" })).collect::<Vec<_>>();
    let messages = vec![
        message("Reading", &[("value", Type::Double)]),
        message("Scan", &[("channel", Type::Uint32), ("rssi_dbm", Type::Double), ("ssid", Type::String)]),
    ];
    definition("execution", "cycle", messages, &streams, "reading", document)
}

/// One Execution of the `delivery` Package: Its Row Messages, Streams With Their Row Message, a `Reading` Record and Its Document.
pub(super) fn definition(
    identifier: &str, cycle: &str, messages: Vec<DescriptorProto>, streams: &[(String, &str)], record: &str, document: Document,
) -> Execution {
    let types = FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("delivery.proto".into()),
            package: Some("delivery".into()),
            syntax: Some("proto3".into()),
            message_type: messages,
            ..Default::default()
        }],
    };
    let suites = Suites {
        suites: vec![Suite {
            metadata: Some(Metadata {
                identifier: "delivery".into(),
                ..Default::default()
            }),
            templates: vec![TestCaseTemplate {
                metadata: Some(Metadata {
                    identifier: "delivery.capture".into(),
                    ..Default::default()
                }),
                documents: vec![document],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let records = RecordContracts {
        records: vec![RecordContract {
            metadata: Some(Metadata {
                identifier: record.into(),
                ..Default::default()
            }),
            message: "delivery.Reading".into(),
        }],
    };
    Execution {
        identifier: identifier.into(),
        cycle: cycle.into(),
        origin: Some(Origin {
            kind: Some(origin::Kind::PluginRun(PluginRun {
                plugin: "delivery".into(),
                template: "delivery.capture".into(),
                digest: vec![1],
                started_at: Some(Timestamp::default()),
                test_case: Some(TestCase {
                    streams: streams.iter().map(|(stream, _)| stream.clone()).collect(),
                    ..Default::default()
                }),
            })),
        }),
        catalog: Declarations {
            types: Some(types),
            streams: streams
                .iter()
                .map(|(stream, row)| model::StreamDeclaration {
                    id: stream.clone(),
                    row: format!("delivery.{row}"),
                    ..Default::default()
                })
                .collect(),
            sections: vec![
                model::Section {
                    r#type: "dokime.model.v1.RecordContracts".into(),
                    payload: records.encode_to_vec(),
                },
                model::Section {
                    r#type: "dokime.model.v1.Suites".into(),
                    payload: suites.encode_to_vec(),
                },
            ],
            ..Default::default()
        }
        .encode_to_vec(),
        ..Default::default()
    }
}

/// A Row Message Whose Fields Are Numbered in Order.
pub(super) fn message(name: &str, fields: &[(&str, Type)]) -> DescriptorProto {
    DescriptorProto {
        name: Some(name.into()),
        field: fields
            .iter()
            .enumerate()
            .map(|(index, (field, kind))| FieldDescriptorProto {
                name: Some((*field).into()),
                number: Some(index as i32 + 1),
                label: Some(Label::Optional as i32),
                r#type: Some(*kind as i32),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

pub(super) fn card(identifier: String, body: Body) -> Card {
    Card {
        metadata: Some(Metadata { identifier, ..Default::default() }),
        body: Some(body),
        ..Default::default()
    }
}

pub(super) fn section(identifier: &str, cards: Vec<Card>) -> Section {
    Section {
        metadata: Some(Metadata {
            identifier: identifier.into(),
            ..Default::default()
        }),
        cards,
        ..Default::default()
    }
}

pub(super) fn plot(identifier: String, plot: Plot) -> Card {
    card(identifier, Body::Plot(plot.message))
}

/* === Tests === */
