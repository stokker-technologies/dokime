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
use harmos::storage::{
    state::{RecoveryEntry, StateBackend},
    *,
};
use sha2::{Digest, Sha256};

// Local Imports
use super::storage::{history, id};
use dokime::storage::sqlite::{Backend, Error, Limits, State, StateLimits};

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

pub(crate) async fn refuses_retired_evidence() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    let metadata = FileMetadata {
        id: id("preserved"),
        scope: None,
        media_type: "text/plain".into(),
        expected_length: None,
        expected_sha256: None,
    };
    let file = backend.create_file(&metadata).await.unwrap();
    backend.write_file(&file, 0, b"preserved").await.unwrap();
    let digest = Sha256::digest(b"preserved").into();
    let reference = backend
        .finish_file(
            &file,
            &StorageCursor {
                history: history(),
                sequence: 1,
            },
            9,
            digest,
        )
        .await
        .unwrap();
    let files = [reference.clone()];
    let parts = [StoragePart::Files(&files)];
    backend
        .write(StorageBatch::new(&parts, NonZeroUsize::new(4096).unwrap()).unwrap())
        .await
        .unwrap();
    let recovery = backend.state(StateLimits::default());
    recovery.open().await.unwrap();
    recovery
        .write_entries(&[RecoveryEntry {
            required_data: StorageCursor {
                history: history(),
                sequence: 0,
            },
            position: RecoveryPosition {
                history: history(),
                sequence: 1,
            },
            payload: b"durable transition".to_vec(),
        }])
        .await
        .unwrap();
    recovery.close().await.unwrap();
    backend.close().await.unwrap();

    let local_path = root.path().join("local.sqlite");
    let connection = rusqlite::Connection::open(&local_path).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        connection
            .query_row("SELECT format FROM storage_data_head", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "dokime.sqlite.storage.v1"
    );
    connection
        .execute(
            "UPDATE storage_data_head SET format='dokime.sqlite.storage.v3'",
            [],
        )
        .unwrap();
    drop(connection);
    let local_bytes = std::fs::read(&local_path).unwrap();
    assert!(!root.path().join("recovery.sqlite").exists());
    let hash: String = reference
        .sha256
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let payload_path = root.path().join("files").join(hash);
    let payload_bytes = std::fs::read(&payload_path).unwrap();

    assert!(matches!(
        State::open(root.path()).await,
        Err(Error::Data("unsupported SQLite data format"))
    ));
    assert_eq!(std::fs::read(&local_path).unwrap(), local_bytes);
    assert_eq!(std::fs::read(&payload_path).unwrap(), payload_bytes);

    // A Refused Open Releases The Root Lease
    let connection = rusqlite::Connection::open(&local_path).unwrap();
    connection
        .execute(
            "UPDATE storage_data_head SET format='dokime.sqlite.storage.v1'",
            [],
        )
        .unwrap();
    drop(connection);
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 1);
    let recovery = backend.state(StateLimits::default());
    assert_eq!(recovery.open().await.unwrap().sequence, 1);
    recovery.close().await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn refuses_separate_database() {
    let root = tempfile::tempdir().unwrap();
    // Initialize The Shared Allocator Before Opening A Fixture Connection
    let allocator_root = tempfile::tempdir().unwrap();
    State::open(allocator_root.path())
        .await
        .unwrap()
        .close()
        .await
        .unwrap();
    let recovery_path = root.path().join("recovery.sqlite");
    let connection = rusqlite::Connection::open(&recovery_path).unwrap();
    connection
        .execute_batch(
            "PRAGMA user_version=2; CREATE TABLE evidence(payload BLOB NOT NULL);
         INSERT INTO evidence VALUES (X'000102ff');",
        )
        .unwrap();
    drop(connection);
    let original = std::fs::read(&recovery_path).unwrap();

    assert!(matches!(
        State::open(root.path()).await,
        Err(Error::Conflict(_))
    ));
    assert_eq!(std::fs::read(&recovery_path).unwrap(), original);
    assert!(!root.path().join("local.sqlite").exists());
}
