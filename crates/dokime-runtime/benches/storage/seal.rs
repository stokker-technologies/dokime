/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Seal-Time Freeze Measurements

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, env, time::Instant};

// Third Party Imports
use prost::Message;
use prost_types::Timestamp;
use prost_types::field_descriptor_proto::Type;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use tonic::transport::Channel;

// Local Imports
use super::{aggregate, boundary, publication, reports};
use dokime_client_internal::ExecutionOutcome;
use dokime_protocol::builders::Plot;
use dokime_protocol::host::cycles_client::CyclesClient;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence_client::EvidenceClient;
use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{
    ArchiveScope, CycleFilter, CyclePhase, CycleSelection, CyclesSealRequest, CyclesWatchRequest, EvidenceArchiveRequest, EvidenceSubject,
    ReportSubject, ReportsCreateRequest, evidence_subject, report_subject,
};
use dokime_protocol::model::{Aggregate, Document, EvidenceCard, FieldReference, Metadata, Summary, SummaryTile, Termination, card::Body};
use dokime_runtime::services;
use dokime_storage::{Evidence, Store};

/* === Constants === */

/// Runs Sealed One After Another Into One Store, in Seconds: Each Later Run Sees the Earlier Ones as History.
const RUNS: [usize; 7] = [1, 180, 1800, 1, 180, 1800, 1];
/// Signal Streams at 10 Hz.
const SIGNALS: usize = 6;
/// Level Streams at 1 Hz.
const LEVELS: usize = 4;
/// One Position Row per Signal Sample.
const POSITION_HZ: usize = 10;
/// One Spectrum Scan per Second Reports Sixteen Bands.
const BANDS: usize = 16;
/// One Event Record Every Ten Seconds.
const EVENT_SECONDS: usize = 10;
const BATCH_ROWS: usize = 4096;
const BASE_NS: i64 = 1_788_220_800_000_000_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct Point {
    #[prost(double, tag = "1")]
    x: f64,

    #[prost(double, tag = "2")]
    y: f64,

    #[prost(double, tag = "3")]
    heading: f64,

    #[prost(double, tag = "4")]
    speed: f64,
}

#[derive(Clone, PartialEq, Message)]
struct Band {
    #[prost(uint32, tag = "1")]
    channel: u32,

    #[prost(double, tag = "2")]
    level: f64,
}

