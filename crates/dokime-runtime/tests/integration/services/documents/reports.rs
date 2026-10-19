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
use dokime_domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn whole() -> harmos::storage::ByteRange {
    harmos::storage::ByteRange { offset: 0, limit: None }
}

/* === Tests === */

#[tokio::test]
async fn report_save_receipt() {
    use dokime_domain::model::execution::saved::ExecutionRevision;
    use dokime_runtime::storage::{
        Backend,
        objects::execution::{self, Execution},
    };
    use harmos::storage::{ObjectVersion, StorageBackends};
    use std::sync::Arc;
    use {
        dokime_domain::model::execution::result::CycleView, dokime_runtime::report::CycleReportInput,
        dokime_runtime::report::saved::CapturedExecutionReportInput,
    };
    use {
        dokime_domain::model::execution::result::TestExecution, dokime_domain::model::execution::snapshot::ExecutionSnapshot,
        dokime_runtime::NodeId, dokime_runtime::definition::TestCase, dokime_runtime::storage::Manager,
    };

    let reads = crate::service_support::budgets::story_settings();
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(Backend::primary(root.path().join("primary")).await.unwrap());
    let settings = reads;
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(primary.as_ref()).clone(),
        dokime_runtime::storage::state_settings(),
    ));
    let manager = Manager::new(StorageBackends { primary, state: recovery }, settings).unwrap();
    let runtime = crate::service_support::reports::storage_runtime(&manager).await;

    // Use An Actual Admitted Execution Revision
    let finished_at = Timestamp::new("2026-10-06T06:00:00Z");
    let cycle_id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    let case = TestCase {
        suite_id: "fixture.registration".into(),
        test_case_id: dokime_runtime::model::identity::new_id::<dokime_runtime::CaseId>(),
        test_case_template_id: dokime_runtime::model::identity::new_id::<TemplateId>(),
        metadata: metadata("fixture.case", "Stored Report Case"),
        suite: None,
        parameters: BTreeMap::new(),
        generation_index: 0,
        label_number: 0,
    };
    let execution_id = dokime_runtime::model::identity::new_id::<TestExecutionId>();
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
        snapshot: ExecutionSnapshot::materialize(&case, None, Default::default(), None, Default::default()).unwrap(),
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
        session_id: dokime_runtime::model::identity::new_id::<dokime_runtime::SessionId>(),
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
        appearance: Some(dokime_runtime::report::value::Value::try_from(serde_json::json!({"app_name": "Later Renderer Brand"})).unwrap()),
        generated_at: finished_at.clone(),
    };
    let row = Execution::try_from(&original_execution).unwrap();
    let receipt = manager.write_object(&row).await.unwrap();
    let mut grouping = input.cycle.clone();
    grouping.test_executions.clear();
    let context = dokime_runtime::execution::context::ExecutionContext {
        scope: original_execution.scope.clone(),
        session_id: input.session_id.clone(),
        cycle: dokime_runtime::execution::context::ExecutionCycleContext {
            cycle: grouping,
            cases: vec![original_execution.snapshot.case().clone()],
            execution_order: vec![original_execution.test_execution_id.clone()],
        },
        execution_documents: BTreeMap::new(),
        prompts: Vec::new(),
        health: vec![model::execution::HealthItem {
            health_item_id: dokime_runtime::HealthItemId::from_seed("captured-health"),
            scope: original_execution.scope.clone(),
            severity: model::execution::HealthSeverity::Warning,
            message: "Captured Warning".into(),
            detail: Some("Original observation".into()),
            observed_at: finished_at.clone(),
        }],
        notes: vec![model::execution::OperatorNote {
            note_id: dokime_runtime::NoteId::from_seed("captured-note"),
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
        execution_id: original_execution.test_execution_id.clone(),
        lifecycle_id: "00000000-0000-4000-8000-000000000002".into(),
        execution: Some(ExecutionRevision {
            execution_id: original_execution.test_execution_id.clone(),
            position: receipt.position.clone(),
            version: receipt.version,
        }),
        evidence_through: receipt.position,
        telemetry_contracts: BTreeMap::new(),
        record_contracts: BTreeMap::new(),
        input,
        context,
    };
    let before = manager.status().accepted;
    let captured = Arc::new(captured);
    let revision = captured.execution.as_ref().unwrap().clone();
    let outputs = Vec::new();
    let mut prepared = dokime_runtime::report::Renderer.execution(captured.clone(), outputs, &manager.cursor_saved()).unwrap();
    assert_eq!(manager.status().accepted, before);
    let sealed_at = finished_at;
    let original = prepared.save_dependencies(&manager, &runtime.operations, &sealed_at, &revision).await.unwrap();
    let saved = manager.status().saved;
    let repeated = prepared.save_dependencies(&manager, &runtime.operations, &sealed_at, &revision).await.unwrap();
    assert_eq!(original, repeated);
    assert_eq!(manager.status().saved, saved);
    assert!(prepared.save_dependencies(&manager, &runtime.operations, &Timestamp::new("2030-01-01T00:00:00Z"), &revision).await.is_err());
    assert_eq!(manager.status().saved, saved);

    // Read The Original Native Captured File
    let bytes = dokime_runtime::storage::read_file_bytes(&manager, &original.source.context).await.unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(bytes.len() as u64, original.source.context.length);
    assert_eq!(sha2::Sha256::digest(&bytes).as_slice(), original.source.context.sha256.as_slice());

    // Flush The Seal's Own Original Receipt
    let mut sealed_execution = original_execution;
    sealed_execution.saved = Some(original.clone());
    let sealed = Execution::try_from(&sealed_execution).unwrap();
    let receipt = manager.write_object(&sealed).await.unwrap();
    execution::flush_seal(&manager, &sealed_execution, &receipt).await.unwrap();
    let restored = execution::read(&manager, &sealed_execution.test_execution_id, ObjectVersion::Exact(receipt.version.clone()))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(restored.position, receipt.position);
    assert_eq!(restored.version, receipt.version);
    // Download The Sealed Bytes Without Rendering Or Admitting New Evidence.
    let accepted = manager.status().accepted;
    let original_bytes = crate::service_support::reports::bytes(&manager, &original.report).await;
    let html = String::from_utf8(original_bytes.clone()).unwrap();
    let payload = crate::service_support::reports::parse(&html);
    assert_eq!(payload["appearance"]["app_name"], "Frozen Fixture");
    assert_eq!(payload["snapshot"]["notes"][0]["text"], "Original operator note");
    assert_eq!(payload["snapshot"]["health"][0]["title"], "Captured Warning");
    assert_eq!(payload["snapshot"]["health"][0]["level"], "warning");
    assert_eq!(payload["snapshot"]["notes"][0]["scope"]["test_execution_id"], sealed_execution.test_execution_id.as_str());
    assert!(!html.contains("Later Renderer Brand"));
    assert_eq!(payload["test_executions"].as_array().unwrap().len(), 1);
    assert_eq!(payload["test_executions"][0]["test_execution_id"], sealed_execution.test_execution_id.as_str());
    assert_eq!(crate::service_support::reports::bytes(&manager, &original.report).await, original_bytes);
    assert_eq!(manager.status().accepted, accepted);

    // Inspect The Original Seal And Preserve Its Earlier Evidence Cut
    let inspection = dokime_runtime::runtime::ExecutionView::open(
        &manager,
        &sealed_execution.test_execution_id,
        ObjectVersion::Exact(receipt.version.clone()),
    )
    .await
    .unwrap()
    .unwrap();
    let checkpoint = inspection.checkpoint().unwrap();
    assert_eq!(checkpoint.position, receipt.position);
    assert_eq!(checkpoint.version, receipt.version);
    assert_eq!(checkpoint.evidence_through, original.source.evidence_through);
    assert_ne!(checkpoint.position, checkpoint.evidence_through);
    let reopened = dokime_runtime::runtime::ExecutionView::open_checkpoint(&manager, &checkpoint).await.unwrap().unwrap();
    assert_eq!(reopened.execution(), &sealed_execution);
    // Reopened Snapshots Keep The Exact Captured Context.
    assert!(reopened.context().is_err());
    let accepted = manager.status().accepted;
    let snapshot = reopened.snapshot().await.unwrap();
    assert_eq!(snapshot.context().unwrap(), &captured.context);
    assert_eq!(snapshot.captured_at().unwrap(), &captured.input.generated_at);
    assert_eq!(snapshot.execution(), &sealed_execution);
    assert_eq!(snapshot.checkpoint().unwrap(), checkpoint);
    let shared = snapshot.snapshot().await.unwrap();
    assert!(std::ptr::eq(snapshot.context().unwrap(), shared.context().unwrap(),));
    assert!(std::ptr::eq(snapshot.execution(), shared.execution()));
    assert_eq!(shared.context().unwrap().notes[0].message, "Original operator note");
    assert_eq!(manager.status().accepted, accepted);
    let mut report = shared.read_file(&original.report, whole()).await.unwrap();
    assert!(report.next().await.unwrap().is_some());
    drop(report);
    let mut foreign = original.report.clone();
    foreign.sha256[0] ^= 1;
    assert!(shared.read_file(&foreign, whole()).await.is_err());
    drop(shared);
    drop(snapshot);

    // Shape-Valid Cut Substitution And Unsealed Revisions Are Refused
    let mut wrong_cut = checkpoint.clone();
    wrong_cut.evidence_through.sequence = 0;
    wrong_cut.validate().unwrap();
    assert!(dokime_runtime::runtime::ExecutionView::open_checkpoint(&manager, &wrong_cut).await.is_err());
    let unsealed = dokime_domain::model::execution::saved::ExecutionCheckpoint {
        execution_id: original.source.execution.execution_id.clone(),
        position: original.source.execution.position.clone(),
        version: original.source.execution.version.clone(),
        evidence_through: original.source.evidence_through.clone(),
    };
    assert!(!matches!(dokime_runtime::runtime::ExecutionView::open_checkpoint(&manager, &unsealed).await, Ok(Some(_))));
    drop(reopened);
    drop(checkpoint);
    drop(inspection);
    assert_eq!(restored.value.saved, Some(original.clone()));
    drop(restored);

    let unchanged = execution::read(&manager, &sealed_execution.test_execution_id, ObjectVersion::Latest).await.unwrap().unwrap();
    assert_eq!(unchanged.value, sealed_execution);
    assert_eq!(unchanged.position, receipt.position);
    drop(unchanged);
    runtime.stop().await.unwrap();
}

#[tokio::test]
async fn report_opens_anytime() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite, false).await;
    runtime.run_cycle_cases(id.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let execution = cycle_snapshot(&runtime, &id).await.test_executions[0].test_execution_id.clone();

    // Running: Both Reports Render From Current Evidence And Seal Nothing
    let running = runtime.open_cycle_report(&id).await.unwrap();
    assert_eq!(running.origin, dokime_runtime::ReportOrigin::Current);
    let payload = crate::service_support::reports::parse(std::str::from_utf8(&running.html).unwrap());
    assert_eq!(payload["test_executions"][0]["test_execution_id"], execution.as_str());
    let running = runtime.open_execution_report(&execution).await.unwrap();
    assert_eq!(running.origin, dokime_runtime::ReportOrigin::Current);
    assert_eq!(running.test_execution_id.as_ref(), Some(&execution));
    assert!(cycle_snapshot(&runtime, &id).await.saved.is_none());

    // Sealed: The Exact Bytes Finalization Published
    executor.release();
    let cycle = wait_for_cycle_lifecycle(&runtime, &id, CycleStatus::Completed).await;
    let seal = cycle.saved.clone().unwrap();
    let published = crate::service_support::reports::bytes(runtime.storage(), &seal.report).await;
    let sealed = runtime.open_cycle_report(&id).await.unwrap();
    assert_eq!(sealed.origin, dokime_runtime::ReportOrigin::Sealed);
    assert_eq!(*sealed.html, published);
    let sealed = runtime.open_execution_report(&execution).await.unwrap();
    assert_eq!(sealed.origin, dokime_runtime::ReportOrigin::Sealed);

    // Missing Published Bytes Are An Error; A Seal Is Never Regenerated
    let name = seal.report.sha256.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let removed = remove_named(root.path(), &name);
    assert!(removed, "the sealed cycle report file must exist under the root");
    assert!(runtime.open_cycle_report(&id).await.is_err());
    assert_eq!(cycle_snapshot(&runtime, &id).await.saved, Some(seal));
    owner.shutdown().await.unwrap();
}

