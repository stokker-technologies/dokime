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
use harmos::storage::{ObjectKey, ObjectPoint, ObjectVersion, StorageBackend};

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::storage::requests::Request;
use {
    dokime_domain::model::execution::request::RunPhase,
    dokime_domain::model::execution::request::RunReceipt,
};

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
    let mut child = crate::service_support::children::ChildStory::spawn(
        "durability::requests::durable_requests_reopens",
        &[("DOKIME_RECEIPT_KILL_ROOT", root.path().as_os_str())],
    );
    child.ready().await;
    child.kill();
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
    let backend = dokime_runtime::storage::Backend::primary(root.path().to_path_buf())
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
    crate::service_support::children::announce_ready();
    std::future::pending::<()>().await;
}
