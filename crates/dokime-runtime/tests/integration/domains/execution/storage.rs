/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use sha2::{Digest, Sha256};

use super::*;
use dokime_domain::model::content::wire;
use dokime_domain::model::execution::request::RunPhase;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};
use {dokime_domain::model::execution::result::CycleView, dokime_domain::model::execution::result::TestExecution};

const UNENCODABLE: &str = "fixture report cannot be encoded";

const UNSEALED: &str = "final execution report was not sealed";

struct UnencodableReport(FakePythonExecutor, Arc<std::sync::atomic::AtomicBool>);

impl ExecutionFixture for UnencodableReport {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a CycleView, target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        self.0.run_cycle(context, cycle, target)
    }

    fn refuse_final_report(&self, _: &TestExecution) -> Option<String> {
        self.1.load(std::sync::atomic::Ordering::SeqCst).then(|| UNENCODABLE.to_owned())
    }
}

#[tokio::test]
async fn execution_save_sealing() {
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle_id, executor) = blocked_cycle(root.path()).await;
    let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Running).await;
    let id = cycle.test_executions[0].test_execution_id.clone();
    handle.storage().flush().await.unwrap();
    let running = dokime_runtime::storage::objects::execution::read(handle.storage(), &id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(running.value.status, ExecutionStatus::Pending);
    assert!(running.value.saved.is_none());
    let snapshot = running.value.snapshot.clone();
    drop(running);

    // Native Reads Use The Requested Batch Size And Shared Storage Admission.
    let large_path = "large.bin";
    let large_bytes = vec![0x5a; 96 * 1024];
    let large_target = handle.execution_file_target(&id, large_path.to_owned(), false, true).await.unwrap().unwrap();
    std::fs::write(&large_target.absolute_path, &large_bytes).unwrap();
    let mut large_reader = handle.open_execution_file_stream(&id, large_path.to_owned()).await.unwrap().unwrap();
    let large_maximum = std::num::NonZeroUsize::new(128 * 1024).unwrap();
    let content = large_reader.next_chunk(large_maximum).await.unwrap().unwrap();
    assert_eq!(content.offset, 0);
    assert_eq!(content.bytes, large_bytes);
    drop(content);
    assert!(large_reader.next_chunk(large_maximum).await.unwrap().is_none());
    drop(large_reader);

    // Resume The Original Live File After Its Viewer Disconnects.
    let path = "follow.bin";
    let target = handle.execution_file_target(&id, path.to_owned(), false, true).await.unwrap().unwrap();
    std::fs::write(&target.absolute_path, b"prefix").unwrap();
    let maximum = std::num::NonZeroUsize::new(3).unwrap();
    let mut follower = handle.follow_execution_file(&id, path, None, 0).await.unwrap().unwrap();
    let identity = follower.metadata().identity.clone().unwrap();
    let first = follower.next_frame(maximum).await.unwrap().unwrap();
    drop(follower);
    let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &first.payload else {
        panic!("live file starts with its retained prefix");
    };
    assert_eq!(chunk.offset, 0);
    assert_eq!(chunk.data, b"pre");
    drop(first);
    let expected = b"prefix-appended";
    {
        let mut file = std::fs::OpenOptions::new().append(true).open(&target.absolute_path).unwrap();
        std::io::Write::write_all(&mut file, b"-appended").unwrap();
    }
    let mut other = identity.clone();
    other.relative_path = "other.bin".to_owned();
    assert!(handle.follow_execution_file(&id, path, Some(&other), 3).await.is_err());
    let mut follower = handle.follow_execution_file(&id, path, Some(&identity), 3).await.unwrap().unwrap();
    assert_eq!(follower.metadata().identity.as_ref(), Some(&identity));
    let mut retained = b"pre".to_vec();
    while retained.len() < expected.len() {
        let frame = follower.next_frame(maximum).await.unwrap().unwrap();
        let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &frame.payload else {
            panic!("live file remains producing until its original owner settles");
        };
        assert_eq!(chunk.offset, retained.len() as u64);
        retained.extend_from_slice(&chunk.data);
    }
    assert_eq!(retained, expected);
    // Exercise Registration And Sealing Beyond The Former Private Source Cap.
    let additional_outputs = 257u32;
    for index in 0..additional_outputs {
        let target = handle.execution_file_target(&id, format!("outputs/{index}.bin"), false, true).await.unwrap().unwrap();
        std::fs::write(&target.absolute_path, index.to_le_bytes()).unwrap();
    }
    executor.release();
    wait_for_cycle_outcome(&handle, &cycle_id, ResultStatus::Passed).await;
    handle.seal_cycle(cycle_id.clone(), CommandContext::default()).await.unwrap();
    let completed = cycle_snapshot(&handle, &cycle_id).await;
    let frame = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else {
        panic!("the original production closes with a sealed reference");
    };
    assert_eq!(terminal.phase, wire::ExecutionFilePhase::Finished);
    let sealed = terminal.sealed.as_ref().unwrap();
    assert_eq!(sealed.identity.as_ref(), Some(&identity));
    assert_eq!(sealed.length, expected.len() as u64);
    assert_eq!(sealed.sha256, Sha256::digest(expected).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    assert!(follower.next_frame(maximum).await.unwrap().is_none());
    drop(frame);
    drop(follower);
    let cycle_seal = completed.saved.as_ref().expect("completed cycle is sealed");
    cycle_seal.validate_view(&completed).unwrap();
    assert_eq!(cycle_seal.executions.len(), 1);
    assert_eq!(cycle_seal.executions[0].execution_id, id);
    let cycle_report = cycle_seal.report.clone();
    let checkpoint = crate::service_support::native::checkpoint(&handle, &id).await;
    let terminal = crate::service_support::native::open(&handle, &checkpoint).await;
    assert_eq!(terminal.execution().status, ExecutionStatus::Passed);
    assert_eq!(terminal.execution().snapshot, snapshot);
    assert!(terminal.execution().saved.is_some());
    let html = crate::service_support::native::report(&handle, &id).await;
    assert!(html.contains("hub-report-data"));
    assert!(html.contains(id.as_str()));
    drop(terminal);
    drop(handle);
    owner.shutdown().await.unwrap();
    let (mut reopened, handle) = booted(root.path()).await;
    let mut large_reader = handle.open_execution_file_stream(&id, large_path.to_owned()).await.unwrap().unwrap();
    let content = large_reader.next_chunk(large_maximum).await.unwrap().unwrap();
    assert_eq!(content.offset, 0);
    assert_eq!(content.bytes, large_bytes);
    drop(content);
    assert!(large_reader.next_chunk(large_maximum).await.unwrap().is_none());
    drop(large_reader);
    assert!(handle.cycles().await.iter().all(|cycle| cycle.test_cycle_id != cycle_id), "sealed cycle stays in saved history");
    let saved_cycle = dokime_runtime::storage::objects::cycle::read(handle.storage(), &cycle_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    let seal = saved_cycle.value.saved.as_ref().unwrap();
    seal.validate(&saved_cycle.value).unwrap();
    assert_eq!(seal.report, cycle_report);
    let bytes = dokime_runtime::storage::read_file_bytes(handle.storage(), &seal.report).await.unwrap();
    assert!(String::from_utf8(bytes.as_slice().to_vec()).unwrap().contains(id.as_str()));
    drop(saved_cycle);
    let restored = crate::service_support::native::open(&handle, &checkpoint).await;
    assert_eq!(restored.execution().snapshot, snapshot);
    assert_eq!(crate::service_support::native::report(&handle, &id).await, html);
    drop(restored);
    let follower = handle.follow_execution_file(&id, path, Some(&identity), expected.len() as u64).await.unwrap().unwrap();
    assert_eq!(follower.metadata().identity.as_ref(), Some(&identity));
    assert_eq!(follower.metadata().phase, wire::ExecutionFilePhase::Finished);
    drop(follower);
    // Every Registered Output Remains Addressable After Reopening Storage.
    for index in 0..additional_outputs {
        let mut output = handle.follow_execution_file(&id, &format!("outputs/{index}.bin"), None, 0).await.unwrap().unwrap();
        assert_eq!(output.metadata().phase, wire::ExecutionFilePhase::Finished);
        let frame = output.next_frame(maximum).await.unwrap().unwrap();
        let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &frame.payload else {
            panic!("saved output must return its bytes");
        };
        assert_eq!(chunk.offset, 0);
        assert_eq!(chunk.data, index.to_le_bytes()[..3]);
        drop(frame);
        let frame = output.next_frame(maximum).await.unwrap().unwrap();
        let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &frame.payload else {
            panic!("saved output must return its remaining byte");
        };
        assert_eq!(chunk.offset, 3);
        assert_eq!(chunk.data, index.to_le_bytes()[3..]);
        drop(frame);
        let frame = output.next_frame(maximum).await.unwrap().unwrap();
        let Some(wire::ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else {
            panic!("saved output must finish with its sealed reference");
        };
        assert_eq!(terminal.phase, wire::ExecutionFilePhase::Finished);
        assert_eq!(terminal.sealed.as_ref().unwrap().length, 4);
        drop(frame);
        assert!(output.next_frame(maximum).await.unwrap().is_none());
    }
    drop(handle);
    reopened.shutdown().await.unwrap();
}

#[tokio::test]
async fn terminal_capture_repaired() {
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle_id, executor) = blocked_cycle(root.path()).await;
    std::fs::write(root.path().join("organization.toml"), "[app]\nlabel = \"Stokker Technologies\"\n").unwrap();
    let mut brand = dokime_runtime::domains::installed_plugins::load_brand(root.path()).unwrap();
    brand.customization.display_name = Some("x".repeat(4 * 1024 * 1024));
    handle.set_brand(Some(brand));
    let id = cycle_snapshot(&handle, &cycle_id).await.test_executions[0].test_execution_id.clone();
    executor.release();
    until_state(&handle, |state| {
        state.cycle_views().any(|cycle| cycle.test_cycle_id == cycle_id && cycle.test_executions[0].finished_at.is_some())
    })
    .await;
    assert!(handle.execution_saved_checkpoint(id.clone()).await.is_err());
    handle.set_brand(None);
    let checkpoint = crate::service_support::native::checkpoint(&handle, &id).await;
    assert_eq!(crate::service_support::native::open(&handle, &checkpoint).await.execution().status, ExecutionStatus::Passed);
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A permanent finalization failure settles the run Failed with its cause and leaves the
// recorded test outcome unchanged; a rerun on a runtime that can encode its report seals.
#[tokio::test(flavor = "multi_thread")]
async fn permanent_finalization_settles() {
    let root = tempdir().unwrap();
    let refuse = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let (mut owner, runtime) =
        started_with_fixture(config(root.path()), Arc::new(UnencodableReport(FakePythonExecutor::fixed("passed"), refuse.clone()))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone()).await;
    let mut watch = runtime.watch_state();
    let receipt = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");

    // The report can never be encoded. The watch stream settles the run
    // instead of leaving it running forever.
    let terminal = watch
        .wait_until(|snapshot| {
            snapshot.state().run_requests.get(&receipt.command_id).is_some_and(|request| request.summary.phase.terminal())
        })
        .await
        .expect("the state watch stays open");
    let cycle = terminal.state().cycle_view(&test_cycle_id).unwrap();
    assert!(cycle.saved.is_none(), "an unproduced report is never sealed");
    assert_eq!(cycle.lifecycle, CycleStatus::Ready);
    assert_eq!(cycle.result.as_ref().map(|result| result.status.clone()), Some(ResultStatus::Passed));
    let failed_id = cycle.test_executions[0].test_execution_id.clone();
    let test_case_id = cycle.test_executions[0].test_case_id.clone();
    assert_eq!(cycle.test_executions[0].status, ExecutionStatus::Passed);
    assert!(cycle.test_executions[0].saved.is_none());
    let diagnosis = terminal.state().run_requests.get(&receipt.command_id).and_then(|request| request.publication_failure.clone());
    assert!(
        diagnosis.as_deref().is_some_and(|cause| cause.starts_with(UNSEALED)),
        "the watch stream carries the terminal diagnosis: {diagnosis:?}"
    );
    assert!(diagnosis.as_deref().is_some_and(|cause| cause.contains(UNENCODABLE)), "the diagnosis names the injected cause: {diagnosis:?}");
    drop(terminal);
    drop(watch);
    drop(runtime);
    // The stop leaves the unsealable cycle to the next start, which seals it once its report encodes.
    owner.shutdown().await.unwrap();
    drop(owner);

    // Restart on the same store with a fixture whose reports encode. The
    // publication failure stays terminal and its cause durable: in the receipt,
    // and as evidence scoped to the execution and to the cycle.
    let (mut owner, runtime) = started_with_fixture(config(root.path()), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    register_python_suite(&runtime, suite.clone()).await;
    let original_cycle =
        dokime_runtime::storage::objects::cycle::read(runtime.storage(), &test_cycle_id, harmos::storage::ObjectVersion::Latest)
            .await
            .unwrap()
            .unwrap();
    assert_eq!(original_cycle.value.lifecycle, CycleStatus::Completed);
    assert!(original_cycle.value.saved.is_some(), "publication repair is durable without replaying the provider");
    let receipt = runtime.run_request(&receipt.command_id).await.unwrap().unwrap();
    assert_eq!(receipt.summary.phase, RunPhase::Failed);
    assert_eq!(receipt.summary.failed, 0);
    assert!(
        receipt.interruption.as_deref().is_some_and(|cause| cause.starts_with(UNSEALED)),
        "the receipt reports the unsealed cause: {:?}",
        receipt.interruption
    );
    let mut reader = runtime
        .storage()
        .read_records::<dokime_runtime::storage::records::RecordRow>(harmos::storage::RecordRead {
            at: runtime.storage().cursor_saved(),
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
    let mut records = Vec::new();
    while let Some(batch) = reader.next().await.unwrap() {
        records.push(crate::service_support::native::record_page(batch).unwrap());
    }
    drop(reader);
    for (record_type, scoped) in [("test.execution.finalization.failed", Some(&failed_id)), ("test.cycle.finalization.failed", None)] {
        let record = records
            .iter()
            .flat_map(|batch| batch.iter())
            .find(|record| record.record_type.as_str() == record_type)
            .unwrap_or_else(|| panic!("{record_type} is recorded as evidence"));
        assert_eq!(record.severity, RecordSeverity::Error);
        assert_eq!(record.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()), scoped, "{record_type} scope");
        assert!(
            record.message.as_deref().is_some_and(|message| message.starts_with(UNSEALED)),
            "{record_type} names the permanent cause: {record:?}"
        );
    }

    // A New Attempt Uses A New Container And Preserves The Repaired Original.
    let checkpoint = crate::service_support::native::checkpoint(&runtime, &failed_id).await;
    let original_view = crate::service_support::native::open(&runtime, &checkpoint).await;
    let original = original_view.execution();
    assert_eq!(original.test_execution_id, failed_id);
    assert_eq!(original.status, ExecutionStatus::Passed);
    assert!(original.saved.is_some(), "publication repair preserves the original outcome");
    assert_eq!(original_cycle.value.execution_ids.len(), 1);
    let fresh_cycle = runtime
        .create_cycle(
            CycleDefinition {
                metadata: original_cycle.value.metadata.clone(),
                continue_automatically: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    runtime
        .append_executions(fresh_cycle.clone(), vec![(dokime_runtime::model::identity::new_id::<TestExecutionId>(), test_case_id.clone())])
        .await
        .unwrap();
    runtime.start_case_execution(fresh_cycle.clone(), test_case_id, CommandContext::default()).await;
    wait_for_cycle_outcome(&runtime, &fresh_cycle, ResultStatus::Passed).await;
    runtime.seal_cycle(fresh_cycle.clone(), CommandContext::default()).await.unwrap();
    let rerun = cycle_snapshot(&runtime, &fresh_cycle).await;
    assert_eq!(rerun.test_executions.len(), 1);
    let attempt = &rerun.test_executions[0];
    assert_ne!(attempt.test_execution_id, failed_id);
    assert_eq!(attempt.status, ExecutionStatus::Passed);
    assert!(attempt.saved.is_some(), "the rerun attempt is sealed");
    drop(runtime);
    owner.shutdown().await.unwrap();
}
