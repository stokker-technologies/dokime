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
use dokime_domain::model;
use {
    dokime_runtime::Identifier, dokime_runtime::Timestamp, dokime_runtime::Value, dokime_runtime::telemetry::TelemetryBatch,
    dokime_runtime::telemetry::TelemetrySample,
};

/* === Constants === */
const ROWS: u64 = 40_000;
const BATCH: u64 = 2_000;
const STORED: u64 = 300;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
// A 10,000-point window is served; the 100,000-point cap is refused with
// InvalidArgument naming what the plot can take, and that maximum is served.
#[tokio::test]
async fn plot_points_admission() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    support::register_fixture_progress_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    for start in (0..ROWS).step_by(BATCH as usize) {
        publish(&handle, &id, start, BATCH).await;
    }
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    let served = first_frame(&client, 10_000).await;
    assert_eq!(
        served.kind,
        model::plots::wire::PlotStreamKind::Snapshot,
        "a 10,000-point window is served: {:?}",
        served.envelope.as_ref().map(|envelope| &envelope.warnings)
    );
    let points = served_points(&served);
    assert!((1..=10_000).contains(&points), "the window keeps its budget: {points}");

    let mut watch = open_watch(&client, 100_000).await;
    let refused = watch.message().await.unwrap().unwrap();
    assert_eq!(refused.kind, model::plots::wire::PlotStreamKind::Error);
    let message = refused.envelope.as_ref().unwrap().warnings[0].clone();
    assert!(!message.contains("plot admission failed"), "the refusal is precise: {message}");
    let maximum = maximum_points(&message);
    assert!((10_000..100_000).contains(&maximum), "the refusal names a maximum between the window and the cap: {message}");
    // The bound is measured on the plot's payload, so the unary query names
    // the same maximum over the same rows.
    let unary = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![request(100_000)] })
        .await
        .unwrap_err();
    assert_eq!(unary.code(), Code::InvalidArgument, "{unary:?}");
    assert_eq!(maximum_points(unary.message()), maximum);
    // A refused plot leaves the watch: a later row reaches it as nothing, not
    // as an append that would hide the missing window.
    publish(&handle, &id, ROWS, 1).await;
    // TODO(source-hook): end a refused plot watch with a terminal frame so silence is observable without a wall-clock window.
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(2), watch.message()).await.is_err(),
        "a refused plot is not served appends"
    );
    drop(watch);

    let largest = first_frame(&client, maximum as u64).await;
    assert_eq!(
        largest.kind,
        model::plots::wire::PlotStreamKind::Snapshot,
        "the named maximum is served: {:?}",
        largest.envelope.as_ref().map(|envelope| &envelope.warnings)
    );

    let beyond = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![request(100_001)] })
        .await
        .unwrap_err();
    assert_eq!(beyond.code(), Code::InvalidArgument, "{beyond:?}");

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A running execution's watch opens with the snapshot of every plot it names,
// then follows with appends. Once sealed and reopened after a restart, the
// in-memory window is empty, yet window All still opens with the stored rows.
#[tokio::test]
async fn sealed_window_all() {
    use dokime_runtime::runtime::command::CommandContext;
    let root = tempfile::tempdir().unwrap();
    let config = || crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.plots", "Plots"));
    let gate = std::sync::Arc::new(Gate::default());
    let mut owner = Runtime::with_execution_fixture(config(), "fixture", gate.clone()).await.unwrap();
    let handle = owner.handle();
    let suite = support::fixture_suite(&["passed"]);
    support::register_fixture_progress_execution(&handle, suite.clone()).await;
    let cycle = support::cycle_with_cases(
        &handle,
        support::metadata("plots.fixture", "Plots"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await;
    let execution_id = handle.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
    handle.start_execution(execution_id, model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    gate.started.notified().await;
    let scope = handle.watch_state().latest().state().cycle_view(&cycle).unwrap().test_executions[0].scope.clone();
    let execution = scope.test_execution_id.clone().unwrap();
    let id = Identifier::new("fixture.progress").unwrap();
    publish_scoped(&handle, &id, &scope, 0, STORED).await;
    let wire_scope = model::identity::wire::EvidenceScope {
        cycle_id: Some(cycle.to_string()),
        execution_id: Some(execution.to_string()),
        ..Default::default()
    };
    let requests = || {
        ["fixture.progress.first", "fixture.progress.second"]
            .map(|plot_id| model::plots::wire::PlotRequest {
                plot_id: Some(plot_id.into()),
                scope: Some(wire_scope.clone()),
                ..request(10_000)
            })
            .to_vec()
    };

    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut watch =
        client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: requests() }).await.unwrap().into_inner();
    for _ in 0..2 {
        let frame = watch.message().await.unwrap().unwrap();
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot, "{:?}", frame.envelope.map(|envelope| envelope.warnings));
        assert_eq!(served_points(&frame), STORED as usize, "a running execution opens with its window");
    }
    publish_scoped(&handle, &id, &scope, STORED, 1).await;
    for _ in 0..2 {
        let frame = watch.message().await.unwrap().unwrap();
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Append);
        assert_eq!(served_points(&frame), 1, "a running execution follows with the new row");
    }
    drop(watch);
    gate.release.notify_one();
    support::wait_for_cycle_outcome(&handle, &cycle, dokime_domain::model::execution::result::ResultStatus::Passed).await;
    support::on_state_change(&handle, async || handle.execution_saved_checkpoint(execution.clone()).await.ok()).await;
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();

    let mut owner = Runtime::with_execution_fixture(config(), "fixture", std::sync::Arc::new(Gate::default())).await.unwrap();
    let handle = owner.handle();
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut watch =
        client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: requests() }).await.unwrap().into_inner();
    for _ in 0..2 {
        let frame = watch.message().await.unwrap().unwrap();
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot, "{:?}", frame.envelope.map(|envelope| envelope.warnings));
        assert_eq!(served_points(&frame), STORED as usize + 1, "a sealed execution opens with its stored rows");
    }
    drop(watch);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */
fn request(max_points: u64) -> model::plots::wire::PlotRequest {
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
        cursor: None,
        technique: contracts::plots::DownsampleTechnique::Unspecified,
        z_expression: None,
        fft: None,
        interval: None,
        histogram: None,
        analysis: None,
        summary_window: None,
    }
}

async fn first_frame(client: &WireClient, max_points: u64) -> model::plots::wire::PlotStreamFrame {
    let mut stream = open_watch(client, max_points).await;
    stream.message().await.unwrap().unwrap()
}

async fn open_watch(client: &WireClient, max_points: u64) -> WireStream<model::plots::wire::PlotStreamFrame> {
    client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: vec![request(max_points)] }).await.unwrap().into_inner()
}

fn served_points(frame: &model::plots::wire::PlotStreamFrame) -> usize {
    frame
        .envelope
        .as_ref()
        .and_then(|envelope| envelope.payload.as_ref())
        .map(|payload| payload.series.iter().map(|series| series.points.len()).sum())
        .unwrap_or_default()
}

async fn publish(handle: &dokime_runtime::RuntimeHandle, id: &Identifier, start: u64, count: u64) {
    let samples = (start..start + count)
        .map(|index| {
            let at = Timestamp::new(format!(
                "2026-09-13T{:02}:{:02}:{:02}.{:03}Z",
                index / 3_600_000,
                index / 60_000 % 60,
                index / 1_000 % 60,
                index % 1_000
            ));
            let mut sample = TelemetrySample::new(dokime_runtime::model::identity::new_id(), id.clone(), index, at, "fixture");
            sample.values.insert("progress".into(), Value::Number(index as f64));
            sample.with_storage_payload(harmos::Message::encode_to_vec(&support::ProgressRow { progress: index as f64 }))
        })
        .collect();
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), id.clone(), samples)).await.unwrap();
}

async fn publish_scoped(
    handle: &dokime_runtime::RuntimeHandle, id: &Identifier, scope: &model::execution::RecordScope, start: u64, count: u64,
) {
    let samples = (start..start + count)
        .map(|index| {
            let at = Timestamp::new(format!("2026-09-13T00:{:02}:{:02}.000Z", index / 60, index % 60));
            let mut sample = TelemetrySample::new(dokime_runtime::model::identity::new_id(), id.clone(), index, at, "fixture");
            sample.values.insert("progress".into(), Value::Number(index as f64));
            sample.scope = Some(scope.clone());
            sample.with_storage_payload(harmos::Message::encode_to_vec(&support::ProgressRow { progress: index as f64 }))
        })
        .collect();
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), id.clone(), samples)).await.unwrap();
}

fn maximum_points(message: &str) -> usize {
    message
        .split("at most ")
        .nth(1)
        .and_then(|rest| rest.split(' ').next())
        .and_then(|number| number.replace(',', "").parse().ok())
        .unwrap_or_else(|| panic!("the refusal names a maximum: {message}"))
}

/* === Tests === */
