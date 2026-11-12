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
use crate::support::plots::{self, data_frame, publish, publish_scoped, request, served_points};
use dokime::Identifier;
use dokime::domain::model;

/* === Constants === */

const STORED: u64 = 300;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[derive(Clone, PartialEq)]
struct MapRow {
    x: f64,
    y: f64,
    z: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn sample_watch_fixed_frontier_and_resume() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    support::register_fixture_progress_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 0, 2).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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

// Oversized Plot Responses Reduce Display Points Without Repeating The Storage Query: the
// 4,000-row guard under a tenth of the message budget; `system/plot_admission.rs` runs 40,000 rows.
pub(crate) async fn plot_points_admission() {
    let message_bytes = Some(Config::local().max_message_bytes() / 10);
    plots::admission(plots::Admission {
        rows: 4_000,
        batch: 1_000,
        message_bytes,
        window: 1_000,
        oversized: 10_000,
    })
    .await;
}

// Late Rows Retain Time Order And Exact Full-Selection Summary Counts.
pub(crate) async fn late_samples_keep_time_order() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    support::register_fixture_progress_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 10, 10).await;
    publish(&handle, &id, 0, 10).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let reply = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![request(100)] })
        .await
        .unwrap()
        .into_inner();
    let payload = reply.envelopes[0].payload.as_ref().unwrap();
    assert_eq!(
        payload.frame.as_ref().unwrap().series[0].y.iter().map(|point| point.value.unwrap()).collect::<Vec<_>>(),
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
pub(crate) async fn sealed_window_all() {
    use dokime::runtime::command::CommandContext;
    let root = tempfile::tempdir().unwrap();
    let config = || crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.plots", "Plots"));
    let gate = std::sync::Arc::new(Gate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = Runtime::with_execution_fixture(prepared_owner.configuration(config()), "fixture", gate.clone()).await.unwrap();
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

    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut watch =
        client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: requests() }).await.unwrap().into_inner();
    // The watch acknowledges every plot before any read, so none sits silent while the snapshots run.
    let mut acknowledged = Vec::new();
    let pending = watch.message().await.unwrap().unwrap();
    assert_eq!(pending.updates.len(), 2);
    for frame in pending.updates {
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Unspecified, "the acknowledgement precedes every snapshot");
        let envelope = frame.envelope.unwrap();
        assert!(envelope.payload.is_none(), "an acknowledgement carries no data");
        acknowledged.push(envelope.plot_id);
    }
    assert_eq!(acknowledged, ["fixture.progress.first", "fixture.progress.second"]);
    for frame in plots::data_frames(&mut watch).await {
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot, "{:?}", frame.envelope.map(|envelope| envelope.warnings));
        assert_eq!(served_points(&frame), STORED as usize, "a running execution opens with its saved rows");
    }
    publish_scoped(&handle, &id, &scope, STORED, 1).await;
    let saved = handle.storage().cursor_saved();
    let mut replacement_cut = None;
    let frames = loop {
        let frames = plots::data_frames(&mut watch).await;
        if frames.iter().all(|frame| frame.envelope.as_ref().is_some_and(|envelope| envelope.revision >= saved.sequence)) {
            break frames;
        }
    };
    assert_eq!(frames.len(), 2);
    for frame in frames {
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Replace);
        assert_eq!(served_points(&frame), STORED as usize + 1);
        let envelope = frame.envelope.unwrap();
        let payload = envelope.payload.unwrap();
        assert_eq!(payload.frame.as_ref().unwrap().series[0].y.last().unwrap().value, Some(STORED as f64));
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

    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner =
        Runtime::with_execution_fixture(prepared_owner.configuration(config()), "fixture", std::sync::Arc::new(Gate::default()))
            .await
            .unwrap();
    let handle = owner.handle();
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut watch =
        client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: requests() }).await.unwrap().into_inner();
    for frame in plots::data_frames(&mut watch).await {
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot, "{:?}", frame.envelope.map(|envelope| envelope.warnings));
        assert_eq!(served_points(&frame), STORED as usize + 1, "a sealed execution opens with its stored rows");
    }
    drop(watch);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A Saved Frontier Refreshes Only Plots Reading The Changed Stream.
