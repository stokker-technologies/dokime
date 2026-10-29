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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::StorageBackend;

// Local Imports
use crate::service_support::prelude::*;
use dokime_domain::model::execution::request::RunSummary;
use dokime_domain::model::identity::CommandId;
use dokime_runtime::storage::requests::Request;
use {dokime_domain::model::execution::request::RunPhase, dokime_domain::model::execution::request::RunReceipt};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn prepared_start_refusal() {
    use sha2::{Digest, Sha256};
    let root = tempdir().unwrap();
    let (_owner, handle) = booted_with_fixture(root.path(), Arc::new(FakePythonExecutor::fixed("passed"))).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let cycle_id = start_cycle(&handle, suite).await;
    let execution_id = cycle_snapshot(&handle, &cycle_id).await.test_executions[0].test_execution_id.clone();
    let node = handle.identity().node.node_id;
    let command_id = CommandId::from_seed(format!("execution-start:{node}:{execution_id}"));
    let mode = dokime_domain::model::execution::ExecutionMode::Automatic;
    let encoded = serde_json::to_vec(&(&cycle_id, Some(&execution_id), mode)).unwrap();
    let fingerprint = Sha256::digest(encoded).iter().map(|byte| format!("{byte:02x}")).collect();
    let receipt = RunReceipt {
        node_id: node,
        summary: RunSummary {
            mode,
            request_id: command_id.clone(),
            cycle_id: cycle_id.clone(),
            phase: RunPhase::Queued,
            paused: false,
            stop_requested: false,
            planned: 1,
            prepared: 0,
            finished: 0,
            unstarted: 0,
            failed: 0,
            accepted_at: dokime_runtime::model::time::now_utc(),
            provider_started_at: None,
            finished_at: None,
        },
        fingerprint,
        submitted_by: None,
        claimed: 0,
        last_claim: None,
        interruption: None,
        commands: BTreeMap::new(),
    };
    let native = Request::new(&receipt).unwrap();
    let saved = handle.storage().write_object(&native).await.unwrap();
    handle.storage().flush_until(&saved.position).await.unwrap();

    for _ in 0..2 {
        let retry = handle.start_execution(execution_id.clone(), mode, CommandContext::default()).await;
        assert_eq!(retry.status, CommandStatus::Rejected);
        assert_eq!(retry.command_id, command_id);
        assert!(handle.watch_state().latest().state().run_requests.is_empty());
        let execution = cycle_snapshot(&handle, &cycle_id).await.test_executions[0].clone();
        assert!(execution.started_at.is_none() && execution.finished_at.is_none());
    }
    let interrupted = handle.run_request(&command_id).await.unwrap().unwrap();
    assert_eq!(interrupted.summary.phase, RunPhase::Interrupted);
    assert!(interrupted.last_claim.is_none());
    assert_eq!(interrupted.claimed, 0);
}

pub(crate) async fn receipt_initialize_storage() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    let receipt = RunReceipt {
        node_id: handle.identity().node.node_id,
        summary: RunSummary {
            mode: Default::default(),
            request_id: dokime_runtime::model::identity::new_id::<CommandId>(),
            cycle_id: dokime_runtime::model::identity::new_id::<TestCycleId>(),
            phase: RunPhase::Queued,
            paused: false,
            stop_requested: false,
            planned: 1,
            prepared: 0,
            finished: 0,
            unstarted: 0,
            failed: 0,
            accepted_at: dokime_runtime::model::time::now_utc(),
            provider_started_at: None,
            finished_at: None,
        },
        fingerprint: "cleanup-fixture".into(),
        submitted_by: None,
        claimed: 0,
        last_claim: None,
        interruption: None,
        commands: BTreeMap::new(),
    };
    let native = Request::new(&receipt).unwrap();
    let admitted = handle.storage().write_object(&native).await.unwrap();
    handle.storage().flush_until(&admitted.position).await.unwrap();
    runtime.shutdown().await.unwrap();
    // Corrupt The Saved Receipt Without Changing Its Recovery Selection
    let backend = dokime_runtime::storage::Backend::primary(root.path().to_path_buf()).await.unwrap();
    let control = backend.local_state();
    control
        .call(|connection| {
            let changed = connection.execute(
                "UPDATE storage_data_entries SET digest=zeroblob(32)
                 WHERE kind='object' AND schema_id='dokime.object.request'",
                [],
            )?;
            assert_eq!(changed, 1);
            Ok(())
        })
        .await
        .unwrap();
    backend.close().await.unwrap();
    drop(control);

    // Recovery Fails After Assembly; Each Failure Must Release The Shared Owner
    for _ in 0..2 {
        let failure = match Runtime::start(config(root.path())).await {
            Ok(mut runtime) => {
                runtime.shutdown().await.unwrap();
                panic!("corrupt native receipt must fail boot");
            }
            Err(error) => error,
        };
        assert!(failure.to_string().contains("stored entry digest differs"), "{failure}");
        assert!(
            !matches!(&failure, dokime_runtime::runtime::Error::StartupCleanup { .. }),
            "successful partial-boot cleanup must return the original cause"
        );
        // A failed pre-public-runtime initialization explicitly closed the
        // local coordinator; the process root can be reopened immediately.
        let local = dokime_runtime::storage::sqlite::State::open(root.path()).await.unwrap();
        local.close().await.unwrap();
        let reopened = dokime_runtime::storage::Backend::primary(root.path().to_path_buf()).await.unwrap();
        let saved = reopened.open().await.unwrap();
        let snapshot = reopened.open_snapshot(&saved).await.unwrap();
        let corrupt = reopened
            .read_objects(
                &snapshot,
                &harmos::storage::query::ObjectSelection {
                    schema: None,
                    key: None,
                    versions: harmos::storage::query::ObjectVersions::Revisions,
                },
                harmos::storage::ScanPage {
                    after: 0,
                    rows: NonZeroUsize::new(1024).unwrap(),
                    bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),
                },
            )
            .await
            .expect_err("the released root still contains the corrupt receipt");
        assert!(format!("{corrupt:?}").contains("stored entry digest differs"), "{corrupt:?}");
        reopened.release_snapshot(&snapshot).await.unwrap();
        reopened.close().await.unwrap();
    }
}

