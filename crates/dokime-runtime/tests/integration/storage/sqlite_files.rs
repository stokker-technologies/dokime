/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SQLite File Publication Proof

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::storage::sqlite::testing::*;
use harmos::storage::{FileMetadata, FileReference, StorageCursor};
use sha2::{Digest, Sha256};

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
async fn shared_content_requires_a_finished_live_owner() {
    let root = tempfile::tempdir().unwrap();
    let state = State::open_with(root.path(), dokime_runtime::config::local_storage_defaults()).await.unwrap();
    let head = state.data_open(None).await.unwrap();
    let metadata = |id: &str| FileMetadata {
        id: harmos::Identifier::new(id).unwrap(),
        scope: None,
        media_type: "application/octet-stream".into(),
        expected_length: None,
        expected_sha256: None,
    };
    let source = state.production_create(root.path().to_owned(), metadata("original")).await.unwrap();
    state.production_create(root.path().to_owned(), metadata("reused")).await.unwrap();
    let reference = FileReference {
        metadata: metadata("reused"),
        position: StorageCursor {
            history: head.history.clone(),
            sequence: 2,
        },
        length: 0,
        sha256: Sha256::digest([]).into(),
    };
    let proven = async |reference: FileReference| {
        state.call(move |connection| Ok(published_content(connection, &reference))).await.unwrap().unwrap()
    };
    assert!(!proven(reference.clone()).await);
    state
        .call(|connection| {
            connection.execute_batch(
                "CREATE TEMP TRIGGER refuse_finish BEFORE UPDATE OF finished ON storage_file_stages
                 BEGIN SELECT RAISE(ABORT,'injected publication failure'); END",
            )
        })
        .await
        .unwrap();
    let at = StorageCursor { history: head.history, sequence: 1 };
    assert!(state.data_finish_file(root.path().to_owned(), source.clone(), at.clone(), 0, reference.sha256).await.is_err());
    assert!(!proven(reference.clone()).await);
    state.call(|connection| connection.execute_batch("DROP TRIGGER refuse_finish")).await.unwrap();
    state.data_finish_file(root.path().to_owned(), source.clone(), at, 0, reference.sha256).await.unwrap();
    assert!(proven(reference.clone()).await);
    state.production_delete(root.path().to_owned(), source).await.unwrap();
    assert!(!proven(reference).await);
    state.close().await.unwrap();
}
