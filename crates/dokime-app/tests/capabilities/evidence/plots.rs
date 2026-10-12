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
use dokime_app::runtime::query::{
    DownsampleTechnique, PlotPayloadCursor, PlotPayloadRequest, RuntimePlotQueryMode,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// snapshot query with all window uses all samples before downsampling.

pub(crate) async fn snapshot_query_samples() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 30, None))
        .await
        .unwrap();

    let envelopes = runtime
        .query_plot_payloads(
            vec![wifi_request(None, Some(5), None)],
            RuntimePlotQueryMode::Snapshot,
        )
        .await
        .unwrap();

    assert_eq!(envelopes.len(), 1);
    assert_eq!(envelopes[0].payload.series[0].raw_points, 30);
    assert_eq!(envelopes[0].payload.series[0].points.len(), 5);
    assert!(envelopes[0].payload.series[0].downsampled);
}

pub(crate) async fn batch_request_cursors() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 4, None))
        .await
        .unwrap();
    let stream = "dokime.tel|-|-|host.wifi.channel_occupancy".to_owned();
    let mut first = wifi_request(
        None,
        Some(10),
        Some(PlotPayloadCursor {
            telemetry_streams: [(stream.clone(), 2)].into(),
            record_sequence: None,
        }),
    );
    first.plot_id = Some("first".to_owned());
    let mut second = wifi_request(
        None,
        Some(10),
        Some(PlotPayloadCursor {
            telemetry_streams: [(stream, 3)].into(),
            record_sequence: None,
        }),
    );
    second.plot_id = Some("second".to_owned());

    let envelopes = runtime
        .query_plot_payloads(vec![first, second], RuntimePlotQueryMode::Append)
        .await
        .unwrap();

    assert_eq!(
        envelopes
            .iter()
            .map(|envelope| envelope.plot_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    assert_eq!(envelopes[0].payload.series[0].raw_points, 2);
    // The second request is the one-cursor case: only the samples after it, and
    // the cursor it hands back is the stream's new high-water.
    assert_eq!(envelopes[1].payload.series[0].raw_points, 1);
    assert_eq!(
        envelopes[1]
            .cursor
            .telemetry_streams
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec![4]
    );
    assert_eq!(envelopes[1].revision, 4);
}

pub(crate) async fn execution_plots_samples() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    register_plot_suite(&runtime).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &cycle, CycleStatus::Running).await;
    let execution = running.test_executions[0].scope.clone();
    for (scope, count) in [
        (None, 7),
        (Some(RecordScope::default().with_cycle(cycle.clone())), 5),
        (Some(execution.clone()), 2),
    ] {
        let receipt = runtime
            .publish_telemetry_batch(samples("host.wifi.channel_occupancy", count, scope))
            .await
            .unwrap();
        assert_eq!(receipt.accepted, count as u64);
        assert_eq!(receipt.dropped, 0);
    }

    let envelopes = runtime
        .query_plot_payloads(
            vec![wifi_request(Some(execution), Some(100), None)],
            RuntimePlotQueryMode::Snapshot,
        )
        .await
        .unwrap();

    assert_eq!(envelopes[0].payload.series[0].raw_points, 2);
    assert_eq!(envelopes[0].payload.series[0].points.len(), 2);
    executor.release();
    wait_for_cycle_lifecycle(&runtime, &cycle, CycleStatus::Completed).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

async fn register_plot_suite(runtime: &RuntimeHandle) {
    runtime
        .set_suites(
            "plot-query",
            dokime::definition::SuiteSet::new([TestSuiteDefinition {
                suite_id: "plot.query.suite".to_owned(),
                product: None,
                tags: Vec::new(),
                workflow: None,
                suite: plot_query_suite(),
                record_contracts: Vec::new(),
                telemetry_contracts: vec![wifi_occupancy_contract()],
            }]),
        )
        .await
        .unwrap();
}

fn plot_query_suite() -> TestSuite {
    TestSuite {
        metadata: Metadata::new(
            Identifier::new("plot.query.suite").unwrap(),
            "Plot Query Suite",
            None,
        ),
        tags: Vec::new(),
        parameter_groups: Vec::new(),
        test_case_templates: Vec::new(),
    }
}

fn wifi_request(
    scope: Option<RecordScope>,
    max_points: Option<usize>,
    cursor: Option<PlotPayloadCursor>,
) -> PlotPayloadRequest {
    PlotPayloadRequest {
        plot_id: Some("wifi.network_count".to_owned()),
        plot_type: None,
        x_scale: Default::default(),
        y_scale: Default::default(),
        stream: "host.wifi.channel_occupancy".to_owned(),
        y_keys: vec!["network_count".to_owned()],
        marker_record_types: Vec::new(),
        filters: Vec::new(),
        x_key: Some("sample_index".to_owned()),
        series_by: None,
        x_expression: None,
        y_expressions: Vec::new(),
        series_by_expression: None,
        z_stream: None,
        z_field: None,
        unit: Some("count".to_owned()),
        x_format: None,
        y_format: None,
        max_points,
        window_s: None,
        scope,
        cursor,
        technique: DownsampleTechnique::Raw,
        ..Default::default()
    }
}

