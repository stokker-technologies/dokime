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
use harmos::storage::*;
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use super::storage::{
    history, id, key, limits, position, read_all, record, schema, view, write_batch,
};
use dokime::storage::sqlite::{Backend, Limits, State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, PartialEq, Message)]
struct Counter {
    #[prost(uint64, tag = "1")]
    counter: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn replace_body(control: &State, sequence: u64, body: Vec<u8>) -> Vec<u8> {
    control
        .call(move |connection| {
            let position = format!("{sequence:016x}");
            let original = connection.query_row(
                "SELECT body FROM storage_data_entries WHERE position=?1",
                [&position],
                |row| row.get::<_, Vec<u8>>(0),
            )?;
            assert_eq!(
                connection.execute(
                    "UPDATE storage_data_entries SET body=?2 WHERE position=?1",
                    rusqlite::params![position, body],
                )?,
                1
            );
            Ok(original)
        })
        .await
        .unwrap()
}

/* === Tests === */

pub(crate) async fn physical_pages_boundaries() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let mut rows = [record(2), record(3), record(4)];
    for row in &mut rows {
        row.payload = Counter { counter: 7 }.encode_to_vec();
    }
    rows[2].scope = Some(id("other"));
    backend
        .write(write_batch(&[StoragePart::Records(&rows)]))
        .await
        .unwrap();
    let old = view(&backend, 3).await;
    let saved = view(&backend, 4).await;
    backend
        .write(write_batch(&[StoragePart::Records(&[record(5)])]))
        .await
        .unwrap();

