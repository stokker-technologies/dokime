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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn unavailable_sidecar_context() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone(), false).await;
    let correlation_id = CorrelationId::new();
    let context = CommandContext {
        correlation_id: Some(correlation_id.clone()),
        idempotency_key: Some("unavailable-provider".to_owned()),
        submitted_by: Some("runtime-test".to_owned()),
    };

    let receipt = runtime
        .run_cycle_cases(test_cycle_id.clone(), context.clone())
        .await;

    assert_eq!(
        receipt.status,
        CommandStatus::Accepted,
        "{:?}",
        receipt.rejection
    );
    assert_eq!(receipt.correlation_id, Some(correlation_id));
    assert_eq!(receipt.idempotency_key, context.idempotency_key);
    assert!(receipt.rejection.is_none());
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Failed).await;
}

pub(crate) async fn append_reopens_cycle() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::case_parameter())).await;
    let suite = fixture_suite(&["failed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone(), false).await;

    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Failed).await;

    let appended = materialized_case(
        "provider:testdb:plt400.ract",
        Some(&suite),
        "append.after_failure",
        "Append After Failure",
        "passed",
    );
    let append = runtime
        .append_cycle_cases(
            test_cycle_id.clone(),
            vec![appended.clone()],
            CommandContext::default(),
        )
        .await;
    let ready = cycle_snapshot(&runtime, &test_cycle_id).await;
    let run = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;

    assert!(append.is_ok(), "{append:?}");
    assert_eq!(ready.lifecycle, CycleStatus::Ready);
    assert!(
        ready
            .cases()
            .any(|case| case.test_case_id == appended.test_case_id)
    );
    assert_eq!(
        latest_execution_for_case(&ready, &appended.test_case_id)
            .unwrap()
            .status,
        ExecutionStatus::Skipped
    );
    assert!(ready.result.is_none());
    assert_eq!(run.status, CommandStatus::Accepted, "{:?}", run.rejection);
}

pub(crate) async fn duplicate_case_executions() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, true).await;

    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    let completed =
        wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    let original = completed.test_executions[0].clone();
    let original_case = completed.cases().next().unwrap().clone();
    let original_case_id = original_case.test_case_id.clone();
    assert!(original.saved.is_some());

    let append = runtime
        .append_cycle_cases(
            test_cycle_id.clone(),
            vec![original_case],
            CommandContext::default(),
        )
        .await;
    assert!(append.is_ok(), "{append:?}");
    let ready = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(ready.cases().count(), 1);
    assert_eq!(ready.test_executions.len(), 2);
    assert_eq!(ready.test_executions[0], original);
    let appended = &ready.test_executions[1];
    assert_eq!(appended.test_case_id, original_case_id);
    assert_ne!(appended.test_execution_id, original.test_execution_id);
    assert_eq!(appended.status, ExecutionStatus::Skipped);
    assert!(appended.saved.is_none());
    assert!(ready.saved.is_none());

    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    let completed =
        wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    assert_eq!(completed.test_executions.len(), 2);
    assert_eq!(completed.test_executions[0], original);
    assert_eq!(
        completed.test_executions[1].test_execution_id,
        appended.test_execution_id
    );
    assert_eq!(completed.test_executions[1].status, ExecutionStatus::Passed);
    assert!(completed.test_executions[1].saved.is_some());
}

