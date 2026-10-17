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
use sha2::{Digest, Sha256};

// Local Imports
use super::*;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};
use dokime_runtime::domains::execution::requests::RunPhase;
use dokime_runtime::execution::{CycleView, TestExecution};
use dokime_contract::operations::content as wire;

/* === Constants === */
/// Why the first runtime refuses every final execution report.
const UNENCODABLE: &str = "fixture report cannot be encoded";

/// How a permanent execution finalization failure opens its diagnosis.
const UNSEALED: &str = "final execution report was not sealed";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Runs every case passed, then refuses to materialize its final report.
struct UnencodableReport(FakePythonExecutor);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionFixture for UnencodableReport {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        self.0.run_cycle(context, cycle, target)
    }

    fn refuse_final_report(&self, _: &TestExecution) -> Option<String> {
        Some(UNENCODABLE.to_owned())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn execution_save_sealing() {
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle_id, executor) = blocked_cycle(root.path()).await;
    let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Running).await;
    let id = cycle.test_executions[0].test_execution_id.clone();
    handle.storage().flush().await.unwrap();
    let running = dokime_runtime::storage::objects::execution::read(
        handle.storage(),
        &id,
        harmos::storage::ObjectVersion::Latest,
    )
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
    let large_target = handle
        .execution_file_target(&id, large_path.to_owned(), false, true)
        .await
        .unwrap()
        .unwrap();
    std::fs::write(&large_target.absolute_path, &large_bytes).unwrap();
    let mut large_reader = handle
        .open_execution_file_stream(&id, large_path.to_owned())
        .await
        .unwrap()
        .unwrap();
    let large_maximum = std::num::NonZeroUsize::new(128 * 1024).unwrap();
    let content = large_reader
        .next_chunk(large_maximum)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(content.offset, 0);
    assert_eq!(content.bytes, large_bytes);
    drop(content);
    assert!(
        large_reader
            .next_chunk(large_maximum)
            .await
            .unwrap()
            .is_none()
    );
    drop(large_reader);

    // Resume The Original Live File After Its Viewer Disconnects.
    let path = "follow.bin";
    let target = handle
        .execution_file_target(&id, path.to_owned(), false, true)
        .await
        .unwrap()
        .unwrap();
    std::fs::write(&target.absolute_path, b"prefix").unwrap();
    let maximum = std::num::NonZeroUsize::new(3).unwrap();
    let mut follower = handle
        .follow_execution_file(&id, path, None, 0)
        .await
        .unwrap()
        .unwrap();
    let identity = follower.metadata().identity.clone().unwrap();
    let first = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &first.payload else {
        panic!("live file starts with its retained prefix");
    };
    assert_eq!(chunk.offset, 0);
    assert_eq!(chunk.data, b"pre");
    drop(first);
    drop(follower);
    let expected = b"prefix-appended";
    {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&target.absolute_path)
            .unwrap();
        std::io::Write::write_all(&mut file, b"-appended").unwrap();
    }
    let mut other = identity.clone();
    other.relative_path = "other.bin".to_owned();
    assert!(
        handle
            .follow_execution_file(&id, path, Some(&other), 3)
            .await
            .is_err()
    );
    let mut follower = handle
        .follow_execution_file(&id, path, Some(&identity), 3)
        .await
        .unwrap()
        .unwrap();
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
        let target = handle
            .execution_file_target(&id, format!("outputs/{index}.bin"), false, true)
            .await
            .unwrap()
            .unwrap();
        std::fs::write(&target.absolute_path, index.to_le_bytes()).unwrap();
    }
    executor.release();
    let completed = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Completed).await;
    let frame = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else {
        panic!("the original production closes with a sealed reference");
    };
    assert_eq!(terminal.phase, wire::ExecutionFilePhase::Finished);
    let sealed = terminal.sealed.as_ref().unwrap();
    assert_eq!(sealed.identity.as_ref(), Some(&identity));
    assert_eq!(sealed.length, expected.len() as u64);
    assert_eq!(
        sealed.sha256,
        Sha256::digest(expected)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
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
    let mut large_reader = handle
        .open_execution_file_stream(&id, large_path.to_owned())
        .await
        .unwrap()
        .unwrap();
    let content = large_reader
        .next_chunk(large_maximum)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(content.offset, 0);
    assert_eq!(content.bytes, large_bytes);
    drop(content);
    assert!(
        large_reader
            .next_chunk(large_maximum)
            .await
            .unwrap()
            .is_none()
    );
    drop(large_reader);
    assert!(
        handle
            .cycles()
            .await
            .iter()
            .all(|cycle| cycle.test_cycle_id != cycle_id),
        "sealed cycle stays in saved history"
    );
    let saved_cycle = dokime_runtime::storage::objects::cycle::read(
        handle.storage(),
        &cycle_id,
        harmos::storage::ObjectVersion::Latest,
    )
    .await
    .unwrap()
    .unwrap();
    let seal = saved_cycle.value.saved.as_ref().unwrap();
    seal.validate(&saved_cycle.value).unwrap();
    assert_eq!(seal.report, cycle_report);
    let bytes = dokime_runtime::storage::read_file_bytes(handle.storage(), &seal.report)
        .await
        .unwrap();
    assert!(String::from_utf8(bytes).unwrap().contains(id.as_str()));
    drop(saved_cycle);
    let restored = crate::service_support::native::open(&handle, &checkpoint).await;
    assert_eq!(restored.execution().snapshot, snapshot);
    assert_eq!(crate::service_support::native::report(&handle, &id).await, html);
    drop(restored);
    let follower = handle
        .follow_execution_file(&id, path, Some(&identity), expected.len() as u64)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(follower.metadata().identity.as_ref(), Some(&identity));
    assert_eq!(
        follower.metadata().phase,
        wire::ExecutionFilePhase::Finished
    );
    drop(follower);
    // Every Registered Output Remains Addressable After Reopening Storage.
    for index in 0..additional_outputs {
        let mut output = handle
            .follow_execution_file(&id, &format!("outputs/{index}.bin"), None, 0)
            .await
            .unwrap()
            .unwrap();
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
    std::fs::write(
        root.path().join("brand.toml"),
        "[app]\nlabel = \"Test\"\nslug = \"test\"\nidentifier = \"com.test.hub\"\n",
    )
    .unwrap();
    let mut brand = dokime_runtime::domains::installed_plugins::load_brand(root.path()).unwrap();
    brand.customization.display_name = Some("x".repeat(4 * 1024 * 1024));
    handle.set_brand(Some(brand));
    let id = cycle_snapshot(&handle, &cycle_id).await.test_executions[0]
        .test_execution_id
        .clone();
    executor.release();
    until_state(&handle, |state| {
        state.cycle_views().any(|cycle| {
            cycle.test_cycle_id == cycle_id && cycle.test_executions[0].finished_at.is_some()
        })
    })
    .await;
    assert!(handle.execution_saved_checkpoint(id.clone()).await.is_err());
    handle.set_brand(None);
    let checkpoint = crate::service_support::native::checkpoint(&handle, &id).await;
    assert_eq!(
        crate::service_support::native::open(&handle, &checkpoint)
            .await
            .execution()
            .status,
        ExecutionStatus::Passed
    );
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A permanent finalization failure settles the run Failed with its cause and leaves the
// recorded test outcome unchanged; a rerun on a runtime that can encode its report seals.
#[tokio::test(flavor = "multi_thread")]
async fn permanent_finalization_settles() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = started_with_fixture(
        config(root.path()),
        Arc::new(UnencodableReport(FakePythonExecutor::fixed("passed"))),
    )
    .await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone(), false).await;
    let mut watch = runtime.watch_state();
    let receipt = runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");

    // The report can never be encoded. The watch stream settles the run
    // instead of leaving it running forever.
    let terminal = watch
        .wait_until(|snapshot| {
            snapshot
                .state()
                .run_requests
                .get(&receipt.command_id)
                .is_some_and(|request| request.summary.phase.terminal())
        })
        .await
        .expect("the state watch stays open");
    let cycle = terminal.state().cycle_view(&test_cycle_id).unwrap();
    assert!(
        cycle.saved.is_none(),
        "an unproduced report is never sealed"
    );
    assert_eq!(cycle.lifecycle, CycleStatus::Completed);
    let failed_id = cycle.test_executions[0].test_execution_id.clone();
    let test_case_id = cycle.test_executions[0].test_case_id.clone();
    assert_eq!(cycle.test_executions[0].status, ExecutionStatus::Passed);
    assert!(cycle.test_executions[0].saved.is_none());
    let diagnosis = terminal
        .state()
        .run_requests
        .get(&receipt.command_id)
        .and_then(|request| request.publication_failure.clone());
    assert!(
        diagnosis
            .as_deref()
            .is_some_and(|cause| cause.starts_with(UNSEALED)),
        "the watch stream carries the terminal diagnosis: {diagnosis:?}"
    );
    assert!(
        diagnosis
            .as_deref()
            .is_some_and(|cause| cause.contains(UNENCODABLE)),
        "the diagnosis names the injected cause: {diagnosis:?}"
    );
    drop(terminal);
    drop(runtime);
    owner.shutdown().await.unwrap();

    // Restart on the same store with a fixture whose reports encode. The
    // publication failure stays terminal and its cause durable: in the receipt,
    // and as evidence scoped to the execution and to the cycle.
    let (mut owner, runtime) = started_with_fixture(
        config(root.path()),
        Arc::new(FakePythonExecutor::fixed("passed")),
    )
    .await;
    register_python_suite(&runtime, suite).await;
    assert_eq!(
        cycle_snapshot(&runtime, &test_cycle_id).await.lifecycle,
        CycleStatus::Completed
    );
    let receipt = runtime
        .run_request(&receipt.command_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(receipt.summary.phase, RunPhase::Failed);
    assert_eq!(receipt.summary.failed, 0);
    assert!(
        receipt
            .interruption
            .as_deref()
            .is_some_and(|cause| cause.starts_with(UNSEALED)),
        "the receipt reports the unsealed cause: {:?}",
        receipt.interruption
    );
    let evidence = runtime.evidence_snapshot().await.unwrap();
    for (record_type, scoped) in [
        ("test.execution.finalization.failed", Some(&failed_id)),
        ("test.cycle.finalization.failed", None),
    ] {
        let record = evidence
            .records
            .iter()
            .find(|record| record.record_type.as_str() == record_type)
            .unwrap_or_else(|| panic!("{record_type} is recorded as evidence"));
        assert_eq!(record.severity, RecordSeverity::Error);
        assert_eq!(
            record
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref()),
            scoped,
            "{record_type} scope"
        );
        assert!(
            record
                .message
                .as_deref()
                .is_some_and(|message| message.starts_with(UNSEALED)),
            "{record_type} names the permanent cause: {record:?}"
        );
    }

    // A rerun runs and seals the whole cycle, the failed attempt included.
    runtime
        .rerun_cycle_case(
            test_cycle_id.clone(),
            test_case_id.clone(),
            CommandContext::default(),
        )
        .await
        .expect("an unsealed finalization leaves the case restartable");
    runtime
        .run_cycle_case(
            test_cycle_id.clone(),
            test_case_id,
            CommandContext::default(),
        )
        .await;
    until_state(&runtime, |state| {
        state.cycle_views().any(|cycle| {
            cycle.test_cycle_id == test_cycle_id
                && cycle.test_executions.len() == 2
                && cycle.saved.is_some()
        })
    })
    .await;
    let rerun = cycle_snapshot(&runtime, &test_cycle_id).await;
    let original = &rerun.test_executions[0];
    assert_eq!(original.test_execution_id, failed_id);
    assert_eq!(original.status, ExecutionStatus::Passed);
    assert!(
        original.saved.is_some(),
        "the failed attempt seals with the cycle"
    );
    let attempt = &rerun.test_executions[1];
    assert_eq!(attempt.status, ExecutionStatus::Passed);
    assert!(attempt.saved.is_some(), "the rerun attempt is sealed");
    drop(runtime);
    owner.shutdown().await.unwrap();
}
