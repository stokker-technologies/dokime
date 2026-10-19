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
use std::sync::atomic::{AtomicBool, Ordering};

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

#[tokio::test]
async fn node_identity_restart() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    let first = handle.runtime_overview().await.expect("runtime overview query succeeds").identity.node.node_id;
    drop(handle);
    let stopped = runtime.shutdown().await.unwrap();
    assert_eq!(runtime.shutdown().await.unwrap(), stopped);

    let decorated = Arc::new(AtomicBool::new(false));
    let observed = decorated.clone();
    let mut rebooted = Runtime::with_fixture_backends(config(root.path()), BTreeMap::new(), move |backends| {
        observed.store(true, Ordering::Release);
        backends
    })
    .await
    .unwrap();
    assert!(decorated.load(Ordering::Acquire));
    let handle = rebooted.handle();
    let second = handle.runtime_overview().await.expect("runtime overview query succeeds").identity.node.node_id;
    drop(handle);

    assert_eq!(second, first);
    assert!(!root.path().join("identity").exists());
    rebooted.shutdown().await.unwrap();
}

#[tokio::test]
async fn starts_shutdown_commands() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = booted(root.path()).await;

    assert!(!handle.runtime_overview().await.expect("runtime overview query succeeds").shutting_down);
    assert!(!handle.is_shutting_down());

    // Shutdown first publishes the product stop marker, then Harmos waits for
    // the final operational handle before joining its journal.
    let stopping = tokio::spawn(async move { runtime.shutdown().await });
    while !handle.is_shutting_down() {
        tokio::task::yield_now().await;
    }
    let rejected = record_health(&handle, "shutdown probe").await;

    assert!(handle.is_shutting_down());
    assert!(rejected.is_err());

    // The last handle is the explicit Harmos drain barrier.
    drop(handle);
    let shutdown = stopping.await.unwrap().unwrap();
    assert!(shutdown.shutting_down);

    // A shutdown marker survives long enough for the old process to drain, but
    // boot clears it for the fresh process before accepting new work.
    let (mut rebooted, rebooted_handle) = booted(root.path()).await;
    assert!(!rebooted_handle.runtime_overview().await.expect("runtime overview query succeeds").shutting_down);
    assert!(record_health(&rebooted_handle, "rebooted probe").await.is_ok());
    drop(rebooted_handle);
    rebooted.shutdown().await.unwrap();
}

#[tokio::test]
async fn old_incompatible_startup() {
    let root = tempdir().unwrap();
    let legacy = root.path().join("harmos");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("journal.jsonl"), b"incompatible old history\n").unwrap();
    std::fs::write(legacy.join("state.json"), b"invalid snapshot").unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    assert!(!handle.is_shutting_down());
    drop(handle);
    runtime.shutdown().await.unwrap();
    assert_eq!(std::fs::read(legacy.join("journal.jsonl")).unwrap(), b"incompatible old history\n");
}

/* === Tests === */

#[tokio::test]
async fn boot_reboots_intact() {
    let root = tempfile::tempdir().unwrap();
    let metadata = Metadata::new(Identifier::new("runtime.hub").unwrap(), "Runtime Hub", None);

    // First boot installs and journals the bundled artifact composition.
    start_bundled_composition(root.path(), metadata.clone()).await.shutdown().await.unwrap();

    // Every boot reconstructs the bundled composition into fresh state.
    // Reusing its storage root must preserve the default definitions.
    start_bundled_composition(root.path(), metadata).await.shutdown().await.unwrap();
}

async fn start_bundled_composition(root: &std::path::Path, metadata: Metadata) -> Runtime {
    let config = crate::service_support::budgets::runtime_config(root, metadata).with_installed_plugins([dokime_host::installed_plugin()]);
    Runtime::start(config).await.expect("the shipped composition boots")
}
