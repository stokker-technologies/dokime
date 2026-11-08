/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SQLite Group Boundaries, Failed Commits And Process Crash Recovery

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

// Third Party Imports
use harmos::storage::{FileMetadata, StorageBackend};
use sha2::{Digest, Sha256, digest::common::hazmat::SerializableState};

// Local Imports
use dokime_runtime::storage::sqlite::Backend;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn metadata(id: &str) -> FileMetadata {
    FileMetadata {
        id: harmos::Identifier::new(id).unwrap(),
        scope: None,
        media_type: "application/octet-stream".into(),
        expected_length: None,
        expected_sha256: None,
    }
}

/* === Tests === */

#[tokio::test]
async fn queued_writers_share_one_durable_commit() {
    for (rows, byte_capacity, expected_groups) in [(8usize, 8usize, 2usize), (2, 8, 5), (8, 2, 5)] {
        let root = tempfile::tempdir().unwrap();
        let mut policy = dokime_runtime::storage::sqlite::commit_policy::defaults();
        policy.rows = rows.try_into().unwrap();
        policy.bytes = (serde_json::to_vec(&metadata("queued-0")).unwrap().len() * byte_capacity).try_into().unwrap();
        let settings = dokime_runtime::storage::sqlite::Settings {
            commit: policy,
            ..dokime_runtime::config::local_storage_defaults()
        };
        let backend = Arc::new(Backend::<()>::primary_with(root.path().to_owned(), settings).await.unwrap());
        backend.open().await.unwrap();
        let state = backend.local_state();
        let commits = Arc::new(AtomicUsize::new(0));
        let count = commits.clone();
        let (entered, entry) = tokio::sync::oneshot::channel();
        let (release, released) = std::sync::mpsc::channel();
        state
            .call(move |connection| {
                let mut entered = Some(entered);
                connection.commit_hook(Some(move || {
                    count.fetch_add(1, Ordering::SeqCst);
                    if let Some(entered) = entered.take() {
                        let _ = entered.send(());
                        released.recv().unwrap();
                    }
                    false
                }))
            })
            .await
            .unwrap();
        let first_backend = backend.clone();
        let first = tokio::spawn(async move { first_backend.create_file(&metadata("first")).await.unwrap() });
        entry.await.unwrap();
        let metadata = (0..8).map(|index| metadata(&format!("queued-{index}"))).collect::<Vec<_>>();
        let mut pending = metadata.iter().map(|metadata| backend.create_file(metadata)).collect::<Vec<_>>();
        for write in &mut pending {
            assert!(futures_util::poll!(write.as_mut()).is_pending());
        }
        let reader = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
        let count: i64 = reader.query_row("SELECT count(*) FROM storage_file_stages", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
        release.send(()).unwrap();
        first.await.unwrap();
        for write in pending {
            write.await.unwrap();
        }
        assert_eq!(commits.load(Ordering::SeqCst), expected_groups);
        state.close().await.unwrap();
        let count: i64 = reader.query_row("SELECT count(*) FROM storage_file_stages", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 9);
    }
}

#[tokio::test]
async fn failed_file_creation_does_not_publish_a_production() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_owned()).await.unwrap();
    backend.open().await.unwrap();
    let staging = root.path().join("staging");
    std::fs::write(&staging, b"blocked directory").unwrap();
    assert!(backend.create_production(&metadata("retry")).await.is_err());
    let count: i64 = backend
        .local_state()
        .call(|connection| connection.query_row("SELECT count(*) FROM storage_productions", [], |row| row.get(0)))
        .await
        .unwrap();
    assert_eq!(count, 0);
    std::fs::remove_file(staging).unwrap();
    backend.create_production(&metadata("retry")).await.unwrap();
    backend.local_state().close().await.unwrap();
    let reopened = Backend::<()>::primary(root.path().to_owned()).await.unwrap();
    reopened.open().await.unwrap();
    let recovered = reopened
        .recover_productions(harmos::storage::ReadLimits {
            rows: 4.try_into().unwrap(),
            bytes: 4096.try_into().unwrap(),
        })
        .await
        .unwrap();
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].metadata.id.as_str(), "retry");
    reopened.local_state().close().await.unwrap();
}

