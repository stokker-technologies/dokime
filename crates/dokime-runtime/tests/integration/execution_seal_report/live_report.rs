/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Keep One Durable Cut While Their Execution Continues.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::prelude::*;
use dokime_domain::model::reports::wire::ReportSubject;
use dokime_runtime::document::{DocumentDefinition, DocumentSection, PlotDefinition, PlotType};
use dokime_runtime::operations::reports::MaterializeReport;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

async fn publish(runtime: &RuntimeHandle, scope: Option<&RecordScope>, values: &[f64], offset: usize) {
    let samples = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let mut sample = tracking_status_sample(offset + index, (offset + index) as f64, *value, true, true);
            sample.stream = identifier("example.tracking");
            sample.scope = scope.cloned();
            sample
        })
        .collect();
    runtime
        .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier("example.tracking"), samples))
        .await
        .unwrap();
    runtime.storage().flush().await.unwrap();
}

fn assert_cut(payload: &serde_json::Value, expected: &[f64]) {
    let samples = payload["snapshot"]["telemetry"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|stream| stream["samples"].as_array().unwrap())
        .collect::<Vec<_>>();
    let rows = samples.iter().map(|sample| sample["distance_m"].as_f64().unwrap()).collect::<Vec<_>>();
    assert_eq!(rows, expected);
    let plot = payload["case_runs"][0]["plots"].as_object().unwrap().values().next().unwrap();
    let series = plot["series"].as_array().unwrap();
    assert_eq!(series.len(), 1, "the report retains the telemetry trace: {plot}");
    assert_eq!(series[0]["raw_points"].as_u64().unwrap() as usize, expected.len());
    let points = series[0]["points"].as_array().unwrap();
    assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).collect::<Vec<_>>(), expected);
    let summary = &plot["summaries"][0];
    assert_eq!(summary["count"].as_u64().unwrap() as usize, rows.len());
    assert_eq!(summary["min"].as_f64().unwrap(), rows.iter().copied().fold(f64::INFINITY, f64::min));
    assert_eq!(summary["max"].as_f64().unwrap(), rows.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    assert_eq!(summary["mean"].as_f64().unwrap(), rows.iter().sum::<f64>() / rows.len() as f64);
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn live_report() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let mut contract = crate::service_support::contracts::tracking_contract();
    contract.stream = identifier("example.tracking");
    contract.interval = Some(dokime_runtime::DurationMillis(1_000));
    let configuration = config(root.path()).with_installed_plugins([dokime_runtime::InstalledPlugin {
        source: "example-plugin".into(),
        version: "test".into(),
        enabled: true,
        telemetry_contracts: vec![contract.clone()],
        ..Default::default()
    }]);
    let (mut owner, runtime) = started_with_fixture(configuration, executor.clone()).await;
    publish(&runtime, None, &[101.0], 0).await;
    let mut suite = custom_suite();
    let mut plot = PlotDefinition::with_type("Distance", PlotType::TimeSeries);
    plot.id("live.distance");
    plot.stream = Some("tracking".into());
    plot.x = Some("timestamp".into());
    plot.y = vec!["distance_m".into()];
    let mut section = DocumentSection::new("Samples");
    section.plot(plot);
    let mut document = DocumentDefinition::new("default", "Live Evidence");
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    runtime
        .set_suites(
            "provider:testdb",
            dokime_runtime::definition::SuiteSet::new([dokime_runtime::domains::TestSuiteDefinition {
                suite_id: "provider:testdb:custom.suite".into(),
                tags: Vec::new(),
                workflow: Some("run".into()),
                suite: suite.clone(),
                record_contracts: crate::service_support::contracts::tracking_record_contracts(),
                telemetry_contracts: Vec::new(),
            }]),
        )
        .await
        .unwrap();
    let admitted =
        runtime.test_suites().await.into_iter().find(|entry| entry.suite.metadata.identifier == suite.metadata.identifier).unwrap().suite;
    let cycle = start_cycle(&runtime, admitted).await;
    let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
    runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let scope = runtime.watch_state().latest().state().executions[&execution].scope.clone();
    let leases = runtime
        .lease_streams(
            dokime_runtime::telemetry::SamplingScope {
                test_cycle_id: Some(cycle.to_string()),
                test_case_id: scope.test_case_id.as_ref().map(ToString::to_string),
                test_execution_id: Some(execution.to_string()),
            },
            &[identifier("example.tracking")],
            "testdb",
        )
        .await
        .unwrap();
    publish(&runtime, Some(&scope), &[2.0, 5.0], 0).await;
    let stale = runtime.storage().cursor_saved();
    publish(&runtime, None, &[11.0], 2).await;

    // Both Entry Points Receive An Older Cursor; Further Commits Follow Their Snapshot.
    for subject in [ReportSubject::Execution(execution.to_string()), ReportSubject::Cycle(cycle.to_string())] {
        let expected = if matches!(&subject, ReportSubject::Execution(_)) { vec![2.0, 5.0, 11.0] } else { vec![2.0, 5.0, 11.0, 19.0] };
        let gate = dokime_runtime::testing::hold_report(&execution, false);
        let task_runtime = runtime.clone();
        let at = stale.clone();
        let mut render =
            tokio::spawn(async move { task_runtime.operations().call::<MaterializeReport>(MaterializeReport { subject, at }).await });
        tokio::select! {
            () = gate.entered() => {},
            result = &mut render => panic!("report settled before its snapshot gate: {:?}", result.map(|result| result.map(|_| ()))),
        }
        publish(&runtime, None, &[if expected.len() == 3 { 19.0 } else { 23.0 }], expected.len()).await;
        gate.release();
        let report = render.await.unwrap().unwrap();
        let payload = crate::service_support::reports::payload(runtime.storage(), &report.report).await;
        assert_cut(&payload, &expected);
    }

    runtime.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
    runtime.release_stream_leases(leases).await.unwrap();
    publish(&runtime, None, &[103.0], 6).await;
    runtime.wait_execution_report(&execution).await.unwrap();
    let mut first = runtime.open_execution_report(&execution).await.unwrap();
    let first_file = first.file.clone();
    let bytes = crate::service_support::reports::opened_bytes(&mut first).await;
    let payload = crate::service_support::reports::parse(std::str::from_utf8(&bytes).unwrap());
    assert_cut(&payload, &[2.0, 5.0, 11.0, 19.0, 23.0]);
    let mut second = runtime.open_execution_report(&execution).await.unwrap();
    assert_eq!(second.file, first_file);
    assert_eq!(crate::service_support::reports::opened_bytes(&mut second).await, bytes);
    runtime.seal_cycle(cycle.clone(), CommandContext::default()).await.unwrap();
    let mut first = runtime.open_cycle_report(&cycle).await.unwrap();
    let first_file = first.file.clone();
    let bytes = crate::service_support::reports::opened_bytes(&mut first).await;
    let payload = crate::service_support::reports::parse(std::str::from_utf8(&bytes).unwrap());
    assert_cut(&payload, &[2.0, 5.0, 11.0, 19.0, 23.0]);
    let mut second = runtime.open_cycle_report(&cycle).await.unwrap();
    assert_eq!(second.file, first_file);
    assert_eq!(crate::service_support::reports::opened_bytes(&mut second).await, bytes);
    owner.shutdown().await.unwrap();
}
