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
use rusqlite::{Connection, params};
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
async fn receipt_history_reopens_with_only_the_newest_unexpired_outcomes() -> Result<(), Box<dyn error::Error>> {
    for indexed_expiry in [true, false] {
        // Configure a Two-Receipt Replay Cache.
        let root = tempfile::tempdir()?;
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

        // Save More Receipts Than the Replay Cache Can Hold.
        let store = Store::open(root.path())?;
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
        let mut mutations = Vec::new();
        let mut requests = Vec::new();
        let mut outcomes = Vec::new();
        for index in 0..3 {
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
            outcomes.push(storage.mutate_once(&mutation, items.clone()).await?);
            mutations.push(mutation);
            requests.push(items);
        }

        // Seed an Expired Receipt Without Waiting for Retention to Elapse.
        let receipts = primary.list_mutation_receipts(ReadLimits { rows: capacity, bytes }).await?;
        let mut expired = receipts[0].clone();
        expired.mutation.key = Identifier::new("expired")?;
        expired.expires_at = clock.date();
        primary.append_mutation_receipts(StorageBatch::new(vec![expired])).await?;
        storage.close().await?;

        // Give Each Surviving Receipt a Distinct Expiry in Reverse Identity Order.
        let connection = Connection::open(root.path().join("storage.sqlite3"))?;
        for (index, mutation) in mutations.iter().enumerate() {
            let id = serde_json::to_string(mutation)?;
            let data: Vec<u8> = connection.query_row("SELECT data FROM storage_mutations WHERE id=?1", [&id], |row| row.get(0))?;
            let mut receipt: MutationReceipt = serde_json::from_slice(&data)?;
            receipt.expires_at = SystemTime::UNIX_EPOCH + Duration::from_secs(u64::try_from(200 - index)?);
            connection.execute(
                "UPDATE storage_mutations SET data=?2,expires_at_s=?3,expires_at_ns=0 WHERE id=?1",
                params![id, serde_json::to_vec(&receipt)?, i64::try_from(200 - index)?],
            )?;
        }

        let total: i64 = connection.query_row("SELECT count(*) FROM storage_mutations", [], |row| row.get(0))?;
        assert_eq!(total, 4);

        // Simulate the Existing Root Layout Before Expiry Columns Were Added.
        if !indexed_expiry {
            connection.execute_batch(
                "DROP INDEX storage_mutation_expiry;
            ALTER TABLE storage_mutations DROP COLUMN expires_at_s;
            ALTER TABLE storage_mutations DROP COLUMN expires_at_ns;",
            )?;
        }

        // Reopen and Replay Only the Two Newest Original Outcomes.
        let (backends, _reopened_lease) = Backend::open_with_execution(&store, catalog.clone(), clock.clone())?;
        let primary = backends.primary.clone();
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
        for index in [0, 1] {
            assert_eq!(storage.mutate_once(&mutations[index], requests[index].clone()).await?, outcomes[index]);
        }
        assert!(matches!(
            storage.mutate_once(&mutations[2], requests[2].clone()).await,
            Err(Error::InvalidInput { field: "storage mutation", .. })
        ));
        let remaining = primary.list_mutation_receipts(ReadLimits { rows: capacity, bytes }).await?;
        assert_eq!(remaining.len(), 3);
        let count: i64 = connection.query_row("SELECT count(*) FROM storage_mutations", [], |row| row.get(0))?;
        assert_eq!(count, 3);
        storage.close().await?;
    }
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
