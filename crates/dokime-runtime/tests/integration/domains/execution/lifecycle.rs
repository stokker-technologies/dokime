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
async fn stored_cycle(
    runtime: &RuntimeHandle, test_cycle_id: &TestCycleId,
) -> (dokime_domain::model::execution::result::TestCycle, harmos::storage::StorageCursor) {
    let stored = dokime_runtime::storage::objects::cycle::read(runtime.storage(), test_cycle_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    (stored.value.clone(), stored.position.clone())
}

type LabelEdit = (Option<TestCycleId>, Option<TestExecutionId>, Value, Value);

async fn label_edits(subscription: &mut dokime_runtime::runtime::RecordSubscription, count: usize) -> Vec<LabelEdit> {
    let mut edits = Vec::new();
    {
        while edits.len() < count {
            let record =
                subscription.next().await.expect("the record subscription stays open").expect("the record subscription stays readable");
            if record.record_type.as_str() != "operator.label_edited" {
                continue;
            }
            let scope = record.scope.clone().unwrap_or_default();
            edits.push((
                scope.test_cycle_id,
                scope.test_execution_id,
                record.details["old_label"].clone(),
                record.details["new_label"].clone(),
            ));
        }
    }
    edits
}

#[tokio::test]
async fn cycle_container_lifecycle() {
    use dokime_domain::model::execution::result::CyclePhase;
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let first = start_cycle(&runtime, suite.clone(), false).await;
    let second = start_cycle(&runtime, suite, false).await;
    assert_eq!(runtime.cycles().await.len(), 2);

    let run = runtime.start_next_execution(first.clone(), CommandContext::default()).await;
    crate::service_support::waits::wait_for_run_settled(&runtime, &run.command_id).await;
    let completed = cycle_snapshot(&runtime, &first).await;
    assert_eq!(completed.phase, CyclePhase::Active);
    assert!(completed.saved.is_none());
    let original = completed.test_executions[0].clone();
    assert_eq!(original.status, ExecutionStatus::Passed);
    assert!(original.saved.is_some());

    let case = original.snapshot.case().clone();
    runtime.append_fixture_cases(first.clone(), vec![case.clone()], CommandContext::default()).await.unwrap();
    let appended = cycle_snapshot(&runtime, &first).await;
    let execution = appended.test_executions[1].test_execution_id.clone();
    let run =
        runtime.start_execution(execution, dokime_domain::model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    crate::service_support::waits::wait_for_run_settled(&runtime, &run.command_id).await;
    let completed = cycle_snapshot(&runtime, &first).await;
    assert_eq!(completed.test_executions[0], original);
    assert!(completed.test_executions[1].saved.is_some());
    assert_eq!(completed.phase, CyclePhase::Active);

    runtime.seal_cycle(first.clone(), CommandContext::default()).await.unwrap();
    let sealed = cycle_snapshot(&runtime, &first).await;
    assert_eq!(sealed.phase, CyclePhase::Sealed);
    assert!(sealed.saved.is_some());
    assert!(sealed.test_executions.iter().all(|execution| execution.saved.is_some()));
    assert_eq!(sealed.test_executions[0], original);
    runtime.seal_cycle(first.clone(), CommandContext::default()).await.unwrap();
    assert_eq!(cycle_snapshot(&runtime, &first).await, sealed);
    assert!(runtime.append_fixture_cases(first.clone(), vec![case], CommandContext::default()).await.is_err());
    assert_eq!(cycle_snapshot(&runtime, &second).await.phase, CyclePhase::Active);

    owner.shutdown().await.unwrap();
    drop(runtime);
    let (_owner, runtime) = booted(root.path()).await;
    let (saved, _) = stored_cycle(&runtime, &second).await;
    assert_eq!(saved.phase, CyclePhase::Sealed);
    let execution_id = saved.execution_ids[0].clone();
    let execution =
        dokime_runtime::storage::objects::execution::read(runtime.storage(), &execution_id, harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(execution.value.status, ExecutionStatus::Cancelled);
    assert!(execution.value.saved.is_some());
}

#[tokio::test]
async fn rerun_fresh_started() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    let completed = wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    let test_case_id = completed.cases().next().unwrap().test_case_id.clone();
    let first_started_at = completed.test_executions[0].started_at.clone().expect("first execution should have a start timestamp");

    let fresh_id = dokime_runtime::model::identity::new_id::<TestExecutionId>();
    let restart = runtime.append_executions(test_cycle_id.clone(), vec![(fresh_id, test_case_id.clone())]).await;
    assert!(restart.is_ok());

    runtime.start_case_execution(test_cycle_id.clone(), test_case_id, CommandContext::default()).await;
    let restarted = wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    let restarted_execution = &restarted.test_executions[1];
    let restarted_at = restarted_execution.started_at.clone().expect("restarted execution should have a fresh start timestamp");

    // The Re-Run Starts After The Original; The Cycle Keeps Its First Start.
    assert!(restarted_at.as_str() >= first_started_at.as_str());
    assert_eq!(restarted.started_at, completed.started_at);

    // The re-run is a new execution appended after the original, which keeps
    // its identity, outcome and sealed evidence exactly as it was.
    let first = &completed.test_executions[0];
    let original = restarted
        .test_executions
        .iter()
        .find(|execution| execution.test_execution_id == first.test_execution_id)
        .expect("the original execution remains in the cycle");
    assert_eq!(restarted.test_executions.len(), 2);
    assert_eq!(restarted.test_executions.last().unwrap().test_execution_id, restarted_execution.test_execution_id);
    assert_ne!(restarted_execution.test_execution_id, first.test_execution_id);
    assert_eq!(original.status, first.status);
    assert_eq!(original.started_at, first.started_at);
    assert_eq!(original.finished_at, first.finished_at);
    assert_eq!(format!("{:?}", original.saved), format!("{:?}", first.saved));
}

#[tokio::test]
async fn stop_archive_state() {
    use dokime_runtime::runtime::query::cycles::{CycleFilter, CycleOrder, select_cycles};
    use dokime_runtime::runtime::query::executions::ArchiveScope;
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = fixture_suite(&["passed"]);
    runtime
        .set_suites(
            "fixture",
            dokime_runtime::definition::SuiteSet::new([dokime_runtime::domains::TestSuiteDefinition::new(
                "fixture.registration",
                suite.clone(),
            )
            .unwrap()]),
        )
        .await
        .unwrap();
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let listed = |scope: ArchiveScope| {
        let snapshot = runtime.watch_state().latest();
        let filter = CycleFilter { archive: scope, ..CycleFilter::default() };
        select_cycles(snapshot.state(), &filter, CycleOrder::default()).iter().any(|cycle| cycle.test_cycle_id == test_cycle_id)
    };

    // A Cycle That Can Still Run Is Not Terminal, So It Cannot Be Archived
    let early = runtime.set_cycle_archived(test_cycle_id.clone(), true, CommandContext::default()).await;
    assert!(early.is_err(), "a ready cycle must refuse archiving");

    let stop = runtime.seal_cycle(test_cycle_id.clone(), CommandContext::default()).await;
    assert!(stop.is_ok(), "{stop:?}");
    let run_stopped = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(run_stopped.status, CommandStatus::Rejected);
    assert!(
        runtime.browse_executions(Default::default()).await.unwrap().entries.iter().all(|entry| entry.checkpoint.is_some()),
        "cycle sealing durably finalizes never-started members"
    );

    // Sealed Containers May Be Archived Without Changing Their Evidence.
    let before = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(before.lifecycle, CycleStatus::Stopped);
    runtime.set_cycle_archived(test_cycle_id.clone(), true, CommandContext::default()).await.unwrap();
    let archived = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(archived.lifecycle, CycleStatus::Stopped);
    assert_eq!(archived.test_executions, before.test_executions);
    assert!(!listed(ArchiveScope::Unarchived));
    assert!(listed(ArchiveScope::Archived));
    assert!(listed(ArchiveScope::All));

    // An Execution Archives On Its Own, And History Lists It Only When Asked
    let execution = archived.test_executions[0].test_execution_id.clone();
    runtime.set_execution_archived(execution.clone(), true, CommandContext::default()).await.unwrap();
    let browse = |scope: ArchiveScope| {
        let runtime = runtime.clone();
        async move {
            let mut query = dokime_runtime::runtime::query::execution_history::ExecutionBrowseQuery::default();
            query.filter.archive = scope;
            runtime.browse_executions(query).await.unwrap().entries.clone()
        }
    };
    assert!(!browse(ArchiveScope::Unarchived).await.iter().any(|entry| entry.metadata.execution_id == execution));
    let shown = browse(ArchiveScope::Archived).await;
    assert!(shown.iter().any(|entry| entry.metadata.execution_id == execution && entry.archived));

    // Unarchive Restores Both Rows
    runtime.set_cycle_archived(test_cycle_id.clone(), false, CommandContext::default()).await.unwrap();
    runtime.set_execution_archived(execution.clone(), false, CommandContext::default()).await.unwrap();
    assert!(listed(ArchiveScope::Unarchived));
    assert!(browse(ArchiveScope::Unarchived).await.iter().any(|entry| entry.metadata.execution_id == execution && !entry.archived));
}

#[tokio::test]
async fn rename_admission_seal() {
    use dokime_runtime::runtime::query::cycles::{CycleFilter, CycleOrder, select_cycles};
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let ready = cycle_snapshot(&runtime, &test_cycle_id).await;
    let first = ready.test_executions[0].clone();

    // Invalid Labels And Missing Executions Leave The Original State Intact
    let cycle = runtime.rename_cycle(test_cycle_id.clone(), "   ".into(), CommandContext::default()).await;
    assert!(matches!(
        cycle,
        Err(dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::Validation { .. }))
    ));
    let execution = runtime.rename_execution(first.test_execution_id.clone(), String::new(), CommandContext::default()).await;
    assert!(matches!(
        execution,
        Err(dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::Validation { .. }))
    ));
    let missing = runtime
        .rename_execution(dokime_runtime::model::identity::new_id::<TestExecutionId>(), "Charger Soak".into(), CommandContext::default())
        .await;
    assert!(matches!(
        missing,
        Err(dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::NotFound { .. }))
    ));
    let unchanged = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(unchanged.metadata.label, ready.metadata.label);
    assert_eq!(unchanged.test_executions, ready.test_executions);

    let old_cycle_label = ready.metadata.label.clone();
    let old_case_label = first.snapshot.case().metadata.label.clone();
    let cycle_before = stored_cycle(&runtime, &test_cycle_id).await;
    let mut records = runtime.subscribe_records();

    // Before The Run Seals, Both Subjects Take A New Label
    let cycle_label = runtime.rename_cycle(test_cycle_id.clone(), "  Charger Burn-In  ".into(), CommandContext::default()).await.unwrap();
    assert_eq!(cycle_label, "Charger Burn-In");
    let case_label =
        runtime.rename_execution(first.test_execution_id.clone(), "Charger Soak".into(), CommandContext::default()).await.unwrap();
    assert_eq!(case_label, "Charger Soak");

    // Each Rename Is An Evidence Record Carrying Both Labels
    assert_eq!(
        label_edits(&mut records, 2).await,
        [
            (Some(test_cycle_id.clone()), None, Value::from(old_cycle_label), Value::from("Charger Burn-In")),
            (Some(test_cycle_id.clone()), Some(first.test_execution_id.clone()), Value::from(old_case_label), Value::from("Charger Soak")),
        ]
    );

    // The Cycle Persists A New Revision, And Lists Read The New Labels
    let cycle_after = stored_cycle(&runtime, &test_cycle_id).await;
    assert_eq!(cycle_after.0.metadata.label, "Charger Burn-In");
    assert_ne!(cycle_after.1, cycle_before.1, "the cycle has a new revision");
    {
        let snapshot = runtime.watch_state().latest();
        let listed = select_cycles(snapshot.state(), &CycleFilter::default(), CycleOrder::default());
        let cycle = listed.iter().find(|cycle| cycle.test_cycle_id == test_cycle_id).unwrap();
        assert_eq!(cycle.metadata.label, "Charger Burn-In");
        assert_eq!(snapshot.state().cases_for_cycle(cycle).map(|case| case.metadata.label.as_str()).collect::<Vec<_>>(), ["Charger Soak"]);
    }

    // The Sealed Execution Keeps The Label It Ran Under
    runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    runtime.seal_cycle(test_cycle_id.clone(), CommandContext::default()).await.unwrap();
    let checkpoint = crate::service_support::native::checkpoint(&runtime, &first.test_execution_id).await;
    let sealed = crate::service_support::native::open(&runtime, &checkpoint).await;
    assert_eq!(sealed.execution().snapshot.case().metadata.label, "Charger Soak");
    drop(sealed);

    // Once Sealed, Neither Subject Can Be Renamed
    let refused = |result: Result<String, dokime_runtime::runtime::Error>| match result {
        Err(dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::Conflict {
            message,
            ..
        })) => message,
        other => panic!("a sealed subject refuses the rename: {other:?}"),
    };
    let cycle = refused(runtime.rename_cycle(test_cycle_id.clone(), "Too Late".into(), CommandContext::default()).await);
    assert!(cycle.contains("sealed"), "{cycle}");
    let execution = refused(runtime.rename_execution(first.test_execution_id.clone(), "Too Late".into(), CommandContext::default()).await);
    assert!(execution.contains("sealed"), "{execution}");

    // A Sealed Container Refuses New Attempts.
    let test_case_id = first.test_case_id.clone();
    let fresh_id = dokime_runtime::model::identity::new_id::<TestExecutionId>();
    assert!(runtime.append_executions(test_cycle_id.clone(), vec![(fresh_id, test_case_id)]).await.is_err());
    let after = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(after.metadata.label, "Charger Burn-In");
    assert!(after.test_executions.iter().all(|execution| execution.snapshot.case().metadata.label == "Charger Soak"));
}

