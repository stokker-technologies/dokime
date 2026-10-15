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
use std::{num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::storage::*;

// Local Imports
use super::storage::{history, id, limits, position, read_all, record, schema, view, write_batch};
use dokime::storage::sqlite::{Backend, Limits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct UnfinishedStatement(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Drop for UnfinishedStatement {
    fn drop(&mut self) {
        // This fixture owns the prepared statement. Every owner operation has
        // settled before this guard drops; no SQLite call runs concurrently.
        unsafe {
            rusqlite::ffi::sqlite3_finalize(self.0 as *mut rusqlite::ffi::sqlite3_stmt);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn saved_views_payloads() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(backend.open().await.unwrap().sequence, 0);
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let first = record(2);
    backend
        .write(write_batch(&[StoragePart::Records(std::slice::from_ref(
            &first,
        ))]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    let old = view(&backend, 2).await;
    let second = record(3);
    backend
        .write(write_batch(&[StoragePart::Records(std::slice::from_ref(
            &second,
        ))]))
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 3);
    backend.cursor_saved().await.unwrap();
    let selection = ReadSelection::new(ReadKind::Records);
    let old_page = backend
        .read_records(&old, &selection, None, limits(10))
        .await
        .unwrap();
    assert_eq!(old_page.entries, vec![first.clone()]);
    assert!(old_page.next.is_none());
    let current = view(&backend, 3).await;
    let page = backend
        .read_records(&current, &selection, None, limits(1))
        .await
        .unwrap();
    assert_eq!(page.entries, vec![first]);
    let next = backend
        .read_records(&current, &selection, page.next.as_ref(), limits(1))
        .await
        .unwrap();
    assert_eq!(next.entries, vec![second]);
    assert!(next.next.is_none());
    let mut descending = selection.clone();
    descending.order = ReadOrder::Descending;
    assert!(
        backend
            .read_records(&current, &descending, page.next.as_ref(), limits(1))
            .await
            .is_err()
    );
    assert_eq!(
        backend
            .read_records(&current, &descending, None, limits(1))
            .await
            .unwrap()
            .entries[0]
            .position
            .sequence,
        3
    );
    let mut filtered = selection.clone();
    filtered.scope = Some(id("another-execution"));
    assert!(
        backend
            .read_records(&current, &filtered, None, limits(5))
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    assert!(
        backend
            .read_stream(&current, &selection, None, limits(1))
            .await
            .is_err()
    );
    let mut tiny = limits(1);
    tiny.bytes = NonZeroUsize::new(1).unwrap();
    assert!(
        backend
            .read_records(&current, &selection, None, tiny)
            .await
            .is_err()
    );
    backend.release_view(old.snapshot()).await.unwrap();
    backend.release_view(old.snapshot()).await.unwrap();
    assert!(matches!(
        backend
            .read_records(&old, &selection, None, limits(1))
            .await,
        Err(harmos::Error::ViewExpired)
    ));
    backend.close().await.unwrap();
    let reopened = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(reopened.open().await.unwrap().sequence, 3);
    let current = view(&reopened, 3).await;
    assert_eq!(
        reopened
            .read_schemas(
                &current,
                &ReadSelection::new(ReadKind::Schemas),
                None,
                limits(1)
            )
            .await
            .unwrap()
            .entries,
        vec![schema()]
    );
    assert_eq!(
        reopened
            .read_records(&current, &selection, None, limits(10))
            .await
            .unwrap()
            .entries
            .len(),
        2
    );
    reopened.close().await.unwrap();
}

pub(crate) async fn stream_rows_views() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(
        root.path().to_path_buf(),
        Limits {
            views: NonZeroUsize::new(1).unwrap(),
            ..Limits::default()
        },
        history(),
    )
    .await
    .unwrap();
    let backend = Arc::new(backend);
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let row = StoredRow {
        record: record(2),
        stream: id("input"),
        sequence: u64::MAX,
    };
    backend
        .write(write_batch(&[StoragePart::Stream(std::slice::from_ref(
            &row,
        ))]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    let current = view(&backend, 2).await;
    assert!(backend.read_view(&position(2)).await.is_err());
    let mut selection = ReadSelection::new(ReadKind::Stream);
    selection.stream = Some(id("input"));
    assert_eq!(
        backend
            .read_stream(&current, &selection, None, limits(1))
            .await
            .unwrap()
            .entries,
        vec![row]
    );
    selection.after = 2;
    assert!(
        backend
            .read_stream(&current, &selection, None, limits(1))
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    backend.release_view(current.snapshot()).await.unwrap();
    backend.read_view(&position(2)).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn cancelled_waiter_owned() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let state = backend.control().await.unwrap();
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    state
        .call(move |connection| {
            let entered_tx = std::sync::Mutex::new(Some(entered_tx));
            connection.create_scalar_function(
                "fixture_block_write",
                0,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                move |_| {
                    entered_tx.lock().unwrap().take().unwrap().send(()).unwrap();
                    release_rx.recv().unwrap();
                    Ok(0)
                },
            )?;
            connection.execute_batch(
                "CREATE TEMP TRIGGER fixture_write_barrier
                 BEFORE INSERT ON storage_data_entries
                 BEGIN SELECT fixture_block_write(); END;",
            )
        })
        .await
        .unwrap();
    let entries = [record(2)];
    let parts = [StoragePart::Records(&entries)];
    let mut write = backend.write(write_batch(&parts));
    assert!(futures_util::poll!(write.as_mut()).is_pending());
    tokio::time::timeout(std::time::Duration::from_secs(10), entered_rx)
        .await
        .unwrap()
        .unwrap();
    drop(write);
    release_tx.send(()).unwrap();
    // The trigger acknowledges physical admission before the waiter is dropped.
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 2);
    state
        .call(|connection| {
            connection.execute_batch("DROP TRIGGER fixture_write_barrier")?;
            connection.remove_function("fixture_block_write", 0)
        })
        .await
        .unwrap();
    let current = view(&backend, 2).await;
    assert_eq!(
        backend
            .read_records(
                &current,
                &ReadSelection::new(ReadKind::Records),
                None,
                limits(1)
            )
            .await
            .unwrap()
            .entries,
        entries
    );
    backend.close().await.unwrap();
}

pub(crate) async fn missing_saved_evidence() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    backend.close().await.unwrap();
    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    let retained = connection
        .query_row(
            "SELECT body,digest FROM storage_data_entries WHERE position='0000000000000001'",
            [],
            |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
        )
        .unwrap();
    connection
        .execute_batch("ALTER TABLE storage_data_head RENAME TO fixture_missing_head")
        .unwrap();
    drop(connection);
    assert!(backend.open().await.is_err());

    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    assert_eq!(
        connection
            .query_row(
                "SELECT body,digest FROM storage_data_entries WHERE position='0000000000000001'",
                [],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .unwrap(),
        retained
    );
    assert!(
        !connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='storage_data_head')",
                [],
                |row| row.get::<_, bool>(0),
            )
            .unwrap()
    );
    connection
        .execute_batch("ALTER TABLE fixture_missing_head RENAME TO storage_data_head")
        .unwrap();
    drop(connection);
    assert_eq!(backend.open().await.unwrap().sequence, 1);
    backend.close().await.unwrap();
}

pub(crate) async fn saved_snapshot_checks() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    backend
        .write(write_batch(&[StoragePart::Records(&[
            record(2),
            record(3),
        ])]))
        .await
        .unwrap();
    let old = view(&backend, 3).await;
    let selection = ReadSelection::new(ReadKind::Records);
    let first = backend
        .read_records(&old, &selection, None, limits(1))
        .await
        .unwrap();
    assert_eq!(first.entries, [record(2)]);
    let cursor = first.next.unwrap();

    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            connection
                .execute_batch("ALTER TABLE storage_data_entries RENAME TO fixture_parked_payload")
        })
        .await
        .unwrap();
    assert!(
        backend
            .read_records(&old, &selection, Some(&cursor), limits(1))
            .await
            .is_err()
    );
    control
        .call(|connection| {
            connection
                .execute_batch("ALTER TABLE fixture_parked_payload RENAME TO storage_data_entries")
        })
        .await
        .unwrap();
    let second = backend
        .read_records(&old, &selection, Some(&cursor), limits(1))
        .await
        .unwrap();
    assert_eq!(second.entries, [record(3)]);
    assert!(second.next.is_none());

    // Interleave Saved Views And Rewind The Original Frontier
    backend
        .write(write_batch(&[StoragePart::Records(&[record(4)])]))
        .await
        .unwrap();
    let new = view(&backend, 4).await;
    assert_eq!(
        read_all(&backend, &new).await,
        [record(2), record(3), record(4)]
    );
    assert_eq!(read_all(&backend, &old).await, [record(2), record(3)]);
    backend.release_view(old.snapshot()).await.unwrap();
    backend.release_view(new.snapshot()).await.unwrap();
    assert!(matches!(
        backend
            .read_records(&old, &selection, None, limits(1))
            .await,
        Err(harmos::Error::ViewExpired)
    ));
    let reopened = view(&backend, 4).await;
    assert_eq!(
        read_all(&backend, &reopened).await,
        [record(2), record(3), record(4)]
    );
    backend.release_view(reopened.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn sparse_pages_statistics() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let mut records = (2..34).map(record).collect::<Vec<_>>();
    records[12].scope = Some(id("sparse"));
    records[31].scope = Some(id("sparse"));
    for chunk in records.chunks(4) {
        backend
            .write(write_batch(&[StoragePart::Records(chunk)]))
            .await
            .unwrap();
    }
    backend.close().await.unwrap();

    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    connection.execute_batch("ANALYZE").unwrap();
    drop(connection);
    for remove_statistics in [false, true] {
        if remove_statistics {
            let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
            assert!(connection.execute("DELETE FROM sqlite_stat1", []).unwrap() > 0);
            drop(connection);
        }

        backend.open().await.unwrap();
        let view = view(&backend, 33).await;
        let mut selection = ReadSelection::new(ReadKind::Records);
        selection.scope = Some(id("sparse"));
        let ascending = backend
            .read_records(&view, &selection, None, limits(16))
            .await
            .unwrap();
        assert_eq!(
            ascending.entries,
            [records[12].clone(), records[31].clone()]
        );
        assert!(ascending.next.is_none());
        selection.order = ReadOrder::Descending;
        let descending = backend
            .read_records(&view, &selection, None, limits(16))
            .await
            .unwrap();
        assert_eq!(
            descending.entries,
            [records[31].clone(), records[12].clone()]
        );
        assert!(descending.next.is_none());
        selection.order = ReadOrder::Ascending;
        selection.scope = Some(id("absent"));
        let empty = backend
            .read_records(&view, &selection, None, limits(16))
            .await
            .unwrap();
        assert!(empty.entries.is_empty());
        assert!(empty.next.is_none());
        backend.release_view(view.snapshot()).await.unwrap();
        backend.close().await.unwrap();
    }
}

// a scoped page resumes at its cursor inside another scope's interleaved rows.
pub(crate) async fn interleaved_scope_pages() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let mut records = (2..90).map(record).collect::<Vec<_>>();
    for record in &mut records {
        if record.position.sequence % 3 != 0 {
            record.scope = Some(id("other"));
        }
    }
    backend
        .write(write_batch(&[StoragePart::Records(&records)]))
        .await
        .unwrap();
    let view = view(&backend, 89).await;

    // Unbounded, and narrowed by the selection's own window, which the
    // continuation must tighten rather than replace.
    for (after, through) in [(0, None), (20, Some(70))] {
        for order in [ReadOrder::Ascending, ReadOrder::Descending] {
            let mut selection = ReadSelection::new(ReadKind::Records);
            selection.scope = Some(id("execution"));
            selection.order = order;
            selection.after = after;
            selection.through = through;
            let mut expected = records
                .iter()
                .filter(|record| {
                    let sequence = record.position.sequence;
                    record.scope == Some(id("execution"))
                        && sequence > after
                        && through.is_none_or(|through| sequence <= through)
                })
                .cloned()
                .collect::<Vec<_>>();
            if order == ReadOrder::Descending {
                expected.reverse();
            }
            let mut read = Vec::new();
            let mut cursor = None;
            loop {
                let page = backend
                    .read_records(&view, &selection, cursor.as_ref(), limits(4))
                    .await
                    .unwrap();
                assert!(page.entries.len() <= 4);
                read.extend(page.entries);
                cursor = page.next;
                if cursor.is_none() {
                    break;
                }
            }
            assert_eq!(
                read, expected,
                "after={after} through={through:?} {order:?}"
            );
        }
    }
    backend.release_view(view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn small_rows_orders() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let records = (2..67).map(record).collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Records(&records)]))
        .await
        .unwrap();
    let view = view(&backend, 66).await;

    for order in [ReadOrder::Ascending, ReadOrder::Descending] {
        let mut selection = ReadSelection::new(ReadKind::Records);
        selection.order = order;
        let mut expected = records.clone();
        if order == ReadOrder::Descending {
            expected.reverse();
        }
        let first = backend
            .read_records(&view, &selection, None, limits(256))
            .await
            .unwrap();
        assert_eq!(first.entries, expected[..64]);
        let cursor = first.next.unwrap();
        let last = backend
            .read_records(&view, &selection, Some(&cursor), limits(256))
            .await
            .unwrap();
        assert_eq!(last.entries, expected[64..]);
        assert!(last.next.is_none());
    }
    backend.release_view(view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn native_query_capacity() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(
        root.path().to_path_buf(),
        Limits {
            views: NonZeroUsize::MIN,
            ..Limits::default()
        },
        history(),
    )
    .await
    .unwrap();
    backend.open().await.unwrap();
    let tiny = QueryLimits {
        memory_bytes: NonZeroUsize::MIN,
        spill_bytes: 0,
        timeout: std::time::Duration::from_secs(5),
        ..StorageSettings::default().query_execution
    };
    assert!(backend.read_query_view(&position(0), tiny).await.is_err());
    for (memory_bytes, spill_bytes) in [(4 * 1024 * 1024, 0), (64 * 1024 * 1024, 64 * 1024 * 1024)]
    {
        let requested = QueryLimits {
            memory_bytes: NonZeroUsize::new(memory_bytes).unwrap(),
            spill_bytes,
            timeout: std::time::Duration::from_secs(5),
            ..StorageSettings::default().query_execution
        };
        let pin = backend
            .read_query_view(&position(0), requested)
            .await
            .unwrap();
        assert!(
            backend
                .read_query_view(&position(0), requested)
                .await
                .is_err()
        );
        backend.release_view(&pin).await.unwrap();
    }
    assert_eq!(backend.cursor_saved().await.unwrap(), position(0));
    backend.close().await.unwrap();
}

pub(crate) async fn restart_committed_conflicts() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    assert!(
        Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
            .await
            .is_err()
    );
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&[record(3)])]))
            .await
            .is_err()
    );
    backend
        .write(write_batch(&[StoragePart::Records(&[record(2)])]))
        .await
        .unwrap();
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            connection.execute_batch(
                "CREATE TRIGGER fail_restart_head BEFORE UPDATE OF saved ON storage_data_head
         BEGIN SELECT RAISE(ABORT,'injected saved-head failure'); END;",
            )
        })
        .await
        .unwrap();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&[record(3)])]))
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), position(2));
    control
        .call(|connection| connection.execute_batch("DROP TRIGGER fail_restart_head"))
        .await
        .unwrap();
    let mut conflict = record(2);
    conflict.payload = vec![8, 9];
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&[conflict])]))
            .await
            .is_err()
    );
    let mut reused = record(3);
    reused.item = record(2).item;
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&[reused])]))
            .await
            .is_err()
    );
    backend.close().await.unwrap();
    drop(control);
    drop(backend);

    let competing = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(competing.open().await.unwrap(), position(2));
    let mut replacement = record(3);
    replacement.payload = vec![8, 7];
    competing
        .write(write_batch(&[StoragePart::Records(&[replacement.clone()])]))
        .await
        .unwrap();
    competing.close().await.unwrap();
    let wrong = StoreHistory {
        epoch: id("wrong"),
        ..history()
    };
    assert!(matches!(
        Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), wrong).await,
        Err(harmos::Error::DifferentHistory)
    ));
    let reopened = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(reopened.open().await.unwrap(), position(3));
    let current = view(&reopened, 3).await;
    assert_eq!(
        read_all(&reopened, &current).await,
        [record(2), replacement]
    );
    reopened.release_view(current.snapshot()).await.unwrap();
    reopened.close().await.unwrap();
}

pub(crate) async fn failed_native_retry() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let control = backend.control().await.unwrap();
    let statement = control
        .call(|connection| {
            let mut statement = std::ptr::null_mut();
            // The original owner exclusively borrows this connection. The SQL and
            // output pointer live through preparation; the fixture owns finalization.
            let result = unsafe {
                rusqlite::ffi::sqlite3_prepare_v2(
                    connection.handle(),
                    c"SELECT 1".as_ptr(),
                    -1,
                    &mut statement,
                    std::ptr::null_mut(),
                )
            };
            if result != rusqlite::ffi::SQLITE_OK {
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(result),
                    None,
                ));
            }
            Ok(statement as usize)
        })
        .await
        .unwrap();
    let statement = UnfinishedStatement(statement);
    assert!(backend.close().await.is_err());
    assert!(
        Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
            .await
            .is_err()
    );
    drop(statement);
    backend.close().await.unwrap();
    let competing = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(competing.open().await.unwrap(), position(1));
    competing.close().await.unwrap();
}