pub(crate) async fn saved_frontier_refreshes_affected_plots() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;
    let handle = owner.handle();
    support::register_fixture_progress_and_idle_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 0, 10).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
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
    let frames = plots::data_frames(&mut watch).await;
    assert_eq!(frames.len(), 4);
    for frame in frames {
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Snapshot);
        let envelope = frame.envelope.unwrap();
        if let Some(cut) = initial_cut {
            assert_eq!(envelope.revision, cut);
        }
        initial_cut = Some(envelope.revision);
        let series = envelope.payload.unwrap().frame.unwrap().series;
        let points: usize = series.iter().map(|series| series.y.len()).sum();
        assert_eq!(points, if envelope.plot_id == idle { 0 } else { 10 });
        assert!(initial.insert(envelope.plot_id, series).is_none());
    }
    let before = handle.storage().cursor_saved();
    publish(&handle, &id, 10, 1).await;
    let saved = handle.storage().cursor_saved();
    let changed = handle.plot_streams_between(&before, &saved).await.unwrap();
    assert_eq!(changed, std::collections::BTreeSet::from(["fixture.progress".to_owned()]));
    let mut refreshed = std::collections::BTreeSet::new();
    let mut replacement_cut = None;
    let frames = plots::data_frames(&mut watch).await;
    assert_eq!(frames.len(), progress.len());
    for frame in frames {
        let envelope = frame.envelope.unwrap();
        if envelope.revision < saved.sequence {
            continue;
        }
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Replace);
        if let Some(cut) = replacement_cut {
            assert_eq!(envelope.revision, cut);
        }
        replacement_cut = Some(envelope.revision);
        let series = envelope.payload.unwrap().frame.unwrap().series;
        let previous = &initial[&envelope.plot_id];
        assert!(progress.contains(&envelope.plot_id.as_str()));
        assert_eq!(series[0].y.len(), 11);
        assert_eq!(series[0].y[..10], previous[0].y);
        assert_eq!(series[0].y.last().unwrap().value, Some(10.0));
        assert!(refreshed.insert(envelope.plot_id));
    }
    assert_eq!(refreshed.len(), progress.len());
    assert!(!refreshed.contains(idle));
    let changed = handle.plot_streams_between(&saved, &handle.storage().cursor_saved()).await.unwrap();
    assert!(changed.is_empty());
    drop(watch);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn fitted_maps_and_playback() {
    use crate::grpc::transcode::Transcode;
    use dokime::domain::model::plots::wire::{self, PlotAnalysisResult};
    use dokime::harmos::{Message, schema::Described};
    use dokime::telemetry::{TelemetryBatch, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetrySample};

    // Start A Host With A Small Transport And The Ordinary Query Budget.
    let root = tempfile::tempdir().unwrap();
    let transport = 8_192;
    let prepared = dokime_server::grpc::Service::prepare(Config::local().with_max_message_bytes(transport)).unwrap();
    let schema = dokime::harmos::schema::graph::Schema::from_declaration(MapRow::DEFINITION, &Default::default()).unwrap();
    let contracts = ["fixture.map", "fixture.grid"].map(|stream| TelemetryContract {
        stream: Identifier::new(stream).unwrap(),
        schema_version: dokime::harmos::domain::streams::revision(&schema, 1).unwrap(),
        storage_schema: Some(schema.clone()),
        metadata: support::metadata(stream, "Map"),
        template_id: None,
        owner: None,
        columns: ["x", "y", "z"]
            .map(|field| TelemetryColumn::new(support::metadata(field, field), dokime::ValueType::Number, TelemetryRole::Value))
            .to_vec(),
        default_x: Some("x".into()),
        interval: None,
        allow_extra_fields: false,
        scalar: false,
        validation_mode: None,
        foreign: false,
    });
    let config =
        support::budgets::runtime_config(root.path(), support::metadata("plots.fitting", "Plot Fitting")).with_installed_plugins([
            dokime::InstalledPlugin {
                source: "fixture".into(),
                enabled: true,
                telemetry_contracts: contracts.to_vec(),
                ..Default::default()
            },
        ]);
    let mut owner = dokime::Runtime::start(prepared.configuration(config)).await.unwrap();
    let handle = owner.handle();

    // Save A Dense Numeric Map With Unequally Populated Cells.
    let mut observed = Vec::new();
    for y in 0..32 {
        for x in 0..32 {
            observed.push((x as f64 * 2.0 + 1.0, y as f64 * 2.0 + 1.0, f64::from(x + y * 2)));
        }
    }
    observed.extend([(1.0, 1.0, 100.0), (1.0, 1.0, 200.0)]);
    let stream = Identifier::new("fixture.map").unwrap();
    let samples = observed
        .iter()
        .enumerate()
        .map(|(index, &(x, y, z))| {
            let timestamp = dokime::Timestamp::new(format!("2026-01-01T00:00:{:02}.{:03}Z", index / 1000, index % 1000));
            let mut sample = TelemetrySample::new(dokime::model::identity::new_id(), stream.clone(), index as u64, timestamp, "fixture");
            sample.values.extend([("x".into(), x.into()), ("y".into(), y.into()), ("z".into(), z.into())]);
            sample.with_storage_payload(MapRow { x, y, z }.encode())
        })
        .collect();
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime::model::identity::new_id(), stream, samples)).await.unwrap();

    // Save Playback Frames With Duplicate Values, Zero And One Missing Cell.
    let stream = Identifier::new("fixture.grid").unwrap();
    let mut samples = Vec::new();
    let frame_count = 16;
    for frame in 0..frame_count {
        for y in 0..8 {
            for x in 0..8 {
                if (x, y) == (7, 7) {
                    continue;
                }
                let base = if (frame, x, y) == (0, 0, 0) { -1.0 } else { f64::from(frame * 100 + x * 10 + y) };
                for z in [base, base + 2.0] {
                    let timestamp = dokime::Timestamp::new(format!("2026-01-01T00:00:{frame:02}Z"));
                    let mut sample =
                        TelemetrySample::new(dokime::model::identity::new_id(), stream.clone(), samples.len() as u64, timestamp, "fixture");
                    sample.values.extend([("x".into(), f64::from(x).into()), ("y".into(), f64::from(y).into()), ("z".into(), z.into())]);
                    samples.push(sample.with_storage_payload(MapRow { x: f64::from(x), y: f64::from(y), z }.encode()));
                }
            }
        }
    }
    handle.publish_telemetry_batch(TelemetryBatch::new(dokime::model::identity::new_id(), stream, samples)).await.unwrap();
    handle.storage().flush().await.unwrap();
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    // Fit The Large Panel Without Changing Bounds Or Averaging Bin Means.
    let contour = wire::PlotRequest {
        plot_id: Some("map".into()),
        plot_type: wire::PlotType::Contour,
        stream: "fixture.map".into(),
        x_key: Some("x".into()),
        y_keys: vec!["y".into()],
        z_field: Some("z".into()),
        panel_width: Some(2048),
        panel_height: Some(512),
        viewport: Some(wire::PlotWindow { from: 0.0, to: 64.0 }),
        y_viewport: Some(wire::PlotWindow { from: 0.0, to: 64.0 }),
        max_points: Some(100_000),
        ..request(100_000)
    };
    let reply = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![contour.clone()] })
        .await
        .unwrap()
        .into_inner();
    assert!(reply.wire_len().unwrap() <= transport);
    let payload = reply.envelopes[0].payload.as_ref().unwrap();
    let Some(PlotAnalysisResult::Contour(grid)) = payload.analysis.as_ref().and_then(|analysis| analysis.result.as_ref()) else {
        panic!("fitted contour missing")
    };
    assert!(grid.columns < 2048 && grid.rows < 512);
    assert_eq!((grid.x_edges.first(), grid.x_edges.last()), (Some(&0.0), Some(&64.0)));
    assert_eq!((grid.y_edges.first(), grid.y_edges.last()), (Some(&0.0), Some(&64.0)));
    assert_eq!(grid.values.len(), grid.columns as usize * grid.rows as usize);
    for row in 0..grid.rows as usize {
        for column in 0..grid.columns as usize {
            let values = observed
                .iter()
                .filter(|(x, y, _)| {
                    grid.x_edges[column] <= *x && *x < grid.x_edges[column + 1] && grid.y_edges[row] <= *y && *y < grid.y_edges[row + 1]
                })
                .map(|(_, _, z)| *z)
                .collect::<Vec<_>>();
            let cell = row * grid.columns as usize + column;
            assert_eq!(grid.valid[cell], !values.is_empty());
            if !values.is_empty() {
                let expected = values.iter().sum::<f64>() / values.len() as f64;
                assert!((grid.values[cell] - expected).abs() < 1e-10, "cell {cell}");
            }
        }
    }
    assert!(!reply.envelopes[0].warnings.is_empty());

    // The Same Map Fits A Stream Frame Including Its Transport Envelope.
    let mut watch =
        client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: vec![contour.clone()] }).await.unwrap().into_inner();
    let frame = data_frame(&mut watch).await;
    assert_eq!(frame.kind, wire::PlotStreamKind::Snapshot);
    assert!(dokime::harmos::grpc::output_item_len(&wire::PlotStreamFrame { updates: vec![frame] }).unwrap() <= transport);
    drop(watch);

    // Fit Complete Playback Frames And Preserve Their Declared Cell Identities.
    let expression = |field: &str| wire::DataExpression {
        source: Some(wire::DataExpressionSource::Telemetry(wire::TelemetryDataSource {
            stream: "fixture.grid".into(),
            field: field.into(),
        })),
        ..Default::default()
    };
    let entries = (0..8)
        .map(|key| dokime::document::GridAxisEntry {
            key: key.to_string(),
            label: key.to_string(),
            order: key,
            group: None,
        })
        .collect::<Vec<_>>();
    let grid_request = wire::PlotRequest {
        plot_id: Some("grid".into()),
        plot_type: wire::PlotType::GridChart,
        stream: "fixture.grid".into(),
        y_keys: vec!["z".into()],
        reduction: Some(dokime::document::Reduction::Mean),
        grid: Some(wire::GridDefinition {
            row: Some(expression("y")),
            column: Some(expression("x")),
            frame: Some(expression("timestamp")),
            rows: entries.clone(),
            columns: entries.clone(),
            highlights: Vec::new(),
            summary_label: "Readings".into(),
        }),
        ..request(100_000)
    };
    let reply = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![grid_request.clone()] })
        .await
        .unwrap()
        .into_inner();
    assert!(reply.wire_len().unwrap() <= transport);
    let payload = reply.envelopes[0].payload.as_ref().unwrap();
    let Some(PlotAnalysisResult::GridChart(data)) = payload.analysis.as_ref().and_then(|analysis| analysis.result.as_ref()) else {
        panic!("fitted playback missing")
    };
    assert!(!data.frames.is_empty() && data.frames.len() < frame_count as usize);
    assert_eq!((&data.rows, &data.columns), (&entries, &entries));
    assert_eq!(data.frames.first().unwrap().instant, "2026-01-01T00:00:00Z");
    assert_eq!(data.frames.last().unwrap().instant, format!("2026-01-01T00:00:{:02}Z", frame_count - 1));
    for frame in &data.frames {
        let second = frame.instant[17..19].parse::<u32>().unwrap();
        assert_eq!(frame.values.len(), 64);
        assert!(!frame.valid[63]);
        for y in 0..8 {
            for x in 0..8 {
                if (x, y) == (7, 7) {
                    continue;
                }
                let cell = y * 8 + x;
                assert!(frame.valid[cell]);
                let expected = if (second, x, y) == (0, 0, 0) { 0.0 } else { f64::from(second * 100) + (x * 10 + y) as f64 + 1.0 };
                assert_eq!(frame.values[cell], expected);
            }
        }
    }
    assert!(!reply.envelopes[0].warnings.is_empty());

    // Several Plots Share One Bounded Atomic Revision.
    let mut watch = client
        .download::<contracts::plots::Watch>(contracts::plots::Watch {
            requests: vec![contour, grid_request.clone()],
        })
        .await
        .unwrap()
        .into_inner();
    let updates = plots::data_frames(&mut watch).await;
    assert_eq!(updates.len(), 2);
    assert!(updates.iter().all(|update| update.kind == wire::PlotStreamKind::Snapshot));
    assert_eq!(updates[0].envelope.as_ref().unwrap().revision, updates[1].envelope.as_ref().unwrap().revision);
    let message = wire::PlotStreamFrame { updates };
    assert!(dokime::harmos::grpc::output_item_len(&message).unwrap() <= transport);
    drop(watch);

    let mut minimal = reply.clone();
    minimal.envelopes[0].warnings.clear();
    minimal.envelopes[0].payload.as_mut().unwrap().max_points = 1;
    minimal.envelopes[0].payload.as_mut().unwrap().generated_at.clear();
    let analysis = minimal.envelopes[0].payload.as_mut().unwrap().analysis.as_mut().unwrap();
    let Some(PlotAnalysisResult::GridChart(data)) = &mut analysis.result else { panic!("grid playback missing") };
    data.frames.truncate(1);
    let excess_label = transport - minimal.wire_len().unwrap() + "Readings".len() + 1;

    // Trajectory Timestamp Columns Fit The Same Limit And Retain Endpoints.
    let trajectory = wire::PlotRequest {
        plot_id: Some("path".into()),
        plot_type: wire::PlotType::Trajectory,
        stream: "fixture.map".into(),
        x_key: Some("x".into()),
        y_keys: vec!["y".into()],
        ..request(100_000)
    };
    let reply = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![trajectory] })
        .await
        .unwrap()
        .into_inner();
    assert!(reply.wire_len().unwrap() <= transport);
    let columns = &reply.envelopes[0].payload.as_ref().unwrap().frame.as_ref().unwrap().series[0];
    assert!(columns.instants.len() > 1 && columns.instants.len() < observed.len());
    assert_eq!(columns.instants.len(), columns.y.len());

    // An Irreducible Frame Reports Its Refused Plot.
    let mut refused = grid_request;
    let refused_id = refused.plot_id.clone().unwrap();
    refused.grid.as_mut().unwrap().summary_label = "x".repeat(excess_label);
    let error =
        client.unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![refused] }).await.unwrap_err();
    assert_eq!(error.code(), tonic::Code::ResourceExhausted);
    assert!(error.message().contains(&refused_id), "{error}");
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn live_plot_cadence() {
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut config = support::budgets::runtime_config(root.path(), support::metadata("plots.cadence", "Plots"));
    config.local_storage.telemetry_interval = dokime::config::local_storage_defaults().telemetry_interval;
    let mut owner = Box::pin(dokime::Runtime::start(prepared.configuration(config))).await.unwrap();
    let handle = owner.handle();
    support::register_fixture_progress_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 0, 2).await;
    // Keep External I/O From Auto-Advancing The Paused Clock.
    let clock = tokio::spawn(async {
        loop {
            tokio::task::yield_now().await;
        }
    });
    tokio::time::pause();
    let cadence = Duration::from_millis(200);
    tokio::time::advance(cadence).await;
    handle.storage().flush().await.unwrap();
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let timed = model::plots::wire::PlotRequest { window_s: Some(60.0), ..request(10_000) };
    let mut watch = client
        .download::<contracts::plots::Watch>(contracts::plots::Watch {
            requests: (0..8)
                .map(|index| model::plots::wire::PlotRequest {
                    plot_id: Some(format!("cadence.{index}")),
                    ..timed.clone()
                })
                .collect(),
        })
        .await
        .unwrap()
        .into_inner();
    let mut saved_only = plots::open_watch(&client, 10_000).await;
    let frames = plots::data_frames(&mut watch).await;
    assert_eq!(frames.len(), 8);
    for frame in frames {
        assert_eq!(served_points(&frame), 2);
    }
    assert_eq!(served_points(&data_frame(&mut saved_only).await), 2);

    let saved = handle.storage().cursor_saved();
    let published = tokio::time::Instant::now();
    plots::publish_pending(&handle, &id, 2, 1).await;
    // Cross Timer Rounding Without Reaching The Next Save Deadline.
    tokio::time::advance(Duration::from_millis(1)).await;
    let frames = plots::data_frames(&mut watch).await;
    assert_eq!(frames.len(), 8);
    assert!(published.elapsed() <= cadence);
    let revision = frames[0].envelope.as_ref().unwrap().revision;
    for (index, frame) in frames.into_iter().enumerate() {
        let envelope = frame.envelope.as_ref().unwrap();
        assert_eq!(envelope.plot_id, format!("cadence.{index}"));
        assert_eq!(envelope.revision, revision);
        assert_eq!(served_points(&frame), 3);
    }
    assert_eq!(handle.storage().cursor_saved(), saved);

    // Finite Timed And All Queries Exclude The Unsaved Row.
    for selected in [timed.clone(), request(10_000)] {
        let mut finite = client
            .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![selected] })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(finite.envelopes.len(), 1);
        let envelope = finite.envelopes.pop().unwrap();
        assert_eq!(envelope.payload.unwrap().frame.unwrap().series[0].y.len(), 2);
        assert!(envelope.revision <= saved.sequence);
    }

    // The Saved-Only Control Uses The Former Live Read Policy. Harmos Also Owns A Periodic Save.
    tokio::time::advance(cadence).await;
    assert_eq!(served_points(&data_frame(&mut saved_only).await), 3);
    eprintln!("saved-only publication-to-frame virtual latency: {:?}", published.elapsed());
    // Saving The Already Delivered Revision Must Not Queue Another Data Frame.

    // Multiple High-Rate Publications Coalesce, Beyond A Small Stream Window.
    let published = tokio::time::Instant::now();
    for batch in 0..16 {
        plots::publish_pending(&handle, &id, 3 + batch * 80, 80).await;
    }
    let accepted = handle.storage().cursor_accepted();
    assert!(accepted.sequence > handle.storage().cursor_saved().sequence);

    // Opening Watches Merge Every Saved And Pending Row Without A Gap.
    let mut opening =
        client.download::<contracts::plots::Watch>(contracts::plots::Watch { requests: vec![timed.clone()] }).await.unwrap().into_inner();
    let frame = data_frame(&mut opening).await;
    let points = &frame.envelope.as_ref().unwrap().payload.as_ref().unwrap().frame.as_ref().unwrap().series[0].y;
    assert_eq!(points.iter().map(|point| point.value.unwrap()).collect::<Vec<_>>(), (0..1283).map(f64::from).collect::<Vec<_>>());
    drop(opening);

    // One Throttled Frame Covers The Burst, Even If A Normal Save Races It.
    let frame = {
        let mut waiting = std::pin::pin!(plots::data_frames(&mut watch));
        assert!(futures_util::poll!(&mut waiting).is_pending());
        tokio::time::advance(cadence).await;
        waiting.await
    };
    eprintln!("coalesced publication-to-frame virtual latency: {:?}", published.elapsed());
    assert!(published.elapsed() <= cadence);
    assert_eq!(frame.len(), 8);
    let revision = frame[0].envelope.as_ref().unwrap().revision;
    for (index, frame) in frame.into_iter().enumerate() {
        let envelope = frame.envelope.as_ref().unwrap();
        assert_eq!(envelope.plot_id, format!("cadence.{index}"));
        assert_eq!(envelope.revision, revision);
        assert_eq!(served_points(&frame), 1283);
    }

    // Exact Statistics Follow The Drawn Cut Without Replacing Its Data.
    loop {
        let statistics = plots::statistics_frames(&mut watch).await;
        if statistics.iter().any(|frame| frame.envelope.as_ref().unwrap().revision < revision) {
            continue;
        }
        assert_eq!(statistics.len(), 8);
        for frame in statistics {
            let envelope = frame.envelope.unwrap();
            assert_eq!(envelope.revision, revision);
            let payload = envelope.payload.unwrap();
            assert!(payload.series.is_empty());
            assert_eq!(payload.summaries[0].count, 1283);
            assert_eq!(payload.summaries[0].median, Some(641.0));
        }
        break;
    }

    // The Same Rows Survive The Tail-To-Saved Handoff Once, Without A Gap.
    handle.storage().flush().await.unwrap();
    let mut finite = client
        .unary::<contracts::plots::QuerySnapshots>(contracts::plots::QuerySnapshots { requests: vec![timed] })
        .await
        .unwrap()
        .into_inner();
    let envelope = finite.envelopes.pop().unwrap();
    let points = &envelope.payload.as_ref().unwrap().frame.as_ref().unwrap().series[0].y;
    assert_eq!(points.iter().map(|point| point.value.unwrap()).collect::<Vec<_>>(), (0..1283).map(f64::from).collect::<Vec<_>>());
    assert!(handle.storage().cursor_saved().covers(&accepted).unwrap());
    // The Saved Handoff Produces Only A Keepalive, Never A Replayed Revision.
    // Processing That Save After This Advance Must Not Postpone The Heartbeat Without Sending Data.
    tokio::time::advance(dokime_server::grpc::WATCH_HEARTBEAT + Duration::from_millis(1)).await;
    let keepalive = watch.message().await.unwrap().unwrap();
    assert_eq!(keepalive.updates.len(), 8);
    assert!(keepalive.updates.iter().all(|update| update.kind == model::plots::wire::PlotStreamKind::Unspecified));
    assert!(keepalive.updates.iter().all(|update| update.envelope.as_ref().unwrap().payload.is_none()));
    tokio::time::resume();
    clock.abort();
    let _ = clock.await;
    drop(watch);
    drop(saved_only);
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}