#[tokio::test]
async fn stopped_telemetry_refused() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    let run = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(run.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let selected = running.test_executions[0].test_execution_id.clone();
    let stop = runtime.stop_execution(selected, CommandContext::default()).await;

    let before = runtime.evidence_snapshot().await.unwrap().telemetry.len();
    publish_tracking_sample(&runtime, &running).await;
    let after = runtime.evidence_snapshot().await.unwrap().telemetry.len();
    executor.release();
    wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Cancelled).await;

    assert!(stop.is_ok());
    assert_eq!(after, before);
}

#[tokio::test]
async fn execution_stop_leaves_cycle_idle() {
    // === Run The First Of Two Executions === //

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let mut suite = plt400_suite();
    suite.test_case_templates[0].parameters[0].expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    suite.test_case_templates[0].parameters[0].expression = ParameterExpr::Array {
        values: vec![ParameterExpr::Literal { value: Value::from("passed") }, ParameterExpr::Literal { value: Value::from("alternate") }],
    };
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let run = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(run.status, CommandStatus::Accepted, "{run:?}");
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    assert_eq!(running.test_executions.len(), 2);

    // === Stop That Execution Alone === //

    let stopped = running.test_executions[0].test_execution_id.clone();
    runtime.stop_execution(stopped.clone(), CommandContext::default()).await.unwrap();
    executor.release();

    // === The Cycle Stays Active And Idle === //

    let idle = wait_for_cycle_idle(&runtime, &test_cycle_id, ResultStatus::Cancelled).await;
    assert_eq!(idle.phase, dokime_domain::model::execution::result::CyclePhase::Active);
    assert_eq!(idle.lifecycle, CycleStatus::Ready, "an execution stop never becomes the cycle's state");
    let execution = |id: &TestExecutionId| idle.test_executions.iter().find(|execution| &execution.test_execution_id == id).unwrap();
    assert_eq!(execution(&stopped).status, ExecutionStatus::Cancelled);
    let sibling = idle.test_executions.iter().find(|execution| execution.test_execution_id != stopped).unwrap();
    assert!(sibling.started_at.is_none(), "the sibling has not run yet");

    // === The Sibling Still Starts === //

    let next = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(next.status, CommandStatus::Accepted, "{next:?}");
    let settled = wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Cancelled).await;
    let sibling = settled.test_executions.iter().find(|execution| execution.test_execution_id == sibling.test_execution_id).unwrap();
    assert!(sibling.finished_at.is_some(), "the sibling ran in the same Active cycle");
}