/* === Tests === */

// distribution query does not limit evidence to the display budget.

pub(crate) async fn distribution_query_budget() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 1000, None))
        .await
        .unwrap();

    let mut request = wifi_request(None, Some(5), None);
    request.plot_type = Some(dokime::document::PlotType::Histogram);
    request.x_key = None;
    request.window_s = Some(2000.0);
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let payload = serde_json::to_value(&envelopes[0].payload).unwrap();

    assert_eq!(payload["analysis"]["series"][0]["count"], 1000);
}

// record only query filters on execution dimensions without a telemetry stream.

pub(crate) async fn record_query_absent() {
    use dokime::document::{DocumentFilter, FilterOperator};
    use dokime::expressions::{DataExpression, DataSource, DataTransform, dimension};

    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    register_custom_suite(&runtime, custom_suite()).await;
    let cycle_id = start_cycle(&runtime, custom_suite(), false).await;
    runtime
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Completed).await;
    let scope = cycle.test_executions[0].scope.clone();

    for (suite, matches) in [("custom.suite", true), ("other.suite", false)] {
        let request = PlotPayloadRequest {
            stream: "default".into(),
            scope: Some(scope.clone()),
            y_expressions: vec![DataExpression {
                label: None,
                source: DataSource::Record {
                    record_type: "test.execution.completed".into(),
                },
                transforms: vec![DataTransform::TimeBetween {
                    previous_record_type: "test.execution.prepared".into(),
                }],
            }],
            filters: vec![DocumentFilter {
                expression: dimension("execution.suite"),
                operator: FilterOperator::Eq,
                value: Some(Value::from(suite)),
            }],
            ..Default::default()
        };
        let result = runtime
            .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap();
        let count = result[0]
            .payload
            .series
            .iter()
            .map(|series| series.points.len())
            .sum::<usize>();
        assert_eq!(count > 0, matches);
        assert!(result[0].cursor.telemetry_streams.is_empty());
    }
}

