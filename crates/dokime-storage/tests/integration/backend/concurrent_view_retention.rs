/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Concurrent File Publication and View Retention

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use harmos_storage::execution::Execution;
use harmos_storage::{
    CompareExchange, FileChunk, FileMetadata, FileReference, Identifier, MutationId, ObjectVersion, Result, StagedFile,
    StorageFilesystemSettings, StorageManager, StoragePolicy, StorageRequestSettings, StorageResponseSettings, StorageSettings,
};
use sha2::{Digest, Sha256};
use tokio::sync::{Barrier, Mutex};

// Local Imports
use super::{StorageWorker, StoryResult, limits};
use dokime_storage::persisted::{self, Administration, Scope};
use dokime_storage::{Backend, Store};

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
async fn concurrent_view_retention_converges_and_isolates_resources() -> StoryResult {
    // Start the Native Storage Worker with the Product Administration Catalog.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let catalog = persisted::catalog();
    let (backends, _lease) = Backend::open_with_execution(&store, catalog.clone(), Execution::default())?;
    let backend = backends.primary.clone();
    let capacity = NonZeroUsize::new(16).unwrap();
    let storage = StorageManager::with_catalog(
        catalog,
        backends,
        StorageSettings {
            transfer: Default::default(),
            policy: StoragePolicy::default(),
            request: StorageRequestSettings {
                max_file_productions: capacity,
                max_mutation_receipts: NonZeroUsize::new(128).unwrap(),
                max_pending_queries: capacity,
                max_active_queries: capacity,
                max_pending_write_batches: capacity,
                max_active_file_operations: capacity,
                max_pending_write_bytes: limits().bytes,
            },
            response: StorageResponseSettings {
                batch_rows: capacity,
                batch_bytes: limits().bytes,
            },
            filesystem: StorageFilesystemSettings {
                poll_interval: Duration::from_millis(100),
                max_entries: capacity,
                event_capacity: capacity,
            },
        },
    )?;
    storage
        .start_with(Arc::new(()), |mut worker| {
            let task = tokio::spawn(async move {
                worker.run().await?;
                worker.finalize().await
            });
            Ok(Box::new(StorageWorker(Mutex::new(Some(task)))))
        })
        .await?;
    let window = storage.mutation_window()?.id;
    let scope = Scope::Root("retention").identifier()?;

    // Retain Identical Content for One Resource, Then Different Content for Two Resources.
    for (scenario, keys, payloads) in [
        ("same", ["view:same:digest", "view:same:digest"], [b"same".as_slice(), b"same".as_slice()]),
        ("different", ["view:first:digest", "view:second:digest"], [b"first".as_slice(), b"second".as_slice()]),
    ] {
        let mutation = |action: &str| -> StoryResult<MutationId> {
            Ok(MutationId {
                window: window.clone(),
                key: Identifier::new(format!("{scenario}/{action}"))?,
            })
        };
        let metadata = |name: &str, payload: &[u8]| -> StoryResult<FileMetadata> {
            Ok(FileMetadata {
                id: Identifier::new(format!("view/{scenario}/{name}"))?,
                scope: Some(scope.clone()),
                media_type: "application/x-protobuf".into(),
                expected_length: Some(payload.len() as u64),
                expected_sha256: Some(Sha256::digest(payload).into()),
            })
        };

        // Capture the Empty Uploads Before Either View Appends Its Bytes.
        let first = storage.create_file_once(&mutation("create-first")?, &metadata("first", payloads[0])?).await?;
        let second = storage.create_file_once(&mutation("create-second")?, &metadata("second", payloads[1])?).await?;
        let older = storage.capture_files(None)?;

        // Append Both Views and Capture Their Complete Prefixes.
        storage.append_file_once(&mutation("append-first")?, &first.session, 0, FileChunk::new(payloads[0].to_vec())).await?;
        storage.append_file_once(&mutation("append-second")?, &second.session, 0, FileChunk::new(payloads[1].to_vec())).await?;
        let newer = storage.capture_files(None)?;
        let boundary = Barrier::new(2);

        // Let the Newer Publication's Checkpoint Overtake the Older Captured Batch.
        let persist_newer = async {
            storage.persist_files(&newer).await?;
            boundary.wait().await;
            Result::Ok(())
        };
        let persist_older = async {
            boundary.wait().await;
            storage.persist_files(&older).await
        };
        let (newer_result, older_result) = tokio::join!(persist_newer, persist_older);
        newer_result?;
        older_result?;

        // Publish Both Files Concurrently and Flush Their Native References.
        let first_mutation = mutation("publish-first")?;
        let second_mutation = mutation("publish-second")?;
        let first_digest = Sha256::digest(payloads[0]).into();
        let second_digest = Sha256::digest(payloads[1]).into();
        let (first, second) = tokio::join!(
            storage.publish_file_production_once(&first_mutation, &first.session, payloads[0].len() as u64, first_digest),
            storage.publish_file_production_once(&second_mutation, &second.session, payloads[1].len() as u64, second_digest),
        );
        let files = [first?, second?];
        storage.flush().await?;

        // Race the Product's Create-If-Absent Index Publications.
        let first_index = Administration {
            key: keys[0].into(),
            scope: scope.clone(),
            payload: serde_json::to_vec(&files[0])?,
        };
        let second_index = Administration {
            key: keys[1].into(),
            scope: scope.clone(),
            payload: serde_json::to_vec(&files[1])?,
        };
        let first_mutation = mutation("retain-first")?;
        let second_mutation = mutation("retain-second")?;
        let (first, second) = tokio::join!(
            storage.compare_exchange_once(&first_mutation, None, &first_index),
            storage.compare_exchange_once(&second_mutation, None, &second_index),
        );
        let outcomes = [first?, second?];
        let applied = outcomes.iter().filter(|outcome| matches!(outcome, CompareExchange::Applied(_))).count();
        assert_eq!(applied, if keys[0] == keys[1] { 1 } else { 2 });
        let mut retained = Vec::new();
        for (index, key) in keys.iter().enumerate() {
            let saved = storage.get_object::<Administration>(key, ObjectVersion::Latest).await?.unwrap();
            let file: FileReference = serde_json::from_slice(&saved.value.payload)?;
            if matches!(outcomes[index], CompareExchange::Conflict { .. }) {
                let unused = StagedFile {
                    history: files[index].position.history.clone(),
                    id: files[index].metadata.id.clone(),
                };
                storage.delete_file_once(&mutation("discard-unused")?, &unused).await?;
            }
            retained.push(file);
        }
        assert_eq!(retained[0] == retained[1], keys[0] == keys[1]);
        storage.persist_files(&older).await?;

        // Read Every Retained File After Duplicate Cleanup.
        let at = backend.get_saved_position().await?;
        let snapshot = backend.create_snapshot(&at).await?;
        for (file, payload) in retained.iter().zip(payloads) {
            assert_eq!(backend.get_file_bytes(&snapshot, file, 0, limits().bytes).await?, payload);
        }
        backend.delete_snapshot(&snapshot).await?;
    }

    // Close the Worker and Its Owned Backend Sessions.
    storage.close().await?;
    Ok(())
}
