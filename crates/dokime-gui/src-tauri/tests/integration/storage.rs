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
use std::{fs, path::Path};
// Third Party Imports

// Local Imports
use dokime_gui::connection::ConnectionSession;
use dokime_gui::native_session::backend::Backend;
use dokime_gui::native_session::storage::StorageRoot;
use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError, StorageResetReason};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn reason(result: Result<dokime_gui::native_session::DesktopSessionSnapshot, DesktopSessionError>) -> Option<StorageResetReason> {
    match result {
        Err(DesktopSessionError::StorageReset { reason, .. }) => Some(reason),
        _ => None,
    }
}

fn recovery_owner(root: &Path) -> DesktopSession {
    let arguments = dokime_server::host::Args {
        shutdown_stdin: false,
        dev_control: false,
        network: dokime_server::host::NetworkArgs {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: Some(0),
            message_limit_mib: Some(2),
        },
        title: None,
        label: None,
        output_root: Some(root.join("data")),
        application_root: Some(root.to_owned()),
        env_file: None,
        plugins: Vec::new(),
        plugins_root: None,
    };
    DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(arguments))
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn reset_refuses_unowned() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join("store");
    let storage = StorageRoot::new(store.clone(), root.path().join("application"));
    fs::create_dir_all(&store).unwrap();
    fs::write(store.join("local.sqlite"), b"kept").unwrap();

    // Neither an attachment nor an owned session that never started owns it.
    let attached = DesktopSession::attached(ConnectionSession::default());
    assert_eq!(reason(attached.reset_local_storage(&store, storage.clone()).await), Some(StorageResetReason::NotOwned));
    let idle = DesktopSession::with_local_configuration(ConnectionSession::default(), Err("never started".into()));
    assert_eq!(reason(idle.reset_local_storage(&store, storage).await), Some(StorageResetReason::NotOwned));
    assert!(store.join("local.sqlite").exists());
}

