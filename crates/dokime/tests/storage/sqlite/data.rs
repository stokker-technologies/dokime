/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod recovery;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::storage::*;
use sha2::{Digest, Sha256};

// Local Imports
use dokime::storage::sqlite::{Backend, Error, Limits};

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

fn bound(bytes: usize) -> NonZeroUsize {
    NonZeroUsize::new(bytes).unwrap()
}

fn record(history: &StoreHistory, sequence: u64, item: &str, bytes: &[u8]) -> StoredRecord {
    StoredRecord {
        position: StorageCursor {
            history: history.clone(),
            sequence,
        },
        schema: SchemaKey {
            payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
            id: harmos::Identifier::new("fixture.record").unwrap(),
            version: std::num::NonZeroU32::MIN,
        },
        scope: None,
        item: harmos::Identifier::new(item).unwrap(),
        source: None,
        source_time: None,
        payload: bytes.to_vec(),
    }
}

pub(crate) async fn native_views_owner() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let control = backend.control().await.unwrap();
    control
        .register_source("fabric", "microsoft.fabric")
        .await
        .unwrap();
    let saved = backend.cursor_saved().await.unwrap();
    let future = StorageCursor {
        history: saved.history.clone(),
        sequence: 1,
    };
    assert!(backend.read_view(&future).await.is_err());
    let pin = backend.read_view(&saved).await.unwrap();
    assert!(matches!(
        dokime::storage::sqlite::State::open(root.path()).await,
        Err(Error::Owned)
    ));
    backend.release_view(&pin).await.unwrap();
    drop(control);
    backend.close().await.unwrap();
    drop(backend);
    let reopened = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let pin = reopened.read_view(&saved).await.unwrap();
    reopened.release_view(&pin).await.unwrap();
    reopened.close().await.unwrap();
}

pub(crate) async fn view_release_reader() {
    let root = tempfile::tempdir().unwrap();
    let backend = Arc::new(
        Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
            .await
            .unwrap(),
    );
    let rows = [record(backend.history(), 1, "active.reader", b"original")];
    let parts = [StoragePart::Records(&rows)];
    let saved = backend
        .write(StorageBatch::new(&parts, bound(4096)).unwrap())
        .await
        .unwrap();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.active.reader").unwrap(),
        saved.clone(),
        saved,
        snapshot.clone(),
    )
    .unwrap();
    let control = backend.control().await.unwrap();
    let (entered, observed) = std::sync::mpsc::channel();
    let (resume, resumed) = std::sync::mpsc::channel();
    let blocker = control.clone();
    let blocked = tokio::spawn(async move {
        blocker
            .call(move |_| {
                entered.send(()).unwrap();
                resumed.recv().unwrap();
                Ok(())
            })
            .await
    });
    tokio::task::spawn_blocking(move || observed.recv().unwrap())
        .await
        .unwrap();
    let selection = ReadSelection::new(ReadKind::Records);
    let mut page = backend.read_records(
        &view,
        &selection,
        None,
        ReadLimits {
            rows: bound(1),
            bytes: bound(4096),
        },
    );
    assert!(futures_util::poll!(&mut page).is_pending());
    let mut released = backend.release_view(&snapshot);
    assert!(futures_util::poll!(&mut released).is_pending());
    resume.send(()).unwrap();
    blocked.await.unwrap().unwrap();
    assert_eq!(page.await.unwrap().entries, rows);
    released.await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn exact_retry_content() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let records = [record(backend.history(), 1, "retry.integrity", b"original")];
    let parts = [StoragePart::Records(&records)];
    let batch = StorageBatch::new(&parts, bound(4096)).unwrap();
    let saved = backend.write(batch).await.unwrap();
    let control = backend.control().await.unwrap();
    let original = control
        .call(|connection| {
            connection.query_row(
                "SELECT body FROM storage_data_entries WHERE position='0000000000000001'",
                [],
                |row| row.get::<_, Vec<u8>>(0),
            )
        })
        .await
        .unwrap();
    control
        .call(|connection| {
            connection.execute(
                "UPDATE storage_data_entries SET body=x'00' WHERE position='0000000000000001'",
                [],
            )
        })
        .await
        .unwrap();
    assert!(backend.write(batch).await.is_err());
    assert_eq!(backend.cursor_saved().await.unwrap(), saved);
    control
        .call(move |connection| {
            connection.execute(
                "UPDATE storage_data_entries SET body=?1 WHERE position='0000000000000001'",
                [original],
            )
        })
        .await
        .unwrap();
    assert_eq!(backend.write(batch).await.unwrap(), saved);
    control
        .call(|connection| {
            connection.execute(
                "UPDATE storage_data_entries SET kind='file' WHERE position='0000000000000001'",
                [],
            )
        })
        .await
        .unwrap();
    assert!(backend.write(batch).await.is_err());
    control
        .call(|connection| {
            connection.execute(
                "UPDATE storage_data_entries SET kind='record' WHERE position='0000000000000001'",
                [],
            )
        })
        .await
        .unwrap();
    assert_eq!(backend.write(batch).await.unwrap(), saved);
    control
        .call(|connection| {
            connection.execute(
                "UPDATE storage_data_head SET saved='0000000000000000' WHERE singleton=1",
                [],
            )
        })
        .await
        .unwrap();
    assert!(backend.write(batch).await.is_err());
    control
        .call(|connection| {
            connection.execute(
                "UPDATE storage_data_head SET saved='0000000000000001' WHERE singleton=1",
                [],
            )
        })
        .await
        .unwrap();
    assert_eq!(backend.write(batch).await.unwrap(), saved);
    drop(control);
    backend.close().await.unwrap();
}

