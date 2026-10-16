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
#[path = "sqlite/main.rs"]
pub(crate) mod sqlite;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::storage::{
    ScanPage, StorageBackend, StorageBatch, StorageCursor, StoragePart, StorageSchema,
    StoredRecord, query::RecordSelection,
};

// Local Imports
use dokime::{
    core::{Identifier, RecordId, RecordScope, SessionId, Timestamp, Value},
    records::{RecordEntry, RecordPublishRequest, RecordSeverity},
    storage::{records, sqlite::Backend},
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
    let mut record = RecordPublishRequest::new(
        Identifier::new("charger.measurement").unwrap(),
        RecordSeverity::Info,
    )
    .into_record(
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
        (
            "limits".into(),
            Value::Array(vec![Value::Integer(-2), Value::Unsigned(u64::MAX)]),
        ),
    ]);
    RecordEntry::from_record(sequence, record)
}

/* === Tests === */

pub(crate) async fn portable_records_identity() {
    let root = tempfile::tempdir().unwrap();
    let writer = Backend::<()>::primary(root.path().to_path_buf())
        .await
        .unwrap();
    writer.open().await.unwrap();
    let history = writer.history().clone();
    let position = |sequence| StorageCursor {
        history: history.clone(),
        sequence,
    };
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
    writer
        .write(
            StorageBatch::new(
                &[StoragePart::Records(&stored)],
                (16 * 1024 * 1024).try_into().unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let saved = writer.cursor_saved().await.unwrap();
    writer.close().await.unwrap();

    let reader = Backend::<()>::primary(root.path().to_path_buf())
        .await
        .unwrap();
    assert_eq!(reader.history(), &history);
    let snapshot = reader.open_snapshot(&saved).await.unwrap();
    let selection = RecordSelection {
        schema,
        scope: None,
        item: None,
        source_start: None,
        source_end: None,
    };
    let mut after = 0;
    let mut actual = Vec::new();
    for stored in &stored {
        let page = reader
            .read_records(
                &snapshot,
                &selection,
                ScanPage {
                    after,
                    rows: 1.try_into().unwrap(),
                    bytes: FRAME_BYTES.try_into().unwrap(),
                },
            )
            .await
            .unwrap();
        assert_eq!(page.as_slice(), std::slice::from_ref(stored));
        actual.push(records::decode(&page[0].payload, FRAME_BYTES).unwrap());
        after = page[0].position.sequence;
    }
    assert!(
        reader
            .read_records(
                &snapshot,
                &selection,
                ScanPage {
                    after,
                    rows: 1.try_into().unwrap(),
                    bytes: FRAME_BYTES.try_into().unwrap(),
                },
            )
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(actual, expected);
    reader.release_snapshot(&snapshot).await.unwrap();
    reader.close().await.unwrap();
}

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn recovery_contract() {
    {
        eprintln!("Scenario: portable_records_identity");
        run_scenario(portable_records_identity, false, None, false);
    }
    {
        eprintln!("Scenario: objects::materialized_cases_reader");
        run_scenario(objects::materialized_cases_reader, false, None, false);
    }
    {
        eprintln!("Scenario: objects::suite_definitions_producer");
        run_scenario(objects::suite_definitions_producer, false, None, false);
    }
    {
        eprintln!("Scenario: objects::cycles_reopen_producers");
        run_scenario(objects::cycles_reopen_producers, false, None, false);
    }
    {
        eprintln!("Scenario: objects::executions_reopen_results");
        run_scenario(objects::executions_reopen_results, false, None, false);
    }
    {
        eprintln!("Scenario: objects::prompt_level_rows");
        objects::prompt_level_rows();
    }
    {
        eprintln!("Scenario: reports::report_publication_reopens");
        run_scenario(reports::report_publication_reopens, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::catalog::source_order_restart");
        run_scenario(sqlite::catalog::source_order_restart, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_previous_data");
        run_scenario(sqlite::refuses_previous_data, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_unversioned_schema");
        run_scenario(sqlite::refuses_unversioned_schema, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_retired_intent");
        run_scenario(sqlite::refuses_retired_intent, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_attachment_receipts");
        run_scenario(sqlite::refuses_attachment_receipts, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_second_drop");
        run_scenario(sqlite::refuses_second_drop, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::sqlite_rolls_reopen");
        run_scenario(sqlite::sqlite_rolls_reopen, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::cancelled_caller_progress");
        run_scenario(sqlite::cancelled_caller_progress, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::close_drains_clones");
        run_scenario(sqlite::close_drains_clones, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::production::checkpoint_recovery_fence");
        run_scenario(
            sqlite::production::checkpoint_recovery_fence,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::production::shared_sha_identity");
        run_scenario(sqlite::production::shared_sha_identity, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::production::mutation_receipts_outcomes");
        run_scenario(
            sqlite::production::mutation_receipts_outcomes,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::production::checkpoints_advance_boundaries");
        run_scenario(
            sqlite::production::checkpoints_advance_boundaries,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::production::late_checkpoints_settle");
        run_scenario(
            sqlite::production::late_checkpoints_settle,
            false,
            None,
            false,
        );
    }
}

#[test]
fn rows_contract() {
    {
        eprintln!("Scenario: sqlite::contract::paged_records_ascend");
        run_scenario(sqlite::contract::paged_records_ascend, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::contract::retries_are_exact");
        run_scenario(sqlite::contract::retries_are_exact, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::contract::durable_reopen");
        run_scenario(sqlite::contract::durable_reopen, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::contract::corrupt_entry_refused");
        run_scenario(sqlite::contract::corrupt_entry_refused, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::contract::snapshots_pin_saved_cuts");
        run_scenario(
            sqlite::contract::snapshots_pin_saved_cuts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::contract::finished_files_publish");
        run_scenario(sqlite::contract::finished_files_publish, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::contract::cancelled_read_isolated");
        run_scenario(
            sqlite::contract::cancelled_read_isolated,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::contract::close_releases_owner");
        run_scenario(sqlite::contract::close_releases_owner, false, None, false);
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
