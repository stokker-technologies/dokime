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

pub(crate) async fn rerun_fresh_started() {
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    let completed =
        wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    let test_case_id = completed.cases().next().unwrap().test_case_id.clone();
    let first_started_at = completed.test_executions[0]
        .started_at
        .clone()
        .expect("first execution should have a start timestamp");

    let restart = runtime
        .rerun_cycle_case(
            test_cycle_id.clone(),
            test_case_id.clone(),
            CommandContext::default(),
        )
        .await;
    assert!(restart.is_ok());

    tokio::time::sleep(Duration::from_millis(5)).await;
    runtime
        .run_cycle_case(
            test_cycle_id.clone(),
            test_case_id,
            CommandContext::default(),
        )
        .await;
    let restarted =
        wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    let restarted_execution = &restarted.test_executions[1];
    let restarted_at = restarted_execution
        .started_at
        .clone()
        .expect("restarted execution should have a fresh start timestamp");

    assert_ne!(restarted_at, first_started_at);
    assert_ne!(Some(restarted_at), restarted.started_at);

    // The re-run is a new execution appended after the original, which keeps
    // its identity, outcome and sealed evidence exactly as it was.
    let first = &completed.test_executions[0];
    let original = restarted
        .test_executions
        .iter()
        .find(|execution| execution.test_execution_id == first.test_execution_id)
        .expect("the original execution remains in the cycle");
    assert_eq!(restarted.test_executions.len(), 2);
    assert_eq!(
        restarted.test_executions.last().unwrap().test_execution_id,
        restarted_execution.test_execution_id
    );
    assert_ne!(
        restarted_execution.test_execution_id,
        first.test_execution_id
    );
    assert_eq!(original.status, first.status);
    assert_eq!(original.started_at, first.started_at);
    assert_eq!(original.finished_at, first.finished_at);
    assert_eq!(
        format!("{:?}", original.saved),
        format!("{:?}", first.saved)
    );
}

pub(crate) async fn stop_clear_state() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = fixture_suite(&["passed"]);
    runtime
        .set_suites(
            "fixture",
            dokime_app::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition::new(
                "fixture.registration",
                suite.clone(),
            )
            .unwrap()]),
        )
        .await
        .unwrap();
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    let stop = runtime
        .stop_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;
    let run_stopped = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    let clear = runtime
        .clear_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;
    let run_cleared = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = cycle_snapshot(&runtime, &test_cycle_id).await;

    assert!(stop.is_ok());
    assert!(
        runtime
            .browse_executions(Default::default(), |page, _| Ok(page.clone()))
            .await
            .unwrap()
            .entries
            .iter()
            .all(|entry| entry.checkpoint.is_none()),
        "never-started execution must remain an unsealed draft"
    );
    assert_eq!(run_stopped.status, CommandStatus::Rejected);
    assert!(clear.is_ok());
    assert_eq!(run_cleared.status, CommandStatus::Rejected);
    assert_eq!(cycle.lifecycle, CycleStatus::Cleared);
    assert!(cycle.cases().next().is_none());
    assert!(cycle.test_executions.is_empty());
}

pub(crate) async fn stopped_cycles_telemetry() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    let run = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(run.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let stop = runtime
        .stop_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Stopping).await;

    let before = runtime.evidence_snapshot().await.unwrap().telemetry.len();
    publish_tracking_sample(&runtime, &running).await;
    let after = runtime.evidence_snapshot().await.unwrap().telemetry.len();
    executor.release();
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Stopped).await;

    assert!(stop.is_ok());
    assert_eq!(after, before);
}

pub(crate) async fn pause_blocks_boundaries() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    let run = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    wait_for_record_type(&runtime, "test.cycle.started").await;
    wait_for_record_type(&runtime, "test.execution.started").await;

    let pause_context = CommandContext {
        idempotency_key: Some("pause-active-request".into()),
        ..Default::default()
    };
    let pause = runtime
        .pause_cycle(test_cycle_id.clone(), pause_context.clone())
        .await;
    let paused = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Paused).await;
    let blocked_runtime = runtime.clone();
    let blocked_test_cycle_id = test_cycle_id.clone();
    let mut paused_wait = tokio::spawn(async move {
        blocked_runtime.wait_if_paused(&blocked_test_cycle_id).await;
    });

    assert_eq!(run.status, CommandStatus::Accepted);
    assert!(pause.is_ok());
    assert_eq!(paused.test_executions[0].status, ExecutionStatus::Running);
    wait_for_record_type(&runtime, "test.execution.paused").await;
    let before_retry = runtime.watch_state().latest().state().record_count;
    runtime
        .pause_cycle(test_cycle_id.clone(), pause_context)
        .await
        .unwrap();
    assert_eq!(
        runtime.watch_state().latest().state().record_count,
        before_retry
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), &mut paused_wait)
            .await
            .is_err(),
        "pause boundary returned before resume",
    );

    let resume = runtime
        .resume_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    wait_for_record_type(&runtime, "test.execution.resumed").await;
    paused_wait.await.unwrap();
    executor.release();
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    wait_for_record_type(&runtime, "test.execution.completed").await;
    wait_for_record_type(&runtime, "test.cycle.completed").await;

    assert!(resume.is_ok());
}

pub(crate) async fn pause_refuses_states() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = fixture_suite(&["passed"]);
    runtime
        .set_suites(
            "fixture",
            dokime_app::definition::SuiteSet::new([dokime::domains::TestSuiteDefinition::new(
                "fixture.registration",
                suite.clone(),
            )
            .unwrap()]),
        )
        .await
        .unwrap();
    let start = runtime
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("cycle.start", "Start Cycle"),
                cases: materialize_cases("fixture.registration", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await;
    let test_cycle_id = start.expect("cycle starts");

    let pause_ready = runtime
        .pause_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;
    let resume_ready = runtime
        .resume_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;

    assert!(pause_ready.is_err());
    assert!(resume_ready.is_err());
}

/* === Tests === */
