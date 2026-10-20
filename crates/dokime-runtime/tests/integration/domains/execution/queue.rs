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
use dokime_domain::model::execution::result::ResultStatus;
use dokime_runtime::runtime::query::execution_history::ExecutionBrowseLifecycle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn container_request_outcomes() {
    use dokime_domain::model::execution::request::RunPhase;

    let root = tempdir().unwrap();
    let executor = Arc::new(FakePythonExecutor::case_parameter());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor).await;
    let suite = fixture_suite(&["failed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite.clone(), true).await;
    let failed = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
    crate::service_support::waits::wait_for_run_settled(&runtime, &failed.command_id).await;
    let original = cycle_snapshot(&runtime, &cycle).await.test_executions[0].clone();
    let case = materialized_case("provider:testdb:plt400.ract", Some(&suite), "later.passed", "Later Passed", "passed");
    runtime.append_fixture_cases(cycle.clone(), vec![case], CommandContext::default()).await.unwrap();
    let passed = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
    crate::service_support::waits::wait_for_run_settled(&runtime, &passed.command_id).await;
    let receipt = runtime.run_request(&passed.command_id).await.unwrap().unwrap();
    assert_eq!(receipt.summary.phase, RunPhase::Completed);
    assert_eq!((receipt.summary.planned, receipt.summary.failed), (1, 0));
    let cycle = cycle_snapshot(&runtime, &cycle).await;
    assert_eq!(cycle.lifecycle, CycleStatus::Ready);
    assert_eq!(cycle.result.as_ref().map(|result| result.status.clone()), Some(ResultStatus::Failed));
    assert_eq!(cycle.test_executions[0], original);
    assert_eq!(cycle.test_executions[1].status, ExecutionStatus::Passed);
}

#[tokio::test]
async fn container_request_order() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let first = start_cycle(&runtime, suite.clone(), true).await;
    let other = start_cycle(&runtime, suite, true).await;
    let first_run = runtime.start_next_execution(first.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let other_run = runtime.start_next_execution(other.clone(), CommandContext::default()).await;

    let before = cycle_snapshot(&runtime, &first).await;
    let case = before.test_executions[0].snapshot.case().clone();
    runtime.append_fixture_cases(first.clone(), vec![case], CommandContext::default()).await.unwrap();
    let last_run = runtime.start_next_execution(first.clone(), CommandContext::default()).await;
    assert_eq!(first_run.status, CommandStatus::Accepted);
    assert_eq!(other_run.status, CommandStatus::Accepted);
    assert_eq!(last_run.status, CommandStatus::Accepted);
    let snapshot = runtime.watch_state().latest();
    assert_eq!(snapshot.state().executions.values().filter(|execution| execution.status == ExecutionStatus::Running).count(), 1);
    drop(snapshot);

    executor.release();
    for request in [&first_run, &other_run, &last_run] {
        crate::service_support::waits::wait_for_run_settled(&runtime, &request.command_id).await;
    }
    wait_for_cycle_outcome(&runtime, &first, ResultStatus::Passed).await;
    wait_for_cycle_outcome(&runtime, &other, ResultStatus::Passed).await;
    let first = cycle_snapshot(&runtime, &first).await;
    let other = cycle_snapshot(&runtime, &other).await;
    let members = &first.test_executions;
    assert_eq!(members.len(), 3);
    assert!(members.iter().all(|execution| execution.saved.is_some()));
    assert!(members[0].finished_at.as_ref().unwrap() <= other.test_executions[0].started_at.as_ref().unwrap());
    assert!(other.test_executions[0].finished_at.as_ref().unwrap() <= members[1].started_at.as_ref().unwrap());
    let first_receipt = runtime.run_request(&first_run.command_id).await.unwrap().unwrap();
    let last_receipt = runtime.run_request(&last_run.command_id).await.unwrap().unwrap();
    assert_eq!((first_receipt.summary.planned, first_receipt.summary.finished), (1, 1));
    assert_eq!((last_receipt.summary.planned, last_receipt.summary.finished), (1, 1));
}

#[tokio::test]
async fn container_seal_isolation() {
    use dokime_domain::model::execution::result::CyclePhase;

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let first = start_cycle(&runtime, suite.clone(), true).await;
    let other = start_cycle(&runtime, suite, true).await;
    let first_run = runtime.start_next_execution(first.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let other_run = runtime.start_next_execution(other.clone(), CommandContext::default()).await;
    let before = cycle_snapshot(&runtime, &first).await;
    let case = before.test_executions[0].snapshot.case().clone();
    runtime.append_fixture_cases(first.clone(), vec![case], CommandContext::default()).await.unwrap();
    let queued = runtime.start_next_execution(first.clone(), CommandContext::default()).await;

    let (first_seal, repeated_seal) = tokio::join!(
        runtime.seal_cycle(first.clone(), CommandContext::default()),
        runtime.seal_cycle(first.clone(), CommandContext::default()),
    );
    first_seal.unwrap();
    repeated_seal.unwrap();
    let sealed = cycle_snapshot(&runtime, &first).await;
    assert_eq!(sealed.phase, CyclePhase::Sealed);
    assert!(sealed.test_executions.iter().all(|execution| execution.saved.is_some()));
    assert!(sealed.test_executions.iter().all(|execution| execution.status == ExecutionStatus::Cancelled));
    {
        use harmos::state::Transaction;

        let snapshot = runtime.watch_state().latest();
        let mut state = snapshot.state().clone();
        let transaction = dokime_runtime::transactions::SealCycle { sealed_cycle: state.cycles[&first].clone() };
        transaction.check(&state).expect("cycle seal replays against individually sealed members");
        let member = &sealed.test_executions[0].test_execution_id;
        state.executions.get_mut(member).unwrap().saved = None;
        assert!(transaction.check(&state).is_err(), "cycle publication cannot adopt an unsealed member");
    }
    for request in [first_run, queued] {
        let receipt = runtime.run_request(&request.command_id).await.unwrap().unwrap();
        assert!(receipt.summary.phase.terminal());
        assert!(receipt.summary.stop_requested);
    }
    assert_eq!(cycle_snapshot(&runtime, &other).await.phase, CyclePhase::Active);
    executor.release();
    crate::service_support::waits::wait_for_run_settled(&runtime, &other_run.command_id).await;
    wait_for_cycle_outcome(&runtime, &other, ResultStatus::Passed).await;
    let other = cycle_snapshot(&runtime, &other).await;
    assert!(other.test_executions.iter().all(|execution| execution.status == ExecutionStatus::Passed));
    assert_eq!(other.phase, CyclePhase::Active);
}

#[tokio::test]
async fn unavailable_sidecar_context() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone(), false).await;
    let correlation_id = dokime_runtime::model::identity::new_id::<CorrelationId>();
    let context = CommandContext {
        correlation_id: Some(correlation_id.clone()),
        idempotency_key: Some("unavailable-provider".to_owned()),
        submitted_by: Some("runtime-test".to_owned()),
    };

    let receipt = runtime.start_next_execution(test_cycle_id.clone(), context.clone()).await;

    assert_eq!(receipt.status, CommandStatus::Accepted, "{:?}", receipt.rejection);
    assert_eq!(receipt.correlation_id, Some(correlation_id));
    assert!(receipt.idempotency_key.is_none());
    assert!(receipt.rejection.is_none());
    wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Failed).await;
}

