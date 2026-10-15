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
use sha2::Digest;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct ReportReader;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::Application for ReportReader {
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = harmos::state::Never;
    type Streams = ();
    type Resources = ();
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn report_save_receipt() {
    use dokime::execution::saved::ExecutionRevision;
    use dokime::storage::sqlite::{Limits, RecoveryLimits};
    use dokime::storage::{
        Backend,
        objects::execution::{self, Execution},
    };
    use dokime::{
        core::NodeId,
        definition::TestCase,
        execution::{ExecutionSnapshot, TestExecution},
        storage::Manager,
    };
    use dokime_app::{
        execution::CycleView,
        report::{
            CycleReportInput,
            saved::{CapturedExecutionReportInput, PreparedExecutionReport},
        },
    };
    use harmos::storage::{ObjectVersion, StorageBackends, StorageSettings};
    use std::{num::NonZeroUsize, sync::Arc, time::Duration};

    let reads = crate::support::budgets::story_settings();
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::primary(root.path().join("primary"), Limits::default())
            .await
            .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(RecoveryLimits::default()));
    let manager = Manager::new(
        StorageBackends {
            primary,
            recovery,
            replicas: vec![],
        },
        // The Runtime Admits Its Configured Execution Read Budget.
        StorageSettings {
            flush_interval: Duration::from_secs(60),
            query_execution: reads.interactive,
            query_execution_ceiling: reads.ceiling,
            read_memory_bytes: NonZeroUsize::new(256 * 1024 * 1024).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    let runtime = harmos::Runtime::<ReportReader>::builder((), ())
        .runtime_with_storage(&manager)
        .await
        .unwrap();

    // Use An Actual Admitted Execution Revision
    let finished_at = Timestamp::new("2026-10-06T06:00:00Z");
    let cycle_id = TestCycleId::new();
    let case = TestCase {
        suite_id: "fixture.registration".into(),
        test_case_id: dokime::core::CaseId::new(),
        test_case_template_id: TemplateId::new(),
        metadata: metadata("fixture.case", "Stored Report Case"),
        suite: None,
        parameters: BTreeMap::new(),
        generation_index: 0,
        label_number: 0,
    };
    let execution_id = TestExecutionId::new();
    let original_execution = TestExecution {
        saved: None,
        mode: Default::default(),
        active_step_attempt: None,
        capture_phase: Default::default(),
        step_attempts: vec![],
        test_cycle_id: cycle_id.clone(),
        test_execution_id: execution_id.clone(),
        test_case_id: case.test_case_id.clone(),
        suite: None,
        status: ExecutionStatus::Skipped,
        // The Captured Context Names Its Own Execution, As Capture Does.
        scope: RecordScope::default()
            .with_cycle(cycle_id.clone())
            .with_test_case(case.test_case_id.clone())
            .with_test_execution(execution_id),
        snapshot: ExecutionSnapshot::materialize(
            &case,
            None,
            Default::default(),
            None,
            Default::default(),
        )
        .unwrap(),
        invalidation: None,
        output_parameters: BTreeMap::new(),
        documents: BTreeMap::new(),
        records: Default::default(),
        started_at: None,
        finished_at: Some(finished_at.clone()),
        steps: vec![],
        traceability: vec![],
        result: None,
    };
    let input = CycleReportInput {
        session_id: dokime::core::SessionId::new(),
        cycle: CycleView {
            saved: None,
            test_cycle_id: cycle_id,
            session_id: None,
            metadata: metadata("fixture.cycle", "Stored Report Cycle"),
            lifecycle: CycleStatus::Stopped,
            created_at: finished_at.clone(),
            started_at: None,
            updated_at: finished_at.clone(),
            test_executions: vec![original_execution.clone()],
            records: Default::default(),
            continue_on_failure: false,
            result: None,
        },
        documents: BTreeMap::new(),
        plots: BTreeMap::new(),
        case_documents: vec![],
        records: vec![],
        telemetry: vec![],
        traceability: vec![],
        artifact_links: vec![],
        appearance: Some(
            dokime_app::report::value::Value::try_from(
                serde_json::json!({"app_name": "Later Renderer Brand"}),
            )
            .unwrap(),
        ),
        generated_at: finished_at.clone(),
        omissions: Vec::new(),
    };
    let row = Execution::try_from(&original_execution).unwrap();
    let receipt = manager.write_object(&row).await.unwrap();
    let mut grouping = input.cycle.clone();
    grouping.test_executions.clear();
    let context = dokime_app::execution::context::ExecutionContext {
        scope: original_execution.scope.clone(),
        session_id: input.session_id.clone(),
        cycle: dokime_app::execution::context::ExecutionCycleContext {
            cycle: grouping,
            cases: vec![original_execution.snapshot.case().clone()],
            execution_order: vec![original_execution.test_execution_id.clone()],
        },
        execution_documents: BTreeMap::new(),
        prompts: Vec::new(),
        health: vec![dokime::execution::HealthItem {
            health_item_id: dokime::core::HealthItemId::from_seed("captured-health"),
            scope: original_execution.scope.clone(),
            severity: dokime::execution::HealthSeverity::Warning,
            message: "Captured Warning".into(),
            detail: Some("Original observation".into()),
            observed_at: finished_at.clone(),
        }],
        notes: vec![dokime::execution::OperatorNote {
            note_id: dokime::core::NoteId::from_seed("captured-note"),
            scope: original_execution.scope.clone(),
            author: None,
            message: "Original operator note".into(),
            created_at: finished_at.clone(),
        }],
        traceability: Vec::new(),
        appearance: serde_json::from_value(serde_json::json!({
            "app_name": "Frozen Fixture", "theme_family_locked": false,
            "mode_policy": "user_or_system", "mode_locked": false
        }))
        .unwrap(),
    };
    let captured = CapturedExecutionReportInput {
        final_report: true,
        owner_node_id: NodeId::from_seed("fixture.report.owner"),
        execution: ExecutionRevision {
            execution_id: original_execution.test_execution_id.clone(),
            position: receipt.position.clone(),
            version: receipt.version,
        },
        evidence_through: receipt.position,
        telemetry_contracts: BTreeMap::new(),
        record_contracts: BTreeMap::new(),
        input,
        context,
    };
    let before = manager.status().accepted;
    let captured = manager
        .produce(
            NonZeroUsize::new(harmos::MemorySize::memory_size(&captured)).unwrap(),
            |_| Ok(captured),
        )
        .unwrap()
        .try_into_value()
        .unwrap()
        .try_into_shared()
        .unwrap();
    let outputs = manager
        .produce(
            NonZeroUsize::new(std::mem::size_of::<
                Vec<dokime::execution::saved::ExecutionOutput>,
            >())
            .unwrap(),
            |_| Ok(Vec::new()),
        )
        .unwrap()
        .try_into_value()
        .unwrap();
    let mut prepared =
        PreparedExecutionReport::prepare(manager.clone(), captured.clone(), outputs).unwrap();
    assert_eq!(manager.status().accepted, before);
    let sealed_at = finished_at;
    let original = prepared.save_dependencies(&sealed_at).await.unwrap();
    let saved = manager.status().saved;
    let repeated = prepared.save_dependencies(&sealed_at).await.unwrap();
    assert_eq!(*original, *repeated);
    assert_eq!(manager.status().saved, saved);
    assert!(
        prepared
            .save_dependencies(&Timestamp::new("2030-01-01T00:00:00Z"))
            .await
            .is_err()
    );
    assert_eq!(manager.status().saved, saved);

    // Read The Original Native Captured File
    let mut reader = manager.read_file(&original.source.context).await.unwrap();
    let mut bytes = Vec::new();
    while let Some(chunk) = reader
        .next_chunk(NonZeroUsize::new(4096).unwrap())
        .await
        .unwrap()
    {
        bytes.extend_from_slice(chunk.as_bytes());
    }
    assert!(!bytes.is_empty());
    assert_eq!(bytes.len() as u64, original.source.context.length);
    assert_eq!(
        sha2::Sha256::digest(&bytes).as_slice(),
        original.source.context.sha256.as_slice()
    );
    drop(reader);

    // Flush The Seal's Own Original Receipt
    let mut sealed_execution = original_execution;
    sealed_execution.saved = Some((*original).clone());
    let sealed = Execution::try_from(&sealed_execution).unwrap();
    let before_seal = manager
        .view_with_limits(crate::support::budgets::story_settings().report)
        .await
        .unwrap();
    let receipt = manager.write_object(&sealed).await.unwrap();
    execution::flush_seal(&manager, &sealed_execution, &receipt)
        .await
        .unwrap();
    let view = manager
        .view_with_limits(crate::support::budgets::story_settings().report)
        .await
        .unwrap();
    let restored = execution::read_in(
        &manager,
        &view,
        &sealed_execution.test_execution_id,
        ObjectVersion::Exact(receipt.version.clone()),
        NonZeroUsize::new(1024 * 1024).unwrap(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(restored.position, receipt.position);
    assert_eq!(restored.version, receipt.version);
    // Download The Sealed Bytes Without Rendering Or Admitting New Evidence.
    let accepted = manager.status().accepted;
    let original_bytes = crate::support::reports::bytes(&manager, &original.report).await;
    let html = String::from_utf8(original_bytes.clone()).unwrap();
    let payload = crate::support::reports::parse(&html);
    assert_eq!(payload["appearance"]["app_name"], "Frozen Fixture");
    assert_eq!(
        payload["snapshot"]["notes"][0]["text"],
        "Original operator note"
    );
    assert_eq!(
        payload["snapshot"]["health"][0]["title"],
        "Captured Warning"
    );
    assert_eq!(payload["snapshot"]["health"][0]["level"], "warning");
    assert_eq!(
        payload["snapshot"]["notes"][0]["scope"]["test_execution_id"],
        sealed_execution.test_execution_id.as_str()
    );
    assert!(!html.contains("Later Renderer Brand"));
    assert_eq!(payload["test_executions"].as_array().unwrap().len(), 1);
    assert_eq!(
        payload["test_executions"][0]["test_execution_id"],
        sealed_execution.test_execution_id.as_str()
    );
    assert_eq!(
        crate::support::reports::bytes(&manager, &original.report).await,
        original_bytes
    );
    assert_eq!(manager.status().accepted, accepted);

    // Inspect The Original Seal And Preserve Its Earlier Evidence Cut
    let limit = NonZeroUsize::new(1024 * 1024).unwrap();
    let inspection = dokime_app::runtime::ExecutionView::open(
        &manager,
        view.clone(),
        &sealed_execution.test_execution_id,
        ObjectVersion::Exact(receipt.version.clone()),
        limit,
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        inspection
            .checkpoint(NonZeroUsize::new(1).unwrap())
            .is_err()
    );
    let checkpoint = inspection.checkpoint(limit).unwrap();
    assert_eq!(checkpoint.position, receipt.position);
    assert_eq!(checkpoint.version, receipt.version);
    assert_eq!(
        checkpoint.evidence_through,
        original.source.evidence_through
    );
    assert_ne!(checkpoint.position, checkpoint.evidence_through);
    let reopened = dokime_app::runtime::ExecutionView::open_checkpoint(
        &manager,
        view.clone(),
        &checkpoint,
        limit,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(reopened.execution(), &sealed_execution);
    // Reopened Snapshots Keep Exact Context And Original Loan Identity.
    assert!(reopened.context().is_err());
    let accepted = manager.status().accepted;
    let snapshot = reopened.snapshot(&manager).await.unwrap();
    assert_eq!(snapshot.context().unwrap(), &captured.context);
    assert_eq!(
        snapshot.captured_at().unwrap(),
        &captured.input.generated_at
    );
    assert_eq!(snapshot.execution(), &sealed_execution);
    assert_eq!(*snapshot.checkpoint(limit).unwrap(), *checkpoint);
    let ceiling = snapshot.view().decoded_bytes().unwrap();
    let oversized = NonZeroUsize::new(ceiling.get().checked_add(1).unwrap()).unwrap();
    assert!(
        snapshot
            .project_context(oversized, |_, _| Ok(true))
            .is_err()
    );
    let shared = snapshot.snapshot(&manager).await.unwrap();
    assert!(std::ptr::eq(
        snapshot.context().unwrap(),
        shared.context().unwrap(),
    ));
    assert!(std::ptr::eq(snapshot.execution(), shared.execution()));
    let matches = shared
        .project_context(
            NonZeroUsize::new(std::mem::size_of::<bool>()).unwrap(),
            |context, _| Ok(context.notes[0].message == "Original operator note"),
        )
        .unwrap();
    assert!(*matches);
    drop(matches);
    assert_eq!(manager.status().accepted, accepted);
    let mut report = shared.read_file(&original.report).await.unwrap();
    assert!(
        report
            .next_chunk(NonZeroUsize::new(1).unwrap())
            .await
            .unwrap()
            .is_some()
    );
    drop(report);
    let mut foreign = original.report.clone();
    foreign.sha256[0] ^= 1;
    assert!(shared.read_file(&foreign).await.is_err());
    drop(shared);
    drop(snapshot);
    assert!(
        dokime_app::runtime::ExecutionView::open_checkpoint(
            &manager,
            before_seal,
            &checkpoint,
            limit,
        )
        .await
        .unwrap()
        .is_none()
    );

    // Shape-Valid Cut Substitution And Unsealed Revisions Are Refused
    let mut wrong_cut = (*checkpoint).clone();
    wrong_cut.evidence_through.sequence = 0;
    wrong_cut.validate().unwrap();
    assert!(
        dokime_app::runtime::ExecutionView::open_checkpoint(
            &manager,
            view.clone(),
            &wrong_cut,
            limit,
        )
        .await
        .is_err()
    );
    let unsealed = dokime::execution::saved::ExecutionCheckpoint {
        execution_id: original.source.execution.execution_id.clone(),
        position: original.source.execution.position.clone(),
        version: original.source.execution.version.clone(),
        evidence_through: original.source.evidence_through.clone(),
    };
    assert!(!matches!(
        dokime_app::runtime::ExecutionView::open_checkpoint(
            &manager,
            view.clone(),
            &unsealed,
            limit,
        )
        .await,
        Ok(Some(_))
    ));
    drop(reopened);
    drop(checkpoint);
    drop(inspection);
    assert_eq!(restored.value.saved, Some((*original).clone()));
    drop(restored);
    drop(view);

    let view = manager
        .view_with_limits(crate::support::budgets::story_settings().report)
        .await
        .unwrap();
    let unchanged = execution::read_in(
        &manager,
        &view,
        &sealed_execution.test_execution_id,
        ObjectVersion::Latest,
        limit,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(unchanged.value, sealed_execution);
    assert_eq!(unchanged.position, receipt.position);
    drop(unchanged);
    drop(view);
    runtime.stop().await.unwrap();
}

pub(crate) async fn open_report_any_state() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let execution = cycle_snapshot(&runtime, &id).await.test_executions[0]
        .test_execution_id
        .clone();

    // Running: Both Reports Render From Current Evidence And Seal Nothing
    let running = runtime.open_cycle_report(&id).await.unwrap();
    assert_eq!(running.origin, dokime_app::ReportOrigin::Current);
    let payload = crate::support::reports::parse(std::str::from_utf8(&running.html).unwrap());
    assert_eq!(
        payload["test_executions"][0]["test_execution_id"],
        execution.as_str()
    );
    let running = runtime.open_execution_report(&execution).await.unwrap();
    assert_eq!(running.origin, dokime_app::ReportOrigin::Current);
    assert_eq!(running.test_execution_id.as_ref(), Some(&execution));
    assert!(cycle_snapshot(&runtime, &id).await.saved.is_none());

    // Sealed: The Exact Bytes Finalization Published
    executor.release();
    let cycle = wait_for_cycle_lifecycle(&runtime, &id, CycleStatus::Completed).await;
    let seal = cycle.saved.clone().unwrap();
    let published = crate::support::reports::bytes(runtime.storage(), &seal.report).await;
    let sealed = runtime.open_cycle_report(&id).await.unwrap();
    assert_eq!(sealed.origin, dokime_app::ReportOrigin::Sealed);
    assert_eq!(*sealed.html, published);
    let sealed = runtime.open_execution_report(&execution).await.unwrap();
    assert_eq!(sealed.origin, dokime_app::ReportOrigin::Sealed);

    // Deleted From The Root: Rendered Again From Sealed Evidence, Never Rewritten
    let name = seal
        .report
        .sha256
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let removed = remove_named(root.path(), &name);
    assert!(
        removed,
        "the sealed cycle report file must exist under the root"
    );
    let regenerated = runtime.open_cycle_report(&id).await.unwrap();
    assert_eq!(regenerated.origin, dokime_app::ReportOrigin::Regenerated);
    let payload = crate::support::reports::parse(std::str::from_utf8(&regenerated.html).unwrap());
    let original = crate::support::reports::parse(std::str::from_utf8(&published).unwrap());
    assert_eq!(payload["case_runs"], original["case_runs"]);
    assert_eq!(cycle_snapshot(&runtime, &id).await.saved, Some(seal));
    owner.shutdown().await.unwrap();
}

// The same running execution's report renders three times in a row: the first
// render releases everything it held, and each carries the same evidence.
pub(crate) async fn open_report_repeatedly() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let execution = cycle_snapshot(&runtime, &id).await.test_executions[0].clone();
    let recorded = execution.steps.len();
    assert!(recorded > 0, "the running execution has step records");

    let mut renders = Vec::new();
    for attempt in 1..=3 {
        let opened = runtime
            .open_execution_report(&execution.test_execution_id)
            .await
            .unwrap_or_else(|error| panic!("render {attempt} of the running report: {error}"));
        assert_eq!(opened.origin, dokime_app::ReportOrigin::Current);
        let payload = crate::support::reports::parse(std::str::from_utf8(&opened.html).unwrap());
        drop(opened);
        let steps = payload["test_steps"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| {
                row["record_type"] == "step"
                    && row["test_execution_id"] == execution.test_execution_id.as_str()
            })
            .count();
        assert_eq!(
            steps, recorded,
            "render {attempt} carries every step the live execution holds"
        );
        renders.push(stable(payload));
    }
    for attempt in 0..3 {
        runtime
            .open_cycle_report(&id)
            .await
            .unwrap_or_else(|error| panic!("cycle render {attempt}: {error}"));
    }
    assert_eq!(renders[0]["case_runs"], renders[2]["case_runs"]);
    assert_eq!(renders[0]["test_steps"], renders[2]["test_steps"]);
    assert_eq!(renders[0]["warnings"], renders[2]["warnings"]);
    assert!(cycle_snapshot(&runtime, &id).await.saved.is_none());

    executor.release();
    wait_for_cycle_lifecycle(&runtime, &id, CycleStatus::Completed).await;
    owner.shutdown().await.unwrap();
}

/// A payload without the times each rendering stamps on itself.
fn stable(mut value: serde_json::Value) -> serde_json::Value {
    match &mut value {
        serde_json::Value::Object(fields) => {
            for key in [
                "generated_at",
                "started_at",
                "finished_at",
                "updated_at",
                "captured_at",
            ] {
                fields.remove(key);
            }
            for field in fields.values_mut() {
                *field = stable(field.take());
            }
        }
        serde_json::Value::Array(items) => {
            for item in items.iter_mut() {
                *item = stable(item.take());
            }
        }
        _ => {}
    }
    value
}

fn remove_named(directory: &std::path::Path, name: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if remove_named(&path, name) {
                return true;
            }
        } else if path.file_name().is_some_and(|file| file == name) {
            std::fs::remove_file(&path).unwrap();
            return true;
        }
    }
    false
}

pub(crate) async fn cycle_report_reboot() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let waiting = tokio::spawn({
        let runtime = runtime.clone();
        let id = id.clone();
        async move { wait_for_cycle_lifecycle(&runtime, &id, CycleStatus::Completed).await }
    });
    tokio::task::yield_now().await;
    waiting.abort();
    assert!(waiting.await.unwrap_err().is_cancelled());
    executor.release();
    let cycle = wait_for_cycle_lifecycle(&runtime, &id, CycleStatus::Completed).await;
    let original = cycle.saved.as_ref().unwrap().clone();
    assert!(original.report.metadata.expected_length.is_none());
    assert!(original.report.metadata.expected_sha256.is_none());
    assert_eq!(original.executions.len(), 1);
    assert!(cycle.test_executions[0].saved.is_some());
    let original_bytes = crate::support::reports::bytes(runtime.storage(), &original.report).await;
    let payload = crate::support::reports::parse(std::str::from_utf8(&original_bytes).unwrap());
    let document = &payload["documents"]["default"];
    assert_eq!(document["selection"]["scope"]["test_cycle_id"], id.as_str());
    assert!(document["selection"]["scope"]["test_execution_id"].is_null());
    assert_eq!(
        document["definition"]["layout"]["nodes"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(document["cards"].as_array().unwrap().len(), 3);
    assert_eq!(payload["test_executions"].as_array().unwrap().len(), 1);
    assert_eq!(
        payload["test_executions"][0]["test_execution_id"],
        cycle.test_executions[0].test_execution_id.as_str()
    );
    let reread = cycle_snapshot(&runtime, &id).await;
    assert_eq!(reread.saved.as_ref(), Some(&original));
    wait_for_record_type(&runtime, "provider.execution.completed").await;

    // Snapshot the durable record set from the running runtime's journaled state.
    let before: Vec<(dokime::core::RecordId, String)> = runtime
        .evidence_snapshot()
        .await
        .unwrap()
        .records
        .iter()
        .map(|record| {
            (
                record.record_id.clone(),
                record.record_type.as_str().to_owned(),
            )
        })
        .collect();
    assert!(
        before
            .iter()
            .any(|(_, record_type)| record_type == "provider.execution.completed"),
        "the completed cycle emitted its terminal event before reboot",
    );

    let checkpoint =
        crate::support::native::checkpoint(&runtime, &cycle.test_executions[0].test_execution_id)
            .await;
    let created = cycle.created_at.clone();
    assert_wall_clock(&created);
    drop(runtime);
    owner.shutdown().await.unwrap();

    let (mut rebooted, handle) = booted(root.path()).await;
    let accepted = handle.storage().status().accepted;
    let downloaded = crate::support::reports::bytes(handle.storage(), &original.report).await;
    assert_eq!(
        downloaded, original_bytes,
        "a reboot serves identical sealed HTML bytes"
    );
    assert_eq!(
        crate::support::reports::parse(&String::from_utf8(downloaded).unwrap()),
        payload
    );
    assert_eq!(handle.storage().status().accepted, accepted);

    assert!(
        handle
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .cycles
            .is_empty()
    );
    let query = harmos::storage::RecordQuery::new(
        dokime::storage::records::schema(),
        |bytes, allowance| {
            dokime::storage::records::RecordRow::from_bytes(bytes, allowance.get()).map_err(
                |source| {
                    harmos::Error::backend(
                        "read saved records",
                        harmos::BackendFailure::Corrupt,
                        source,
                    )
                },
            )
        },
    );
    let mut reader = handle.storage().query(query).await.unwrap();
    let mut after = Vec::new();
    while let Some(page) = reader.next_page().await.unwrap() {
        for row in page.iter() {
            let entry = row
                .payload
                .clone()
                .into_entry(std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap())
                .unwrap();
            after.push((entry.record_id.clone(), entry.record_type.to_string()));
        }
    }
    for record in &before {
        assert!(
            after.contains(record),
            "event {record:?} was lost or changed identity across a reboot (after: {after:?})",
        );
    }
    // The stored presentation carries the cycle's captured wall clock, without
    // constructing a live cycle to read it from.
    assert_eq!(
        handle
            .execution_checkpoint_context(checkpoint, |context, _| Ok(context
                .cycle
                .cycle
                .created_at))
            .await
            .unwrap()
            .as_str(),
        created.as_str(),
        "the checkpoint preserves the original wall-clock created_at",
    );
    let view = handle
        .storage()
        .view_with_limits(crate::support::budgets::story_settings().report)
        .await
        .unwrap();
    let stored = dokime::storage::objects::cycle::read_in(
        handle.storage(),
        &view,
        &id,
        harmos::storage::ObjectVersion::Latest,
        view.decoded_bytes().unwrap(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(stored.value.saved.as_ref(), Some(&original));
    assert_eq!(
        crate::support::reports::payload(handle.storage(), &original.report).await,
        payload,
        "reboot reads the original verified report without rendering another revision",
    );
    drop(stored);
    drop(view);
    drop(handle);
    rebooted.shutdown().await.unwrap();
}
