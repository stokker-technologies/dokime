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
use std::{num::NonZeroUsize, process::Command};

// Third Party Imports
use harmos::storage::{ObjectKey, ObjectPoint, ObjectVersion, StorageBackend};

// Local Imports
use crate::support::prelude::*;
use dokime::core::CommandId;
use dokime::execution::request::RunSummary;
use dokime::storage::requests::Request;
use dokime_app::domains::execution::requests::{RunPhase, RunReceipt};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn durable_requests_reopens() {
    if let Some(root) = std::env::var_os("DOKIME_RECEIPT_KILL_ROOT") {
        run_child(PathBuf::from(root)).await;
        return;
    }

    let root = tempdir().unwrap();
    let marker = root.path().join("request-ready.json");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "storage::requests::durable_requests_reopens",
            "--nocapture",
        ])
        .env("DOKIME_RECEIPT_KILL_ROOT", root.path())
        .spawn()
        .unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    while !marker.exists() && tokio::time::Instant::now() < deadline {
        assert!(
            child.try_wait().unwrap().is_none(),
            "writer exited before saving the receipt"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    // Reap the writer even if the assertion below fails.
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(marker.exists(), "writer did not save a receipt");
    let pending: RunReceipt = serde_json::from_slice(&std::fs::read(marker).unwrap()).unwrap();
    assert!(!pending.summary.phase.terminal());
    assert!(pending.claimed > 0);
    assert!(pending.last_claim.is_some());

    let (mut runtime, handle) = booted(root.path()).await;
    let recovered = handle
        .run_request(&pending.summary.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recovered.summary.phase, RunPhase::Interrupted);
    assert_eq!(recovered.claimed, pending.claimed);
    assert_eq!(recovered.fingerprint, pending.fingerprint);
    assert_eq!(recovered.submitted_by, pending.submitted_by);
    assert_eq!(
        recovered.summary.provider_started_at,
        pending.summary.provider_started_at
    );
    assert_eq!(
        recovered.last_claim.as_ref().unwrap().execution_id,
        pending.last_claim.as_ref().unwrap().execution_id
    );
    assert!(recovered.summary.finished_at.is_some());
    drop(handle);
    runtime.shutdown().await.unwrap();

    let (mut runtime, handle) = booted(root.path()).await;
    let reread = handle
        .run_request(&pending.summary.request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reread.summary.phase, RunPhase::Interrupted);
    assert_eq!(reread.summary.finished_at, recovered.summary.finished_at);
    drop(handle);
    runtime.shutdown().await.unwrap();

    // Read The Saved Native Parent After The Runtime Has Closed
    let backend = dokime::storage::Backend::primary(root.path().to_path_buf())
        .await
        .unwrap();
    let saved = backend.open().await.unwrap();
    let snapshot = backend.open_snapshot(&saved).await.unwrap();
    let query = ObjectPoint {
        schema: Identifier::new("dokime.object.request").unwrap(),
        item: ObjectKey::new(pending.summary.request_id.as_str(), NonZeroUsize::MAX).unwrap(),
        version: ObjectVersion::Latest,
    };
    let object = backend
        .read_object(&snapshot, &query)
        .await
        .unwrap()
        .unwrap();
    let native = <Request as harmos::Message>::decode(&object.value.payload).unwrap();
    let receipt = native.restore().unwrap();
    assert_eq!(receipt.summary.phase, RunPhase::Interrupted);
    assert_eq!(receipt.claimed, pending.claimed);
    assert_eq!(receipt.last_claim, pending.last_claim);
    assert_eq!(receipt.summary.finished_at, recovered.summary.finished_at);
    backend.release_snapshot(&snapshot).await.unwrap();
    backend.close().await.unwrap();
}

async fn run_child(root: PathBuf) {
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_runtime, handle) = booted_with_fixture(&root, executor.clone()).await;
    let suite = fixture_suite(&["passed"]);
    register_python_suite(&handle, suite.clone()).await;
    let cycle = start_cycle(&handle, suite, false).await;
    let receipt = handle
        .run_cycle_cases(
            cycle,
            CommandContext {
                submitted_by: Some("operator".into()),
                idempotency_key: Some("crash-receipt".into()),
                ..Default::default()
            },
        )
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;
    let pending = handle
        .run_request(&receipt.command_id)
        .await
        .unwrap()
        .unwrap();
    // Recovery promises saved receipts, not every accepted progress update.
    // This explicit barrier fixes the durable frontier before the parent kills
    // the writer; the periodic worker may otherwise legitimately save later.
    let accepted = handle.storage().status().accepted;
    assert!(
        handle
            .storage()
            .flush()
            .await
            .unwrap()
            .covers(&accepted)
            .unwrap()
    );
    let temporary = root.join("request-ready.tmp");
    std::fs::write(&temporary, serde_json::to_vec(&*pending).unwrap()).unwrap();
    std::fs::rename(temporary, root.join("request-ready.json")).unwrap();
    std::future::pending::<()>().await;
}