#[dokime_tracing::trace_test]
#[test]
fn backup_archives_store() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join("store");
    let storage = StorageRoot::new(store.clone(), root.path().join("application"));
    fs::create_dir_all(store.join("artifacts/cycle-1/nested")).unwrap();
    fs::create_dir_all(store.join("uploads/pending")).unwrap();
    fs::write(store.join("local.sqlite"), b"rows").unwrap();
    fs::write(store.join("local.sqlite-wal"), b"wal").unwrap();
    fs::write(store.join("local.sqlite-shm"), b"shm").unwrap();
    fs::write(store.join("artifacts/cycle-1/evidence.bin"), b"bytes").unwrap();
    fs::write(store.join("artifacts/cycle-1/nested/trace.csv"), vec![b'7'; 3 << 20]).unwrap();

    // A Backend Holding The Lease Refuses The Backup
    let lease = fs::OpenOptions::new().create(true).truncate(false).write(true).open(store.join("local.lock")).unwrap();
    lease.try_lock().unwrap();
    let destination = storage.backup_destination(time::OffsetDateTime::UNIX_EPOCH);
    assert_eq!(destination, root.path().join("store-backup-19700101T000000Z.zip"));
    assert!(matches!(
        storage.archive(&destination, |_| {}),
        Err(DesktopSessionError::StorageReset {
            reason: StorageResetReason::BackendRunning,
            ..
        })
    ));
    assert!(!destination.exists());
    drop(lease);

    // A Destination Inside The Root Is Refused
    assert!(matches!(
        storage.archive(&store.join("inside.zip"), |_| {}),
        Err(DesktopSessionError::StorageReset {
            reason: StorageResetReason::OutsideRoot,
            ..
        })
    ));

    // The Archive Holds Every Nested File, The WAL Included, But The Lease
    let mut reports = Vec::new();
    let archive = storage.archive(&destination, |bytes| reports.push(bytes)).unwrap();
    assert_eq!(archive, destination);
    let written = fs::metadata(&archive).unwrap().len();
    assert_eq!(reports.last(), Some(&written));
    assert!(reports.len() > 1, "{reports:?}");
    assert!(reports.is_sorted());
    let mut zip = zip::ZipArchive::new(fs::File::open(&archive).unwrap()).unwrap();
    let mut names = zip.file_names().map(|name| name.unwrap().into_owned()).collect::<Vec<_>>();
    names.sort();
    assert_eq!(
        names,
        [
            "artifacts/",
            "artifacts/cycle-1/",
            "artifacts/cycle-1/evidence.bin",
            "artifacts/cycle-1/nested/",
            "artifacts/cycle-1/nested/trace.csv",
            "local.sqlite",
            "local.sqlite-shm",
            "local.sqlite-wal",
            "uploads/",
            "uploads/pending/",
        ]
    );
    let mut evidence = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("artifacts/cycle-1/evidence.bin").unwrap(), &mut evidence).unwrap();
    assert_eq!(evidence, "bytes");
    let mut wal = String::new();
    std::io::Read::read_to_string(&mut zip.by_name("local.sqlite-wal").unwrap(), &mut wal).unwrap();
    assert_eq!(wal, "wal");
    assert_eq!(zip.by_name("artifacts/cycle-1/nested/trace.csv").unwrap().size(), 3 << 20);

    // An Existing Archive Is Never Overwritten
    assert!(storage.archive(&destination, |_| {}).is_err());
    assert_eq!(fs::metadata(&archive).unwrap().len(), written);

    // The Reset Then Empties The Root
    storage.clear().unwrap();
    assert_eq!(fs::read_dir(&store).unwrap().count(), 0);
    assert!(archive.exists());
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn backup_reset_backup_failure_leaves_storage_untouched_and_can_retry() {
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("local.sqlite"), b"corrupt storage").unwrap();
    std::fs::write(data.join("evidence"), b"retained evidence").unwrap();
    let storage = StorageRoot::new(data.clone(), root.path().to_owned());
    let owner = recovery_owner(root.path());
    assert!(matches!(owner.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::Startup { .. })));
    let occupied = root.path().join("occupied.zip");
    std::fs::write(&occupied, b"existing archive").unwrap();
    assert!(owner.backup_and_reset_local_storage(storage.clone(), occupied.clone()).await.is_err());
    assert_eq!(std::fs::read(data.join("local.sqlite")).unwrap(), b"corrupt storage");
    assert_eq!(std::fs::read(data.join("evidence")).unwrap(), b"retained evidence");
    assert_eq!(std::fs::read(occupied).unwrap(), b"existing archive");

    let archive = root.path().join("saved.zip");
    let recovered = owner.backup_and_reset_local_storage(storage, archive.clone()).await.unwrap();
    assert_eq!(recovered.ownership, BackendOwnership::OwnedLocal);
    assert_eq!(recovered.archive.as_deref(), Some(archive.to_str().unwrap()));
    assert!(archive.is_file());
    assert!(!data.join("evidence").exists());
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn backup_reset_refuses_idle_and_another_backends_storage() {
    let root = tempfile::tempdir().unwrap();
    let storage = StorageRoot::new(root.path().join("data"), root.path().to_owned());
    let owner = recovery_owner(root.path());
    assert!(matches!(
        owner.backup_and_reset_local_storage(storage.clone(), root.path().join("idle.zip")).await,
        Err(DesktopSessionError::StorageReset { reason: StorageResetReason::NotOwned, .. })
    ));
    owner.select(BackendSelection::OwnedLocal).await.unwrap();
    let other = recovery_owner(root.path());
    assert!(matches!(other.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::BackendRunning { .. })));
    assert!(matches!(
        other.backup_and_reset_local_storage(storage, root.path().join("occupied.zip")).await,
        Err(DesktopSessionError::StorageReset { reason: StorageResetReason::NotOwned, .. })
    ));
    assert!(!root.path().join("occupied.zip").exists());
    owner.shutdown().await.unwrap();
}
