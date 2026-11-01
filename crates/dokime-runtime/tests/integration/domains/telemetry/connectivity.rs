/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Boolean Connectivity In Live And Frozen Plots

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use dokime_runtime::document::{DocumentDefinition, DocumentSection, PlotDefinition, PlotType};
use dokime_runtime::runtime::query::{PlotPayloadRequest, RuntimePlotQueryMode};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn assert_states(payload: &serde_json::Value) {
    let stream = &payload["snapshot"]["telemetry"]["host.network"];
    let column = stream["spec"]["columns"].as_array().unwrap().iter().find(|column| column["name"] == "internet").unwrap();
    assert_eq!(column["value_type"], "boolean");
    assert_eq!(stream["samples"][0]["internet"], false);
    assert_eq!(stream["samples"][1]["internet"], true);
    let series = &payload["case_runs"][0]["plots"]["connectivity"]["series"][0];
    assert_eq!(series["raw_points"], 4);
    let points = series["points"].as_array().unwrap();
    assert_eq!(points.iter().map(|point| point["y"].as_f64()).collect::<Vec<_>>(), [Some(0.0), Some(1.0), None, Some(0.0)]);
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn boolean_connectivity() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let mut suite = custom_suite();
    let mut plot = PlotDefinition::with_type("Internet Connectivity", PlotType::TimeSeries);
    plot.id("connectivity");
    plot.stream = Some("host.network".into());
    plot.y = vec!["internet".into()];
    let mut section = DocumentSection::new("Connectivity");
    section.plot(plot);
    let mut document = DocumentDefinition::new("default", "Network");
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    register_custom_suite(&runtime, suite.clone()).await;
    let admitted =
        runtime.test_suites().await.into_iter().find(|entry| entry.suite.metadata.identifier == suite.metadata.identifier).unwrap().suite;
    let cycle = start_cycle(&runtime, admitted).await;
    let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
    runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let scope = runtime.watch_state().latest().state().executions[&execution].scope.clone();
    let mut samples = Vec::new();
    for (index, value) in [Value::Bool(false), Value::Bool(true), Value::Null, Value::Bool(false)].into_iter().enumerate() {
        let mut sample = TelemetrySample::new(
            dokime_runtime::model::identity::new_id(),
            identifier("host.network"),
            index as u64,
            Timestamp::new(format!("2026-05-28T10:00:0{index}Z")),
            "example-plugin",
        );
        sample.scope = Some(scope.clone());
        sample.values.insert("internet".into(), value);
        samples.push(typed_fixture_sample(sample));
    }
    runtime
        .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier("host.network"), samples))
        .await
        .unwrap();
    let request = PlotPayloadRequest {
        plot_type: Some(PlotType::TimeSeries),
        stream: "host.network".into(),
        y_keys: vec!["internet".into()],
        scope: Some(scope),
        ..Default::default()
    };
    // The Live Tail And Saved Reads Use The Same Declared Boolean Encoding.
    for _ in 0..2 {
        let live = runtime.query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot).await.unwrap();
        assert_eq!(live[0].payload.series[0].raw_points, 4);
        assert_eq!(
            live[0].payload.series[0].points.iter().map(|point| point.y).collect::<Vec<_>>(),
            [Some(0.0), Some(1.0), None, Some(0.0)]
        );
        runtime.storage().flush().await.unwrap();
    }
    let mut report = runtime.open_execution_report(&execution).await.unwrap();
    let bytes = crate::service_support::reports::opened_bytes(&mut report).await;
    assert_states(&crate::service_support::reports::parse(std::str::from_utf8(&bytes).unwrap()));
    runtime.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
    runtime.wait_execution_report(&execution).await.unwrap();
    runtime.seal_cycle(cycle, CommandContext::default()).await.unwrap();
    let mut report = runtime.open_execution_report(&execution).await.unwrap();
    let bytes = crate::service_support::reports::opened_bytes(&mut report).await;
    assert_states(&crate::service_support::reports::parse(std::str::from_utf8(&bytes).unwrap()));
    owner.shutdown().await.unwrap();
}
