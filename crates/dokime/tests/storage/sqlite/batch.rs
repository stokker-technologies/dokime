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
use harmos::{
    schema::{
        Cardinality,
        graph::{Field, Kind},
    },
    storage::*,
};

// Local Imports
use super::storage::{history, limits, position, read_all, record, schema, view, write_batch};
use dokime::storage::sqlite::{Backend, Limits, State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn observe_commits(control: &State) {
    control
        .call(|connection| {
            connection.execute_batch(
                "CREATE TABLE fixture_batch_commits (saved TEXT NOT NULL);
             CREATE TRIGGER observe_batch_commit AFTER UPDATE OF saved ON storage_data_head
             WHEN NEW.saved != OLD.saved BEGIN
                 INSERT INTO fixture_batch_commits VALUES (NEW.saved);
             END;",
            )
        })
        .await
        .unwrap();
}

async fn commits(control: &State) -> i64 {
    control
        .call(|connection| {
            connection.query_row("SELECT COUNT(*) FROM fixture_batch_commits", [], |row| {
                row.get(0)
            })
        })
        .await
        .unwrap()
}

/* === Tests === */

pub(crate) async fn native_batch_unchanged() {
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
    observe_commits(&control).await;
    let before = commits(&control).await;
    let records = (2..66).map(record).collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Records(&records)]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    assert_eq!(commits(&control).await, before + 1);
    backend
        .write(write_batch(&[StoragePart::Records(&records)]))
        .await
        .unwrap();
    assert_eq!(commits(&control).await, before + 1);
    let view = view(&backend, 65).await;
    assert_eq!(read_all(&backend, &view).await, records);
    backend.release_view(view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn batch_conflicts_prefix() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let records = (2..10).map(record).collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Records(&records)]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    let control = backend.control().await.unwrap();
    observe_commits(&control).await;
    let before = commits(&control).await;

    let mut duplicate = record(10);
    duplicate.item = records[0].item.clone();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&[
                duplicate,
                record(11)
            ])]))
            .await
            .is_err()
    );
    assert_eq!(commits(&control).await, before);

    let mut changed = records.clone();
    changed[0].payload = vec![8, 9];
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&changed)]))
            .await
            .is_err()
    );
    assert_eq!(commits(&control).await, before);

    let view = view(&backend, 9).await;
    assert_eq!(read_all(&backend, &view).await, records);
    backend.release_view(view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn cancelled_batch_owned() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let records = (2..66).map(record).collect::<Vec<_>>();
    let parts = [StoragePart::Records(&records)];
    let mut writing = backend.write(write_batch(&parts));
    assert!(futures_util::poll!(writing.as_mut()).is_pending());
    drop(writing);
    backend.cursor_saved().await.unwrap();
    backend.close().await.unwrap();

    let reopened = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(reopened.open().await.unwrap().sequence, 65);
    let view = view(&reopened, 65).await;
    assert_eq!(read_all(&reopened, &view).await, records);
    reopened.release_view(view.snapshot()).await.unwrap();
    reopened.close().await.unwrap();
}