    let mut selection = ReadSelection::new(ReadKind::Records);
    selection.schema = Some(key());
    let mut cursor = None;
    for expected in [2, 3, 4] {
        let page = backend
            .read_records(&saved, &selection, cursor.as_ref(), limits(1))
            .await
            .unwrap();
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.entries[0].position.sequence, expected);
        assert_eq!(
            Counter::decode(page.entries[0].payload.as_slice())
                .unwrap()
                .counter,
            7
        );
        cursor = page.next;
    }
    assert!(cursor.is_none());
    selection.after = 4;
    assert!(
        backend
            .read_records(&saved, &selection, None, limits(1))
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    selection.after = 3;
    assert!(
        backend
            .read_records(&old, &selection, None, limits(1))
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    selection.after = 0;
    selection.scope = Some(id("other"));
    let selected = backend
        .read_records(&saved, &selection, None, limits(1))
        .await
        .unwrap();
    assert_eq!(selected.entries, rows[2..]);
    selection.after = 5;
    assert!(
        backend
            .read_records(&saved, &selection, None, limits(1))
            .await
            .is_err()
    );
    let mut foreign = position(4);
    foreign.history.epoch = id("foreign");
    assert!(backend.read_view(&foreign).await.is_err());
    selection.after = 0;
    let tiny = ReadLimits {
        rows: NonZeroUsize::MIN,
        bytes: NonZeroUsize::MIN,
    };
    assert!(
        backend
            .read_records(&saved, &selection, None, tiny)
            .await
            .is_err()
    );
    backend.release_view(old.snapshot()).await.unwrap();
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn native_pages_payloads() {
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
    let saved = view(&backend, 3).await;
    let control = backend.control().await.unwrap();
    let original = replace_body(&control, 2, Vec::new()).await;
    let mut selection = ReadSelection::new(ReadKind::Records);
    selection.schema = Some(key());
    selection.after = 2;
    let good = backend
        .read_records(&saved, &selection, None, limits(1))
        .await
        .unwrap();
    assert_eq!(good.entries, [record(3)]);
    selection.after = 0;
    assert!(
        backend
            .read_records(&saved, &selection, None, limits(1))
            .await
            .is_err()
    );
    replace_body(&control, 2, original).await;
    assert_eq!(read_all(&backend, &saved).await, [record(2), record(3)]);
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn native_owner_positions() {
    for duplicate_inventory in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let backend =
            Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
                .await
                .unwrap();
        backend.open().await.unwrap();
        backend
            .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
            .await
            .unwrap();
        backend
            .write(write_batch(&[StoragePart::Records(&[record(2)])]))
            .await
            .unwrap();
        let control = backend.control().await.unwrap();
        let failure = control
            .call(move |connection| {
                let table = if duplicate_inventory {
                    "storage_data_inventory"
                } else {
                    "storage_data_entries"
                };
                let sql = format!(
                    "INSERT INTO {table} SELECT * FROM {table} WHERE position='0000000000000002'"
                );
                connection.execute(&sql, [])
            })
            .await
            .unwrap_err();
        assert!(
            matches!(failure, dokime::storage::sqlite::Error::Sqlite(rusqlite::Error::SqliteFailure(error, _))
            if error.code == rusqlite::ErrorCode::ConstraintViolation)
        );
        assert_eq!(backend.cursor_saved().await.unwrap(), position(2));
        backend.close().await.unwrap();
        backend.open().await.unwrap();
        let saved = view(&backend, 2).await;
        assert_eq!(read_all(&backend, &saved).await, [record(2)]);
        backend.release_view(saved.snapshot()).await.unwrap();
        backend.close().await.unwrap();
    }
}

pub(crate) async fn physical_page_positions() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    for sequence in [2, 3] {
        backend
            .write(write_batch(&[StoragePart::Records(&[record(sequence)])]))
            .await
            .unwrap();
    }
    let control = backend.control().await.unwrap();
    let original = replace_body(&control, 3, Vec::new()).await;
    backend.close().await.unwrap();
    backend.open().await.unwrap();
    let saved = view(&backend, 3).await;
    let selection = ReadSelection::new(ReadKind::Records);
    let first = backend
        .read_records(&saved, &selection, None, limits(1))
        .await
        .unwrap();
    assert_eq!(first.entries, [record(2)]);
    assert!(first.next.is_some());
    assert!(
        backend
            .read_records(&saved, &selection, first.next.as_ref(), limits(1))
            .await
            .is_err()
    );
    let control = backend.control().await.unwrap();
    replace_body(&control, 3, original).await;
    let second = backend
        .read_records(&saved, &selection, first.next.as_ref(), limits(1))
        .await
        .unwrap();
    assert_eq!(second.entries, [record(3)]);
    assert!(second.next.is_none());
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn native_page_position() {
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
        .write(write_batch(&[StoragePart::Records(&[record(2)])]))
        .await
        .unwrap();
    let control = backend.control().await.unwrap();
    let mut wrong = record(2);
    wrong.position = position(3);
    let body = serde_json::to_vec(&wrong).unwrap();
    let digest: [u8; 32] = Sha256::digest(&body).into();
    let original = replace_body(&control, 2, body).await;
    control
        .call(move |connection| {
            for table in ["storage_data_entries", "storage_data_inventory"] {
                let sql = format!("UPDATE {table} SET digest=?1 WHERE position='0000000000000002'");
                assert_eq!(connection.execute(&sql, [digest.as_slice()])?, 1);
            }
            Ok(())
        })
        .await
        .unwrap();
    backend.close().await.unwrap();
    backend.open().await.unwrap();
    let saved = view(&backend, 2).await;
    let failure = backend
        .read_records(
            &saved,
            &ReadSelection::new(ReadKind::Records),
            None,
            limits(1),
        )
        .await
        .unwrap_err();
    assert!(
        failure.to_string().contains("entry index position differs"),
        "{failure}"
    );

    // Restore The Sole Admitted Payload Without Changing Its Frontier
    let control = backend.control().await.unwrap();
    let digest: [u8; 32] = Sha256::digest(&original).into();
    replace_body(&control, 2, original).await;
    control
        .call(move |connection| {
            for table in ["storage_data_entries", "storage_data_inventory"] {
                let sql = format!("UPDATE {table} SET digest=?1 WHERE position='0000000000000002'");
                assert_eq!(connection.execute(&sql, [digest.as_slice()])?, 1);
            }
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(read_all(&backend, &saved).await, [record(2)]);
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn native_records_boundaries() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();
    let declared = view(&backend, 1).await;
    let mut selection = ReadSelection::new(ReadKind::Records);
    selection.schema = Some(key());
    assert!(
        backend
            .read_records(&declared, &selection, None, limits(1))
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    let mut first = record(2);
    first.scope = Some(id("request-a"));
    backend
        .write(write_batch(&[StoragePart::Records(&[first.clone()])]))
        .await
        .unwrap();
    let old = view(&backend, 2).await;
    let mut update = record(3);
    update.scope = Some(id("request-a"));
    backend
        .write(write_batch(&[StoragePart::Records(&[update.clone()])]))
        .await
        .unwrap();
    assert!(backend.read_view(&position(4)).await.is_err());
    assert_eq!(
        backend
            .read_records(&old, &selection, None, limits(10))
            .await
            .unwrap()
            .entries,
        [first.clone()]
    );
    let mut other = record(4);
    other.scope = Some(id("request-b"));
    other.payload = Counter { counter: u64::MAX }.encode_to_vec();
    backend
        .write(write_batch(&[StoragePart::Records(&[other.clone()])]))
        .await
        .unwrap();
    let mut other_schema = schema();
    other_schema.position = position(5);
    other_schema.key.id = id("other");
    backend
        .write(write_batch(&[StoragePart::Schemas(
            &[other_schema.clone()],
        )]))
        .await
        .unwrap();
    let mut foreign = record(6);
    foreign.schema = other_schema.key;
    backend
        .write(write_batch(&[StoragePart::Records(&[foreign])]))
        .await
        .unwrap();
    backend
        .write(write_batch(&[StoragePart::Stream(&[StoredRow {
            record: record(7),
            stream: id("samples"),
            sequence: 1,
        }])]))
        .await
        .unwrap();
    let saved = view(&backend, 7).await;
    let expected = [first.clone(), update, other];
    let page = backend
        .read_records(&saved, &selection, None, limits(10))
        .await
        .unwrap();
    assert_eq!(page.entries, expected);
    assert_eq!(
        Counter::decode(page.entries[2].payload.as_slice())
            .unwrap()
            .counter,
        u64::MAX
    );
    assert_eq!(
        backend
            .read_records(&old, &selection, None, limits(10))
            .await
            .unwrap()
            .entries,
        [first.clone()]
    );
    backend.release_view(declared.snapshot()).await.unwrap();
    backend.release_view(old.snapshot()).await.unwrap();
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
    backend.open().await.unwrap();
    let old = view(&backend, 2).await;
    let saved = view(&backend, 7).await;
    assert_eq!(
        backend
            .read_records(&old, &selection, None, limits(10))
            .await
            .unwrap()
            .entries,
        [first]
    );
    assert_eq!(
        backend
            .read_records(&saved, &selection, None, limits(10))
            .await
            .unwrap()
            .entries,
        expected
    );
    backend.release_view(old.snapshot()).await.unwrap();
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}
