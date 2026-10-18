/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg(feature = "storage")]

/* === Modules === */
mod cancellation;
mod nominal;
pub(crate) mod objects;
pub(crate) mod reports;
pub(crate) mod sqlite;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::storage::{
    ScanPage, StorageBackend, StorageBatch, StorageCursor, StoragePart, StorageSchema, StoredRecord, query::RecordSelection,
};

// Local Imports
use dokime_domain::model;
use {
    dokime_runtime::Identifier, dokime_runtime::RecordId, dokime_runtime::RecordScope, dokime_runtime::SessionId,
    dokime_runtime::Timestamp, dokime_runtime::Value, dokime_runtime::records::RecordSeverity, dokime_runtime::storage::records,
    dokime_runtime::storage::sqlite::Backend, model::records::evidence::RecordEntry, model::records::publication::RecordPublishRequest,
};

/* === Constants === */
const FRAME_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn entry(sequence: u64, volts: f64) -> RecordEntry {
    let mut record = RecordPublishRequest::new(Identifier::new("charger.measurement").unwrap(), RecordSeverity::Info).into_record(
        RecordId::from_seed(sequence.to_string()),
        1,
        Timestamp::new("2026-10-04T12:00:00Z"),
        "bench.charger",
    );
    record.scope = Some(RecordScope::session(SessionId::from_seed("bench")));
    record.details = BTreeMap::from([
        ("voltage".into(), Value::Number(volts)),
        ("current".into(), Value::Number(3.25)),
        ("ready".into(), Value::Bool(true)),
        ("limits".into(), Value::Array(vec![Value::Integer(-2), Value::Unsigned(u64::MAX)])),
    ]);
    RecordEntry::from_record(sequence, record)
}

/* === Tests === */

#[tokio::test]
async fn portable_records_identity() {
    let root = tempfile::tempdir().unwrap();
    let writer = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
    writer.open().await.unwrap();
    let history = writer.history().clone();
    let position = |sequence| StorageCursor { history: history.clone(), sequence };
    let schema = records::schema();
    writer
        .write(
            StorageBatch::new(
                &[StoragePart::Schemas(&[StorageSchema::new(
                    position(1),
                    schema.clone(),
                    sqlite::storage::declared::<records::RecordRow>(),
                )])],
                (16 * 1024 * 1024).try_into().unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let expected = [entry(17, 229.5), entry(29, 230.5)];
    let stored = expected
        .iter()
        .enumerate()
        .map(|(index, entry)| StoredRecord {
            position: position(index as u64 + 2),
            schema: schema.clone(),
            scope: Some(Identifier::new("bench-epoch").unwrap()),
            item: Identifier::new(entry.record_id.to_string()).unwrap(),
            source: None,
            source_time: None,
            payload: records::encode(entry, FRAME_BYTES).unwrap(),
        })
        .collect::<Vec<_>>();
    writer.write(StorageBatch::new(&[StoragePart::Records(&stored)], (16 * 1024 * 1024).try_into().unwrap()).unwrap()).await.unwrap();
    let saved = writer.cursor_saved().await.unwrap();
    writer.close().await.unwrap();

    let reader = Backend::<()>::primary(root.path().to_path_buf()).await.unwrap();
    assert_eq!(reader.history(), &history);
    let snapshot = reader.open_snapshot(&saved).await.unwrap();
    let selection = RecordSelection { schema, scope: None, item: None, source_start: None, source_end: None };
    let mut after = 0;
    let mut actual = Vec::new();
    for stored in &stored {
        let page = reader
            .read_records(&snapshot, &selection, ScanPage { after, rows: 1.try_into().unwrap(), bytes: FRAME_BYTES.try_into().unwrap() })
            .await
            .unwrap();
        assert_eq!(page.as_slice(), std::slice::from_ref(stored));
        actual.push(records::decode(&page[0].payload, FRAME_BYTES).unwrap());
        after = page[0].position.sequence;
    }
    assert!(
        reader
            .read_records(&snapshot, &selection, ScanPage { after, rows: 1.try_into().unwrap(), bytes: FRAME_BYTES.try_into().unwrap() },)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(actual, expected);
    reader.release_snapshot(&snapshot).await.unwrap();
    reader.close().await.unwrap();
}
