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
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::execution::ExecutionMode;
use dokime_domain::model::execution::result::CyclePhase;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn refusal_code(result: Result<(), dokime_runtime::runtime::Error>) -> String {
    match result {
        Err(dokime_runtime::runtime::Error::Admission(AdmissionFailure::Conflict { code, .. })) => code,
        other => panic!("cancellation is refused as a conflict: {other:?}"),
    }
}

#[tokio::test]
async fn cancel_skips_unstarted() {
    // === Plan Three Unstarted Attempts === //

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let mut suite = plt400_suite();
    let literals = ["passed", "alternate", "third"].map(|value| ParameterExpr::Literal { value: Value::from(value) });
    suite.test_case_templates[0].parameters[0].expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    suite.test_case_templates[0].parameters[0].expression = ParameterExpr::Array { values: literals.to_vec() };
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite).await;
    let planned = cycle_snapshot(&runtime, &test_cycle_id).await;
    let ids = planned.test_executions.iter().map(|execution| execution.test_execution_id.clone()).collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    assert!(planned.continue_automatically);

    // === Cancel The First Before Anything Runs === //

    let cancel = runtime.cancel_execution(ids[0].clone(), CommandContext::default()).await;
    let repeated = runtime.cancel_execution(ids[0].clone(), CommandContext::default()).await;
    let cancelled_cycle = cycle_snapshot(&runtime, &test_cycle_id).await;
    let evidence = runtime.evidence_snapshot().await.unwrap();

    assert!(cancel.is_ok(), "{cancel:?}");
    assert!(repeated.is_ok(), "a repeated Cancel resolves the durable seal");
    assert_eq!(cancelled_cycle.phase, CyclePhase::Active);
    assert_eq!(cancelled_cycle.lifecycle, CycleStatus::Ready);
    assert!(cancelled_cycle.continue_automatically, "Cancel leaves continuation unchanged");
    assert_eq!(cancelled_cycle.result.as_ref().map(|result| result.skipped), Some(1));
    let cancelled = &cancelled_cycle.test_executions[0];
    let outcome = cancelled.result.as_ref().map(|result| (result.skipped, &result.status));
    assert_eq!(cancelled.status, ExecutionStatus::Skipped);
    assert!(cancelled.finished_at.is_some(), "a cancelled member is finished, unlike a planned one");
    assert_eq!(outcome, Some((1, &ResultStatus::Cancelled)));
    assert!(cancelled.steps.iter().all(|step| step.status == StepStatus::Skipped));
    assert!(cancelled.saved.is_some(), "Cancel seals the skipped attempt");
    assert!(cancelled_cycle.test_executions[1..].iter().all(|execution| execution.started_at.is_none() && execution.saved.is_none()));
    assert!(evidence.records.iter().any(|record| record.record_type.as_str() == "test.execution.stopped"));

    // === Start The Second And Queue The Third Behind It === //

    let started = runtime.start_execution(ids[1].clone(), ExecutionMode::Automatic, CommandContext::default()).await;
    assert_eq!(started.status, CommandStatus::Accepted, "{:?}", started.rejection);
    executor.wait_until_entered().await;
    let queued = runtime.start_execution(ids[2].clone(), ExecutionMode::Automatic, CommandContext::default()).await;
    assert_eq!(queued.status, CommandStatus::Accepted, "{:?}", queued.rejection);

    let running = refusal_code(runtime.cancel_execution(ids[1].clone(), CommandContext::default()).await);
    let dequeued = runtime.cancel_execution(ids[2].clone(), CommandContext::default()).await;
    let during = cycle_snapshot(&runtime, &test_cycle_id).await;
    executor.release();
    wait_for_run_settled(&runtime, &started.command_id).await;
    let settled = cycle_snapshot(&runtime, &test_cycle_id).await;
    let dequeued_request = runtime.run_request(&queued.command_id).await.unwrap().unwrap();

    assert_eq!(running, "execution_already_started");
    assert!(dequeued.is_ok(), "{dequeued:?}");
    assert_eq!(during.test_executions[1].status, ExecutionStatus::Running);
    assert_eq!(during.test_executions[2].status, ExecutionStatus::Skipped);
    assert!(during.test_executions[2].saved.is_some());
    assert_eq!(dequeued_request.summary.prepared, 0);
    assert!(dequeued_request.summary.phase.terminal());
    assert_eq!(settled.phase, CyclePhase::Active);
    assert!(settled.continue_automatically);
    assert!(settled.test_executions[1].finished_at.is_some());
    assert_eq!(settled.test_executions[2].started_at, None, "continuation skips the cancelled member");
}

/* === Tests === */
