/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Recover the Newest Unexpired Mutation Receipts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    error,
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, SystemTime},
};

// Third Party Imports
use harmos_storage::execution::Execution;
use harmos_storage::{
    Error, Identifier, MemorySize, MutationId, MutationItem, MutationReceipt, MutationValue, ObjectCatalog, ReadLimits, StorageBatch,
    StorageCursor, StorageFilesystemSettings, StorageManager, StoragePolicy, StorageRequestSettings, StorageResponseSettings,
    StorageSettings,
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
async fn controlled_receipts_survive_reopen_until_their_execution_clock_expires_them() -> Result<(), Box<dyn error::Error>> {
    for seconds in [100, 4_000_000_000] {
        // Share One Controlled Execution Between Receipt Stamping and Recovery.
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let catalog = persisted::catalog();
        let (clock, controller) = Execution::virtual_at(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds));
        let capacity = NonZeroUsize::new(64).unwrap();
        let bytes = NonZeroUsize::new(1024 * 1024).unwrap();
        let settings = StorageSettings {
            transfer: Default::default(),
            policy: StoragePolicy::default(),
            request: StorageRequestSettings {
                max_file_productions: capacity,
                max_mutation_receipts: capacity,
                max_pending_queries: capacity,
                max_active_queries: capacity,
                max_pending_write_batches: capacity,
                max_active_file_operations: capacity,
                max_pending_write_bytes: bytes,
            },
            response: StorageResponseSettings { batch_rows: capacity, batch_bytes: bytes },
            filesystem: StorageFilesystemSettings {
                poll_interval: Duration::from_millis(100),
                max_entries: capacity,
                event_capacity: capacity,
            },
        };
        let (backends, _lease) = Backend::open_with_execution(&store, catalog.clone(), clock.clone())?;
        let primary = backends.primary.clone();
        let storage = StorageManager::with_catalog_execution(catalog.clone(), backends, settings.clone(), clock.clone())?;
        storage
            .start_with(Arc::new(()), |mut worker| {
                let task = tokio::spawn(async move {
                    worker.run().await?;
                    worker.cleanup().await
                });
                Ok(Box::new(StorageWorker(Mutex::new(Some(task)))))
            })
            .await?;

        // Persist an Original Mutation Result with Library-Stamped Expiry.
        let mutation = MutationId {
            window: storage.mutation_window()?.id,
            key: Identifier::new("controlled-replay")?,
        };
        let value = Administration {
            key: "controlled-replay".into(),
            scope: Scope::Root("receipt-history").identifier()?,
            payload: vec![1],
        };
        let items = vec![MutationItem::Create(catalog.encode(&value, bytes)?)];
        let outcome = storage.mutate_once(&mutation, items.clone()).await?;
        let limits = ReadLimits { rows: capacity, bytes };
        let receipts = primary.list_mutation_receipts(limits).await?;
        assert_eq!(receipts.len(), 1);
        let expires_at = receipts[0].expires_at;
        storage.close().await?;

        // Reopen Just Before Expiry and Replay the Original Result.
        controller.advance_by(expires_at.duration_since(clock.date())? - Duration::from_nanos(1)).await;
        let (backends, _reopened_lease) = Backend::open_with_execution(&store, catalog.clone(), clock.clone())?;
        let primary = backends.primary.clone();
        let storage = StorageManager::with_catalog_execution(catalog, backends, settings, clock.clone())?;
        storage
            .start_with(Arc::new(()), |mut worker| {
                let task = tokio::spawn(async move {
                    worker.run().await?;
                    worker.cleanup().await
                });
                Ok(Box::new(StorageWorker(Mutex::new(Some(task)))))
            })
            .await?;
        assert_eq!(storage.mutate_once(&mutation, items).await?, outcome);
        assert_eq!(primary.list_mutation_receipts(limits).await?, receipts);
        storage.close().await?;

        // Advance Beyond Expiry and Verify Recovery Physically Prunes the Receipt.
        controller.advance_by(Duration::from_nanos(2)).await;
        let (backends, _expired_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), clock)?;
        backends.primary.create_session().await?;
        assert!(backends.primary.list_mutation_receipts(limits).await?.is_empty());
        let connection = Connection::open(root.path().join("storage.sqlite3"))?;
        let count: i64 = connection.query_row("SELECT count(*) FROM storage_mutations", [], |row| row.get(0))?;
        assert_eq!(count, 0);
        backends.primary.delete_session().await?;
    }
    Ok(())
}