// A download always serves the report: a member whose final report is sealed
// while its cycle still runs serves those bytes, the running cycle materializes
// its current state, and after finalization both serve the sealed artifacts.
#[tokio::test]
async fn report_download_serves() {
    let root = tempdir().unwrap();
    let executor = Arc::new(SecondGroupHoldExecutor::default());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let first = plt400_suite();
    let second = plt400_diagnostics_suite();
    register_python_suite_pair(&runtime, first.clone(), second.clone()).await;
    let id = start_cycle(&runtime, first, false).await;
    runtime
        .append_cycle_cases(id.clone(), materialize_cases("provider:testdb:plt400.diagnostics", &second), CommandContext::default())
        .await
        .unwrap();
    let receipt = runtime.run_cycle_cases(id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{:?}", receipt.rejection);
    executor.wait_until_entered().await;
    let settled = cycle_snapshot(&runtime, &id)
        .await
        .test_executions
        .into_iter()
        .find(|execution| execution.status == ExecutionStatus::Passed)
        .expect("the first group settled before the second runs")
        .test_execution_id;

    // Running: The Settled Member Seals Its Final Report While The Cycle Runs On
    runtime.finish_execution_for_test(&settled).await.unwrap();
    let durable = dokime_runtime::storage::objects::execution::read(runtime.storage(), &settled, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    let seal = durable.value.saved.clone().expect("the settled member is sealed durably");
    drop(durable);
    let published = crate::service_support::reports::bytes(runtime.storage(), &seal.report).await;
    assert!(cycle_snapshot(&runtime, &id).await.saved.is_none());
    for attempt in 1..=2 {
        let opened = runtime
            .open_execution_report(&settled)
            .await
            .unwrap_or_else(|error| panic!("download {attempt} of the sealed member: {error}"));
        assert_eq!(opened.origin, dokime_runtime::ReportOrigin::Sealed);
        assert_eq!(opened.html, published, "download {attempt} serves the final report bytes");
    }
    let running = runtime.open_cycle_report(&id).await.unwrap();
    assert_eq!(running.origin, dokime_runtime::ReportOrigin::Current);
    let payload = crate::service_support::reports::parse(std::str::from_utf8(&running.html).unwrap());
    assert_eq!(payload["test_executions"].as_array().unwrap().len(), 2);

    // Finalized: Both Downloads Serve The Sealed Artifacts
    executor.release();
    let cycle = wait_for_cycle_lifecycle(&runtime, &id, CycleStatus::Completed).await;
    let cycle_seal = cycle.saved.clone().expect("the completed cycle is sealed");
    let sealed = runtime.open_cycle_report(&id).await.unwrap();
    assert_eq!(sealed.origin, dokime_runtime::ReportOrigin::Sealed);
    assert_eq!(sealed.html, crate::service_support::reports::bytes(runtime.storage(), &cycle_seal.report).await);
    let sealed = runtime.open_execution_report(&settled).await.unwrap();
    assert_eq!(sealed.origin, dokime_runtime::ReportOrigin::Sealed);
    assert_eq!(sealed.html, published, "finalization keeps the member's final report");
    owner.shutdown().await.unwrap();
}

// The same running execution's report renders three times in a row: the first
// render releases everything it held, and each carries the same evidence.
#[tokio::test]
async fn open_report_repeatedly() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite, false).await;
    runtime.run_cycle_cases(id.clone(), CommandContext::default()).await;
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
        assert_eq!(opened.origin, dokime_runtime::ReportOrigin::Current);
        let payload = crate::service_support::reports::parse(std::str::from_utf8(&opened.html).unwrap());
        drop(opened);
        let steps = payload["test_executions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["test_execution_id"] == execution.test_execution_id.as_str())
            .flat_map(|row| row["steps"].as_array().unwrap())
            .filter(|row| row["record_type"] == "step")
            .count();
        assert_eq!(steps, recorded, "render {attempt} carries every step the live execution holds");
        renders.push(stable(payload));
    }
    for attempt in 0..3 {
        runtime.open_cycle_report(&id).await.unwrap_or_else(|error| panic!("cycle render {attempt}: {error}"));
    }
    assert_eq!(renders[0]["case_runs"], renders[2]["case_runs"]);
    let steps = |render: &serde_json::Value| {
        render["test_executions"].as_array().unwrap().iter().map(|row| row["steps"].clone()).collect::<Vec<_>>()
    };
    assert_eq!(steps(&renders[0]), steps(&renders[2]));
    assert_eq!(renders[0]["warnings"], renders[2]["warnings"]);
    assert!(cycle_snapshot(&runtime, &id).await.saved.is_none());

    executor.release();
    wait_for_cycle_lifecycle(&runtime, &id, CycleStatus::Completed).await;
    owner.shutdown().await.unwrap();
}