#[tokio::test]
async fn pause_blocks_boundaries() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = plt400_suite();
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;

    let pause_ready = runtime.pause_cycle(test_cycle_id.clone(), CommandContext::default()).await;
    let resume_ready = runtime.resume_cycle(test_cycle_id.clone(), CommandContext::default()).await;
    assert!(pause_ready.is_err());
    assert!(resume_ready.is_err());

    let run = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    wait_for_record_type(&runtime, "test.cycle.started").await;
    wait_for_record_type(&runtime, "test.execution.started").await;

    let pause_context = CommandContext { idempotency_key: Some("pause-active-request".into()), ..Default::default() };
    let pause = runtime.pause_cycle(test_cycle_id.clone(), pause_context.clone()).await;
    let paused = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Paused).await;
    let paused_wait = runtime.wait_if_paused(&test_cycle_id);
    tokio::pin!(paused_wait);

    assert_eq!(run.status, CommandStatus::Accepted);
    assert!(pause.is_ok());
    assert_eq!(paused.test_executions[0].status, ExecutionStatus::Running);
    wait_for_record_type(&runtime, "test.execution.paused").await;
    let before_retry = runtime.watch_state().latest().state().record_count;
    runtime.pause_cycle(test_cycle_id.clone(), pause_context).await.unwrap();
    assert_eq!(runtime.watch_state().latest().state().record_count, before_retry);
    assert!(futures_util::FutureExt::now_or_never(&mut paused_wait).is_none(), "pause boundary returned before resume");

    let resume = runtime.resume_cycle(test_cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    wait_for_record_type(&runtime, "test.execution.resumed").await;
    paused_wait.await;
    executor.release();
    wait_for_cycle_outcome(&runtime, &test_cycle_id, ResultStatus::Passed).await;
    wait_for_record_type(&runtime, "test.execution.completed").await;
    wait_for_record_type(&runtime, "test.cycle.completed").await;

    assert!(resume.is_ok());
}