#[tokio::test]
async fn rejected_group_acknowledges_no_writer() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_owned()).await.unwrap();
    backend.open().await.unwrap();
    backend.create_production(&metadata("acknowledged")).await.unwrap();
    let state = backend.local_state();
    state.call(|connection| connection.commit_hook(Some(|| true))).await.unwrap();
    assert!(backend.create_production(&metadata("rejected")).await.is_err());
    state.close().await.unwrap();
    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    let ids = connection
        .prepare("SELECT id FROM storage_file_stages ORDER BY id")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(ids, ["acknowledged"]);
}

#[tokio::test]
async fn crashes_preserve_acknowledged_productions() {
    for mode in ["commit", "checkpoint"] {
        let root = tempfile::tempdir().unwrap();
        let status = dokime_domain::process::command(std::env::current_exe().unwrap())
            .args(["--exact", "storage::group_commit::crash_child", "--ignored", "--nocapture"])
            .env("DOKIME_COMMIT_CRASH_ROOT", root.path())
            .env("DOKIME_COMMIT_CRASH_MODE", mode)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(if mode == "commit" { 73 } else { 74 }));
        let backend = Backend::<()>::primary(root.path().to_owned()).await.unwrap();
        backend.open().await.unwrap();
        let limits = harmos::storage::ReadLimits {
            rows: 16.try_into().unwrap(),
            bytes: 1_048_576.try_into().unwrap(),
        };
        let recovered = backend.recover_productions(limits).await.unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].metadata.id.as_str(), "acknowledged");
        assert_eq!(recovered[0].length, 7);
        let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
        let name: String = connection
            .query_row("SELECT uuid FROM storage_staging WHERE id='acknowledged' AND phase='write'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(std::fs::read(root.path().join("staging").join(name)).unwrap(), b"payload");
        backend.local_state().close().await.unwrap();
    }
}

#[ignore = "Invoked By The Crash Recovery Parent"]
#[tokio::test]
async fn crash_child() {
    let root = std::path::PathBuf::from(std::env::var_os("DOKIME_COMMIT_CRASH_ROOT").unwrap());
    let backend = Backend::<()>::primary(root).await.unwrap();
    backend.open().await.unwrap();
    let metadata = metadata("acknowledged");
    let file = backend.create_production(&metadata).await.unwrap();
    backend.write_file(&file, 0, b"payload".to_vec().into()).await.unwrap();
    let digest = Sha256::new_with_prefix(b"payload");
    let checkpoint = harmos::storage::FileCheckpoint {
        metadata,
        session: harmos::storage::FileSession {
            history: file.history.clone(),
            id: file.id.clone(),
            generation: 1,
        },
        length: 7,
        hash_format: 1,
        hash_state: digest.serialize().to_vec(),
    };
    backend.save_production(harmos::storage::StorageBatch::new(vec![checkpoint])).await.unwrap();
    let state = backend.local_state();
    if std::env::var("DOKIME_COMMIT_CRASH_MODE").unwrap() == "commit" {
        state.call(|connection| connection.commit_hook(Some(|| std::process::exit(73)))).await.unwrap();
        let position = harmos::storage::StorageCursor {
            history: file.history.clone(),
            sequence: 1,
        };
        backend.finish_file(&file, &position, 7, digest.finalize().into()).await.unwrap();
        panic!("commit crash hook did not run");
    }
    state
        .call(|connection| {
            let mut file: *mut rusqlite::ffi::sqlite3_file = std::ptr::null_mut();
            // SAFETY: SQLite Writes Its Live Main File Pointer Into The Provided Slot.
            let result = unsafe {
                rusqlite::ffi::sqlite3_file_control(
                    connection.handle(),
                    c"main".as_ptr(),
                    rusqlite::ffi::SQLITE_FCNTL_FILE_POINTER,
                    (&mut file as *mut *mut rusqlite::ffi::sqlite3_file).cast(),
                )
            };
            assert_eq!(result, rusqlite::ffi::SQLITE_OK);
            assert!(!file.is_null());
            // SAFETY: The Table Copy Retains Every Original Method Except xSync. Its
            // Leaked Lifetime Extends Through The Intentional Process Exit In xSync.
            unsafe {
                unsafe extern "C" fn crash_checkpoint_sync(_: *mut rusqlite::ffi::sqlite3_file, _: i32) -> i32 {
                    std::process::exit(74)
                }
                let mut methods = Box::new(std::ptr::read((*file).pMethods));
                methods.xSync = Some(crash_checkpoint_sync);
                (*file).pMethods = Box::into_raw(methods);
            }
            connection.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        })
        .await
        .unwrap();
    panic!("checkpoint crash hook did not run");
}
