/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SQLite Backend Contract Scenarios: Paging, Retries, Durability, Integrity,
// Snapshots, Finished Files And Read Cancellation.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports
use harmos::storage::{query::FileSelection, *};
use sha2::{Digest, Sha256};

// Local Imports
use super::storage::{Store, id, key, page, read_records, rows};
use super::{Backend, Error, State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// A Store Holding The Fixture Schema And Records `2..=last`.
async fn seeded(root: &std::path::Path, last: u64) -> (Store, Vec<StoredRecord>) {
    let store = Store::<()>::open(root).await;
    store.backend.write_schemas(store.batch(vec![store.schema()])).await.unwrap();
    let records = (2..=last).map(|sequence| store.record(sequence)).collect::<Vec<_>>();
    store.backend.write_records(store.batch(records.clone())).await.unwrap();
    (store, records)
}

async fn exact_retry(store: &Store, records: &[StoredRecord], through: u64) {
    let saved = store.backend.write_records(store.batch(records.to_vec())).await.unwrap();
    assert_eq!(saved, store.at(through));
    let mut conflicting = records.to_vec();
    conflicting[1].payload = vec![8, 99];
    assert!(store.backend.write_records(store.batch(conflicting)).await.is_err());
    assert_eq!(store.backend.cursor_saved().await.unwrap(), store.at(through));
    assert_eq!(store.records(through).await, records);
}

fn file_metadata(name: &str, bytes: &[u8]) -> FileMetadata {
    FileMetadata {
        id: id(name),
        scope: Some(id("execution")),
        media_type: "text/plain".into(),
        expected_length: Some(bytes.len() as u64),
        expected_sha256: Some(Sha256::digest(bytes).into()),
    }
}

/* === Tests === */

// Pages ascend by position, always carry at least one entry, and an empty page
// means the selection is exhausted at the snapshot's frontier.
#[tokio::test]
async fn paged_records_ascend() {
    let root = tempfile::tempdir().unwrap();
    let (store, records) = seeded(root.path(), 7).await;
    let backend = &store.backend;
    let snapshot = backend.open_snapshot(&store.at(7)).await.unwrap();
    let selection = query::RecordSelection {
        schema: key(),
        scope: None,
        item: None,
        source_start: None,
        source_end: None,
    };
    let first = backend.read_records(&snapshot, &selection, page(0, rows(2))).await.unwrap();
    assert_eq!(first, records[..2]);
    let oversized = backend.read_records(&snapshot, &selection, ScanPage { after: 0, rows: rows(64), bytes: rows(1) }).await.unwrap();
    assert_eq!(oversized, records[..1], "the first entry always fits");
    assert_eq!(read_records(backend, &snapshot, rows(2)).await.unwrap(), records);
    assert!(backend.read_records(&snapshot, &selection, page(7, rows(2))).await.unwrap().is_empty());
    backend.release_snapshot(&snapshot).await.unwrap();
    assert_eq!(store.records(4).await, records[..3], "an earlier cut");
    backend.close().await.unwrap();
}

// An identical retry of a durable batch is accepted without change; a retry
// whose contents differ is refused and leaves the durable prefix untouched.
#[tokio::test]
async fn retries_are_exact() {
    let root = tempfile::tempdir().unwrap();
    let (store, records) = seeded(root.path(), 4).await;
    let backend = &store.backend;
    exact_retry(&store, &records, 4).await;
    backend.close().await.unwrap();
}

// A Late Batch Failure Rolls Back Its Earlier Rows And Leaves Cached Statements Reusable.
#[tokio::test]
async fn batch_rollback_retries() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    store.backend.write_schemas(store.batch(vec![store.schema()])).await.unwrap();

    let invalid = vec![store.record(2), store.record(4)];
    assert!(store.backend.write_records(store.batch(invalid)).await.is_err());
    assert_eq!(store.backend.cursor_saved().await.unwrap(), store.at(1));
    assert!(store.records(1).await.is_empty());

    let records = (2..=4).map(|sequence| store.record(sequence)).collect::<Vec<_>>();
    store.backend.write_records(store.batch(records[..1].to_vec())).await.unwrap();
    let saved = store.backend.write_records(store.batch(records.clone())).await.unwrap();
    assert_eq!(saved, store.at(4));
    assert_eq!(store.records(4).await, records);
    store.backend.close().await.unwrap();
    drop(store);

    let reopened = Store::<()>::open(root.path()).await;
    assert_eq!(reopened.backend.cursor_saved().await.unwrap(), reopened.at(4));
    assert_eq!(reopened.records(4).await, records);
    exact_retry(&reopened, &records, 4).await;
    reopened.backend.close().await.unwrap();
}