pub(crate) async fn conflicting_retry_prefix() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let first = [record(backend.history(), 1, "first", b"original")];
    let parts = [StoragePart::Records(&first)];
    let batch = StorageBatch::new(&parts, bound(4096)).unwrap();
    let saved = backend.write(batch).await.unwrap();
    assert_eq!(backend.write(batch).await.unwrap(), saved);
    let next = [
        record(backend.history(), 2, "second", b"new"),
        record(backend.history(), 3, "first", b"conflict"),
    ];
    let parts = [StoragePart::Records(&next)];
    assert!(
        backend
            .write(StorageBatch::new(&parts, bound(4096)).unwrap())
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 1);
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.retry").unwrap(),
        saved.clone(),
        saved,
        snapshot.clone(),
    )
    .unwrap();
    let page = backend
        .read_records(
            &view,
            &ReadSelection::new(ReadKind::Records),
            None,
            ReadLimits {
                rows: bound(64),
                bytes: bound(4096),
            },
        )
        .await
        .unwrap();
    assert_eq!(page.entries, first);
    assert!(page.next.is_none());
    backend.release_view(&snapshot).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn invalid_file_identity() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let bytes = b"verified immutable file";
    let sha256: [u8; 32] = Sha256::digest(bytes).into();
    let staging = backend
        .create_file(&FileMetadata {
            id: harmos::Identifier::new("file.retry.finish").unwrap(),
            scope: None,
            media_type: "application/octet-stream".into(),
            expected_length: None,
            expected_sha256: None,
        })
        .await
        .unwrap();
    backend.write_file(&staging, 0, bytes).await.unwrap();
    let position = StorageCursor {
        history: backend.history().clone(),
        sequence: 1,
    };
    assert!(
        backend
            .finish_file(&staging, &position, bytes.len() as u64 + 1, sha256)
            .await
            .is_err()
    );
    assert!(
        backend
            .finish_file(&staging, &position, bytes.len() as u64, [0; 32])
            .await
            .is_err()
    );
    let control = backend.control().await.unwrap();
    assert!(control.call(|connection| {
        connection.query_row("SELECT finish_intent IS NULL AND finished IS NULL FROM storage_file_stages WHERE id='file.retry.finish'", [], |row| row.get::<_, bool>(0))
    }).await.unwrap());
    control.call(|connection| connection.execute_batch(
        "CREATE TRIGGER fail_file_receipt BEFORE UPDATE OF finished ON storage_file_stages WHEN NEW.finished IS NOT NULL BEGIN SELECT RAISE(ABORT,'injected receipt failure'); END;"
    )).await.unwrap();
    assert!(
        backend
            .finish_file(&staging, &position, bytes.len() as u64, sha256)
            .await
            .is_err()
    );
    assert!(control.call(|connection| connection.query_row(
        "SELECT finish_intent IS NOT NULL AND finished IS NULL FROM storage_file_stages WHERE id='file.retry.finish'", [], |row| row.get::<_, bool>(0)
    )).await.unwrap());
    control
        .call(|connection| connection.execute_batch("DROP TRIGGER fail_file_receipt"))
        .await
        .unwrap();
    drop(control);
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let reference = backend
        .finish_file(&staging, &position, bytes.len() as u64, sha256)
        .await
        .unwrap();
    assert_eq!(reference.sha256, sha256);
    assert_eq!(
        backend
            .finish_file(&staging, &position, bytes.len() as u64, sha256)
            .await
            .unwrap(),
        reference
    );
    assert!(
        backend
            .finish_file(&staging, &position, bytes.len() as u64, [0; 32])
            .await
            .is_err()
    );
    let changed_position = StorageCursor {
        history: position.history.clone(),
        sequence: 2,
    };
    assert!(
        backend
            .finish_file(&staging, &changed_position, bytes.len() as u64, sha256)
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 0);
    backend.close().await.unwrap();
}

