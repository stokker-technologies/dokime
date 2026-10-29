/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Transactions, Indexed Admissions And Reopen Fault Boundaries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model::{
    execution::RecordScope,
    identity::{Identifier, RecordId, TestExecutionId},
    records::{
        RecordSeverity,
        evidence::{Record, RecordEntry},
    },
    time::Timestamp,
};
use dokime_runtime::storage::{records::RecordRow, sqlite::Backend};
use harmos::storage::{StoredRecord, query::RecordSelection};
use harmos::{
    schema::Described,
    storage::{StorageBackend, StorageBatch, StorageCursor},
};

/* === Constants === */

pub const CRASH_READY: &str = "DOKIME_EVIDENCE_READY";
const EXECUTION: &str = "durability.execution";
const OTHER: &str = "durability.other";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn evidence_durability(days: usize, rows_per_day: usize) {
    use dokime_runtime::storage::records;
    use harmos::storage::{StorageBackend, StorageSchema};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    };

    let root = tempfile::tempdir().unwrap();
    let mut store = open(root.path()).await;
    let schema = StorageSchema::new(position(&store, 1), records::schema(), record_schema());
    store.write_schemas(StorageBatch::new(vec![schema])).await.unwrap();
    let execution = TestExecutionId::from_seed(EXECUTION);
    let other = TestExecutionId::from_seed(OTHER);

    // Abort After A Row, During Its Capture Index, And Before Publishing The Frontier.
    for (table, when) in [
        ("storage_data_entries", "AFTER INSERT"),
        ("storage_record_captures", "AFTER INSERT"),
        ("storage_data_head", "BEFORE UPDATE OF saved"),
    ] {
        let fired = Arc::new(AtomicBool::new(false));
        let observed = Arc::clone(&fired);
        store
            .local_state()
            .call(move |connection| {
                connection.create_scalar_function("fixture_fault", 0, rusqlite::functions::FunctionFlags::SQLITE_UTF8, move |_| {
                    observed.store(true, Ordering::Relaxed);
                    Ok(0_i64)
                })?;
                let condition = if table == "storage_data_entries" {
                    "WHEN NEW.position='0000000000000003'"
                } else if table == "storage_record_captures" {
                    "WHEN NEW.ordinal='0000000000000002'"
                } else {
                    ""
                };
                connection.execute_batch(&format!(
                    "CREATE TEMP TRIGGER fixture_failure {when} ON {table} {condition}
                BEGIN SELECT fixture_fault(); SELECT RAISE(ABORT,'fixture'); END;"
                ))
            })
            .await
            .unwrap();
        let batch = record_batch(&store, 0, 3);
        assert!(store.write_records(StorageBatch::new(batch)).await.is_err());
        assert!(fired.load(Ordering::Relaxed));
        assert_counts(&store, 0).await;
        assert_eq!(store.cursor_saved().await.unwrap(), position(&store, 1));
        store.close().await.unwrap();
        drop(store);
        store = open(root.path()).await;
        assert_counts(&store, 0).await;
        assert_eq!(store.cursor_saved().await.unwrap(), position(&store, 1));
    }

    // Count The Actual Writer VM Work, Including Its Indexed Retry Queries.
    let steps = Arc::new(AtomicU64::new(0));
    let observed = Arc::clone(&steps);
    store
        .local_state()
        .call(move |connection| {
            connection.progress_handler(
                1,
                Some(move || {
                    observed.fetch_add(1, Ordering::Relaxed);
                    false
                }),
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let mut baseline = None;
    for day in 0..days {
        let rows = record_batch(&store, day * rows_per_day, rows_per_day);
        let before = steps.load(Ordering::Relaxed);
        let at = store.write_records(StorageBatch::new(rows.clone())).await.unwrap();
        let admitted = steps.load(Ordering::Relaxed) - before;
        let before = steps.load(Ordering::Relaxed);
        assert_eq!(store.write_records(StorageBatch::new(rows)).await.unwrap(), at);
        let retried = steps.load(Ordering::Relaxed) - before;
        let (first_admission, first_retry) = *baseline.get_or_insert((admitted, retried));
        assert!(admitted <= first_admission * 2, "admission work grew with retained history: {admitted}");
        assert!(retried <= first_retry * 2, "retry work grew with retained history: {retried}");
        assert_counts(&store, (day + 1) * rows_per_day).await;
    }
    let count = days * rows_per_day;
    let at = store.cursor_saved().await.unwrap();
    assert_eq!(at.sequence, count as u64 + 1);
    let history = store.history().clone();
    store.close().await.unwrap();
    drop(store);
    let store = open(root.path()).await;
    assert_eq!(store.history(), &history);
    assert_counts(&store, count).await;
    // A Lost Acknowledgement Can Retry An Older Saved Batch After Reopening.
    let retry = record_batch(&store, 0, rows_per_day);
    assert_eq!(store.write_records(StorageBatch::new(retry)).await.unwrap(), at);
    assert_counts(&store, count).await;
    let snapshot = store.open_snapshot(&at).await.unwrap();
    let selection = RecordSelection {
        schema: records::schema(),
        scope: None,
        item: None,
        source_start: None,
        source_end: None,
    };
    let mut after = 0;
    let mut read = 0;
    loop {
        let rows = store
            .read_records(
                &snapshot,
                &selection,
                harmos::storage::ScanPage {
                    after,
                    rows: std::num::NonZeroUsize::new(64).unwrap(),
                    bytes: std::num::NonZeroUsize::new(64 * 1024).unwrap(),
                },
            )
            .await
            .unwrap();
        assert!(rows.len() <= 64);
        if rows.is_empty() {
            break;
        }
        for row in &rows {
            let entry = records::decode(&row.payload, usize::MAX).unwrap();
            read += 1;
            assert_eq!(entry.ordinal, read as u64);
            assert_eq!(entry.captured_scopes.len(), 2);
            assert_eq!(entry.captured_scopes[1].test_execution_id.as_ref(), Some(&other));
            assert_eq!(entry.record.scope.as_ref().unwrap().test_execution_id.as_ref(), Some(&execution));
        }
        after = rows.last().unwrap().position.sequence;
    }
    assert_eq!(read, count);
    store.release_snapshot(&snapshot).await.unwrap();
    store.close().await.unwrap();
}

pub async fn crash_writer(root: std::path::PathBuf, phase: String) {
    use dokime_runtime::storage::records;
    use harmos::storage::{StorageBackend, StorageSchema};
    let store = open(&root).await;
    let schema = StorageSchema::new(position(&store, 1), records::schema(), record_schema());
    store.write_schemas(StorageBatch::new(vec![schema])).await.unwrap();
    if phase != "saved" {
        let (table, when, condition) = match phase.as_str() {
            "row" => ("storage_data_entries", "AFTER INSERT", "WHEN NEW.position='0000000000000003'"),
            "capture" => ("storage_record_captures", "AFTER INSERT", "WHEN NEW.ordinal='0000000000000002'"),
            "frontier" => ("storage_data_head", "BEFORE UPDATE OF saved", ""),
            _ => panic!("invalid crash fixture phase"),
        };
        store
            .local_state()
            .call(move |connection| {
                connection.create_scalar_function(
                    "fixture_stop",
                    0,
                    rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                    |_| -> rusqlite::Result<i64> { announce_and_wait() },
                )?;
                connection.execute_batch(&format!(
                    "CREATE TEMP TRIGGER fixture_stop {when} ON {table} {condition}
                BEGIN SELECT fixture_stop(); END;"
                ))
            })
            .await
            .unwrap();
    }
    store.write_records(StorageBatch::new(record_batch(&store, 0, 3))).await.unwrap();
    announce_and_wait();
}

pub async fn recovered_after_crash(root: std::path::PathBuf, committed: bool) {
    use harmos::storage::StorageBackend;
    let store = open(&root).await;
    let count = if committed { 3 } else { 0 };
    assert_counts(&store, count).await;
    assert_eq!(store.cursor_saved().await.unwrap(), position(&store, count as u64 + 1));
    let batch = record_batch(&store, 0, 3);
    let at = store.write_records(StorageBatch::new(batch.clone())).await.unwrap();
    assert_eq!(at, position(&store, 4));
    assert_eq!(store.write_records(StorageBatch::new(batch)).await.unwrap(), at);
    assert_counts(&store, 3).await;
    store.close().await.unwrap();
}

/* === Private Functions === */

async fn open(root: &std::path::Path) -> Backend<()> {
    let backend = Backend::primary(root.to_path_buf()).await.unwrap();
    backend.open().await.unwrap();
    backend
}

fn position(store: &Backend<()>, sequence: u64) -> StorageCursor {
    StorageCursor {
        history: store.history().clone(),
        sequence,
    }
}

fn record_schema() -> harmos::schema::graph::Schema {
    harmos::schema::graph::Schema::from_declaration(RecordRow::DEFINITION, &Default::default()).unwrap()
}

async fn assert_counts(store: &Backend<()>, count: usize) {
    let counts = store
        .local_state()
        .call(|connection| {
            connection.query_row(
                "SELECT (SELECT count(*) FROM storage_data_entries WHERE kind='record'),
            (SELECT count(*) FROM storage_record_visibility), (SELECT count(*) FROM storage_record_captures)",
                [],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)),
            )
        })
        .await
        .unwrap();
    let expected = i64::try_from(count).unwrap();
    assert_eq!(counts, (expected, expected, expected * 2));
}