#[tokio::test]
async fn append_retains_active() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::case_parameter())).await;
    let suite = fixture_suite(&["failed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone(), false).await;

    runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Failed).await;

    let appended = materialized_case("provider:testdb:plt400.ract", Some(&suite), "append.after_failure", "Append After Failure", "passed");
    let append = runtime.append_fixture_cases(test_cycle_id.clone(), vec![appended.clone()], CommandContext::default()).await;
    let ready = cycle_snapshot(&runtime, &test_cycle_id).await;
    let run = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;

    assert!(append.is_ok(), "{append:?}");
    assert_eq!(ready.lifecycle, CycleStatus::Ready);
    assert!(ready.cases().any(|case| case.test_case_id == appended.test_case_id));
    assert_eq!(latest_execution_for_case(&ready, &appended.test_case_id).unwrap().status, ExecutionStatus::Skipped);
    assert!(ready.result.is_none());
    assert_eq!(run.status, CommandStatus::Accepted, "{:?}", run.rejection);
}

#[tokio::test]
async fn duplicate_case_executions() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, true).await;

    runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    let completed = wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    let original = completed.test_executions[0].clone();
    let original_case = completed.cases().next().unwrap().clone();
    let original_case_id = original_case.test_case_id.clone();
    assert!(original.saved.is_some());

    let append = runtime.append_fixture_cases(test_cycle_id.clone(), vec![original_case], CommandContext::default()).await;
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

    let rerun = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    crate::service_support::waits::wait_for_run_settled(&runtime, &rerun.command_id).await;
    let completed = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(completed.lifecycle, CycleStatus::Ready);
    assert_eq!(completed.result.as_ref().map(|result| result.status.clone()), Some(ResultStatus::Passed));
    assert_eq!(completed.test_executions.len(), 2);
    assert_eq!(completed.test_executions[0], original);
    assert_eq!(completed.test_executions[1].test_execution_id, appended.test_execution_id);
    assert_eq!(completed.test_executions[1].status, ExecutionStatus::Passed);
    let settled = runtime.run_request(&rerun.command_id).await.unwrap().unwrap();
    assert!(completed.test_executions[1].saved.is_some(), "the appended execution seals: {:?}", settled.interruption);
}