pub(crate) async fn run_queued_results() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::case_parameter())).await;
    let suite = fixture_suite(&["passed", "failed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let stop_on_failure = start_cycle(&runtime, suite.clone(), false).await;
    let continue_on_failure = start_cycle(&runtime, suite, true).await;

    runtime
        .run_cycle_cases(stop_on_failure.clone(), CommandContext::default())
        .await;
    runtime
        .run_cycle_cases(continue_on_failure.clone(), CommandContext::default())
        .await;
    let stopped = wait_for_cycle_lifecycle(&runtime, &stop_on_failure, CycleStatus::Failed).await;
    let continued =
        wait_for_cycle_lifecycle(&runtime, &continue_on_failure, CycleStatus::Failed).await;

    assert_eq!(stopped.lifecycle, CycleStatus::Failed);
    // This fixture runs all cases before returning; the fold must retain the
    // already-produced third result regardless of sequential scheduling policy.
    assert_eq!(stopped.test_executions.len(), 3);
    assert_eq!(stopped.test_executions[2].status, ExecutionStatus::Passed);
    assert_eq!(stopped.test_executions[0].status, ExecutionStatus::Passed);
    assert_eq!(stopped.test_executions[1].status, ExecutionStatus::Failed);
    assert_eq!(stopped.cases().count(), 3);
    assert_eq!(stopped.result.as_ref().unwrap().passed, 2);
    assert_eq!(stopped.result.as_ref().unwrap().failed, 1);
    assert_eq!(stopped.result.as_ref().unwrap().blocked, 0);
    assert_eq!(stopped.test_executions[0].steps.len(), 1);
    assert_eq!(
        stopped.test_executions[0].steps[0]
            .metadata
            .identifier
            .as_str(),
        "fixture.phase"
    );
    assert_eq!(
        stopped.test_executions[0].steps[0].metadata.label,
        "Fixture Phase"
    );
    assert_eq!(continued.test_executions.len(), 3);
    assert_eq!(continued.result.as_ref().unwrap().passed, 2);
    assert_eq!(continued.result.as_ref().unwrap().failed, 1);
}

pub(crate) async fn later_cases_start() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone(), false).await;

    let run = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(run.status, CommandStatus::Accepted, "{:?}", run.rejection);
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let first_execution = &running.test_executions[0];
    let second_execution = &running.test_executions[1];

    assert_eq!(first_execution.status, ExecutionStatus::Running);
    assert_eq!(second_execution.status, ExecutionStatus::Pending);
    assert!(first_execution.started_at.is_some());
    assert!(second_execution.started_at.is_none());

    tokio::time::sleep(Duration::from_millis(10)).await;
    executor.release();
    let completed =
        wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    let first_started_at = completed.test_executions[0]
        .started_at
        .as_ref()
        .expect("first execution started");
    let second_started_at = completed.test_executions[1]
        .started_at
        .as_ref()
        .expect("second execution started");

    assert_eq!(completed.test_executions[0].status, ExecutionStatus::Passed);
    assert_eq!(completed.test_executions[1].status, ExecutionStatus::Passed);
    assert_ne!(first_started_at, second_started_at);
}

pub(crate) async fn request_retries_receipt() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"; 3]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite, true).await;
    let context = CommandContext {
        idempotency_key: Some("large-request".into()),
        submitted_by: Some("operator".into()),
        ..Default::default()
    };
    let accepted = runtime
        .run_cycle_cases(cycle_id.clone(), context.clone())
        .await;
    assert_eq!(
        accepted.status,
        CommandStatus::Accepted,
        "{:?}",
        accepted.rejection
    );
    let duplicate = runtime
        .run_cycle_cases(cycle_id.clone(), context.clone())
        .await;
    assert_eq!(duplicate.command_id, accepted.command_id);
    let mut watch = runtime.watch_state();
    let snapshot = tokio::time::timeout(
        Duration::from_secs(10),
        watch.wait_until(|snapshot| {
            snapshot.state().cycles[&cycle_id].lifecycle == CycleStatus::Completed
        }),
    )
    .await
    .unwrap()
    .unwrap();
    let cycle = snapshot.state().cycle_view(&cycle_id).unwrap();
    assert_eq!(cycle.test_executions.len(), 3);
    // The cycle fold follows execution sealing; the durable receipt also waits
    // for the final cycle row to be published.
    let settled = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let receipt = runtime
                .run_request(&accepted.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        settled.summary.phase,
        dokime_app::domains::execution::requests::RunPhase::Completed
    );
    assert_eq!(settled.summary.prepared, 3);
    assert_eq!(settled.summary.finished, 3);
    assert_eq!(settled.summary.unstarted, 0);
    assert_eq!(
        runtime
            .run_cycle_cases(cycle_id.clone(), context.clone())
            .await
            .command_id,
        accepted.command_id
    );
    assert_eq!(
        cycle_snapshot(&runtime, &cycle_id)
            .await
            .test_executions
            .len(),
        3
    );
    drop(runtime);
    owner.shutdown().await.unwrap();
    let config = config(root.path());
    let (mut rebooted, handle) = started(config).await;
    assert!(
        handle
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .cycles
            .is_empty()
    );
    let resolved = handle.run_cycle_cases(cycle_id, context).await;
    assert_eq!(resolved.status, CommandStatus::Accepted);
    assert_eq!(resolved.command_id, accepted.command_id);
    assert_eq!(
        handle
            .run_request(&resolved.command_id)
            .await
            .unwrap()
            .unwrap()
            .summary
            .finished,
        3
    );
    drop(handle);
    rebooted.shutdown().await.unwrap();
}

/* === Tests === */