pub(crate) async fn receipt_initialize_storage() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    let receipt = RunReceipt {
        node_id: handle.identity().node.node_id,
        summary: RunSummary {
            mode: Default::default(),
            request_id: CommandId::new(),
            cycle_id: TestCycleId::new(),
            phase: RunPhase::Queued,
            paused: false,
            stop_requested: false,
            planned: 1,
            prepared: 0,
            finished: 0,
            unstarted: 0,
            failed: 0,
            accepted_at: Timestamp::now_utc(),
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
    handle
        .storage()
        .flush_until(&admitted.position)
        .await
        .unwrap();
    runtime.shutdown().await.unwrap();
    // Corrupt The Saved Receipt Without Changing Its Recovery Selection
    let backend = dokime::storage::Backend::primary(root.path().to_path_buf())
        .await
        .unwrap();
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
        assert!(
            failure.to_string().contains("stored entry digest differs"),
            "{failure}"
        );
        assert!(
            !matches!(&failure, dokime_app::Error::StartupCleanup { .. }),
            "successful partial-boot cleanup must return the original cause"
        );
        // A failed pre-public-runtime initialization explicitly closed the
        // local coordinator; the process root can be reopened immediately.
        let local = dokime::storage::sqlite::State::open(root.path())
            .await
            .unwrap();
        local.close().await.unwrap();
        let reopened = dokime::storage::Backend::primary(root.path().to_path_buf())
            .await
            .unwrap();
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
            .err()
            .expect("the released root still contains the corrupt receipt");
        assert!(
            format!("{corrupt:?}").contains("stored entry digest differs"),
            "{corrupt:?}"
        );
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
    let cycle = start_cycle(&handle, suite, false).await;
    let receipt = handle
        .run_cycle_cases(cycle, CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    executor.wait_until_entered().await;

    let progress = handle
        .run_request(&receipt.command_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(progress.claimed, 1);
    assert_eq!(progress.summary.prepared, 1);
    assert!(!progress.summary.phase.terminal());
    let progress_boundary = handle.storage().status().accepted;
    assert!(
        !handle
            .storage()
            .status()
            .saved
            .covers(&progress_boundary)
            .unwrap()
    );
    executor.release();
    let terminal = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let current = handle
                .run_request(&receipt.command_id)
                .await
                .unwrap()
                .unwrap();
            if current.summary.phase.terminal() {
                break current;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("terminal receipt must settle before the periodic save");
    assert_eq!(terminal.summary.phase, RunPhase::Completed);
    assert!(
        handle
            .storage()
            .status()
            .saved
            .covers(&progress_boundary)
            .unwrap()
    );
    drop(handle);
    runtime.shutdown().await.unwrap();

    let (mut runtime, handle) = booted(root.path()).await;
    let saved = handle
        .run_request(&receipt.command_id)
        .await
        .unwrap()
        .unwrap();
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
    let active_cycle = start_cycle(&handle, suite.clone(), false).await;
    let queued_cycle = start_cycle(&handle, suite, false).await;

    let active = handle
        .run_cycle_cases(active_cycle, CommandContext::default())
        .await;
    assert_eq!(active.status, CommandStatus::Accepted);
    tokio::time::timeout(Duration::from_secs(5), executor.wait_until_entered())
        .await
        .expect("first provider did not enter");
    let queued = handle
        .run_cycle_cases(queued_cycle, CommandContext::default())
        .await;
    assert_eq!(queued.status, CommandStatus::Accepted);
    let pending = handle
        .run_request(&queued.command_id)
        .await
        .unwrap()
        .unwrap();
    assert!(!pending.summary.phase.terminal());
    assert!(pending.summary.provider_started_at.is_none());

    tokio::time::timeout(Duration::from_secs(15), runtime.shutdown())
        .await
        .expect("shutdown did not join admitted provider monitors")
        .unwrap();
    drop(handle);
    let (mut reopened, handle) = booted(root.path()).await;
    for receipt in [&active, &queued] {
        let saved = handle
            .run_request(&receipt.command_id)
            .await
            .unwrap()
            .unwrap();
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
    let active_cycle = start_cycle(&handle, suite.clone(), false).await;
    let queued_cycle = start_cycle(&handle, suite, false).await;
    let active = handle
        .run_cycle_cases(active_cycle, CommandContext::default())
        .await;
    assert_eq!(active.status, CommandStatus::Accepted);
    tokio::time::timeout(Duration::from_secs(5), executor.wait_until_entered())
        .await
        .unwrap();

    let id = cycle_snapshot(&handle, &queued_cycle).await.test_executions[0]
        .test_execution_id
        .clone();
    let queued = handle
        .run_execution(
            id.clone(),
            dokime::execution::ExecutionMode::Automatic,
            CommandContext::default(),
        )
        .await;
    assert_eq!(queued.status, CommandStatus::Accepted);
    handle
        .pause_execution(id.clone(), CommandContext::default())
        .await
        .unwrap();
    assert!(
        handle
            .run_request(&queued.command_id)
            .await
            .unwrap()
            .unwrap()
            .summary
            .paused
    );
    handle
        .resume_execution(id.clone(), CommandContext::default())
        .await
        .unwrap();
    assert!(
        !handle
            .run_request(&queued.command_id)
            .await
            .unwrap()
            .unwrap()
            .summary
            .paused
    );
    handle
        .stop_execution(id, CommandContext::default())
        .await
        .unwrap();
    let active_receipt = handle
        .run_request(&active.command_id)
        .await
        .unwrap()
        .unwrap();
    assert!(!active_receipt.summary.paused);
    assert!(!active_receipt.summary.stop_requested);
    executor.release();

    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let receipt = handle
                .run_request(&queued.command_id)
                .await
                .unwrap()
                .unwrap();
            if receipt.summary.phase.terminal() {
                assert!(receipt.summary.stop_requested);
                assert_ne!(receipt.summary.phase, RunPhase::Completed);
                assert_eq!(receipt.claimed, 0);
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("stopped execution request did not settle");
    runtime.shutdown().await.unwrap();
}