pub(crate) async fn admitted_progress_stop() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut runtime, handle) = started_with_fixture(config(root.path()), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let cycle = start_cycle(&handle, suite).await;
    let receipt = handle.start_next_execution(cycle, CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;

    let progress = handle.run_request(&receipt.command_id).await.unwrap().unwrap();
    assert_eq!(progress.claimed, 1);
    assert_eq!(progress.summary.prepared, 1);
    assert!(!progress.summary.phase.terminal());
    let progress_boundary = handle.storage().status().accepted;
    assert!(handle.storage().status().accepted.covers(&progress_boundary).unwrap());
    executor.release();
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let terminal = handle.run_request(&receipt.command_id).await.unwrap().unwrap();
    assert_eq!(terminal.summary.phase, RunPhase::Completed);
    assert!(handle.storage().status().saved.covers(&progress_boundary).unwrap());
    drop(handle);
    runtime.shutdown().await.unwrap();

    let (mut runtime, handle) = booted(root.path()).await;
    let saved = handle.run_request(&receipt.command_id).await.unwrap().unwrap();
    assert_eq!(saved.summary.phase, RunPhase::Completed);
    assert_eq!(saved.summary.finished, terminal.summary.finished);
    assert_eq!(saved.summary.finished_at, terminal.summary.finished_at);
    drop(handle);
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn shutdown_settles_requests() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut runtime, handle) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let active_cycle = start_cycle(&handle, suite.clone()).await;
    let queued_cycle = start_cycle(&handle, suite).await;

    let active = handle.start_next_execution(active_cycle, CommandContext::default()).await;
    assert_eq!(active.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let queued = handle.start_next_execution(queued_cycle, CommandContext::default()).await;
    assert_eq!(queued.status, CommandStatus::Accepted);
    let pending = handle.run_request(&queued.command_id).await.unwrap().unwrap();
    assert!(!pending.summary.phase.terminal());
    assert!(pending.summary.provider_started_at.is_none());

    runtime.shutdown().await.unwrap();
    drop(handle);
    let (mut reopened, handle) = booted(root.path()).await;
    for receipt in [&active, &queued] {
        let saved = handle.run_request(&receipt.command_id).await.unwrap().unwrap();
        assert!(saved.summary.phase.terminal());
        assert_ne!(saved.summary.phase, RunPhase::Completed);
        assert_ne!(saved.summary.phase, RunPhase::Interrupted);
        assert!(saved.summary.finished_at.is_some());
        if receipt.command_id == queued.command_id {
            assert!(saved.summary.provider_started_at.is_none());
            assert_eq!(saved.claimed, 0);
        }
    }
    drop(handle);
    reopened.shutdown().await.unwrap();
}

pub(crate) async fn queued_execution_request() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut runtime, handle) = booted_with_fixture(root.path(), executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let active_cycle = start_cycle(&handle, suite.clone()).await;
    let queued_cycle = start_cycle(&handle, suite).await;
    let active = handle.start_next_execution(active_cycle, CommandContext::default()).await;
    assert_eq!(active.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;

    let id = cycle_snapshot(&handle, &queued_cycle).await.test_executions[0].test_execution_id.clone();
    let queued =
        handle.start_execution(id.clone(), dokime_domain::model::execution::ExecutionMode::Automatic, CommandContext::default()).await;
    assert_eq!(queued.status, CommandStatus::Accepted);
    handle.pause_execution(id.clone(), CommandContext::default()).await.unwrap();
    assert!(handle.run_request(&queued.command_id).await.unwrap().unwrap().summary.paused);
    handle.resume_execution(id.clone(), CommandContext::default()).await.unwrap();
    assert!(!handle.run_request(&queued.command_id).await.unwrap().unwrap().summary.paused);
    handle.stop_execution(id, CommandContext::default()).await.unwrap();
    let active_receipt = handle.run_request(&active.command_id).await.unwrap().unwrap();
    assert!(!active_receipt.summary.paused);
    assert!(!active_receipt.summary.stop_requested);
    executor.release();

    wait_for_run_settled(&handle, &queued.command_id).await;
    let receipt = handle.run_request(&queued.command_id).await.unwrap().unwrap();
    assert!(receipt.summary.stop_requested);
    assert_ne!(receipt.summary.phase, RunPhase::Completed);
    assert_eq!(receipt.claimed, 0);
    runtime.shutdown().await.unwrap();
}

/* === Tests === */
