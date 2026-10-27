/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Plot Admission Path, Shared By Its Integration Guard And Its Full-Scale System Story.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use tonic::Code;

// Local Imports
use super::wire::{WireClient, WireStream, runtime};
use dokime::client::contracts;
use dokime::domain::model;
use dokime::storage::sqlite::counters;
use dokime_server::grpc::{Config, Server};
use {dokime::Identifier, dokime::Timestamp, dokime::Value, dokime::telemetry::TelemetryBatch, dokime::telemetry::TelemetrySample};

/* === Constants === */

// The Query Limit Of One Plot Request, Independent Of The Message Budget.
const QUERY_POINTS: u64 = 100_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One Size Of The Admission Path: Saved Rows, The Message Budget And The Two Requested Windows.
#[derive(Clone, Copy, Debug)]
pub struct Admission {
    pub rows: u64,
    pub batch: u64,
    pub message_bytes: Option<usize>,
    pub window: u64,
    pub oversized: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Oversized Plot Responses Reduce Display Points Without Repeating The Storage Query.
pub async fn admission(size: Admission) {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    super::register_fixture_progress_execution(&handle, super::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    for start in (0..size.rows).step_by(size.batch as usize) {
        publish(&handle, &id, start, size.batch).await;
    }
    let mut config = Config::local();
    if let Some(bytes) = size.message_bytes {
        config = config.with_max_message_bytes(bytes);
    }
    let server = Server::start(handle.clone(), config).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    let before = counters().data_scan_rows;
    let served = first_frame(&client, size.window).await;
    let window_scan = counters().data_scan_rows - before;
    assert_eq!(
        served.kind,
        model::plots::wire::PlotStreamKind::Snapshot,
        "a {}-point window is served: {:?}",
        size.window,
        served.envelope.as_ref().map(|envelope| &envelope.warnings)
    );
    let points = served_points(&served);
    assert!((1..=size.window as usize).contains(&points), "the window keeps its budget: {points}");

    let before = counters().data_scan_rows;
    let mut watch = open_watch(&client, size.oversized).await;
    let reduced = data_frame(&mut watch).await;
    let reduced_scan = counters().data_scan_rows - before;
    assert_eq!(
        reduced.kind,
        model::plots::wire::PlotStreamKind::Snapshot,
        "{:?}",
        reduced.envelope.as_ref().map(|envelope| &envelope.warnings)
    );
    let envelope = reduced.envelope.as_ref().unwrap();
    assert!(!envelope.warnings.is_empty());
    let maximum = envelope.payload.as_ref().unwrap().max_points as usize;
    assert!((size.window as usize..size.oversized as usize).contains(&maximum), "{maximum}");
    assert!((1..=maximum).contains(&served_points(&reduced)), "the reduced snapshot keeps its budget");
    // Reducing An Oversized Response Reads The Saved Rows Once, Exactly Like The Window That Fits.
    assert_eq!(reduced_scan, window_scan, "the reduction reuses the one storage read");
    assert!(window_scan >= size.rows, "the read covers every saved row: {window_scan}");
    // The plot stays in the watch under its reduced budget: a later row reaches it.
    publish(&handle, &id, size.rows, 1).await;
    let followed = data_frame(&mut watch).await;
    assert_ne!(
        followed.kind,
        model::plots::wire::PlotStreamKind::Error,
        "{:?}",
        followed.envelope.as_ref().map(|envelope| &envelope.warnings)
    );
    drop(watch);
    // The unary query reduces the same plot to the same budget.
    let unary = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![request(size.oversized)] })
        .await
        .unwrap()
        .into_inner();
    let envelope = &unary.envelopes[0];
    assert!(!envelope.warnings.is_empty());
    assert_eq!(envelope.payload.as_ref().unwrap().max_points as usize, maximum);