/* === Tests === */

#[tokio::test]
async fn continuation_follows_seal() {
    let root = tempdir().unwrap();
    let executor = Arc::new(SecondExecutionHoldExecutor::default());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite, false).await;
    let receipt = runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;

    let during = cycle_snapshot(&runtime, &cycle_id).await;
    assert_eq!(during.phase, dokime_domain::model::execution::result::CyclePhase::Active);
    assert!(during.saved.is_none());
    assert_eq!(during.test_executions[0].status, ExecutionStatus::Passed);
    assert!(during.test_executions[0].saved.is_some());
    assert_eq!(during.test_executions[1].status, ExecutionStatus::Running);
    let saved = dokime_runtime::storage::objects::execution::read(
        runtime.storage(),
        &during.test_executions[0].test_execution_id,
        harmos::storage::ObjectVersion::Latest,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(saved.value.saved, during.test_executions[0].saved);
    assert!(runtime.run_request(&receipt.command_id).await.unwrap().unwrap().summary.phase.terminal());

    executor.release();
    let completed = wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    assert!(completed.test_executions.iter().all(|execution| execution.saved.is_some()));
    assert_eq!(completed.phase, dokime_domain::model::execution::result::CyclePhase::Active);
    assert!(completed.saved.is_none());
}

#[tokio::test]
async fn queued_execution_identity() {
    use dokime_domain::model::execution::{ExecutionMode, result::CyclePhase};
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let running = start_cycle(&runtime, suite.clone(), false).await;
    runtime.update_cycle_policy(running.clone(), false, false, CommandContext::default()).await.unwrap();
    runtime.start_next_execution(running.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;

    let queued = start_cycle(&runtime, suite, false).await;
    let members = cycle_snapshot(&runtime, &queued).await.test_executions;
    let id = members[0].test_execution_id.clone();
    let first = CommandContext { submitted_by: Some("native-listener".into()), ..Default::default() };
    let other = CommandContext { submitted_by: Some("remote-listener".into()), ..Default::default() };
    let (first, duplicate) = tokio::join!(
        runtime.start_execution(id.clone(), ExecutionMode::Automatic, first),
        runtime.start_execution(id.clone(), ExecutionMode::Automatic, other),
    );
    assert_eq!(first.status, CommandStatus::Accepted);
    assert_eq!(duplicate.status, CommandStatus::Accepted);
    assert_eq!(first.command_id, duplicate.command_id);
    let conflict = runtime.start_execution(id.clone(), ExecutionMode::Interactive, CommandContext::default()).await;
    assert_eq!(conflict.status, CommandStatus::Rejected);

    let (stopped_once, stopped_twice) = tokio::join!(
        runtime.stop_execution(id.clone(), CommandContext::default()),
        runtime.stop_execution(id.clone(), CommandContext::default()),
    );
    stopped_once.unwrap();
    stopped_twice.unwrap();
    let stopped = cycle_snapshot(&runtime, &queued).await;
    assert_eq!(stopped.phase, CyclePhase::Active);
    assert!(!stopped.continue_automatically);
    assert_eq!(stopped.test_executions[0].status, ExecutionStatus::Cancelled);
    assert!(stopped.test_executions[0].saved.is_some());
    assert_eq!(stopped.test_executions[1], members[1]);
    assert_eq!(cycle_snapshot(&runtime, &running).await.test_executions[0].status, ExecutionStatus::Running);
    let retry = runtime.start_execution(id, ExecutionMode::Automatic, CommandContext::default()).await;
    assert_eq!(retry.command_id, first.command_id);
    assert_eq!(retry.status, CommandStatus::Accepted);
    executor.release();
}

#[tokio::test]
async fn continuation_failure_retained() {
    use dokime_domain::model::execution::{ExecutionMode, result::CyclePhase};
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::case_parameter())).await;
    let suite = fixture_suite(&["failed", "passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite, false).await;
    let before = cycle_snapshot(&runtime, &cycle).await;
    let first = before.test_executions[0].test_execution_id.clone();
    let receipt = runtime.start_execution(first, ExecutionMode::Automatic, CommandContext::default()).await;
    wait_for_run_settled(&runtime, &receipt.command_id).await;
    let failed = cycle_snapshot(&runtime, &cycle).await;
    assert_eq!(failed.phase, CyclePhase::Active);
    assert_eq!(failed.test_executions[0].status, ExecutionStatus::Failed);
    assert!(failed.test_executions[0].saved.is_some());
    assert_eq!(failed.test_executions[1], before.test_executions[1]);

    let case = before.test_executions[1].snapshot.case().clone();
    runtime.append_fixture_cases(cycle.clone(), vec![case], CommandContext::default()).await.unwrap();
    let idle = cycle_snapshot(&runtime, &cycle).await;
    assert_eq!(idle.test_executions.len(), 3);
    assert!(idle.test_executions[1..].iter().all(|execution| execution.started_at.is_none() && execution.finished_at.is_none()));
    assert_eq!(runtime.watch_state().latest().state().run_requests.len(), 1);
}