#[tokio::test]
async fn run_queued_results() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::case_parameter())).await;
    let suite = fixture_suite(&["passed", "failed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let stop_on_failure = start_cycle(&runtime, suite.clone(), false).await;
    let continue_on_failure = start_cycle(&runtime, suite, true).await;

    runtime.start_next_execution(stop_on_failure.clone(), CommandContext::default()).await;
    runtime.start_next_execution(continue_on_failure.clone(), CommandContext::default()).await;
    let stopped = wait_for_cycle_outcome(&runtime, &stop_on_failure, ResultStatus::Failed).await;
    let mut changes = runtime.watch_state();
    let continued = changes
        .wait_until(|snapshot| {
            snapshot.state().cycles[&continue_on_failure].execution_ids.iter().all(|id| snapshot.state().executions[id].saved.is_some())
        })
        .await
        .unwrap()
        .state()
        .cycle_view(&continue_on_failure)
        .unwrap();

    assert_eq!(stopped.lifecycle, CycleStatus::Failed);
    assert_eq!(stopped.test_executions.len(), 3);
    assert_eq!(stopped.test_executions[2].status, ExecutionStatus::Skipped);
    assert!(stopped.test_executions[2].started_at.is_none());
    assert!(stopped.test_executions[2].saved.is_none());
    assert_eq!(stopped.test_executions[0].status, ExecutionStatus::Passed);
    assert_eq!(stopped.test_executions[1].status, ExecutionStatus::Failed);
    assert_eq!(stopped.cases().count(), 3);
    assert_eq!(stopped.result.as_ref().unwrap().passed, 1);
    assert_eq!(stopped.result.as_ref().unwrap().failed, 1);
    assert_eq!(stopped.result.as_ref().unwrap().blocked, 0);
    assert_eq!(stopped.test_executions[0].steps.len(), 1);
    assert_eq!(stopped.test_executions[0].steps[0].metadata.identifier.as_str(), "fixture.phase");
    assert_eq!(stopped.test_executions[0].steps[0].metadata.label, "Fixture Phase");
    assert_eq!(continued.test_executions.len(), 3);
    assert_eq!(continued.result.as_ref().unwrap().passed, 2);
    assert_eq!(continued.result.as_ref().unwrap().failed, 1);
}

#[tokio::test]
async fn later_cases_start() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone(), false).await;

    let run = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(run.status, CommandStatus::Accepted, "{:?}", run.rejection);
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let first_execution = &running.test_executions[0];
    let second_execution = &running.test_executions[1];

    assert_eq!(first_execution.status, ExecutionStatus::Running);
    assert_eq!(second_execution.status, ExecutionStatus::Skipped);
    assert!(first_execution.started_at.is_some());
    assert!(second_execution.started_at.is_none());

    executor.release();
    let completed = wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    let first_started_at = completed.test_executions[0].started_at.as_ref().expect("first execution started");
    let second_started_at = completed.test_executions[1].started_at.as_ref().expect("second execution started");

    assert_eq!(completed.test_executions[0].status, ExecutionStatus::Passed);
    assert_eq!(completed.test_executions[1].status, ExecutionStatus::Passed);
    // The Second Case Starts Only Once The First Has.
    assert!(second_started_at.as_str() >= first_started_at.as_str());
}

