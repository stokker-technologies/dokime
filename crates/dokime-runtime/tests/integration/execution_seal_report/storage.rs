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
use dokime_domain::model::content::wire;
use dokime_domain::model::execution::request::RunPhase;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};
use {dokime_domain::model::execution::result::CycleView, dokime_domain::model::execution::result::TestExecution};

/* === Constants === */
const UNENCODABLE: &str = "fixture report cannot be encoded";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct UnencodableReport(FakePythonExecutor, Arc<std::sync::atomic::AtomicBool>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn execution_save_sealing() {
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

pub(crate) async fn terminal_capture_repaired() {
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle_id, executor) = blocked_cycle(root.path()).await;
    let original = handle.read_application_settings().await.unwrap();
    let mut settings = original.settings.clone();
    settings.report.source_bytes = 1;
    let limited = handle
        .update_application_settings(dokime_domain::model::settings::UpdateSettings {
            expected_revision: original.revision,
            settings,
        })
        .await
        .unwrap();
    let id = cycle_snapshot(&handle, &cycle_id).await.test_executions[0].test_execution_id.clone();
    executor.release();
    until_state(&handle, |state| state.report_finalizations.get(&id).is_some_and(|intent| intent.failure.is_some())).await;
    assert!(handle.execution_saved_checkpoint(id.clone()).await.is_err());
    handle
        .update_application_settings(dokime_domain::model::settings::UpdateSettings {
            expected_revision: limited.revision,
            settings: original.settings,
        })
        .await
        .unwrap();
    handle.finish_execution_for_test(&id).await.unwrap();
    let checkpoint = crate::service_support::native::checkpoint(&handle, &id).await;
    assert_eq!(crate::service_support::native::open(&handle, &checkpoint).await.execution().status, ExecutionStatus::Passed);
    drop(handle);
    owner.shutdown().await.unwrap();
}

// A diagnostic recorded after the report lifecycle froze never changes the frozen execution, and
// a cycle that cannot be sealed at startup stays retryable instead of stopping the runtime boot.
pub(crate) async fn unsealed_cycle_survives_restart() {
    let root = tempdir().unwrap();
    let refuse = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let fixture = || Arc::new(UnencodableReport(FakePythonExecutor::fixed("passed"), refuse.clone()));
    let (mut owner, runtime) = started_with_fixture(config(root.path()), fixture()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite.clone()).await;
    let mut watch = runtime.watch_state();
    let receipt = runtime.start_next_execution(test_cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    let terminal = watch
        .wait_until(|snapshot| {
            snapshot.state().run_requests.get(&receipt.command_id).is_some_and(|request| request.summary.phase.terminal())
        })
        .await
        .expect("the state watch stays open");
    let member = terminal.state().cycle_view(&test_cycle_id).unwrap().test_executions[0].clone();
    let frozen = terminal.state().report_finalizations[&member.test_execution_id].lifecycle.clone().expect("the seal froze its lifecycle");
    drop(terminal);
    drop(watch);

    // Cycle Diagnostics Remain Writable After The Execution Closes.
    runtime
        .add_health_item(
            dokime_domain::model::execution::RecordScope::default().with_cycle(test_cycle_id.clone()),
            dokime_domain::model::execution::HealthSeverity::Error,
            "late diagnostic".into(),
            None,
        )
        .await
        .unwrap();
    drop(runtime);
    owner.shutdown().await.unwrap();
    drop(owner);

    // Still Unsealable: The Runtime Boots And Leaves The Cycle For A Later Seal.
    let (mut owner, runtime) = started_with_fixture(config(root.path()), fixture()).await;
    let reopened = runtime.watch_state().latest().state().cycle_view(&test_cycle_id).expect("the unsealed cycle stays live");
    assert!(reopened.saved.is_none());
    assert!(reopened.test_executions[0].saved.is_none());
    drop(runtime);
    owner.shutdown().await.unwrap();
    drop(owner);

    // Once Reports Encode, Startup Seals The Member From Its Original Frozen Revision.
    refuse.store(false, std::sync::atomic::Ordering::SeqCst);
    let (mut owner, runtime) = started_with_fixture(config(root.path()), fixture()).await;
    let sealed = dokime_runtime::storage::objects::cycle::read(runtime.storage(), &test_cycle_id, harmos::storage::ObjectVersion::Latest)
        .await
        .unwrap()
        .unwrap();
    assert!(sealed.value.saved.is_some(), "startup seals the retried cycle");
    let checkpoint = crate::service_support::native::checkpoint(&runtime, &member.test_execution_id).await;
    let view = crate::service_support::native::open(&runtime, &checkpoint).await;
    let seal = view.execution().saved.clone().expect("the member is sealed");
    assert_eq!(seal.source.execution, frozen.execution, "the seal reproduces the frozen lifecycle revision");
    assert_eq!(view.execution().status, member.status);
    drop((view, sealed, runtime));
    owner.shutdown().await.unwrap();
}

// Publication Failure Is Independent Of The Settled Provider Outcome.
pub(crate) async fn permanent_finalization_settles() {
    let root = tempdir().unwrap();
    let refuse = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let (mut owner, runtime) =
        started_with_fixture(config(root.path()), Arc::new(UnencodableReport(FakePythonExecutor::fixed("passed"), refuse))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle_id = start_cycle(&runtime, suite).await;
    let execution = runtime.watch_state().latest().state().cycles[&cycle_id].execution_ids[0].clone();
    let mut watch = runtime.watch_state();
    let receipt = runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    watch
        .wait_until(|snapshot| {
            snapshot.state().run_requests[&receipt.command_id].summary.phase.terminal()
                && snapshot.state().report_finalizations.get(&execution).is_some_and(|intent| intent.failure.is_some())
        })
        .await
        .unwrap();
    assert!(runtime.wait_execution_report(&execution).await.is_err());
    assert_eq!(runtime.watch_state().latest().state().executions[&execution].status, ExecutionStatus::Passed);
    runtime.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
    drop(watch);
    owner.shutdown().await.unwrap();
    drop(runtime);
    drop(owner);
    let (mut owner, runtime) = started_with_fixture(config(root.path()), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let receipt = runtime.run_request(&receipt.command_id).await.unwrap().unwrap();
    assert_eq!(receipt.summary.phase, RunPhase::Completed);
    assert!(receipt.interruption.is_none());
    let mut report = runtime.open_execution_report(&execution).await.unwrap();
    assert!(!crate::service_support::reports::opened_bytes(&mut report).await.is_empty());
    owner.shutdown().await.unwrap();
}
