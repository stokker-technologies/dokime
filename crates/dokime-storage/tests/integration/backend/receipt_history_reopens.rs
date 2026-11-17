/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reopen Storage After Receipt History Crosses One Page

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error, num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use harmos_storage::{
    Identifier, MutationId, MutationItem, ObjectVersion, StorageFilesystemSettings, StorageManager, StoragePolicy, StorageRequestSettings,
    StorageResponseSettings, StorageSettings,
};
use rusqlite::Connection;
use tokio::sync::Mutex;

// Local Imports
use super::StorageWorker;
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
async fn receipt_history_reopens_beyond_one_recovery_page() -> Result<(), Box<dyn error::Error>> {
    // Configure the Smallest Receipt Page.
    let root = tempfile::tempdir()?;
    let catalog = persisted::catalog();
    let capacity = NonZeroUsize::new(64).unwrap();
    let bytes = NonZeroUsize::new(1024 * 1024).unwrap();
    let settings = StorageSettings {
        transfer: Default::default(),
        policy: StoragePolicy::default(),
        request: StorageRequestSettings {
            max_file_productions: capacity,
            max_mutation_receipts: NonZeroUsize::MIN,
            max_pending_queries: capacity,
            max_active_queries: capacity,
            max_pending_write_batches: capacity,
            max_active_file_operations: capacity,
            max_pending_write_bytes: bytes,
        },
        response: StorageResponseSettings {
            batch_rows: NonZeroUsize::MIN,
            batch_bytes: bytes,
        },
        filesystem: StorageFilesystemSettings {
            poll_interval: Duration::from_millis(100),
            max_entries: capacity,
            event_capacity: capacity,
        },
    };

    // Write, Close and Reopen at the Page Boundary, Then Cross It by One Receipt.
    for index in 0..=2 {
        let store = Store::open(root.path())?;
        let (backends, _lease) = Backend::open(&store, catalog.clone())?;
        let storage = StorageManager::with_catalog(catalog.clone(), backends, settings.clone())?;
        storage
            .start_with(Arc::new(()), |mut worker| {
                let task = tokio::spawn(async move {
                    worker.run().await?;
                    worker.cleanup().await
                });
                Ok(Box::new(StorageWorker(Mutex::new(Some(task)))))
            })
            .await?;

        // Read Every Object Written Before This Reopen.
        for previous in 0..index {
            let key = previous.to_string();
            let stored = storage.get_object::<Administration>(&key, ObjectVersion::Latest).await?.unwrap();
            assert_eq!(stored.value.payload, vec![previous]);
        }
        if index == 2 {
            storage.close().await?;
            break;
        }

        // Persist One Minimal Object and Its Mutation Receipt.
        let value = Administration {
            key: index.to_string(),
            scope: Scope::Root("receipt-history").identifier()?,
            payload: vec![index],
        };
        let mutation = MutationId {
            window: storage.mutation_window()?.id,
            key: Identifier::new(index.to_string())?,
        };
        let items = vec![MutationItem::Create(catalog.encode(&value, bytes)?)];
        storage.mutate_once(&mutation, items).await?;
        storage.close().await?;

        // Confirm the Saved History Crosses Rows While Remaining Below the Byte Bound.
        let connection = Connection::open(root.path().join("storage.sqlite3"))?;
        let (rows, encoded_bytes): (i64, i64) =
            connection.query_row("SELECT count(*),sum(length(data)) FROM storage_mutations", [], |row| Ok((row.get(0)?, row.get(1)?)))?;
        assert_eq!(rows, i64::from(index) + 1);
        assert!(usize::try_from(encoded_bytes)? <= bytes.get());
        eprintln!("Receipt History: {rows} Rows, {encoded_bytes} Encoded Bytes");
    }
    Ok(())
}