#[tokio::test]
async fn keyed_replay_recovers_an_outcome_omitted_from_the_cache() -> Result<(), Box<dyn error::Error>> {
    // Open a Manager With Two Receipt Slots and a Controlled Backend Clock.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let catalog = persisted::catalog();
    let (clock, _controller) = Execution::virtual_at(SystemTime::UNIX_EPOCH + Duration::from_secs(100));
    let capacity = NonZeroUsize::new(64).unwrap();
    let bytes = NonZeroUsize::new(1024 * 1024).unwrap();
    let settings = StorageSettings {
        transfer: Default::default(),
        policy: StoragePolicy::default(),
        request: StorageRequestSettings {
            max_file_productions: capacity,
            max_mutation_receipts: NonZeroUsize::new(2).unwrap(),
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
    let (backends, _lease) = Backend::open_with_execution(&store, catalog.clone(), clock.clone())?;
    let primary = backends.primary.clone();
    let storage = StorageManager::with_catalog_execution(catalog.clone(), backends, settings.clone(), clock.clone())?;
    storage
        .start_with(Arc::new(()), |mut worker| {
            let task = tokio::spawn(async move {
                worker.run().await?;
                worker.cleanup().await
            });
            Ok(Box::new(StorageWorker(Mutex::new(Some(task)))))
        })
        .await?;

    // Commit Three Distinct Outcomes Under One Unchanged Window.
    let window = storage.mutation_window()?;
    let mut committed = Vec::new();
    for index in 0..3 {
        let value = Administration {
            key: index.to_string(),
            scope: Scope::Root("receipt-history").identifier()?,
            payload: vec![index],
        };
        let mutation = MutationId {
            window: window.id.clone(),
            key: Identifier::new(index.to_string())?,
        };
        let items = vec![MutationItem::Create(catalog.encode(&value, bytes)?)];
        let outcome = storage.mutate_once(&mutation, items.clone()).await?;
        committed.push((mutation, items, outcome));
    }
    assert_eq!(storage.mutation_window()?, window);
    let limits = ReadLimits {
        rows: NonZeroUsize::new(2).unwrap(),
        bytes,
    };
    let recovered = primary.list_mutation_receipts(limits).await?;
    assert_eq!(recovered.len(), 2);
    let (mutation, items, original) = committed.iter().find(|(id, _, _)| !recovered.iter().any(|receipt| &receipt.mutation == id)).unwrap();
    let accepted = storage.get_saved_position();
    storage.close().await?;

    // Reopen and Replay the Omitted Receipt Without Executing Its Create Again.
    let (backends, _reopened_lease) = Backend::open_with_execution(&store, catalog.clone(), clock.clone())?;
    let storage = StorageManager::with_catalog_execution(catalog, backends, settings, clock)?;
    storage
        .start_with(Arc::new(()), |mut worker| {
            let task = tokio::spawn(async move {
                worker.run().await?;
                worker.cleanup().await
            });
            Ok(Box::new(StorageWorker(Mutex::new(Some(task)))))
        })
        .await?;
    assert_eq!(storage.mutate_once(mutation, items.clone()).await?, *original);
    assert_eq!(storage.get_saved_position(), accepted);
    let unknown = MutationId {
        key: Identifier::new("unknown-old-window")?,
        ..mutation.clone()
    };
    assert!(matches!(storage.mutate_once(&unknown, items.clone()).await, Err(Error::InvalidInput { field: "storage mutation", .. })));
    storage.close().await?;
    Ok(())
}

#[tokio::test]
async fn expiry_index_reconstruction_preserves_keyed_receipts() -> Result<(), Box<dyn error::Error>> {
    // Seed the Existing Receipt Format With Unexpired and Expired Outcomes.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
    let (clock, _controller) = Execution::virtual_at(now);
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), clock.clone())?;
    let primary = &backends.primary;
    primary.create_session().await?;
    let receipt = MutationReceipt {
        mutation: MutationId {
            window: Identifier::new("window")?,
            key: Identifier::new("retained")?,
        },
        fingerprint: [1; 32],
        expires_at: now + Duration::from_secs(10),
        value: MutationValue::Cursor(StorageCursor {
            history: primary.get_history().clone(),
            sequence: 0,
        }),
    };
    let expired = MutationReceipt {
        mutation: MutationId {
            key: Identifier::new("expired")?,
            ..receipt.mutation.clone()
        },
        expires_at: now,
        ..receipt.clone()
    };
    primary.append_mutation_receipts(StorageBatch::new(vec![receipt.clone(), expired.clone()])).await?;
    primary.delete_session().await?;

    // Reconstruct the Existing Expiry Index When Reopening an Older Root.
    let connection = Connection::open(root.path().join("storage.sqlite3"))?;
    connection.execute_batch(
        "DROP INDEX storage_mutation_expiry;
        ALTER TABLE storage_mutations DROP COLUMN expires_at_s;
        ALTER TABLE storage_mutations DROP COLUMN expires_at_ns;",
    )?;
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), clock)?;
    let primary = &reopened.primary;
    primary.create_session().await?;
    assert_eq!(primary.get_mutation_receipt(&receipt.mutation).await?, Some(receipt.clone()));
    assert!(primary.get_mutation_receipt(&expired.mutation).await?.is_none());
    let limits = ReadLimits {
        rows: NonZeroUsize::MIN,
        bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
    };
    assert_eq!(primary.list_mutation_receipts(limits).await?, vec![receipt]);
    primary.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn receipt_recovery_stops_at_memory_bounds_and_prunes_with_retention() -> Result<(), Box<dyn error::Error>> {
    // Use One Fixed Clock for Receipt Expiry and Backend Retention.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
    let (clock, _controller) = Execution::virtual_at(now);
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), clock.clone())?;
    let primary = &backends.primary;
    primary.create_session().await?;
    let newest = MutationReceipt {
        mutation: MutationId {
            window: Identifier::new("window")?,
            key: Identifier::new("newest")?,
        },
        fingerprint: [0; 32],
        expires_at: now + Duration::from_secs(10),
        value: MutationValue::Cursor(StorageCursor {
            history: primary.get_history().clone(),
            sequence: 0,
        }),
    };
    let middle = MutationReceipt {
        mutation: MutationId {
            key: Identifier::new("middle".repeat(100))?,
            ..newest.mutation.clone()
        },
        expires_at: newest.expires_at - Duration::from_nanos(100),
        ..newest.clone()
    };
    let oldest = MutationReceipt {
        mutation: MutationId {
            key: Identifier::new("oldest")?,
            ..newest.mutation.clone()
        },
        expires_at: middle.expires_at - Duration::from_nanos(100),
        ..newest.clone()
    };
    primary.append_mutation_receipts(StorageBatch::new(vec![oldest.clone(), newest.clone(), middle.clone()])).await?;

    // Stop Before the First Oversized Receipt Without Skipping It.
    let one_receipt = size_of::<Vec<MutationReceipt>>() + newest.memory_size();
    let limits = ReadLimits {
        rows: NonZeroUsize::new(3).unwrap(),
        bytes: NonZeroUsize::new(one_receipt).unwrap(),
    };
    assert_eq!(primary.list_mutation_receipts(limits).await?, vec![newest.clone()]);
    assert_eq!(primary.get_mutation_receipt(&oldest.mutation).await?, Some(oldest.clone()));
    let unknown = MutationId {
        key: Identifier::new("unknown")?,
        ..newest.mutation.clone()
    };
    assert!(primary.get_mutation_receipt(&unknown).await?.is_none());
    let too_small = ReadLimits {
        bytes: NonZeroUsize::new(one_receipt - 1).unwrap(),
        ..limits
    };
    assert!(primary.list_mutation_receipts(too_small).await?.is_empty());
    let stop_at_middle = ReadLimits {
        bytes: NonZeroUsize::new(one_receipt + oldest.memory_size()).unwrap(),
        ..limits
    };
    assert_eq!(primary.list_mutation_receipts(stop_at_middle).await?, vec![newest.clone()]);

    // Respect Nanosecond Expiry Order and Keep Unselected Unexpired History.
    let full = ReadLimits {
        bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
        ..limits
    };
    assert_eq!(primary.list_mutation_receipts(full).await?, vec![newest.clone(), middle.clone(), oldest.clone()]);
    let connection = Connection::open(root.path().join("storage.sqlite3"))?;
    let count: i64 = connection.query_row("SELECT count(*) FROM storage_mutations", [], |row| row.get(0))?;
    assert_eq!(count, 3);

    // Reopen Without Decoding an Older Receipt Outside the Requested Row Bound.
    connection.execute("UPDATE storage_mutations SET data=X'00' WHERE id=?1", [serde_json::to_string(&oldest.mutation)?])?;
    primary.delete_session().await?;
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), clock)?;
    let primary = &reopened.primary;
    primary.create_session().await?;
    let one_row = ReadLimits { rows: NonZeroUsize::MIN, ..full };
    assert_eq!(primary.list_mutation_receipts(one_row).await?, vec![newest.clone()]);
    assert_eq!(primary.get_mutation_receipt(&middle.mutation).await?, Some(middle));
    assert!(primary.get_mutation_receipt(&oldest.mutation).await.is_err());

    // Remove Expired Receipts Through the Library's Existing Retention Hook.
    let expired = MutationReceipt {
        mutation: MutationId {
            key: Identifier::new("expired")?,
            ..newest.mutation.clone()
        },
        expires_at: now,
        ..newest
    };
    primary.append_mutation_receipts(StorageBatch::new(vec![expired.clone()])).await?;
    assert!(primary.get_mutation_receipt(&expired.mutation).await?.is_none());
    let through = primary.get_saved_position().await?;
    assert!(primary.delete_expired_object_revisions(&through, NonZeroUsize::MIN, &ObjectCatalog::new()).await?);
    let count: i64 =
        connection.query_row("SELECT count(*) FROM storage_mutations WHERE id=?1", [serde_json::to_string(&expired.mutation)?], |row| {
            row.get(0)
        })?;
    assert_eq!(count, 0);
    let count: i64 = connection.query_row("SELECT count(*) FROM storage_mutations", [], |row| row.get(0))?;
    assert_eq!(count, 3);
    primary.delete_session().await?;
    Ok(())
}