pub(crate) async fn grouped_pages_limits() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let records = (2..10).map(record).collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Records(&records[..4])]))
        .await
        .unwrap();
    backend
        .write(write_batch(&[StoragePart::Records(&records[4..])]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    let view = view(&backend, 9).await;

    backend
        .write(write_batch(&[StoragePart::Records(
            &(10..14).map(record).collect::<Vec<_>>(),
        )]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    assert_eq!(read_all(&backend, &view).await, records);

    let mut selection = ReadSelection::new(ReadKind::Records);
    selection.order = ReadOrder::Descending;
    let mut cursor = None;
    let mut actual = Vec::new();
    loop {
        let page = backend
            .read_records(&view, &selection, cursor.as_ref(), limits(3))
            .await
            .unwrap();
        assert!(page.entries.len() <= 3);
        actual.extend(page.entries);
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(actual, records.into_iter().rev().collect::<Vec<_>>());
    let tiny = ReadLimits {
        rows: NonZeroUsize::new(3).unwrap(),
        bytes: NonZeroUsize::new(1).unwrap(),
    };
    assert!(
        backend
            .read_records(&view, &selection, None, tiny)
            .await
            .is_err()
    );
    backend.release_view(view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn saved_page_damage() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let first = (2..6).map(record).collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Records(&first)]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    let saved_view = view(&backend, 5).await;
    backend
        .write(write_batch(&[StoragePart::Records(
            &(6..10).map(record).collect::<Vec<_>>(),
        )]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();

    let control = backend.control().await.unwrap();
    let original = control
        .call(|connection| {
            let mut statement = connection.prepare(
                "SELECT position,body FROM storage_data_entries WHERE position>='0000000000000006' ORDER BY position",
            )?;
            let original = statement
                .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            connection.execute(
                "UPDATE storage_data_entries SET body=x'00' WHERE position>='0000000000000006'", [],
            )?;
            Ok(original)
        })
        .await
        .unwrap();
    let result = backend
        .read_records(
            &saved_view,
            &ReadSelection::new(ReadKind::Records),
            None,
            limits(8),
        )
        .await;
    let later = view(&backend, 9).await;
    assert!(
        backend
            .read_records(
                &later,
                &ReadSelection::new(ReadKind::Records),
                None,
                limits(8)
            )
            .await
            .is_err()
    );
    backend.release_view(later.snapshot()).await.unwrap();
    control
        .call(move |connection| {
            for (position, body) in original {
                assert_eq!(
                    connection.execute(
                        "UPDATE storage_data_entries SET body=?2 WHERE position=?1",
                        rusqlite::params![position, body],
                    )?,
                    1
                );
            }
            Ok(())
        })
        .await
        .unwrap();
    let page = result.unwrap();
    assert_eq!(page.entries, first);
    assert!(page.next.is_none());
    backend.release_view(saved_view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn failed_batch_retry() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let saved = (2..5).map(record).collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Records(&saved)]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();

    let pending = (5..9).map(record).collect::<Vec<_>>();
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            connection.execute_batch(
                "CREATE TRIGGER fail_batch_head BEFORE UPDATE OF saved ON storage_data_head
             BEGIN SELECT RAISE(ABORT,'injected saved-head failure'); END;",
            )
        })
        .await
        .unwrap();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&pending)]))
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), position(4));
    control
        .call(|connection| {
            for table in ["storage_data_entries", "storage_data_inventory"] {
                let sql = format!("SELECT COUNT(*) FROM {table} WHERE position>'0000000000000004'");
                assert_eq!(
                    connection.query_row(&sql, [], |row| row.get::<_, i64>(0))?,
                    0
                );
            }
            connection.execute_batch("DROP TRIGGER fail_batch_head")
        })
        .await
        .unwrap();
    backend.close().await.unwrap();
    drop(control);
    drop(backend);
    let reopened = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    assert_eq!(reopened.open().await.unwrap().sequence, 4);
    reopened
        .write(write_batch(&[StoragePart::Records(&pending)]))
        .await
        .unwrap();
    reopened
        .write(write_batch(&[StoragePart::Records(
            &(9..13).map(record).collect::<Vec<_>>(),
        )]))
        .await
        .unwrap();
    reopened.cursor_saved().await.unwrap();
    let view = view(&reopened, 12).await;
    assert_eq!(
        read_all(&reopened, &view).await,
        (2..13).map(record).collect::<Vec<_>>()
    );
    // The Native Primary Key Keeps One Physical Payload Per Position
    let control = reopened.control().await.unwrap();
    let rows = control
        .call(|connection| connection.query_row(
            "SELECT COUNT(*),COUNT(DISTINCT position) FROM storage_data_entries WHERE kind='record'",
            [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        ))
        .await
        .unwrap();
    assert_eq!(rows, (11, 11));
    reopened.release_view(view.snapshot()).await.unwrap();
    reopened.close().await.unwrap();
}

