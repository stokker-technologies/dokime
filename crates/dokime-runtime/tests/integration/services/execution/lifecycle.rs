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
async fn wait_for_cycle_seal(
    runtime: &RuntimeHandle,
    test_cycle_id: &TestCycleId,
) -> dokime_domain::model::execution::result::CycleView {
    until_state(runtime, |state| {
        state
            .cycle_views()
            .any(|cycle| &cycle.test_cycle_id == test_cycle_id && cycle.saved.is_some())
    })
    .await;
    cycle_snapshot(runtime, test_cycle_id).await
}

async fn stored_cycle(
    runtime: &RuntimeHandle,
    test_cycle_id: &TestCycleId,
) -> (
    dokime_domain::model::execution::result::TestCycle,
    harmos::storage::StorageCursor,
) {
    let stored = dokime_runtime::storage::objects::cycle::read(
        runtime.storage(),
        test_cycle_id,
        harmos::storage::ObjectVersion::Latest,
    )
    .await
    .unwrap()
    .unwrap();
    (stored.value.clone(), stored.position.clone())
}

type LabelEdit = (Option<TestCycleId>, Option<TestExecutionId>, Value, Value);

async fn label_edits(
    subscription: &mut dokime_runtime::runtime::RecordSubscription,
    count: usize,
) -> Vec<LabelEdit> {
    let mut edits = Vec::new();
    {
        while edits.len() < count {
            let record = subscription
                .next()
                .await
                .expect("the record subscription stays open")
                .expect("the record subscription stays readable");
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
async fn rerun_fresh_started() {
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
        let filter = CycleFilter {
            archive: scope,
            ..CycleFilter::default()
        };
        select_cycles(snapshot.state(), &filter, CycleOrder::default())
            .iter()
            .any(|cycle| cycle.test_cycle_id == test_cycle_id)
    };

    // A Cycle That Can Still Run Is Not Terminal, So It Cannot Be Archived
    let early = runtime
        .set_cycle_archived(test_cycle_id.clone(), true, CommandContext::default())
        .await;
    assert!(early.is_err(), "a ready cycle must refuse archiving");

    let stop = runtime
        .stop_cycle(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert!(stop.is_ok(), "{stop:?}");
    let run_stopped = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(run_stopped.status, CommandStatus::Rejected);
    assert!(
        runtime
            .browse_executions(Default::default())
            .await
            .unwrap()
            .entries
            .iter()
            .all(|entry| entry.checkpoint.is_none()),
        "never-started execution must remain an unsealed draft"
    );

    // Stopped And Unsealed Is Terminal: Archiving Hides It And Changes Nothing Else
    let before = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(before.lifecycle, CycleStatus::Stopped);
    runtime
        .set_cycle_archived(test_cycle_id.clone(), true, CommandContext::default())
        .await
        .unwrap();
    let archived = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(archived.lifecycle, CycleStatus::Stopped);
    assert_eq!(archived.test_executions, before.test_executions);
    assert!(!listed(ArchiveScope::Unarchived));
    assert!(listed(ArchiveScope::Archived));
    assert!(listed(ArchiveScope::All));

    // An Execution Archives On Its Own, And History Lists It Only When Asked
    let execution = archived.test_executions[0].test_execution_id.clone();
    runtime
        .set_execution_archived(execution.clone(), true, CommandContext::default())
        .await
        .unwrap();
    let browse = |scope: ArchiveScope| {
        let runtime = runtime.clone();
        async move {
            let mut query =
                dokime_runtime::runtime::query::execution_history::ExecutionBrowseQuery::default();
            query.filter.archive = scope;
            runtime
                .browse_executions(query)
                .await
                .unwrap()
                .entries
                .clone()
        }
    };
    assert!(
        !browse(ArchiveScope::Unarchived)
            .await
            .iter()
            .any(|entry| entry.metadata.execution_id == execution)
    );
    let shown = browse(ArchiveScope::Archived).await;
    assert!(
        shown
            .iter()
            .any(|entry| entry.metadata.execution_id == execution && entry.archived)
    );

    // Unarchive Restores Both Rows
    runtime
        .set_cycle_archived(test_cycle_id.clone(), false, CommandContext::default())
        .await
        .unwrap();
    runtime
        .set_execution_archived(execution.clone(), false, CommandContext::default())
        .await
        .unwrap();
    assert!(listed(ArchiveScope::Unarchived));
    assert!(
        browse(ArchiveScope::Unarchived)
            .await
            .iter()
            .any(|entry| entry.metadata.execution_id == execution && !entry.archived)
    );
}

#[tokio::test]
async fn rename_refuses_sealed() {
    use dokime_runtime::runtime::query::cycles::{CycleFilter, CycleOrder, select_cycles};
    let root = tempdir().unwrap();
    let (_owner, runtime) =
        booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let ready = cycle_snapshot(&runtime, &test_cycle_id).await;
    let first = ready.test_executions[0].clone();
    let old_cycle_label = ready.metadata.label.clone();
    let old_case_label = first.snapshot.case().metadata.label.clone();
    let cycle_before = stored_cycle(&runtime, &test_cycle_id).await;
    let mut records = runtime.subscribe_records();

    // Before The Run Seals, Both Subjects Take A New Label
    let cycle_label = runtime
        .rename_cycle(
            test_cycle_id.clone(),
            "  Charger Burn-In  ".into(),
            CommandContext::default(),
        )
        .await
        .unwrap();
    assert_eq!(cycle_label, "Charger Burn-In");
    let case_label = runtime
        .rename_execution(
            first.test_execution_id.clone(),
            "Charger Soak".into(),
            CommandContext::default(),
        )
        .await
        .unwrap();
    assert_eq!(case_label, "Charger Soak");

    // Each Rename Is An Evidence Record Carrying Both Labels
    assert_eq!(
        label_edits(&mut records, 2).await,
        [
            (
                Some(test_cycle_id.clone()),
                None,
                Value::from(old_cycle_label),
                Value::from("Charger Burn-In")
            ),
            (
                Some(test_cycle_id.clone()),
                Some(first.test_execution_id.clone()),
                Value::from(old_case_label),
                Value::from("Charger Soak")
            ),
        ]
    );

    // The Cycle Persists A New Revision, And Lists Read The New Labels
    let cycle_after = stored_cycle(&runtime, &test_cycle_id).await;
    assert_eq!(cycle_after.0.metadata.label, "Charger Burn-In");
    assert_ne!(
        cycle_after.1, cycle_before.1,
        "the cycle has a new revision"
    );
    {
        let snapshot = runtime.watch_state().latest();
        let listed = select_cycles(
            snapshot.state(),
            &CycleFilter::default(),
            CycleOrder::default(),
        );
        let cycle = listed
            .iter()
            .find(|cycle| cycle.test_cycle_id == test_cycle_id)
            .unwrap();
        assert_eq!(cycle.metadata.label, "Charger Burn-In");
        assert_eq!(
            snapshot
                .state()
                .cases_for_cycle(cycle)
                .map(|case| case.metadata.label.as_str())
                .collect::<Vec<_>>(),
            ["Charger Soak"]
        );
    }

    // The Sealed Execution Keeps The Label It Ran Under
    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Completed).await;
    wait_for_cycle_seal(&runtime, &test_cycle_id).await;
    let checkpoint = crate::service_support::native::checkpoint(&runtime, &first.test_execution_id).await;
    let sealed = crate::service_support::native::open(&runtime, &checkpoint).await;
    assert_eq!(
        sealed.execution().snapshot.case().metadata.label,
        "Charger Soak"
    );
    drop(sealed);

    // Once Sealed, Neither Subject Can Be Renamed
    let refused = |result: Result<String, dokime_runtime::runtime::Error>| match result {
        Err(dokime_runtime::runtime::Error::Conflict { message, .. }) => message,
        other => panic!("a sealed subject refuses the rename: {other:?}"),
    };
    let cycle = refused(
        runtime
            .rename_cycle(
                test_cycle_id.clone(),
                "Too Late".into(),
                CommandContext::default(),
            )
            .await,
    );
    assert!(cycle.contains("sealed"), "{cycle}");
    let execution = refused(
        runtime
            .rename_execution(
                first.test_execution_id.clone(),
                "Too Late".into(),
                CommandContext::default(),
            )
            .await,
    );
    assert!(execution.contains("sealed"), "{execution}");

    // A Re-Run Shares The Sealed Case, So It Refuses Too
    let test_case_id = first.test_case_id.clone();
    runtime
        .rerun_cycle_case(
            test_cycle_id.clone(),
            test_case_id,
            CommandContext::default(),
        )
        .await
        .unwrap();
    let rerun = cycle_snapshot(&runtime, &test_cycle_id).await;
    let attempt = rerun.test_executions.last().unwrap().clone();
    assert_ne!(attempt.test_execution_id, first.test_execution_id);
    assert_eq!(attempt.snapshot.case().metadata.label, "Charger Soak");
    refused(
        runtime
            .rename_execution(
                attempt.test_execution_id,
                "Too Late".into(),
                CommandContext::default(),
            )
            .await,
    );
    let after = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(after.metadata.label, "Charger Burn-In");
    assert!(
        after
            .test_executions
            .iter()
            .all(|execution| execution.snapshot.case().metadata.label == "Charger Soak")
    );
}

#[tokio::test]
async fn rename_refuses_empty() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    let before = cycle_snapshot(&runtime, &test_cycle_id).await;
    let execution = before.test_executions[0].test_execution_id.clone();

    let cycle = runtime
        .rename_cycle(
            test_cycle_id.clone(),
            "   ".into(),
            CommandContext::default(),
        )
        .await;
    assert!(matches!(cycle, Err(dokime_runtime::runtime::Error::Validation { .. })));
    let execution = runtime
        .rename_execution(execution, String::new(), CommandContext::default())
        .await;
    assert!(matches!(
        execution,
        Err(dokime_runtime::runtime::Error::Validation { .. })
    ));
    let missing = runtime
        .rename_execution(
            dokime_runtime::model::identity::new_id::<TestExecutionId>(),
            "Charger Soak".into(),
            CommandContext::default(),
        )
        .await;
    assert!(matches!(missing, Err(dokime_runtime::runtime::Error::NotFound { .. })));

    let after = cycle_snapshot(&runtime, &test_cycle_id).await;
    assert_eq!(after.metadata.label, before.metadata.label);
    assert_eq!(after.test_executions, before.test_executions);
}

#[tokio::test]
async fn stopped_cycles_telemetry() {
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

#[tokio::test]
async fn pause_blocks_boundaries() {
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
    let paused_wait = tokio::spawn(async move {
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
    tokio::task::yield_now().await;
    assert!(
        !paused_wait.is_finished(),
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

#[tokio::test]
async fn pause_refuses_states() {
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
    let test_cycle_id = start_cases(
        &runtime,
        metadata("cycle.start", "Start Cycle"),
        materialize_cases("fixture.registration", &suite),
        false,
    )
    .await;

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
