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
use std::fs;
// Third Party Imports

// Local Imports
use dokime_gui::connection::ConnectionSession;
use dokime_gui::native_session::storage::StorageRoot;
use dokime_gui::native_session::{
    BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError, StorageResetReason,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn refuses_unowned() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join("store");
    let storage = StorageRoot::new(store.clone(), root.path().join("application"));
    fs::create_dir_all(&store).unwrap();
    fs::write(store.join("local.sqlite"), b"kept").unwrap();

    // Neither an attachment nor an owned session that never started owns it.
    let attached = DesktopSession::attached(ConnectionSession::default());
    assert_eq!(
        reason(attached.reset_local_storage(&store, storage.clone()).await),
        Some(StorageResetReason::NotOwned)
    );
    let idle = DesktopSession::with_local_configuration(
        ConnectionSession::default(),
        Err("never started".into()),
    );
    assert_eq!(
        reason(idle.reset_local_storage(&store, storage).await),
        Some(StorageResetReason::NotOwned)
    );
    assert!(store.join("local.sqlite").exists());
}

#[cfg(unix)]
pub(crate) async fn refusals_and_reset() {
    let root = tempfile::tempdir().unwrap();
    let backend = super::Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let application = root.path().join("application");
    let storage = StorageRoot::new(store.clone(), application.clone());
    let executable = super::stub(root.path(), super::Publish::Ready(&backend.endpoint()));
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        super::owned(executable, &store),
    );
    let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    populate(&store, root.path());

    // A Path Other Than The Data Root Leaves The Backend Running
    let artifacts = store.join("artifacts");
    for outside in [root.path(), application.as_path(), artifacts.as_path()] {
        assert_eq!(
            reason(owner.reset_local_storage(outside, storage.clone()).await),
            Some(StorageResetReason::OutsideRoot)
        );
    }
    let enclosing = StorageRoot::new(root.path().to_path_buf(), application.clone());
    assert_eq!(
        reason(owner.reset_local_storage(root.path(), enclosing).await),
        Some(StorageResetReason::OutsideRoot)
    );
    assert_eq!(
        owner.status().connection.generation,
        started.connection.generation
    );
    assert!(super::store_owner(&store).is_some());

    // A Backend Still Holding The Store Keeps Every File
    let lease = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(store.join("local.lock"))
        .unwrap();
    lease.try_lock().unwrap();
    assert_eq!(
        reason(owner.reset_local_storage(&store, storage.clone()).await),
        Some(StorageResetReason::BackendRunning)
    );
    assert_eq!(owner.status().ownership, BackendOwnership::None);
    assert!(store.join("local.sqlite").exists());
    assert!(store.join("artifacts/evidence.bin").exists());
    drop(lease);

    // The Reset Empties The Root And Starts Again On It
    let restarted = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    let reset = owner
        .reset_local_storage(&store, storage.clone())
        .await
        .unwrap();
    assert_eq!(reset.ownership, BackendOwnership::OwnedLocal);
    assert_ne!(reset.connection.generation, restarted.connection.generation);
    assert_eq!(owner.status().phase, None);
    let mut remaining = fs::read_dir(&store)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    remaining.sort();
    // Only the restarted backend's own ownership marker is left.
    assert_eq!(remaining, vec!["owner.pid".to_owned()]);
    // A link inside the root is removed, never followed.
    assert!(root.path().join("outside.txt").exists());

    owner.shutdown().await.unwrap();
    backend.shutdown().await;
}

/* === Private Functions === */
fn reason(
    result: Result<dokime_gui::native_session::DesktopSessionSnapshot, DesktopSessionError>,
) -> Option<StorageResetReason> {
    match result {
        Err(DesktopSessionError::StorageReset { reason, .. }) => Some(reason),
        _ => None,
    }
}

#[cfg(unix)]
fn populate(store: &std::path::Path, outside: &std::path::Path) {
    fs::write(store.join("local.sqlite"), b"rows").unwrap();
    fs::write(store.join("local.sqlite-wal"), b"wal").unwrap();
    fs::create_dir_all(store.join("artifacts")).unwrap();
    fs::write(store.join("artifacts/evidence.bin"), b"bytes").unwrap();
    fs::create_dir_all(store.join("uploads/pending")).unwrap();
    fs::write(outside.join("outside.txt"), b"not ours").unwrap();
    std::os::unix::fs::symlink(outside.join("outside.txt"), store.join("linked.txt")).unwrap();
}

/* === Tests === */