// Distinct Records Share An Item Without Weakening Position Retry Integrity.
#[tokio::test]
async fn records_share_item_with_exact_retries() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let schemas = [store.schema()];
    store.backend.write_schemas(store.batch(schemas.to_vec())).await.unwrap();

    let records = (2..=3)
        .map(|sequence| StoredRecord {
            item: id("shared-item"),
            ..store.record(sequence)
        })
        .collect::<Vec<_>>();
    for record in &records {
        store.backend.write_records(store.batch(vec![record.clone()])).await.unwrap();
    }
    let saved = store.backend.write_records(store.batch(records.clone())).await.unwrap();
    assert_eq!(saved, store.at(3));

    let snapshot = store.backend.open_snapshot(&store.at(3)).await.unwrap();
    let selection = query::RecordSelection {
        schema: key(),
        scope: None,
        item: Some(id("shared-item")),
        source_start: None,
        source_end: None,
    };
    assert_eq!(store.backend.read_records(&snapshot, &selection, page(0, rows(2))).await.unwrap(), records);
    store.backend.release_snapshot(&snapshot).await.unwrap();

    exact_retry(&store, &records, 3).await;
    store.backend.close().await.unwrap();
    drop(store);

    let reopened = Store::<()>::open(root.path()).await;
    assert_eq!(reopened.records(3).await, records);
    let saved = reopened.backend.write_records(reopened.batch(records.clone())).await.unwrap();
    assert_eq!(saved, reopened.at(3));
    reopened.backend.close().await.unwrap();
}

// A closed root reopens at its durable cursor with the same history and rows.
#[tokio::test]
async fn durable_reopen() {
    let root = tempfile::tempdir().unwrap();
    let (store, records) = seeded(root.path(), 6).await;
    let history = store.history.clone();
    store.backend.close().await.unwrap();
    drop(store);
    let reopened = Store::<()>::open(root.path()).await;
    assert_eq!(reopened.history, history);
    assert_eq!(reopened.backend.cursor_saved().await.unwrap(), reopened.at(6));
    assert_eq!(reopened.records(6).await, records);
    reopened.backend.close().await.unwrap();
}

#[tokio::test]
async fn bootstrap_identity_survives_before_admitted_open() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
    let history = backend.history().clone();
    let empty = StorageCursor { history: history.clone(), sequence: 0 };
    let complete =
        backend.prune_objects(&empty, std::num::NonZeroUsize::new(16).unwrap(), |_| Ok(ObjectRetention::UntilReleased)).await.unwrap();
    assert!(complete);
    let local = backend.local_state();
    let deferred = local
        .call(|connection| {
            let count = |row: &rusqlite::Row<'_>| row.get::<_, i64>(0);
            connection.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('storage_data_entries','storage_file_stages','storage_staging')",
                [],
                count,
            )
        })
        .await
        .unwrap();
    assert_eq!(deferred, 0);
    local.close().await.unwrap();
    drop(backend);

    let reopened = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
    assert_eq!(reopened.history(), &history);

    let at = reopened.open().await.unwrap();
    assert_eq!(at, StorageCursor { history, sequence: 0 });
    let initialized = reopened
        .local_state()
        .call(|connection| {
            let count = |row: &rusqlite::Row<'_>| row.get::<_, i64>(0);
            connection.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('storage_data_entries','storage_file_stages','storage_staging')",
                [],
                count,
            )
        })
        .await
        .unwrap();
    assert_eq!(initialized, 3);
    reopened.local_state().close().await.unwrap();
}

#[tokio::test]
async fn bootstrap_defers_saved_sequence_validation_to_admitted_open() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
    let history = backend.history().clone();
    backend.local_state().call(|connection| connection.execute("UPDATE storage_data_head SET saved='zzzzzzzzzzzzzzzz'", [])).await.unwrap();
    backend.local_state().close().await.unwrap();
    drop(backend);

    let reopened = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
    assert_eq!(reopened.history(), &history);

    assert!(reopened.open().await.is_err());
    let saved = reopened
        .local_state()
        .call(|connection| {
            let text = |row: &rusqlite::Row<'_>| row.get::<_, String>(0);
            connection.query_row("SELECT saved FROM storage_data_head", [], text)
        })
        .await
        .unwrap();
    assert_eq!(saved, "zzzzzzzzzzzzzzzz");
    reopened.local_state().close().await.unwrap();
}

