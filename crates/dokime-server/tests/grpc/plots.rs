/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use dokime_app::{
    core::{Identifier, Timestamp, Value},
    telemetry::{TelemetryBatch, TelemetrySample},
};

/* === Constants === */
/// More rows than one message can carry as points, so the cap is refused.
const ROWS: u64 = 40_000;
const BATCH: u64 = 2_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
// A 10,000-point window is served; the 100,000-point cap is refused with
// InvalidArgument naming what the plot can take, and that maximum is served.
pub(crate) async fn plot_points_admission() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    common::register_fixture_progress_execution(&handle, common::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    for start in (0..ROWS).step_by(BATCH as usize) {
        publish(&handle, &id, start, BATCH).await;
    }
    let server = Server::start(handle.clone(), Config::local())
        .await
        .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    let served = first_frame(&client, 10_000).await;
    assert_eq!(
        served.kind,
        contracts::plots::PlotStreamKind::Snapshot,
        "a 10,000-point window is served: {:?}",
        served.envelope.as_ref().map(|envelope| &envelope.warnings)
    );
    let points = served_points(&served);
    assert!(
        (1..=10_000).contains(&points),
        "the window keeps its budget: {points}"
    );

    let mut watch = open_watch(&client, 100_000).await;
    let refused = watch.message().await.unwrap().unwrap();
    assert_eq!(refused.kind, contracts::plots::PlotStreamKind::Error);
    let message = refused.envelope.as_ref().unwrap().warnings[0].clone();
    assert!(
        !message.contains("plot admission failed"),
        "the refusal is precise: {message}"
    );
    let maximum = maximum_points(&message);
    assert!(
        (10_000..100_000).contains(&maximum),
        "the refusal names a maximum between the window and the cap: {message}"
    );
    // A refused plot leaves the watch: a later row reaches it as nothing, not
    // as an append that would hide the missing window.
    publish(&handle, &id, ROWS, 1).await;
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(2), watch.message())
            .await
            .is_err(),
        "a refused plot is not served appends"
    );
    drop(watch);
    let unary = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::PlotSnapshotsRequest {
            requests: vec![request(100_000)],
        })
        .await
        .unwrap_err();
    assert_eq!(unary.code(), Code::InvalidArgument, "{unary:?}");
    assert_eq!(maximum_points(unary.message()), maximum);

    let largest = first_frame(&client, maximum as u64).await;
    assert_eq!(
        largest.kind,
        contracts::plots::PlotStreamKind::Snapshot,
        "the named maximum is served: {:?}",
        largest.envelope.as_ref().map(|envelope| &envelope.warnings)
    );

    let beyond = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::PlotSnapshotsRequest {
            requests: vec![request(100_001)],
        })
        .await
        .unwrap_err();
    assert_eq!(beyond.code(), Code::InvalidArgument, "{beyond:?}");

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */
fn request(max_points: u64) -> contracts::plots::PlotRequest {
    contracts::plots::PlotRequest {
        plot_id: Some("fixture.progress.plot".into()),
        plot_type: contracts::plots::PlotType::TimeSeries,
        stream: "fixture.progress".into(),
        y_keys: vec!["progress".into()],
        max_points: Some(max_points),
        x_scale: contracts::plots::AxisScale::Unspecified,
        y_scale: contracts::plots::AxisScale::Unspecified,
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
        cursor: None,
        technique: contracts::plots::DownsampleTechnique::Unspecified,
        z_expression: None,
        fft: None,
        interval: None,
        histogram: None,
        analysis: None,
    }
}

async fn first_frame(client: &WireClient, max_points: u64) -> contracts::plots::PlotStreamFrame {
    let mut stream = open_watch(client, max_points).await;
    stream.message().await.unwrap().unwrap()
}

async fn open_watch(
    client: &WireClient,
    max_points: u64,
) -> WireStream<contracts::plots::PlotStreamFrame> {
    client
        .download::<contracts::plots::Watch>(contracts::plots::PlotWatchRequest {
            requests: vec![request(max_points)],
        })
        .await
        .unwrap()
        .into_inner()
}

fn served_points(frame: &contracts::plots::PlotStreamFrame) -> usize {
    frame
        .envelope
        .as_ref()
        .and_then(|envelope| envelope.payload.as_ref())
        .map(|payload| {
            payload
                .series
                .iter()
                .map(|series| series.points.len())
                .sum()
        })
        .unwrap_or_default()
}

/// Publishes `count` progress rows from `start`, one millisecond apart.
async fn publish(handle: &dokime_app::RuntimeHandle, id: &Identifier, start: u64, count: u64) {
    let samples = (start..start + count)
        .map(|index| {
            let at = Timestamp::new(format!(
                "2026-09-13T{:02}:{:02}:{:02}.{:03}Z",
                index / 3_600_000,
                index / 60_000 % 60,
                index / 1_000 % 60,
                index % 1_000
            ));
            let mut sample = TelemetrySample::new(id.clone(), index, at, "fixture");
            sample
                .values
                .insert("progress".into(), Value::Number(index as f64));
            sample
        })
        .collect();
    handle
        .publish_telemetry_batch(TelemetryBatch::new(id.clone(), samples))
        .await
        .unwrap();
}

/// The number the refusal names after "at most".
fn maximum_points(message: &str) -> usize {
    message
        .split("at most ")
        .nth(1)
        .and_then(|rest| rest.split(' ').next())
        .and_then(|number| number.replace(',', "").parse().ok())
        .unwrap_or_else(|| panic!("the refusal names a maximum: {message}"))
}

/* === Tests === */