pub(crate) async fn live_owner_restored() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let records = [record(2), record(3)];
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            connection.execute_batch("ALTER TABLE storage_data_head RENAME TO fixture_parked_head")
        })
        .await
        .unwrap();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&records)]))
            .await
            .is_err()
    );
    assert!(backend.cursor_saved().await.is_err());
    control
        .call(|connection| {
            assert_eq!(
                connection.query_row(
                    "SELECT saved FROM fixture_parked_head WHERE singleton=1",
                    [],
                    |row| row.get::<_, String>(0),
                )?,
                "0000000000000001"
            );
            assert!(!connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='storage_data_head')",
                [],
                |row| row.get::<_, bool>(0),
            )?);
            connection.execute_batch("ALTER TABLE fixture_parked_head RENAME TO storage_data_head")
        })
        .await
        .unwrap();
    assert_eq!(
        backend
            .write(write_batch(&[StoragePart::Records(&records)]))
            .await
            .unwrap()
            .sequence,
        3
    );

    control
        .call(|connection| {
            connection.execute_batch(
                "ALTER TABLE storage_data_inventory RENAME TO fixture_parked_inventory",
            )
        })
        .await
        .unwrap();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&[record(4)])]))
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 3);
    control
        .call(|connection| {
            assert!(!connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='storage_data_inventory')",
                [],
                |row| row.get::<_, bool>(0),
            )?);
            assert_eq!(
                connection.query_row(
                    "SELECT COUNT(*) FROM storage_data_entries WHERE position='0000000000000004'",
                    [],
                    |row| row.get::<_, i64>(0),
                )?,
                0
            );
            connection.execute_batch(
                "ALTER TABLE fixture_parked_inventory RENAME TO storage_data_inventory",
            )
        })
        .await
        .unwrap();
    backend
        .write(write_batch(&[StoragePart::Records(&[record(4)])]))
        .await
        .unwrap();
    backend.close().await.unwrap();
    assert_eq!(backend.open().await.unwrap().sequence, 4);
    let saved = view(&backend, 4).await;
    assert_eq!(
        read_all(&backend, &saved).await,
        [record(2), record(3), record(4)]
    );
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn live_owner_publication() {
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
        .write(write_batch(&[StoragePart::Records(
            &(2..5).map(record).collect::<Vec<_>>(),
        )]))
        .await
        .unwrap();
    let control = backend.control().await.unwrap();
    observe_commits(&control).await;
    control
        .call(|connection| {
            connection
                .execute_batch("ALTER TABLE storage_data_entries RENAME TO fixture_parked_entries")
        })
        .await
        .unwrap();
    let before = commits(&control).await;
    let next = (5..7).map(record).collect::<Vec<_>>();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&next)]))
            .await
            .is_err()
    );
    assert_eq!(commits(&control).await, before);
    assert_eq!(backend.cursor_saved().await.unwrap(), position(4));
    control
        .call(|connection| {
            assert!(!connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='storage_data_entries')",
                [],
                |row| row.get::<_, bool>(0),
            )?);
            assert_eq!(
                connection.query_row(
                    "SELECT COUNT(*) FROM storage_data_inventory WHERE position>'0000000000000004'",
                    [],
                    |row| row.get::<_, i64>(0),
                )?,
                0
            );
            connection
                .execute_batch("ALTER TABLE fixture_parked_entries RENAME TO storage_data_entries")
        })
        .await
        .unwrap();
    backend
        .write(write_batch(&[StoragePart::Records(&next)]))
        .await
        .unwrap();
    backend.close().await.unwrap();
    assert_eq!(backend.open().await.unwrap(), position(6));
    let saved = view(&backend, 6).await;
    assert_eq!(
        read_all(&backend, &saved).await,
        (2..7).map(record).collect::<Vec<_>>()
    );
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn large_native_limits() {
    #[derive(Clone, PartialEq, prost::Message)]
    struct Payload {
        #[prost(bytes, tag = "1")]
        value: Vec<u8>,
    }
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(
        root.path().to_path_buf(),
        Limits {
            entry_bytes: NonZeroUsize::new(256 * 1024).unwrap(),
            ..Limits::default()
        },
        history(),
    )
    .await
    .unwrap();
    backend.open().await.unwrap();
    let mut declared = schema();
    declared.schema = super::storage::graph(vec![Field {
        name: "value".into(),
        tag: 1,
        kind: Kind::Bytes,
        cardinality: Cardinality::Required,
    }]);
    backend
        .write(write_batch(&[StoragePart::Schemas(&[declared])]))
        .await
        .unwrap();
    let payload = prost::Message::encode_to_vec(&Payload {
        value: vec![0; 96 * 1024],
    });
    let records = (2..10)
        .map(|sequence| {
            let mut record = record(sequence);
            record.payload = payload.clone();
            record
        })
        .collect::<Vec<_>>();
    let control = backend.control().await.unwrap();
    observe_commits(&control).await;
    let before = commits(&control).await;
    let saved = backend
        .write(write_batch(&[StoragePart::Records(&records)]))
        .await
        .unwrap();
    assert_eq!(saved.sequence, 9);
    assert_eq!(commits(&control).await, before + 1);
    let saved_view = view(&backend, 9).await;
    let selection = ReadSelection::new(ReadKind::Records);
    let limits = ReadLimits {
        rows: NonZeroUsize::new(8).unwrap(),
        bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
    };
    let mut cursor = None;
    let mut restored = Vec::new();
    loop {
        let page = backend
            .read_records(&saved_view, &selection, cursor.as_ref(), limits)
            .await
            .unwrap();
        // The Explicit Encoded Bound Splits Before The Sixth Large Record
        if restored.is_empty() {
            assert_eq!(page.entries.len(), 5);
            assert!(page.next.is_some());
        }
        let bytes: usize = page
            .entries
            .iter()
            .map(|record| serde_json::to_vec(record).unwrap().len())
            .sum();
        assert!(bytes <= 1024 * 1024);
        restored.extend(page.entries);
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(restored, records);
    backend.release_view(saved_view.snapshot()).await.unwrap();
    backend.close().await.unwrap();

    let backend = Backend::<()>::replica(
        root.path().to_path_buf(),
        Limits {
            entry_bytes: NonZeroUsize::new(256 * 1024).unwrap(),

            ..Limits::default()
        },
        history(),
    )
    .await
    .unwrap();
    backend.open().await.unwrap();
    let saved_view = view(&backend, 9).await;
    let oversized = ReadLimits {
        rows: NonZeroUsize::new(8).unwrap(),
        bytes: NonZeroUsize::new(2 * 1024 * 1024).unwrap(),
    };
    let page = backend
        .read_records(&saved_view, &selection, None, oversized)
        .await
        .unwrap();
    assert_eq!(page.entries, records);
    assert!(page.next.is_none());
    assert_eq!(backend.cursor_saved().await.unwrap(), position(9));
    let bounded = ReadLimits {
        rows: NonZeroUsize::MIN,
        bytes: NonZeroUsize::new(256 * 1024).unwrap(),
    };
    let mut cursor = None;
    let mut restored = Vec::new();
    loop {
        let page = backend
            .read_records(&saved_view, &selection, cursor.as_ref(), bounded)
            .await
            .unwrap();
        assert_eq!(page.entries.len(), 1);
        restored.extend(page.entries);
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(restored, records);
    backend.release_view(saved_view.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn rolled_back_schema() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            connection.execute_batch(
                "CREATE TRIGGER fail_cached_schema BEFORE UPDATE OF saved ON storage_data_head
         BEGIN SELECT RAISE(ABORT,'injected schema rollback'); END;",
            )
        })
        .await
        .unwrap();
    let row = StoredRow {
        record: record(2),
        stream: harmos::Identifier::new("samples").unwrap(),
        sequence: 1,
    };
    assert!(
        backend
            .write(write_batch(&[
                StoragePart::Schemas(&[schema()]),
                StoragePart::Stream(std::slice::from_ref(&row)),
            ]))
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), position(0));
    control
        .call(|connection| connection.execute_batch("DROP TRIGGER fail_cached_schema"))
        .await
        .unwrap();

    let mut replacement = schema();
    replacement.schema.definitions[0][0].kind = Kind::String;
    let mut replacement_row = row;
    replacement_row.record.payload = vec![10, 1, b'x'];
    backend
        .write(write_batch(&[
            StoragePart::Schemas(&[replacement]),
            StoragePart::Stream(&[replacement_row]),
        ]))
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap(), position(2));
    let kind: String = control
        .call(|connection| {
            connection.query_row(
                "SELECT kind FROM storage_data_entries WHERE position='0000000000000002'",
                [],
                |row| row.get(0),
            )
        })
        .await
        .unwrap();
    assert_eq!(kind, "stream");
    backend.close().await.unwrap();
}