pub(crate) async fn file_publication_corruption() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let bytes = b"immutable report";
    let sha256: [u8; 32] = Sha256::digest(bytes).into();
    let staging = backend
        .create_file(&FileMetadata {
            id: harmos::Identifier::new("file.corrupted").unwrap(),
            scope: None,
            media_type: "application/octet-stream".into(),
            expected_length: Some(bytes.len() as u64),
            expected_sha256: Some(sha256),
        })
        .await
        .unwrap();
    backend.write_file(&staging, 0, bytes).await.unwrap();
    let position = StorageCursor {
        history: backend.history().clone(),
        sequence: 1,
    };
    let reference = backend
        .finish_file(&staging, &position, bytes.len() as u64, sha256)
        .await
        .unwrap();
    let path = std::fs::read_dir(root.path().join("files"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(&path, vec![0; bytes.len()]).unwrap();
    let files = [reference.clone()];
    let parts = [StoragePart::Files(&files)];
    let batch = StorageBatch::new(&parts, bound(4096)).unwrap();
    assert!(backend.write(batch).await.is_err());
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 0);
    std::fs::write(&path, bytes).unwrap();
    assert_eq!(backend.write(batch).await.unwrap(), position);
    std::fs::write(&path, vec![0; bytes.len()]).unwrap();
    assert!(backend.write(batch).await.is_err());
    assert_eq!(backend.cursor_saved().await.unwrap(), position);
    std::fs::write(&path, bytes).unwrap();
    assert_eq!(backend.write(batch).await.unwrap(), position);
    backend.close().await.unwrap();
}

#[cfg(unix)]
pub(crate) async fn file_writes_opening() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let metadata = FileMetadata {
        id: harmos::Identifier::new("file.special").unwrap(),
        scope: None,
        media_type: "application/octet-stream".into(),
        expected_length: None,
        expected_sha256: None,
    };
    let staging = backend.create_file(&metadata).await.unwrap();
    let path = std::fs::read_dir(root.path().join("staging"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    // Bind A Short Address Before Moving The Socket To The Staged Payload Path.
    let socket = root.path().join("socket");
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::rename(&socket, &path).unwrap();
    assert!(backend.write_file(&staging, 0, b"immutable").await.is_err());
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 0);
    drop(listener);
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, b"").unwrap();
    backend.write_file(&staging, 0, b"immutable").await.unwrap();
    backend.abort_file(&staging).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn finished_files_frontier() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let head = backend.cursor_saved().await.unwrap();
    let data = b"immutable report bytes";
    let hash: [u8; 32] = Sha256::digest(data).into();
    let metadata = FileMetadata {
        id: harmos::Identifier::new("report.owned").unwrap(),
        scope: Some(harmos::Identifier::new("execution.owned").unwrap()),
        media_type: "text/html".into(),
        expected_length: Some(data.len() as u64),
        expected_sha256: Some(hash),
    };
    let staging = backend.create_file(&metadata).await.unwrap();
    backend.write_file(&staging, 0, data).await.unwrap();
    backend.write_file(&staging, 0, data).await.unwrap();
    assert!(backend.write_file(&staging, 0, b"different").await.is_err());
    let position = StorageCursor {
        history: head.history.clone(),
        sequence: 1,
    };
    let reference = backend
        .finish_file(&staging, &position, data.len() as u64, hash)
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 0);
    backend.abort_file(&staging).await.unwrap();
    let snapshot = backend.read_view(&head).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.file").unwrap(),
        position.clone(),
        head,
        snapshot.clone(),
    )
    .unwrap();
    assert_eq!(
        backend
            .read_file(&view, &reference, 0, bound(4096))
            .await
            .unwrap(),
        data
    );
    backend.release_view(&snapshot).await.unwrap();
    let references = [reference];
    let parts = [StoragePart::Files(&references)];
    backend
        .write(StorageBatch::new(&parts, bound(4096)).unwrap())
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 1);
    backend.close().await.unwrap();
    let reopened = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    assert_eq!(reopened.cursor_saved().await.unwrap(), position);
    reopened.close().await.unwrap();
}

