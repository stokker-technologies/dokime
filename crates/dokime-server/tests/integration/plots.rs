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
use dokime::domain::model;
use {dokime::Identifier, dokime::Timestamp, dokime::Value, dokime::telemetry::TelemetryBatch, dokime::telemetry::TelemetrySample};

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

#[tokio::test]
async fn sample_watch_fixed_frontier_and_resume() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    support::register_fixture_progress_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 0, 2).await;
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let selection = contracts::telemetry::ListSamples {
        streams: vec![id.to_string()],
        limit: 1,
        schema_versions: Vec::new(),
        sources: Vec::new(),
        signals: Vec::new(),
        scope: None,
        after_stream_cursors: Vec::new(),
        start: None,
        end: None,
        latest: false,
    };
    let mut watch = client
        .download::<contracts::telemetry::WatchSamples>(contracts::telemetry::WatchSamples {
            selection: Some(selection.clone()),
            follow: false,
        })
        .await
        .unwrap()
        .into_inner();
    let first = watch.message().await.unwrap().unwrap();
    assert_eq!(first.samples.len(), 1);
    // A New Durable Row Must Not Extend An Already Open Finite Replay.
    publish(&handle, &id, 2, 1).await;
    let mut count = first.samples.len();
    let mut through = first.through_stream_cursors;
    while let Some(page) = watch.message().await.unwrap() {
        count += page.samples.len();
        through = page.through_stream_cursors;
    }
    assert_eq!(count, 2);
    let after = through
        .into_iter()
        .map(|entry| model::telemetry::wire::ListTelemetrySamplesRequestAfterStreamCursorsEntry { key: entry.key, value: entry.value })
        .collect();
    let mut resumed = client
        .download::<contracts::telemetry::WatchSamples>(contracts::telemetry::WatchSamples {
            selection: Some(contracts::telemetry::ListSamples { after_stream_cursors: after, ..selection }),
            follow: false,
        })
        .await
        .unwrap()
        .into_inner();
    let mut count = 0;
    while let Some(page) = resumed.message().await.unwrap() {
        count += page.samples.len();
    }
    assert_eq!(count, 1);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Oversized Plot Responses Reduce Display Points Without Repeating The Storage Query.
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
    let reduced = data_frame(&mut watch).await;
    assert_eq!(
        reduced.kind,
        model::plots::wire::PlotStreamKind::Snapshot,
        "{:?}",
        reduced.envelope.as_ref().map(|envelope| &envelope.warnings)
    );
    let envelope = reduced.envelope.as_ref().unwrap();
    assert!(!envelope.warnings.is_empty());
    let maximum = envelope.payload.as_ref().unwrap().max_points as usize;
    assert!((10_000..100_000).contains(&maximum));
    assert!((1..=maximum).contains(&served_points(&reduced)), "the reduced snapshot keeps its budget");
    // The plot stays in the watch under its reduced budget: a later row reaches it.
    publish(&handle, &id, ROWS, 1).await;
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
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![request(100_000)] })
        .await
        .unwrap()
        .into_inner();
    let envelope = &unary.envelopes[0];
    assert!(!envelope.warnings.is_empty());
    assert_eq!(envelope.payload.as_ref().unwrap().max_points as usize, maximum);

    let beyond = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![request(100_001)] })
        .await
        .unwrap_err();
    assert_eq!(beyond.code(), Code::InvalidArgument, "{beyond:?}");

    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Late Rows Retain Time Order And Exact Full-Selection Summary Counts.
