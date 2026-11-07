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

// One Running-Capable Cycle Whose Member Document Plots The Tracking Stream.
async fn plotted_cycle(
    root: &std::path::Path,
) -> (
    crate::service_support::system_reports::Runtime,
    RuntimeHandle,
    Arc<BlockingPublishingExecutor>,
    TestCycleId,
    dokime_domain::model::identity::TestExecutionId,
) {
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let mut contract = crate::service_support::contracts::tracking_contract();
    contract.stream = identifier("example.tracking");
    contract.interval = Some(dokime_runtime::DurationMillis(1_000));
    let configuration = config(root).with_installed_plugins([dokime_runtime::InstalledPlugin {
        source: "example-plugin".into(),
        version: "test".into(),
        enabled: true,
        telemetry_contracts: vec![contract],
        ..Default::default()
    }]);
    let (owner, runtime) = started_with_fixture(configuration, executor.clone()).await;
    let mut suite = custom_suite();
    let mut plot = PlotDefinition::with_type("Distance", PlotType::TimeSeries);
    plot.id("live.distance");
    plot.stream = Some("tracking".into());
    plot.x = Some("timestamp".into());
    plot.y = vec!["distance_m".into()];
    let mut section = DocumentSection::new("Samples");
    section.plot(plot);
    let mut document = DocumentDefinition::new("Live Evidence");
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
    (owner, runtime, executor, cycle, execution)
}

async fn lease_tracking(
    runtime: &RuntimeHandle, cycle: &TestCycleId, execution: &dokime_domain::model::identity::TestExecutionId, scope: &RecordScope,
) -> Vec<String> {
    runtime
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
        .unwrap()
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
    let (mut owner, runtime, executor, cycle, execution) = plotted_cycle(root.path()).await;
    publish(&runtime, None, &[101.0], 0).await;
    runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let scope = runtime.watch_state().latest().state().executions[&execution].scope.clone();
    let leases = lease_tracking(&runtime, &cycle, &execution, &scope).await;
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
            tokio::spawn(async move { task_runtime.operations().operation::<MaterializeReport>(MaterializeReport { subject, at }).await });
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
    let current = runtime
        .operations()
        .operation::<MaterializeReport>(MaterializeReport {
            subject: ReportSubject::Cycle(cycle.to_string()),
            at: stale,
        })
        .await
        .unwrap();
    let current = crate::service_support::reports::payload(runtime.storage(), &current.report).await;
    assert_cut(&current, &[2.0, 5.0, 11.0, 19.0, 23.0]);
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

// A Member Persists Its Finalization Boundary While An Open Cycle Report Admits It.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn finalization_during_cycle_report() {
    let root = tempdir().unwrap();
    let (mut owner, runtime, executor, cycle, execution) = plotted_cycle(root.path()).await;

    // The Member Has Finished but Not Yet Persisted Its Finalization Boundary
    let request = dokime_runtime::testing::hold_report_request(&execution);
    let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    executor.release();
    request.entered().await;
    {
        let snapshot = runtime.watch_state().latest();
        assert!(snapshot.state().executions[&execution].finished_at.is_some());
        assert!(!snapshot.state().report_finalizations.contains_key(&execution));
    }

    // The Report Reads the Cycle, Then the Boundary Lands Beyond Its Cursor Before the Member Is Captured
    let capture = dokime_runtime::testing::hold_member_capture(&execution);
    let task_runtime = runtime.clone();
    let task_cycle = cycle.clone();
    let mut open = tokio::spawn(async move { task_runtime.open_cycle_report(&task_cycle).await });
    tokio::select! {
        () = capture.entered() => {},
        result = &mut open => panic!("the cycle report settled before its member capture: {:?}", result.map(|result| result.map(|_| ()))),
    }
    publish(&runtime, None, &[7.0], 0).await;
    request.release();
    let mut watch = runtime.watch_state();
    loop {
        if watch.latest().state().report_finalizations.contains_key(&execution) {
            break;
        }
        watch.next().await.expect("the runtime keeps publishing state");
    }
    capture.release();

    let mut opened = open.await.unwrap().unwrap();
    assert_eq!(opened.origin, dokime_runtime::ReportOrigin::Current);
    assert!(!crate::service_support::reports::opened_bytes(&mut opened).await.is_empty());
    runtime.wait_execution_report(&execution).await.unwrap();
    owner.shutdown().await.unwrap();
}

