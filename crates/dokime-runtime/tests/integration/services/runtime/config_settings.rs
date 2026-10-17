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
use std::time::Duration;

#[tokio::test]
async fn backend_policy_restart() {
    use crate::service_support::{booted, config};
    let root = tempfile::tempdir().unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    let before = handle.storage_policy().await.unwrap();
    assert_eq!(before.revision, 0);
    assert!(!before.restart_required);
    let mut policy = before.configured.clone();
    policy.compaction_interval_ms = 7_200_000;
    policy.local_retention_ms = Some(604_800_000);
    let saved = handle
        .update_storage_policy(0, policy.clone())
        .await
        .unwrap();
    assert_eq!(saved.revision, 1);
    assert!(saved.restart_required);
    assert_eq!(saved.effective, before.effective);
    assert_eq!(saved.configured, policy);
    assert!(matches!(
        handle.update_storage_policy(0, policy.clone()).await,
        Err(dokime_runtime::runtime::Error::Conflict { .. })
    ));
    let mut invalid = policy.clone();
    invalid.compaction_interval_ms = 0;
    assert!(handle.update_storage_policy(1, invalid).await.is_err());
    assert_eq!(handle.storage_policy().await.unwrap(), saved);
    assert_eq!(
        handle
            .update_storage_policy(1, policy.clone())
            .await
            .unwrap(),
        saved
    );
    runtime.shutdown().await.unwrap();
    assert!(matches!(
        handle.update_storage_policy(1, policy.clone()).await,
        Err(dokime_runtime::runtime::Error::ShuttingDown)
    ));
    drop(handle);

    let mut fallback = config(root.path());
    fallback.compaction_interval = Duration::from_secs(3600);
    let mut runtime = dokime_runtime::Runtime::start(fallback).await.unwrap();
    let restored = runtime.handle().storage_policy().await.unwrap();
    assert_eq!(restored.revision, 0, "revisions count saves in one process");
    assert_eq!(restored.effective, policy);
    assert_eq!(restored.configured, policy);
    assert!(!restored.restart_required);
    runtime.shutdown().await.unwrap();
    let settings = root.path().join("application/settings.json");
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&settings).unwrap()).unwrap();
    assert_eq!(saved["storage"]["compaction_interval_ms"], 7_200_000);
    assert!(saved.get("restart_required").is_none());
    std::fs::write(settings, b"broken").unwrap();
    assert!(
        dokime_runtime::Runtime::start(config(root.path()))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn concurrent_policy_revision() {
    let root = tempfile::tempdir().unwrap();
    let (mut runtime, handle) = crate::service_support::booted(root.path()).await;
    let original = handle.storage_policy().await.unwrap();
    let mut a = original.configured.clone();
    a.compaction_interval_ms = 3_600_000;
    let mut b = original.configured;
    b.compaction_interval_ms = 10_800_000;
    let (a, b) = tokio::join!(
        handle.update_storage_policy(0, a),
        handle.update_storage_policy(0, b)
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let saved = handle.storage_policy().await.unwrap();
    assert_eq!(saved.revision, 1);
    assert_eq!(saved, a.or(b).unwrap());
    runtime.shutdown().await.unwrap();
}