#[tokio::test]
async fn late_samples_keep_time_order() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    support::register_fixture_progress_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 10, 10).await;
    publish(&handle, &id, 0, 10).await;
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let reply = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![request(100)] })
        .await
        .unwrap()
        .into_inner();
    let payload = reply.envelopes[0].payload.as_ref().unwrap();
    assert_eq!(
        payload.series[0].points.iter().map(|point| point.y.unwrap()).collect::<Vec<_>>(),
        (0..20).map(f64::from).collect::<Vec<_>>()
    );
    let summary = &payload.summaries[0];
    assert_eq!(summary.count, 20);
    assert_eq!(summary.min, Some(0.0));
    assert_eq!(summary.max, Some(19.0));
    assert!((summary.mean.unwrap() - 9.5).abs() <= 16.0 * f64::EPSILON);
    assert_eq!(summary.median, Some(9.5));
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// Running And Reopened Sealed Executions Read Complete Saved Storage Snapshots.
#[tokio::test]
async fn sealed_window_all() {
    use dokime::runtime::command::CommandContext;
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
    // The watch acknowledges every plot before any read, so none sits silent while the snapshots run.
    let mut acknowledged = Vec::new();
    for _ in 0..2 {
        let frame = watch.message().await.unwrap().unwrap();
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Unspecified, "the acknowledgement precedes every snapshot");
        let envelope = frame.envelope.unwrap();
        assert!(envelope.payload.is_none(), "an acknowledgement carries no data");
        acknowledged.push(envelope.plot_id);
    }
    assert_eq!(acknowledged, ["fixture.progress.first", "fixture.progress.second"]);
    for _ in 0..2 {
        let frame = data_frame(&mut watch).await;
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot, "{:?}", frame.envelope.map(|envelope| envelope.warnings));
        assert_eq!(served_points(&frame), STORED as usize, "a running execution opens with its saved rows");
    }
    publish_scoped(&handle, &id, &scope, STORED, 1).await;
    let saved = handle.storage().cursor_saved();
    let mut replacement_cut = None;
    for _ in 0..2 {
        let frame = loop {
            let frame = data_frame(&mut watch).await;
            if frame.envelope.as_ref().is_some_and(|envelope| envelope.revision >= saved.sequence) {
                break frame;
            }
        };
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Replace);
        assert_eq!(served_points(&frame), STORED as usize + 1);
        let envelope = frame.envelope.unwrap();
        let payload = envelope.payload.unwrap();
        assert_eq!(payload.series[0].points.last().unwrap().y, Some(STORED as f64));
        if let Some(cut) = replacement_cut {
            assert_eq!(envelope.revision, cut);
        }
        replacement_cut = Some(envelope.revision);
    }
    drop(watch);
    gate.release.notify_one();
    support::wait_for_cycle_outcome(&handle, &cycle, dokime::domain::model::execution::result::ResultStatus::Passed).await;
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
        let frame = data_frame(&mut watch).await;
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot, "{:?}", frame.envelope.map(|envelope| envelope.warnings));
        assert_eq!(served_points(&frame), STORED as usize + 1, "a sealed execution opens with its stored rows");
    }
    drop(watch);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Saved Frontier Refreshes Only Plots Reading The Changed Stream.