// A Member That Finished Before Persisting Its Boundary Reports Its Settled Evidence At Any Earlier Cursor.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn settling_member_report() {
    let root = tempdir().unwrap();
    let (mut owner, runtime, executor, cycle, execution) = plotted_cycle(root.path()).await;

    // The Member's Last Samples Persist After a Report Cursor Was Chosen
    let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let scope = runtime.watch_state().latest().state().executions[&execution].scope.clone();
    let leases = lease_tracking(&runtime, &cycle, &execution, &scope).await;
    publish(&runtime, Some(&scope), &[2.0, 5.0], 0).await;
    let earlier = runtime.storage().cursor_saved();
    publish(&runtime, Some(&scope), &[11.0], 2).await;
    runtime.release_stream_leases(leases).await.unwrap();

    // It Finishes, and Its Finalization Has Not Persisted a Boundary Yet
    let request = dokime_runtime::testing::hold_report_request(&execution);
    executor.release();
    request.entered().await;
    {
        let snapshot = runtime.watch_state().latest();
        assert!(snapshot.state().executions[&execution].finished_at.is_some());
        assert!(!snapshot.state().report_finalizations.contains_key(&execution));
    }
    for subject in [ReportSubject::Execution(execution.to_string()), ReportSubject::Cycle(cycle.to_string())] {
        let report = runtime.operations().operation::<MaterializeReport>(MaterializeReport { subject, at: earlier.clone() }).await.unwrap();
        let payload = crate::service_support::reports::payload(runtime.storage(), &report.report).await;
        assert_cut(&payload, &[2.0, 5.0, 11.0]);
    }
    request.release();
    runtime.wait_execution_report(&execution).await.unwrap();
    owner.shutdown().await.unwrap();
}

// A Running Member Freezes Its Terminal Lifecycle While An Open Cycle Report Admits It.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn lifecycle_during_cycle_report() {
    let root = tempdir().unwrap();
    let (mut owner, runtime, executor, cycle, execution) = plotted_cycle(root.path()).await;
    let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;

    // The Report Reads the Running Member, Which Then Finishes and Freezes Its Lifecycle
    let capture = dokime_runtime::testing::hold_member_capture(&execution);
    let task_runtime = runtime.clone();
    let task_cycle = cycle.clone();
    let mut open = tokio::spawn(async move { task_runtime.open_cycle_report(&task_cycle).await });
    tokio::select! {
        () = capture.entered() => {},
        result = &mut open => panic!("the cycle report settled before its member capture: {:?}", result.map(|result| result.map(|_| ()))),
    }
    executor.release();
    let mut watch = runtime.watch_state();
    loop {
        if watch.latest().state().report_finalizations.get(&execution).is_some_and(|intent| intent.lifecycle.is_some()) {
            break;
        }
        watch.next().await.expect("the runtime keeps publishing state");
    }
    capture.release();

    let mut opened = open.await.unwrap().unwrap();
    assert_eq!(opened.origin, dokime_runtime::ReportOrigin::Current);
    assert!(!crate::service_support::reports::opened_bytes(&mut opened).await.is_empty());
    runtime.wait_execution_report(&execution).await.unwrap();
    owner.shutdown().await.unwrap();
}

// An Execution Report Requested At An Earlier Cursor Uses The Finalization Its Member Froze Since.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn finalizing_member_execution_report() {
    let root = tempdir().unwrap();
    let (mut owner, runtime, executor, cycle, execution) = plotted_cycle(root.path()).await;
    let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let scope = runtime.watch_state().latest().state().executions[&execution].scope.clone();
    let leases = lease_tracking(&runtime, &cycle, &execution, &scope).await;
    publish(&runtime, Some(&scope), &[2.0, 5.0], 0).await;
    let earlier = runtime.storage().cursor_saved();
    publish(&runtime, Some(&scope), &[11.0], 2).await;
    runtime.release_stream_leases(leases).await.unwrap();

    // Its Lifecycle Is Frozen, but Its Background Report Has Not Started
    let job = dokime_runtime::testing::hold_report_job(&execution);
    executor.release();
    job.entered().await;
    assert!(runtime.watch_state().latest().state().report_finalizations[&execution].lifecycle.is_some());
    let subject = ReportSubject::Execution(execution.to_string());
    let report = runtime.operations().operation::<MaterializeReport>(MaterializeReport { subject, at: earlier }).await.unwrap();
    let payload = crate::service_support::reports::payload(runtime.storage(), &report.report).await;
    assert_cut(&payload, &[2.0, 5.0, 11.0]);
    job.release();
    runtime.wait_execution_report(&execution).await.unwrap();
    assert_eq!(runtime.watch_state().latest().state().executions[&execution].saved.as_ref().map(|seal| &seal.report), Some(&report.report));
    owner.shutdown().await.unwrap();
}
