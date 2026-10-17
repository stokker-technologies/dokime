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
use super::storage::{Store, id, key, page, read_records, rows, write_batch};
use super::{Error, State};

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
    store.backend.write(write_batch(&[StoragePart::Schemas(&[store.schema()])])).await.unwrap();
    let records = (2..=last).map(|sequence| store.record(sequence)).collect::<Vec<_>>();
    store.backend.write(write_batch(&[StoragePart::Records(&records)])).await.unwrap();
    (store, records)
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
    let selection = query::RecordSelection { schema: key(), scope: None, item: None, source_start: None, source_end: None };
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
    let saved = backend.write(write_batch(&[StoragePart::Records(&records)])).await.unwrap();
    assert_eq!(saved, store.at(4));
    let mut conflicting = records.clone();
    conflicting[1].payload = vec![8, 99];
    assert!(backend.write(write_batch(&[StoragePart::Records(&conflicting)])).await.is_err());
    assert_eq!(backend.cursor_saved().await.unwrap(), store.at(4));
    assert_eq!(store.records(4).await, records);
    backend.close().await.unwrap();
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
    backend.write_file(&staging, 0, &bytes[..4]).await.unwrap();
    assert!(backend.write_file(&staging, 9, b"gap").await.is_err());
    backend.write_file(&staging, 4, &bytes[4..]).await.unwrap();
    assert!(backend.finish_file(&staging, &store.at(1), bytes.len() as u64, [0; 32]).await.is_err());
    let file = backend.finish_file(&staging, &store.at(1), bytes.len() as u64, digest).await.unwrap();
    assert_eq!(backend.finish_file(&staging, &store.at(1), bytes.len() as u64, digest).await.unwrap(), file, "finishing is idempotent");
    let aborted = backend.create_file(&file_metadata("aborted", b"x")).await.unwrap();
    backend.write_file(&aborted, 0, b"x").await.unwrap();
    backend.abort_file(&aborted).await.unwrap();
    backend.write(write_batch(&[StoragePart::Files(std::slice::from_ref(&file))])).await.unwrap();

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

// A read whose caller is cancelled after dispatch neither poisons nor
// interrupts the reads that follow it.
#[tokio::test]
async fn cancelled_read_isolated() {
    let root = tempfile::tempdir().unwrap();
    let (store, records) = seeded(root.path(), 401).await;
    let backend = &store.backend;
    let snapshot = backend.open_snapshot(&store.at(401)).await.unwrap();
    let selection = query::RecordSelection { schema: key(), scope: None, item: None, source_start: None, source_end: None };
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