#[tokio::test]
async fn bootstrap_rejects_invalid_history_without_initializing_data() {
    for bytes in [br#"{"store":"store","epoch":"epoch","extra":0}"#.to_vec(), vec![b' '; 4097]] {
        let root = tempfile::tempdir().unwrap();
        let backend = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
        let changed = bytes.clone();
        backend
            .local_state()
            .call(move |connection| connection.execute("UPDATE storage_data_head SET history=?1", [changed]))
            .await
            .unwrap();
        backend.local_state().close().await.unwrap();
        drop(backend);

        assert!(Backend::<()>::primary(root.path().to_path_buf()).await.is_err());
        let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
        let blob = |row: &rusqlite::Row<'_>| row.get::<_, Vec<u8>>(0);
        let retained = connection.query_row("SELECT history FROM storage_data_head", [], blob).unwrap();
        assert_eq!(retained, bytes);
        let count = |row: &rusqlite::Row<'_>| row.get::<_, i64>(0);
        let initialized = connection.query_row("SELECT COUNT(*) FROM sqlite_master WHERE name='storage_data_entries'", [], count).unwrap();
        assert_eq!(initialized, 0);
    }
}

#[tokio::test]
async fn bootstrap_refuses_missing_head_with_saved_entries() {
    let root = tempfile::tempdir().unwrap();
    let (store, records) = seeded(root.path(), 2).await;
    let local = store.backend.local_state();
    local.call(|connection| connection.execute("DELETE FROM storage_data_head", [])).await.unwrap();
    store.backend.close().await.unwrap();
    drop(store);

    assert!(Backend::<()>::primary(root.path().to_path_buf()).await.is_err());
    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    let count = |row: &rusqlite::Row<'_>| row.get::<_, i64>(0);
    let heads = connection.query_row("SELECT COUNT(*) FROM storage_data_head", [], count).unwrap();
    let entries = connection.query_row("SELECT COUNT(*) FROM storage_data_entries", [], count).unwrap();
    assert_eq!(heads, 0);
    assert_eq!(entries, 1 + records.len() as i64);
}

// A stored entry whose digest no longer matches its body is refused on read.
#[tokio::test]
async fn corrupt_entry_refused() {
    let root = tempfile::tempdir().unwrap();
    let (store, _) = seeded(root.path(), 4).await;
    store
        .backend
        .local_state()
        .call(|connection| {
            let changed = connection.execute("UPDATE storage_data_entries SET digest=zeroblob(32) WHERE kind='record'", [])?;
            assert_eq!(changed, 3);
            Ok(())
        })
        .await
        .unwrap();
    let snapshot = store.backend.open_snapshot(&store.at(4)).await.unwrap();
    let error = read_records(&store.backend, &snapshot, rows(2)).await.unwrap_err();
    assert!(format!("{error:?}").contains("stored entry digest differs"), "{error:?}");
    store.backend.release_snapshot(&snapshot).await.unwrap();
    store.backend.close().await.unwrap();
}

// Snapshots pin a saved cut: a released snapshot reads nothing, and no
// snapshot opens beyond the saved frontier.
#[tokio::test]
async fn snapshots_pin_cuts() {
    let root = tempfile::tempdir().unwrap();
    let (store, _) = seeded(root.path(), 3).await;
    let backend = &store.backend;
    assert!(backend.open_snapshot(&store.at(4)).await.is_err());
    let snapshot = backend.open_snapshot(&store.at(3)).await.unwrap();
    backend.release_snapshot(&snapshot).await.unwrap();
    assert!(read_records(backend, &snapshot, rows(2)).await.is_err());
    backend.close().await.unwrap();
}