fn record_batch(store: &Backend<()>, first: usize, count: usize) -> Vec<StoredRecord> {
    use dokime_runtime::storage::records;
    let execution = TestExecutionId::from_seed(EXECUTION);
    let other = TestExecutionId::from_seed(OTHER);
    (first..first + count)
        .map(|index| {
            let ordinal = index as u64 + 1;
            let scope = RecordScope {
                test_execution_id: Some(execution.clone()),
                ..Default::default()
            };
            let mut record = Record::new(
                RecordId::from_seed(format!("durability:{ordinal}")),
                Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH + time::Duration::minutes(index as i64)),
                Identifier::new("durability.reading").unwrap(),
                1,
                "fixture",
                RecordSeverity::Info,
            );
            record.scope = Some(scope.clone());
            let mut entry = RecordEntry::from_record(ordinal, record);
            entry.ordinal = ordinal;
            entry.captured_scopes = vec![
                scope,
                RecordScope {
                    test_execution_id: Some(other.clone()),
                    ..Default::default()
                },
            ];
            StoredRecord {
                position: position(store, ordinal + 1),
                schema: records::schema(),
                scope: Some(Identifier::new(execution.as_str()).unwrap()),
                item: Identifier::new(entry.record.record_id.as_str()).unwrap(),
                source: None,
                source_time: None,
                payload: records::encode(&entry, usize::MAX).unwrap(),
            }
        })
        .collect::<Vec<_>>()
}

fn announce_and_wait() -> ! {
    use std::io::Write;
    println!("\n{CRASH_READY}");
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::park();
    }
}

/* === Tests === */