pub(crate) async fn cancelled_waiter_settlement() {
    let root = tempfile::tempdir().unwrap();
    let backend = Arc::new(
        Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
            .await
            .unwrap(),
    );
    let records = [record(backend.history(), 1, "record.one", &[1])];
    let parts = [StoragePart::Records(&records)];
    let saved = backend
        .write(StorageBatch::new(&parts, bound(4096)).unwrap())
        .await
        .unwrap();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.one").unwrap(),
        saved.clone(),
        saved.clone(),
        snapshot.clone(),
    )
    .unwrap();
    let state = backend.control().await.unwrap();
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let blocker = tokio::spawn(async move {
        state
            .call(move |_| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
            .await
            .unwrap();
    });
    tokio::time::timeout(std::time::Duration::from_secs(10), entered_rx)
        .await
        .unwrap()
        .unwrap();
    let selection = ReadSelection::new(ReadKind::Records);
    let mut reader = backend.read_records(
        &view,
        &selection,
        None,
        ReadLimits {
            rows: bound(64),
            bytes: bound(4096),
        },
    );
    std::future::poll_fn(|context| {
        assert!(reader.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(reader);
    let mut released = backend.release_view(&snapshot);
    assert!(futures_util::poll!(&mut released).is_pending());
    assert!(matches!(
        backend
            .read_records(
                &view,
                &selection,
                None,
                ReadLimits {
                    rows: bound(64),
                    bytes: bound(4096)
                }
            )
            .await,
        Err(harmos::Error::ViewExpired)
    ));
    release_tx.send(()).unwrap();
    blocker.await.unwrap();
    released.await.unwrap();
    let next = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if let Ok(snapshot) = backend.read_view(&saved).await {
                break snapshot;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    backend.release_view(&next).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn exhausted_read_retry() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(
        root.path().to_path_buf(),
        Limits {
            entry_bytes: bound(2 * 1024 * 1024),
            ..Limits::default()
        },
    )
    .await
    .unwrap();
    let records = [
        record(backend.history(), 1, "record.small", &[1]),
        record(backend.history(), 2, "record.large", &vec![255; 256 * 1024]),
    ];
    let maximum = bound(2 * 1024 * 1024);
    let parts = [StoragePart::Records(&records)];
    let saved = backend
        .write(StorageBatch::new(&parts, maximum).unwrap())
        .await
        .unwrap();
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(
        root.path().to_path_buf(),
        Limits {
            memory_bytes: bound(3 * 1024 * 1024),
            entry_bytes: bound(2 * 1024 * 1024),
            ..Limits::default()
        },
    )
    .await
    .unwrap();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.memory").unwrap(),
        saved.clone(),
        saved.clone(),
        snapshot.clone(),
    )
    .unwrap();
    let limits = ReadLimits {
        rows: bound(1),
        bytes: maximum,
    };
    let mut selection = ReadSelection::new(ReadKind::Records);
    selection.after = 1;
    let error = backend
        .read_records(&view, &selection, None, limits)
        .await
        .unwrap_err();
    assert!(
        matches!(
            error,
            harmos::Error::InvalidInput {
                reason: "owned query state exceeds memory bound",
                ..
            }
        ),
        "the decoded allocation must be refused before deserialization: {error}"
    );
    selection.after = 0;
    selection.through = Some(1);
    let page = backend
        .read_records(&view, &selection, None, limits)
        .await
        .unwrap();
    assert_eq!(page.entries, records[..1]);
    assert!(page.next.is_none());
    backend.release_view(&snapshot).await.unwrap();
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(
        root.path().to_path_buf(),
        Limits {
            entry_bytes: bound(2 * 1024 * 1024),
            ..Limits::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap(), saved);
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.retry").unwrap(),
        saved.clone(),
        saved,
        snapshot.clone(),
    )
    .unwrap();
    selection.after = 1;
    selection.through = None;
    let page = backend
        .read_records(&view, &selection, None, limits)
        .await
        .unwrap();
    assert_eq!(page.entries, records[1..]);
    assert!(page.next.is_none());
    backend.release_view(&snapshot).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn typed_page_view() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let records = [
        record(backend.history(), 1, "first", &[7; 128]),
        record(backend.history(), 2, "second", &[8; 128]),
        record(backend.history(), 3, "third", &[9; 128]),
    ];
    let parts = [StoragePart::Records(&records)];
    let saved = backend
        .write(StorageBatch::new(&parts, bound(4096)).unwrap())
        .await
        .unwrap();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.typed").unwrap(),
        saved.clone(),
        saved,
        snapshot,
    )
    .unwrap();
    let selection = ReadSelection::new(ReadKind::Records);
    let bytes = records
        .iter()
        .map(|row| serde_json::to_vec(row).unwrap().len())
        .max()
        .unwrap();
    let impossible = serde_json::to_vec(&records[0]).unwrap().len() - 1;
    assert!(
        backend
            .read_records(
                &view,
                &selection,
                None,
                ReadLimits {
                    rows: bound(64),
                    bytes: bound(impossible)
                }
            )
            .await
            .is_err()
    );
    let limits = ReadLimits {
        rows: bound(64),
        bytes: bound(bytes),
    };
    let mut cursor = None;
    let mut actual = Vec::new();
    loop {
        let page = backend
            .read_records(&view, &selection, cursor.as_ref(), limits)
            .await
            .unwrap();
        let encoded: usize = page
            .entries
            .iter()
            .map(|row| serde_json::to_vec(row).unwrap().len())
            .sum();
        assert!(encoded <= bytes);
        assert_eq!(page.entries.len(), 1);
        actual.extend(page.entries);
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(actual, records);
    backend.release_view(view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn page_lookahead_blobs() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(
        root.path().to_path_buf(),
        Limits {
            memory_bytes: bound(4 * 1024 * 1024),
            ..Limits::default()
        },
    )
    .await
    .unwrap();
    let rows = [
        record(backend.history(), 1, "first", &[255; 180_000]),
        record(backend.history(), 2, "second", &[255; 180_000]),
    ];
    // Prepare Each Large Row Without Charging A Second Write Blob.
    for row in &rows {
        let parts = [StoragePart::Records(std::slice::from_ref(row))];
        backend
            .write(StorageBatch::new(&parts, bound(1024 * 1024)).unwrap())
            .await
            .unwrap();
    }
    let saved = rows[1].position.clone();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.lookahead").unwrap(),
        saved.clone(),
        saved,
        snapshot,
    )
    .unwrap();
    let selection = ReadSelection::new(ReadKind::Records);
    let limits = ReadLimits {
        rows: NonZeroUsize::MIN,
        bytes: bound(1024 * 1024),
    };
    let first = backend
        .read_records(&view, &selection, None, limits)
        .await
        .unwrap();
    assert_eq!(first.entries, rows[..1]);
    let next = first
        .next
        .clone()
        .expect("lookahead preserves continuation");
    drop(first);
    let second = backend
        .read_records(&view, &selection, Some(&next), limits)
        .await
        .unwrap();
    assert_eq!(second.entries, rows[1..]);
    assert!(second.next.is_none());
    backend.release_view(view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}