// A finished file is staged, verified against its declared digest, and
// readable once its reference is written; an aborted staging leaves nothing.
#[tokio::test]
async fn finished_files_publish() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let bytes = b"charger evidence";
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    let staging = backend.create_file(&file_metadata("trace", bytes)).await.unwrap();
    backend.write_file(&staging, 0, store.batch(bytes[..4].to_vec())).await.unwrap();
    assert!(backend.write_file(&staging, 9, store.batch(b"gap".to_vec())).await.is_err());
    backend.write_file(&staging, 4, store.batch(bytes[4..].to_vec())).await.unwrap();
    assert!(backend.finish_file(&staging, &store.at(1), bytes.len() as u64, [0; 32]).await.is_err());
    let file = backend.finish_file(&staging, &store.at(1), bytes.len() as u64, digest).await.unwrap();
    assert_eq!(backend.finish_file(&staging, &store.at(1), bytes.len() as u64, digest).await.unwrap(), file, "finishing is idempotent");
    let aborted = backend.create_file(&file_metadata("aborted", b"x")).await.unwrap();
    backend.write_file(&aborted, 0, store.batch(b"x".to_vec())).await.unwrap();
    backend.abort_file(&aborted).await.unwrap();
    backend.write_files(store.batch(vec![file.clone()])).await.unwrap();

    let snapshot = backend.open_snapshot(&store.at(1)).await.unwrap();
    let listed = backend.read_files(&snapshot, &FileSelection { scope: None, item: None }, page(0, rows(8))).await.unwrap();
    assert_eq!(listed, std::slice::from_ref(&file));
    assert_eq!(backend.read_file(&snapshot, &file, 8, rows(100)).await.unwrap(), &bytes[8..]);
    backend.release_snapshot(&snapshot).await.unwrap();
    backend.close().await.unwrap();
    drop(store);

    let reopened = Store::<()>::open(root.path()).await;
    let snapshot = reopened.backend.open_snapshot(&reopened.at(1)).await.unwrap();
    assert_eq!(reopened.backend.read_file(&snapshot, &file, 0, rows(100)).await.unwrap(), bytes);
    reopened.backend.release_snapshot(&snapshot).await.unwrap();
    reopened.backend.close().await.unwrap();
}

// File Finish And Publication Preserve Independent SHA-256 Digests.
// Invalid Claims Do Not Bind The File Identity, Including At Block Boundaries.
#[tokio::test]
async fn file_digests_match() {
    for length in [0, 1, 63, 64, 65, 65_549] {
        let root = tempfile::tempdir().unwrap();
        let store = Store::<()>::open(root.path()).await;
        let backend = &store.backend;
        let bytes = (0..length).map(|index| (index % 251) as u8).collect::<Vec<_>>();
        let expected: [u8; 32] = Sha256::digest(&bytes).into();
        let mut metadata = file_metadata("digest", &bytes);
        metadata.expected_sha256 = None;
        let staged = backend.create_file(&metadata).await.unwrap();
        let split = bytes.len().min(31);
        if split > 0 {
            backend.write_file(&staged, 0, store.batch(bytes[..split].to_vec())).await.unwrap();
        }
        if split < bytes.len() {
            backend.write_file(&staged, split as u64, store.batch(bytes[split..].to_vec())).await.unwrap();
        }

        assert!(backend.finish_file(&staged, &store.at(1), length as u64, [0; 32]).await.is_err());
        let file = backend.finish_file(&staged, &store.at(1), length as u64, expected).await.unwrap();
        assert_eq!(file.sha256, expected);
        backend.write_files(store.batch(vec![file.clone()])).await.unwrap();
        let snapshot = backend.open_snapshot(&store.at(1)).await.unwrap();
        assert_eq!(backend.read_file(&snapshot, &file, 0, rows(bytes.len().max(1))).await.unwrap(), bytes);
        backend.release_snapshot(&snapshot).await.unwrap();
        backend.close().await.unwrap();
    }
}

// A read whose caller is cancelled after dispatch neither poisons nor
// interrupts the reads that follow it.
#[tokio::test]
async fn cancelled_read_isolated() {
    let root = tempfile::tempdir().unwrap();
    let (store, records) = seeded(root.path(), 401).await;
    let backend = &store.backend;
    let snapshot = backend.open_snapshot(&store.at(401)).await.unwrap();
    let selection = query::RecordSelection {
        schema: key(),
        scope: None,
        item: None,
        source_start: None,
        source_end: None,
    };
    for _ in 0..8 {
        let mut cancelled = Box::pin(backend.read_records(&snapshot, &selection, page(0, rows(400))));
        std::future::poll_fn(|context| {
            let _ = cancelled.as_mut().poll(context);
            std::task::Poll::Ready(())
        })
        .await;
        drop(cancelled);
        assert_eq!(read_records(backend, &snapshot, rows(64)).await.unwrap(), records);
    }
    backend.release_snapshot(&snapshot).await.unwrap();
    backend.close().await.unwrap();
}

// The backend owns its root until closed; closing releases the local owner.
#[tokio::test]
async fn close_releases_owner() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    assert!(matches!(State::open(root.path()).await, Err(Error::Owned)));
    store.backend.close().await.unwrap();
    drop(store);
    State::open(root.path()).await.unwrap().close().await.unwrap();
}