    let beyond = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots {
            requests: vec![request(QUERY_POINTS + 1)],
        })
        .await
        .unwrap_err();
    assert_eq!(beyond.code(), Code::InvalidArgument, "{beyond:?}");

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub fn request(max_points: u64) -> model::plots::wire::PlotRequest {
    model::plots::wire::PlotRequest {
        plot_id: Some("fixture.progress.plot".into()),
        plot_type: model::plots::wire::PlotType::TimeSeries,
        stream: "fixture.progress".into(),
        y_keys: vec!["progress".into()],
        max_points: Some(max_points),
        x_scale: model::plots::wire::AxisScale::Unspecified,
        y_scale: model::plots::wire::AxisScale::Unspecified,
        marker_record_types: Vec::new(),
        filters: Vec::new(),
        x_key: None,
        series_by: None,
        x_expression: None,
        y_expressions: Vec::new(),
        series_by_expression: None,
        z_stream: None,
        z_field: None,
        unit: None,
        window_s: None,
        scope: None,
        technique: contracts::plots::DownsampleTechnique::Unspecified,
        z_expression: None,
        fft: None,
        interval: None,
        histogram: None,
        analysis: None,
        summary_window: None,
    }
}

pub async fn first_frame(client: &WireClient, max_points: u64) -> model::plots::wire::PlotStreamFrame {
    let mut stream = open_watch(client, max_points).await;
    data_frame(&mut stream).await
}

// The next frame that carries a plot's data or error, past the watch's acknowledgements and keepalives.
pub async fn data_frame(watch: &mut WireStream<model::plots::wire::PlotStreamFrame>) -> model::plots::wire::PlotStreamFrame {
    let mut frame = watch.message().await.unwrap().unwrap();
    while frame.kind == model::plots::wire::PlotStreamKind::Unspecified {
        frame = watch.message().await.unwrap().unwrap();
    }
    frame
}

pub async fn open_watch(client: &WireClient, max_points: u64) -> WireStream<model::plots::wire::PlotStreamFrame> {
    client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: vec![request(max_points)] }).await.unwrap().into_inner()
}

pub fn served_points(frame: &model::plots::wire::PlotStreamFrame) -> usize {
    let Some(payload) = frame.envelope.as_ref().and_then(|envelope| envelope.payload.as_ref()) else {
        return 0;
    };
    payload.series.iter().map(|series| series.points.len()).sum()
}

pub async fn publish(handle: &dokime::RuntimeHandle, id: &Identifier, start: u64, count: u64) {
    let samples = (start..start + count).map(|index| sample(id, index, clock(index), None)).collect();
    save(handle, id, samples).await;
}

pub async fn publish_scoped(
    handle: &dokime::RuntimeHandle, id: &Identifier, scope: &model::execution::RecordScope, start: u64, count: u64,
) {
    let samples = (start..start + count).map(|index| sample(id, index, minutes(index), Some(scope))).collect();
    save(handle, id, samples).await;
}

/* === Private Functions === */

// Millisecond Steps From Midnight, So Tens Of Thousands Of Rows Keep Distinct Times.
fn clock(index: u64) -> Timestamp {
    let (hours, minutes, seconds, milliseconds) = (index / 3_600_000, index / 60_000 % 60, index / 1_000 % 60, index % 1_000);
    Timestamp::new(format!("2026-09-13T{hours:02}:{minutes:02}:{seconds:02}.{milliseconds:03}Z"))
}

fn minutes(index: u64) -> Timestamp {
    Timestamp::new(format!("2026-09-13T00:{:02}:{:02}.000Z", index / 60, index % 60))
}

fn sample(id: &Identifier, index: u64, at: Timestamp, scope: Option<&model::execution::RecordScope>) -> TelemetrySample {
    let mut sample = TelemetrySample::new(dokime::model::identity::new_id(), id.clone(), index, at, "fixture");
    sample.values.insert("progress".into(), Value::Number(index as f64));
    sample.scope = scope.cloned();
    let payload = dokime::harmos::Message::encode_to_vec(&super::ProgressRow { progress: index as f64 });
    sample.with_storage_payload(payload)
}

async fn save(handle: &dokime::RuntimeHandle, id: &Identifier, samples: Vec<TelemetrySample>) {
    let batch = TelemetryBatch::new(dokime::model::identity::new_id(), id.clone(), samples);
    handle.publish_telemetry_batch(batch).await.unwrap();
    handle.storage().flush().await.unwrap();
}

/* === Tests === */