#[tokio::test]
async fn running_queued_counts() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed", "passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let tally = |runtime: &RuntimeHandle| {
        let snapshot = runtime.watch_state().latest();
        let state = snapshot.state();
        dokime_runtime::runtime::query::case_tally(state, &state.cycles[&test_cycle_id])
    };
    let browsed = |runtime: RuntimeHandle| {
        let test_cycle_id = test_cycle_id.clone();
        async move {
            let page = runtime.browse_executions(Default::default()).await.unwrap();
            page.cycles.iter().find(|cycle| cycle.cycle_id == test_cycle_id).cloned().expect("browse reports the cycle")
        }
    };

    // Each Explicitly Accepted Execution Waits In The Serialized Lane.
    let run = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(run.status, CommandStatus::Accepted, "{:?}", run.rejection);
    executor.wait_until_entered().await;
    for _ in 0..2 {
        let queued = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
        assert_eq!(queued.status, CommandStatus::Accepted);
    }
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let waiting = tally(&runtime);
    assert_eq!(waiting.running, 1);
    assert_eq!(waiting.queued, 2);
    let cycle = browsed(runtime.clone()).await;
    assert_eq!(cycle.lifecycle, ExecutionBrowseLifecycle::Running);
    assert_eq!(cycle.running_count, 1);
    assert_eq!(cycle.queued_count, 2);
    assert_eq!(cycle.execution_count, 3);
    assert_eq!(cycle.case_count, 3);
    assert_eq!(cycle.unrun_count, 0);
    assert_eq!(cycle.result_status, None);
    assert!(cycle.started_at.is_some());

    // Settled Executions Have Nothing Running Or Waiting And All Passed.
    executor.release();
    let completed = wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    for execution in &completed.test_executions {
        crate::service_support::native::checkpoint(&runtime, &execution.test_execution_id).await;
    }
    let settled = tally(&runtime);
    assert_eq!(settled.running, 0);
    assert_eq!(settled.queued, 0);
    let cycle = browsed(runtime.clone()).await;
    assert_eq!(cycle.lifecycle, ExecutionBrowseLifecycle::Completed);
    assert_eq!(cycle.result_status, Some(ResultStatus::Passed));
    assert_eq!(cycle.running_count, 0);
    assert_eq!(cycle.queued_count, 0);
    assert_eq!(cycle.passed_count, 3);
    assert!(cycle.finished_at.is_some());
}

#[tokio::test]
async fn execution_retry_receipt() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"; 3]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite, true).await;
    let execution_id = cycle_snapshot(&runtime, &cycle_id).await.test_executions[0].test_execution_id.clone();
    let context =
        CommandContext { idempotency_key: Some("large-request".into()), submitted_by: Some("operator".into()), ..Default::default() };
    let mode = dokime_domain::model::execution::ExecutionMode::Automatic;
    let accepted = runtime.start_execution(execution_id.clone(), mode, context.clone()).await;
    assert_eq!(accepted.status, CommandStatus::Accepted, "{:?}", accepted.rejection);
    let other = CommandContext { submitted_by: Some("another-listener".into()), ..Default::default() };
    let duplicate = runtime.start_execution(execution_id.clone(), mode, other).await;
    assert_eq!(duplicate.command_id, accepted.command_id);
    let cycle = wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    assert_eq!(cycle.test_executions.len(), 3);
    // The cycle fold follows execution sealing; the durable receipt also waits
    // for the final cycle row to be published.
    wait_for_run_settled(&runtime, &accepted.command_id).await;
    let settled = runtime.run_request(&accepted.command_id).await.unwrap().unwrap();
    assert_eq!(settled.summary.phase, dokime_domain::model::execution::request::RunPhase::Completed);
    assert_eq!(settled.summary.prepared, 1);
    assert_eq!(settled.summary.finished, 1);
    assert_eq!(settled.summary.unstarted, 0);
    assert_eq!(runtime.start_execution(execution_id.clone(), mode, context.clone()).await.command_id, accepted.command_id);
    assert_eq!(cycle_snapshot(&runtime, &cycle_id).await.test_executions.len(), 3);
    let sealed_execution = cycle_snapshot(&runtime, &cycle_id).await.test_executions[0].clone();
    runtime.stop_execution(execution_id.clone(), CommandContext::default()).await.unwrap();
    let halted = cycle_snapshot(&runtime, &cycle_id).await;
    assert!(!halted.continue_automatically);
    assert_eq!(halted.test_executions[0], sealed_execution);
    drop(runtime);
    owner.shutdown().await.unwrap();
    let config = config(root.path());
    let (mut rebooted, handle) = started(config).await;
    assert!(handle.runtime_overview().await.expect("runtime overview query succeeds").cycles.is_empty());
    let resolved = handle.start_execution(execution_id, mode, context).await;
    assert_eq!(resolved.status, CommandStatus::Accepted);
    assert_eq!(resolved.command_id, accepted.command_id);
    assert_eq!(handle.run_request(&resolved.command_id).await.unwrap().unwrap().summary.finished, 1);
    drop(handle);
    rebooted.shutdown().await.unwrap();
}

/* === Tests === */