fn stable(mut value: serde_json::Value) -> serde_json::Value {
    match &mut value {
        serde_json::Value::Object(fields) => {
            for key in ["generated_at", "started_at", "finished_at", "updated_at", "captured_at"] {
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

#[tokio::test]
async fn cycle_report_reboot() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite, false).await;
    runtime.run_cycle_cases(id.clone(), CommandContext::default()).await;
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
    // The Whole Report Is Rendered Before Publication, So Storage Verifies It.
    assert_eq!(original.report.metadata.expected_length, Some(original.report.length));
    assert_eq!(original.report.metadata.expected_sha256.as_ref().map(|digest| digest.as_slice()), Some(original.report.sha256.as_slice()));
    assert_eq!(original.executions.len(), 1);
    assert!(cycle.test_executions[0].saved.is_some());
    let original_bytes = crate::service_support::reports::bytes(runtime.storage(), &original.report).await;
    let payload = crate::service_support::reports::parse(std::str::from_utf8(&original_bytes).unwrap());
    let document = &payload["documents"]["default"];
    assert_eq!(document["selection"]["scope"]["test_cycle_id"], id.as_str());
    assert!(document["selection"]["scope"]["test_execution_id"].is_null());
    assert_eq!(document["definition"]["layout"]["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(document["cards"].as_array().unwrap().len(), 3);
    assert_eq!(payload["test_executions"].as_array().unwrap().len(), 1);
    assert_eq!(payload["test_executions"][0]["test_execution_id"], cycle.test_executions[0].test_execution_id.as_str());
    let reread = cycle_snapshot(&runtime, &id).await;
    assert_eq!(reread.saved.as_ref(), Some(&original));
    wait_for_record_type(&runtime, "provider.execution.completed").await;

    // Snapshot the durable record set from the running runtime's journaled state.
    let before: Vec<(dokime_runtime::RecordId, String)> = runtime
        .evidence_snapshot()
        .await
        .unwrap()
        .records
        .iter()
        .map(|record| (record.record_id.clone(), record.record_type.as_str().to_owned()))
        .collect();
    assert!(
        before.iter().any(|(_, record_type)| record_type == "provider.execution.completed"),
        "the completed cycle emitted its terminal event before reboot",
    );

    let checkpoint = crate::service_support::native::checkpoint(&runtime, &cycle.test_executions[0].test_execution_id).await;
    let created = cycle.created_at.clone();
    assert_wall_clock(&created);
    drop(runtime);
    owner.shutdown().await.unwrap();

    let (mut rebooted, handle) = booted(root.path()).await;
    let accepted = handle.storage().status().accepted;
    let downloaded = crate::service_support::reports::bytes(handle.storage(), &original.report).await;
    assert_eq!(downloaded, original_bytes, "a reboot serves identical sealed HTML bytes");
    assert_eq!(crate::service_support::reports::parse(&String::from_utf8(downloaded).unwrap()), payload);
    assert_eq!(handle.storage().status().accepted, accepted);

    assert!(handle.runtime_overview().await.expect("runtime overview query succeeds").cycles.is_empty());
    let mut reader = handle
        .storage()
        .read_records::<dokime_runtime::storage::records::RecordRow>(harmos::storage::RecordRead {
            at: handle.storage().cursor_saved(),
            selection: harmos::storage::query::RecordSelection {
                schema: dokime_runtime::storage::records::schema(),
                scope: None,
                item: None,
                source_start: None,
                source_end: None,
            },
            after: 0,
        })
        .await
        .unwrap();
    let mut after = Vec::new();
    while let Some(batch) = reader.next().await.unwrap() {
        for row in batch {
            let entry = row.payload.entry().unwrap();
            after.push((entry.record_id.clone(), entry.record_type.to_string()));
        }
    }
    for record in &before {
        assert!(after.contains(record), "event {record:?} was lost or changed identity across a reboot (after: {after:?})",);
    }
    // The stored presentation carries the cycle's captured wall clock, without
    // constructing a live cycle to read it from.
    assert_eq!(
        handle.execution_checkpoint_context(checkpoint).await.unwrap().cycle.cycle.created_at.as_str(),
        created.as_str(),
        "the checkpoint preserves the original wall-clock created_at",
    );
    let stored = dokime_runtime::storage::objects::cycle::read(handle.storage(), &id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.value.saved.as_ref(), Some(&original));
    assert_eq!(
        crate::service_support::reports::payload(handle.storage(), &original.report).await,
        payload,
        "reboot reads the original verified report without rendering another revision",
    );
    drop(stored);
    drop(handle);
    rebooted.shutdown().await.unwrap();
}
