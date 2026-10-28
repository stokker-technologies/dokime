/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

// A cycle is created empty, refuses to run until cases are appended, then
// runs the appended cases to completion.
#[tokio::test]
async fn empty_cycle_grows() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;

    // Created: Ready, Empty, And Visible
    let cycle_id = start_cases(&runtime, metadata("cycle.explicit", "Explicit Cycle"), Vec::new()).await;
    let created = cycle_snapshot(&runtime, &cycle_id).await;
    assert_eq!(created.lifecycle, CycleStatus::Ready);
    assert_eq!(created.cases().count(), 0);
    assert!(created.test_executions.is_empty());
    let query = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::Cycle,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();
    let RuntimeViewPayload::Cycles { frame } = query.payload else {
        panic!("expected cycle query frame");
    };
    assert_eq!(frame.count, 1);

    // Empty: Running Is Refused
    let refused = runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(refused.status, CommandStatus::Rejected);
    assert_eq!(refused.rejection.as_ref().map(|rejection| rejection.code.as_str()), Some("execution_not_found"));
    assert_eq!(cycle_snapshot(&runtime, &cycle_id).await.lifecycle, CycleStatus::Ready);

    // Appended: Cases Join Unstarted
    let registration = runtime.test_suites().await.into_iter().find(|entry| entry.suite == suite).expect("fixture suite is installed");
    runtime
        .append_fixture_cases(cycle_id.clone(), materialize_cases(&registration.suite_id, &registration.suite), CommandContext::default())
        .await
        .unwrap();
    let appended = cycle_snapshot(&runtime, &cycle_id).await;
    assert_eq!(appended.lifecycle, CycleStatus::Ready);
    assert!(appended.started_at.is_none());
    assert_eq!(appended.cases().count(), 1);
    assert_eq!(appended.test_executions.len(), 1);
    assert_eq!(appended.test_executions[0].status, ExecutionStatus::Skipped);
    assert!(appended.test_executions[0].started_at.is_none());
    assert!(appended.test_executions[0].finished_at.is_none());

    // Run: The Appended Cases Complete
    let accepted = runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(accepted.status, CommandStatus::Accepted, "{accepted:?}");
    let completed = wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    assert_eq!(completed.test_executions.len(), 1);
    assert_eq!(completed.test_executions[0].status, ExecutionStatus::Passed);
}

// Sealing An Empty Cycle Persists An Empty Report.
#[tokio::test]
async fn empty_cycle_seals() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let cycle_id = start_cases(&runtime, metadata("cycle.unused", "Unused Cycle"), Vec::new()).await;

    runtime.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    let stopped = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Stopped).await;
    assert!(stopped.test_executions.is_empty());
    assert!(stopped.saved.is_some(), "stopping seals the empty cycle");

    let mut report = runtime.open_cycle_report(&cycle_id).await.unwrap();
    assert_eq!(report.origin, dokime_runtime::ReportOrigin::Sealed);
    let payload = crate::service_support::reports::parse(
        std::str::from_utf8(&crate::service_support::reports::opened_bytes(&mut report).await).unwrap(),
    );
    assert!(payload["test_executions"].as_array().is_none_or(|executions| executions.is_empty()));
}