/// The Clients a Seal Reaches on a Restarted Host.
struct Clients {
    server: services::Server,
    reports: ReportsClient<Channel>,
    evidence: EvidenceClient<Channel>,
    cycles: CyclesClient<Channel>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// The Seal Reproduction Runs Alone, Selected by Its Name.
pub(super) fn selected() -> bool {
    env::args().any(|argument| argument == "seal")
}

/// Seal Runs of 1 s, 3 min and 30 min One After Another, Freezing Each Report and Evidence Archive as Sealing Does.
pub(super) fn measure(executor: &Runtime, times: &reports::Times) {
    boundary(
        "Seal-Time Freeze, Growing Store",
        "runs of 1 s, 3 min and 30 min sealed one after another into one store: six 10 Hz signals, four 1 Hz levels, a 10 Hz position, sixteen spectrum bands per second, an event record every 10 s; a 29-plot document of time series, event-conditioned histograms, XY plots and heat strips; after each seal the host restarts, Reports.Create freezes the report, Evidence.Archive the archive, then Cycles.Seal seals the run's cycle; printed once per run",
    );
    let root = tempfile::tempdir().unwrap();
    for (index, seconds) in RUNS.into_iter().enumerate() {
        let samples = executor.block_on(seal(&root, index, seconds));
        executor.block_on(freeze(&root, index, seconds, samples, times));
    }
    let database = rusqlite::Connection::open(root.path().join("storage.sqlite3")).unwrap();
    let mut query = database.prepare("SELECT item FROM storage_rows WHERE kind=1").unwrap();
    let mut counts = BTreeMap::<String, usize>::new();
    for item in query.query_map([], |row| row.get::<_, String>(0)).unwrap() {
        let item = item.unwrap();
        *counts.entry(item.split(['/', ':']).next().unwrap_or_default().to_owned()).or_default() += 1;
    }
    println!("Seal Freeze Objects | {counts:?}");
}

/* === Private Functions === */

/// Write and Seal One Run Directly Into the Shared Storage Root, Returning Its Sample Count.
async fn seal(root: &TempDir, index: usize, seconds: usize) -> usize {
    let store = Store::open(root.path()).unwrap();
    let execution = identifier(index);
    let streams = streams();
    let messages = vec![
        reports::message("Reading", &[("value", Type::Double)]),
        reports::message("Point", &[("x", Type::Double), ("y", Type::Double), ("heading", Type::Double), ("speed", Type::Double)]),
        reports::message("Band", &[("channel", Type::Uint32), ("level", Type::Double)]),
    ];
    store.create(&reports::definition(&execution, &cycle(index), messages, &streams, "event", document())).unwrap();
    let names = streams.iter().map(|(stream, _)| stream.clone()).collect::<Vec<_>>();
    store.open_streams(&execution, &names).unwrap();
    let mut count = 0;
    for stream in &names {
        let (rate, rows) = match stream.as_str() {
            "position" => (POSITION_HZ, 1),
            "spectrum" => (1, BANDS),
            name if name.starts_with("signal.") => (10, 1),
            _ => (1, 1),
        };
        let total = seconds * rate * rows;
        for first in (0..total).step_by(BATCH_ROWS) {
            let batch = (first..(first + BATCH_ROWS).min(total))
                .map(|ordinal| {
                    let at = BASE_NS + (ordinal / rows) as i64 * 1_000_000_000 / rate as i64;
                    let phase = ordinal as f64 * 0.01;
                    let row = match stream.as_str() {
                        "position" => Point {
                            x: phase.cos() * 50.0,
                            y: phase.sin() * 30.0,
                            heading: (phase * 7.0) % 360.0,
                            speed: 2.0 + phase.sin().abs(),
                        }
                        .encode_to_vec(),
                        "spectrum" => Band {
                            channel: (ordinal % rows) as u32 + 1,
                            level: -40.0 - (ordinal % rows) as f64 * 1.5 - (phase * 0.3).sin() * 4.0,
                        }
                        .encode_to_vec(),
                        _ => reports::Reading {
                            value: 10.0 * phase.sin() + (ordinal % 7) as f64 * 0.1,
                        }
                        .encode_to_vec(),
                    };
                    reports::sample(stream, ordinal, at, row)
                })
                .collect::<Vec<_>>();
            store.sample_batch(&execution, &batch).unwrap();
        }
        count += total;
    }
    for event in 0..seconds.div_ceil(EVENT_SECONDS) {
        let source_ns = BASE_NS + (event * EVENT_SECONDS) as i64 * 1_000_000_000;
        let record = RecordsAppendRequest {
            execution: execution.clone(),
            kind: "event".into(),
            payload: reports::Reading { value: event as f64 }.encode_to_vec(),
            source_at: Some(Timestamp {
                seconds: source_ns / 1_000_000_000,
                nanos: 0,
            }),
            request_id: format!("event-{event}"),
        };
        let evidence = Evidence {
            kind: "record:event".into(),
            payload: record.encode_to_vec(),
            source_ns,
            ..Default::default()
        };
        store.append(&execution, evidence).unwrap();
    }
    store.close_streams(&execution).unwrap();
    store.close(&execution, Termination::Completed, ExecutionOutcome::default()).unwrap();
    store.seal(&execution, Default::default()).unwrap();
    store.shutdown_samples().await.unwrap();
    count
}

/// Restart the Host, Then Freeze the Report and Archive and Seal the Cycle.
async fn freeze(root: &TempDir, index: usize, seconds: usize, samples: usize, times: &reports::Times) {
    let execution = identifier(index);
    let mut clients = serve(root).await;
    times.0.lock().unwrap().clear();
    let started = Instant::now();
    let request = ReportsCreateRequest {
        subject: Some(ReportSubject {
            kind: Some(report_subject::Kind::Execution(execution.clone())),
        }),
        request_id: format!("freeze-{index}"),
        ..Default::default()
    };
    clients.reports.create(request).await.unwrap();
    let report_ms = started.elapsed().as_secs_f64() * 1000.0;
    let report = std::mem::take(&mut *times.0.lock().unwrap());
    let started = Instant::now();
    let request = EvidenceArchiveRequest {
        subject: Some(EvidenceSubject {
            kind: Some(evidence_subject::Kind::Execution(execution.clone())),
        }),
        at: None,
    };
    clients.evidence.archive(request).await.unwrap();
    let evidence_ms = started.elapsed().as_secs_f64() * 1000.0;
    let archive = std::mem::take(&mut *times.0.lock().unwrap());
    let started = Instant::now();
    seal_cycle(&mut clients, &cycle(index)).await;
    let cycle_ms = started.elapsed().as_secs_f64() * 1000.0;
    let sealing = std::mem::take(&mut *times.0.lock().unwrap());
    println!(
        "Seal Freeze | run {index} | {seconds} s | {index} earlier runs | {samples} samples | report={report_ms:.1} ms | evidence={evidence_ms:.1} ms | cycle seal={cycle_ms:.1} ms"
    );
    for (case, stages) in [("Report", report), ("Evidence", archive), ("Cycle Seal", sealing)] {
        let line = stages
            .iter()
            .filter(|(phase, _)| !phase.contains('.'))
            .map(|(phase, values)| format!("{phase}={:.1}", values.iter().sum::<f64>()))
            .collect::<Vec<_>>()
            .join(" | ");
        println!("Seal Freeze Stages | run {index} | {case} | {line}");
    }
    clients.server.close().await.unwrap();
    services::remove_credentials(root.path(), &publication::credentials()).unwrap();
}

async fn serve(root: &TempDir) -> Clients {
    let server = reports::host(root.path()).await;
    let endpoint = format!("http://{}", server.endpoint());
    Clients {
        reports: ReportsClient::connect(endpoint.clone()).await.unwrap(),
        evidence: EvidenceClient::connect(endpoint.clone()).await.unwrap(),
        cycles: CyclesClient::connect(endpoint).await.unwrap(),
        server,
    }
}

/// Seal the Run's Cycle and Observe Its Completion as the SDK's `cycle.seal()` Does.
async fn seal_cycle(clients: &mut Clients, cycle: &str) {
    let request = CyclesSealRequest {
        name: cycle.into(),
        request_id: format!("seal-{cycle}"),
    };
    clients.cycles.seal(request).await.unwrap();
    let request = CyclesWatchRequest {
        selection: Some(CycleSelection {
            cycle_ids: vec![cycle.into()],
            ..Default::default()
        }),
        filter: Some(CycleFilter {
            archive: Some(ArchiveScope::All as i32),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut frames = clients.cycles.watch(request).await.unwrap().into_inner();
    while let Some(frame) = frames.message().await.unwrap() {
        let Some(summary) = frame.cycles.into_iter().find(|summary| summary.cycle_id == cycle) else { continue };
        assert_ne!(summary.phase, CyclePhase::SealFailed as i32, "Seal Cycle: {:?}", summary.seal_failure);
        if summary.sealed {
            return;
        }
    }
    panic!("Seal Cycle: The Watch Ended Before the Seal Settled");
}

fn identifier(index: usize) -> String {
    format!("run-{index:02}")
}

fn cycle(index: usize) -> String {
    format!("cycle-{index:02}")
}

fn streams() -> Vec<(String, &'static str)> {
    let signals = (0..SIGNALS).map(|index| (format!("signal.{index}"), "Reading"));
    let levels = (0..LEVELS).map(|index| (format!("level.{index}"), "Reading"));
    signals.chain(levels).chain([("position".into(), "Point"), ("spectrum".into(), "Band")]).collect()
}

/// A Document Shaped Like a Real One: Summary, Records, Time Series, Event-Conditioned Histograms, XY Plots and Heat Strips.
fn document() -> Document {
    let value = |stream: &str| FieldReference::new(stream).field("value");
    let monitored = |stream: String| {
        vec![
            reports::plot(format!("{stream}.series"), Plot::time_series(value(&stream))),
            reports::plot(format!("{stream}.events"), Plot::histogram(value(&stream).at("event"))),
        ]
    };
    let tiles = [Aggregate::Count, Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Rms]
        .into_iter()
        .map(|kind| SummaryTile {
            value: Some(aggregate("signal.0", kind)),
            ..Default::default()
        })
        .collect();
    let histogram = |index: usize| reports::plot(format!("signal.{index}.histogram"), Plot::histogram(value(&format!("signal.{index}"))));
    let position = FieldReference::new("position");
    let spectrum = FieldReference::new("spectrum");
    Document {
        metadata: Some(Metadata {
            identifier: "seal.report".into(),
            ..Default::default()
        }),
        sections: vec![
            reports::section(
                "summary",
                vec![
                    reports::card("statistics".into(), Body::Summary(Summary { tiles })),
                    reports::card("records".into(), Body::Evidence(EvidenceCard::Records as i32)),
                ],
            ),
            reports::section(
                "signals",
                (0..SIGNALS).flat_map(|index| monitored(format!("signal.{index}"))).chain((0..3).map(histogram)).collect(),
            ),
            reports::section(
                "position",
                vec![
                    reports::plot("position.path".into(), Plot::xy(position.field("y")).x(position.field("x"))),
                    reports::plot("position.speed".into(), Plot::xy(position.field("speed")).x(position.field("heading"))),
                    reports::plot("position.events".into(), Plot::xy(position.field("y").at("event")).x(position.field("x").at("event"))),
                ],
            ),
            reports::section(
                "spectrum",
                vec![
                    reports::plot("spectrum.bands".into(), Plot::heat_strip(spectrum.field("level")).cells(spectrum.field("channel"))),
                    reports::plot("spectrum.histogram".into(), Plot::histogram(spectrum.field("level"))),
                    reports::plot("spectrum.series".into(), Plot::time_series(spectrum.field("level"))),
                ],
            ),
            reports::section("levels", (0..LEVELS).flat_map(|index| monitored(format!("level.{index}"))).collect()),
        ],
    }
}

/* === Tests === */
