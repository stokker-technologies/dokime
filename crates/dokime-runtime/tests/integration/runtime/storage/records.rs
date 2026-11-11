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

// Third Party Imports
use harmos::{Message, storage::RecordInput};

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::domains::evidence::RecordSelection;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn recovery_continues_from_durable_record_frontier() {
    let root = tempdir().unwrap();
    let settings = config(root.path());
    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty()).await.unwrap();
    let handle = runtime.handle();
    handle
        .add_operator_note(OperatorNoteRequest {
            scope: Some(RecordScope::default()),
            author: None,
            message: "Before Recovery".into(),
        })
        .await
        .unwrap();
    let mut durable = handle.selected_records(RecordSelection::default()).await.unwrap().pop().unwrap();
    let restored_count = handle.watch_state().latest().state().record_count;
    durable.record.record_id = dokime_runtime::model::identity::new_id();
    durable.ordinal = restored_count + 7;
    let row = dokime_runtime::storage::records::RecordRow::from_entry(&durable).unwrap();
    // Commit Evidence Without Advancing The State Journal, As In The Crash Window.
    let written = handle
        .storage()
        .write_records(vec![RecordInput {
            schema: dokime_runtime::storage::records::schema(),
            scope: None,
            item: harmos::Identifier::new(durable.record_id.as_str()).unwrap(),
            source: None,
            source_time: None,
            payload: row.encode(),
        }])
        .await
        .unwrap();
    handle.storage().flush_until(&written).await.unwrap();
    assert_eq!(handle.watch_state().latest().state().record_count, restored_count);
    drop(handle);
    runtime.shutdown().await.unwrap();

    for offset in 1..=2 {
        let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty()).await.unwrap();
        let handle = runtime.handle();
        handle
            .add_operator_note(OperatorNoteRequest {
                scope: Some(RecordScope::default()),
                author: None,
                message: "After Recovery".into(),
            })
            .await
            .unwrap();
        let mut records = handle.selected_records(RecordSelection::default()).await.unwrap();
        records.sort_by_key(|record| record.ordinal);
        assert!(records.iter().any(|record| record.record_id == durable.record_id && record.ordinal == durable.ordinal));
        assert_eq!(records.last().unwrap().ordinal, durable.ordinal + offset);
        assert!(records.windows(2).all(|pair| pair[0].ordinal < pair[1].ordinal));
        drop(handle);
        runtime.shutdown().await.unwrap();
    }
}

pub(crate) async fn managed_records_spool() {
    let root = tempdir().unwrap();
    let settings = config(root.path());
    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty()).await.unwrap();
    runtime.shutdown().await.unwrap();
    let persistent = root_files(root.path());

    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings.clone(), std::iter::empty()).await.unwrap();
    let handle = runtime.handle();
    handle
        .add_operator_note(OperatorNoteRequest {
            scope: Some(RecordScope::default()),
            author: Some("operator".into()),
            message: "A durable observation".into(),
        })
        .await
        .unwrap();
    let selection = RecordSelection {
        record_types: vec![Identifier::new("operator.note_added").unwrap()],
        ..RecordSelection::default()
    };
    let records = handle.selected_records(selection.clone()).await.unwrap();
    assert_eq!(records.len(), 1);
    drop(handle);
    runtime.shutdown().await.unwrap();
    assert_eq!(root_files(root.path()), persistent, "record capture creates no separate process spool files");

    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings, std::iter::empty()).await.unwrap();
    let recovered = runtime.handle().selected_records(selection).await.unwrap();
    assert_eq!(recovered, records, "global record visibility survives managed recovery");
    runtime.shutdown().await.unwrap();
    assert_eq!(root_files(root.path()), persistent);
}

fn root_files(root: &Path) -> Vec<std::ffi::OsString> {
    let mut files = std::fs::read_dir(root).unwrap().map(|entry| entry.unwrap().file_name()).collect::<Vec<_>>();
    files.sort();

    files
}