#[tokio::test]
async fn saved_frontier_refreshes_affected_plots() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    support::register_fixture_progress_and_idle_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 0, 10).await;
    let server = Server::start(handle.clone(), Config::local()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let progress = ["fixture.progress.a", "fixture.progress.b", "fixture.progress.c"];
    let idle = "fixture.idle.plot";
    let mut requests = progress
        .map(|plot_id| model::plots::wire::PlotRequest {
            plot_id: Some(plot_id.into()),
            ..request(10_000)
        })
        .to_vec();
    requests.push(model::plots::wire::PlotRequest {
        plot_id: Some(idle.into()),
        stream: "fixture.idle".into(),
        ..request(10_000)
    });
    let mut watch = client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests }).await.unwrap().into_inner();
    let mut initial = std::collections::BTreeMap::new();
    let mut initial_cut = None;
    for _ in 0..4 {
        let frame = data_frame(&mut watch).await;
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot);
        let envelope = frame.envelope.unwrap();
        if let Some(cut) = initial_cut {
            assert_eq!(envelope.revision, cut);
        }
        initial_cut = Some(envelope.revision);
        let series = envelope.payload.unwrap().series;
        let points: usize = series.iter().map(|series| series.points.len()).sum();
        assert_eq!(points, if envelope.plot_id == idle { 0 } else { 10 });
        assert!(initial.insert(envelope.plot_id, series).is_none());
    }
    let before = handle.storage().cursor_saved();
    publish(&handle, &id, 10, 1).await;
    let saved = handle.storage().cursor_saved();
    let changed = handle.plot_streams_saved_between(&before, &saved).await.unwrap();
    assert_eq!(changed, std::collections::BTreeSet::from(["fixture.progress".to_owned()]));
    let mut refreshed = std::collections::BTreeSet::new();
    let mut replacement_cut = None;
    while refreshed.len() < progress.len() {
        let frame = data_frame(&mut watch).await;
        let envelope = frame.envelope.unwrap();
        if envelope.revision < saved.sequence {
            continue;
        }
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Replace);
        if let Some(cut) = replacement_cut {
            assert_eq!(envelope.revision, cut);
        }
        replacement_cut = Some(envelope.revision);
        let series = envelope.payload.unwrap().series;
        let previous = &initial[&envelope.plot_id];
        assert!(progress.contains(&envelope.plot_id.as_str()));
        assert_eq!(series[0].points.len(), 11);
        assert_eq!(series[0].points[..10], previous[0].points);
        assert_eq!(series[0].points.last().unwrap().y, Some(10.0));
        assert!(refreshed.insert(envelope.plot_id));
    }
    assert_eq!(refreshed.len(), progress.len());
    assert!(!refreshed.contains(idle));
    let changed = handle.plot_streams_saved_between(&saved, &handle.storage().cursor_saved()).await.unwrap();
    assert!(changed.is_empty());
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
    data_frame(&mut stream).await
}

// The next frame that carries a plot's data or error, past the watch's acknowledgements and keepalives.
async fn data_frame(watch: &mut WireStream<model::plots::wire::PlotStreamFrame>) -> model::plots::wire::PlotStreamFrame {
    loop {
        let frame = watch.message().await.unwrap().unwrap();
        if frame.kind != model::plots::wire::PlotStreamKind::Unspecified {
            return frame;
        }
    }
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

async fn publish(handle: &dokime::RuntimeHandle, id: &Identifier, start: u64, count: u64) {
    let samples = (start..start + count)
        .map(|index| {
            let at = Timestamp::new(format!(
                "2026-09-13T{:02}:{:02}:{:02}.{:03}Z",
                index / 3_600_000,
                index / 60_000 % 60,
                index / 1_000 % 60,
                index % 1_000
            ));
            let mut sample = TelemetrySample::new(dokime::model::identity::new_id(), id.clone(), index, at, "fixture");
            sample.values.insert("progress".into(), Value::Number(index as f64));
            sample.with_storage_payload(dokime::harmos::Message::encode_to_vec(&support::ProgressRow { progress: index as f64 }))
        })
        .collect();
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime::model::identity::new_id(), id.clone(), samples)).await.unwrap();
    handle.storage().flush().await.unwrap();
}

async fn publish_scoped(handle: &dokime::RuntimeHandle, id: &Identifier, scope: &model::execution::RecordScope, start: u64, count: u64) {
    let samples = (start..start + count)
        .map(|index| {
            let at = Timestamp::new(format!("2026-09-13T00:{:02}:{:02}.000Z", index / 60, index % 60));
            let mut sample = TelemetrySample::new(dokime::model::identity::new_id(), id.clone(), index, at, "fixture");
            sample.values.insert("progress".into(), Value::Number(index as f64));
            sample.scope = Some(scope.clone());
            sample.with_storage_payload(dokime::harmos::Message::encode_to_vec(&support::ProgressRow { progress: index as f64 }))
        })
        .collect();
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime::model::identity::new_id(), id.clone(), samples)).await.unwrap();
    handle.storage().flush().await.unwrap();
}

/* === Tests === */