pub(crate) async fn selected_analysis_provenance() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None))
        .await
        .unwrap();
    let definitions = runtime.analysis_definitions().await.unwrap();
    assert!(
        definitions
            .iter()
            .any(|definition| definition.id == "dokime.analysis.mean")
    );
    assert_eq!(
        definitions
            .iter()
            .any(|definition| definition.id == "dokime.analysis.fft"),
        cfg!(feature = "fft")
    );
    let mut request = wifi_request(None, Some(1), None);
    request.analysis = Some(dokime::analysis::Selection {
        id: "dokime.analysis.mean".into(),
        version: 1,
        parameters: Default::default(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let result = envelopes[0].payload.computed.as_ref().unwrap();
    assert_eq!(result.series[0].input_samples, 100);
    assert_eq!(result.series[0].output.outputs[0].values, [49.5]);
    assert_eq!(result.series[0].output.outputs[0].unit, "count");
    assert_eq!(result.source.epoch, runtime.process_epoch().to_string());
    assert_eq!(result.source.cursor, envelopes[0].cursor);
    assert_eq!(result.source.fields, ["network_count"]);
    assert_eq!(result.selection, request.analysis.clone().unwrap());
    assert!(envelopes[0].payload.series.is_empty());

    let mut expressions = request.clone();
    expressions.y_keys.clear();
    expressions.x_expression = Some(dokime::expressions::telemetry(
        "host.wifi.channel_occupancy",
        "sample_index",
    ));
    expressions.y_expressions = vec![dokime::expressions::telemetry(
        "host.wifi.channel_occupancy",
        "network_count",
    )];
    let equivalent = runtime
        .query_plot_payloads(vec![expressions.clone()], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    assert_eq!(equivalent[0].payload.computed.as_ref().unwrap(), result);
    expressions.y_expressions[0] = expressions.y_expressions[0].clone().mean();
    assert!(
        runtime
            .query_plot_payloads(vec![expressions], RuntimePlotQueryMode::Snapshot)
            .await
            .is_err()
    );

    request.cursor = Some(envelopes[0].cursor.clone());
    let batches = runtime.query_plot_stream_batches(&[request]).await.unwrap();
    assert_eq!(batches[0].mode, RuntimePlotQueryMode::Replace);
    assert_eq!(
        batches[0]
            .envelope
            .payload
            .computed
            .as_ref()
            .unwrap()
            .series[0]
            .input_samples,
        100
    );
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn selected_original_coordinates() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None))
        .await
        .unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.analysis = Some(dokime::analysis::Selection {
        id: "dokime.analysis.lttb".into(),
        version: 1,
        parameters: [("points".into(), Value::Integer(7))].into(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let result = &envelopes[0].payload.computed.as_ref().unwrap().series[0];
    assert_eq!(result.input_samples, 100);
    let output = &result.output.outputs[0];
    assert_eq!(result.display_indices[&output.key], [0]);
    assert_eq!(output.axis.len(), 7);
    assert_eq!(output.axis, output.values);
    assert_eq!(output.axis[0], 0.0);
    assert_eq!(output.axis[6], 99.0);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[cfg(feature = "fft")]
pub(crate) async fn selected_fft_manager() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    let mut batch = samples("host.wifi.channel_occupancy", 64, None);
    for (index, sample) in batch.samples.iter_mut().enumerate() {
        sample.values.insert(
            "network_count".into(),
            Value::Integer(if index % 2 == 0 { 1 } else { -1 }),
        );
    }
    runtime.publish_telemetry_batch(batch).await.unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.x_key = Some("timestamp".into());
    request.analysis = Some(dokime::analysis::Selection {
        id: "dokime.analysis.fft".into(),
        version: 1,
        parameters: [("hann".into(), Value::Bool(false))].into(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let result = &envelopes[0].payload.computed.as_ref().unwrap().series[0];
    assert_eq!(result.origin.as_deref(), Some("2026-01-01T00:00:00Z"));
    let output = &result.output.outputs[0];
    assert_eq!(output.axis_unit, "Hz");
    assert_eq!(output.axis.last(), Some(&0.5));
    assert!((output.values.last().unwrap() - 1.0).abs() < 1e-12);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn selected_analysis_read() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    for (id, version, parameters, expected) in [
        (
            "dokime.analysis.missing",
            1,
            std::collections::BTreeMap::new(),
            "unavailable",
        ),
        (
            "dokime.analysis.mean",
            999,
            std::collections::BTreeMap::new(),
            "version",
        ),
        (
            "dokime.analysis.mean",
            1,
            [("unknown".into(), Value::Integer(1))].into(),
            "undeclared",
        ),
    ] {
        let mut request = wifi_request(None, Some(1), None);
        request.analysis = Some(dokime::analysis::Selection {
            id: id.into(),
            version,
            parameters,
        });
        let error = runtime
            .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn selected_analysis_axes() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    let mut batch = samples("host.wifi.channel_occupancy", 100, None);
    for sample in &mut batch.samples {
        if let Some(Value::Integer(value)) = sample.values.get_mut("sample_index") {
            *value *= 10;
        }
    }
    runtime.publish_telemetry_batch(batch).await.unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.window_s = Some(9.0);
    request.analysis = Some(dokime::analysis::Selection {
        id: "dokime.analysis.mean".into(),
        version: 1,
        parameters: Default::default(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let payload = &envelopes[0].payload;
    let series = &payload.computed.as_ref().unwrap().series[0];
    assert_eq!(series.input_samples, 10);
    assert_eq!(series.first_axis, 900.0);
    assert_eq!(series.last_axis, 990.0);
    assert_eq!(series.output.outputs[0].values, [94.5]);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn external_analysis_reload() {
    let root = tempdir().unwrap();
    let store = crate::support::plugins::plugin_store(root.path());
    let bundle = crate::support::plugins::fixture_bundle_bytes("dokime-route-plugin");
    let staged = store.stage(&bundle).unwrap();
    store
        .consent(staged.installation.install_id, &[], &Default::default())
        .unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None))
        .await
        .unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.analysis = Some(dokime::analysis::Selection {
        id: "route-fixture.peak-to-peak".into(),
        version: 1,
        parameters: Default::default(),
    });
    for reload in [false, true] {
        if reload {
            owner.reinitialize_plugin("route-fixture").await.unwrap();
        }
        let definitions = runtime.analysis_definitions().await.unwrap();
        assert_eq!(
            definitions
                .iter()
                .filter(|definition| definition.id == "route-fixture.peak-to-peak")
                .count(),
            1
        );
        let envelopes = runtime
            .query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap();
        let computed = envelopes[0].payload.computed.as_ref().unwrap();
        assert_eq!(computed.selection.id, "route-fixture.peak-to-peak");
        assert_eq!(computed.series[0].input_samples, 100);
        assert_eq!(computed.series[0].output.outputs[0].values, [99.0]);
        assert_eq!(computed.series[0].output.outputs[0].unit, "count");
    }
    drop(runtime);
    owner.shutdown().await.unwrap();
}
