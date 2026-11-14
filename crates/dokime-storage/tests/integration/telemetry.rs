/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow Durability and Saved Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{Cursor, Read},
    num::NonZeroUsize,
};

// Third Party Imports
use arrow_array::{Array, StructArray, UInt64Array};
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use super::fixture::{self, Row};
use dokime_protocol::v1;
use dokime_storage::{BatchSelection, ExecutionState, NumericWindow, Result, SampleWindow, Store, storage_defaults};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn selection(through: u64) -> BatchSelection<'static> {
    BatchSelection {
        execution: "execution",
        stream: "readings",
        after: 0,
        through,
        start_ns: i64::MIN,
        end_ns: i64::MAX,
    }
}
fn numeric(through: u64) -> NumericWindow<'static> {
    NumericWindow {
        execution: "execution",
        stream: "readings",
        field: "value",
        after: 0,
        through,
        start_ns: i64::MIN,
        end_ns: i64::MAX,
        bins: NonZeroUsize::new(16).unwrap(),
    }
}

/* === Tests === */

#[test]
fn accepted_tail_saved_cut_and_failed_catalog_commit() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    store.sample_batch("execution", &fixture::samples(1024))?;
    assert_eq!(store.cut("execution")?.frontier, 0);
    let mut live = 0;
    store.read_batches(selection(store.accepted_frontier("execution")?), &["value".into()], |batch| {
        live += batch.num_rows();
        Ok(true)
    })?;
    assert_eq!(live, 1024);
    let db = rusqlite::Connection::open(root.path().join("execution.sqlite3"))?;
    db.execute_batch(
        "CREATE TRIGGER refuse_batch BEFORE INSERT ON telemetry_batches BEGIN SELECT RAISE(ABORT,'injected catalog failure'); END;",
    )?;
    assert!(store.flush_samples().is_err());
    assert_eq!(store.cut("execution")?.frontier, 0);
    db.execute_batch("DROP TRIGGER refuse_batch")?;
    store.flush_samples()?;
    let cut = store.cut("execution")?;
    assert_eq!(cut.frontier, 1024);
    let (finite, _cancellation) = store.read_session()?;
    store.sample_batch("execution", &fixture::samples(10))?;
    store.flush_samples()?;
    let mut count = 0;
    finite.read_batches(selection(cut.frontier), &["value".into()], |batch| {
        count += batch.num_rows();
        let values = batch.column(1).as_any().downcast_ref::<StructArray>().unwrap();
        assert_eq!(values.num_columns(), 1);
        assert!(values.column(0).as_any().downcast_ref::<UInt64Array>().is_some());
        Ok(true)
    })?;
    assert_eq!(count, 1024);
    Ok(())
}
#[test]
fn compressed_long_execution_restarts_and_archive_preserves_bytes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    let rows = fixture::samples(65_536);
    store.sample_batch("execution", &rows)?;
    store.flush_samples()?;
    drop(store);
    let store = Store::open(root.path())?;
    store.sample_batch("execution", &fixture::samples(4096))?;
    store.close_streams("execution")?;
    store.close(
        "execution",
        v1::Termination::Completed,
        v1::ExecutionOutcome {
            attribution: Some(v1::execution_outcome::Attribution::NotEvaluated(v1::UnevaluatedOutcome::default())),
            ..Default::default()
        },
    )?;
    store.seal("execution")?;
    assert_eq!(store.cut("execution")?.state, ExecutionState::Sealed);
    let mut archive = Vec::new();
    store.write_execution_archive("execution", &mut archive)?;
    let mut zip = zip::ZipArchive::new(Cursor::new(archive))?;
    let mut copied = Vec::new();
    zip.by_name("telemetry/readings.arrow")?.read_to_end(&mut copied)?;
    let file = std::fs::read_dir(root.path().join("telemetry"))?.next().unwrap()?.path();
    let source = std::fs::read(file)?;
    assert_eq!(copied, source);
    let manifest: serde_json::Value = serde_json::from_reader(zip.by_name("manifest.json")?)?;
    assert_eq!(manifest["telemetry"][0]["size"], source.len());
    assert_eq!(manifest["telemetry"][0]["sha256"], Sha256::digest(&source).iter().map(|b| format!("{b:02x}")).collect::<String>());
    let window = SampleWindow {
        execution: "execution",
        streams: &[],
        after: &[],
        through: u64::MAX,
        start_ns: i64::MIN,
        end_ns: i64::MAX,
        latest: true,
        limit: NonZeroUsize::new(2).unwrap(),
    };
    assert_eq!(store.sample_window(window)?.len(), 2);
    Ok(())
}
#[test]
fn large_integer_document_median_is_exact() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    let mut samples = fixture::samples(3);
    let values = [u64::MAX, u64::MAX - 2, u64::MAX - 1];
    for (sample, value) in samples.iter_mut().zip(values) {
        sample.row = Row { value, label: "large".into() }.encode_to_vec();
    }
    store.sample_batch("execution", &samples)?;
    store.flush_samples()?;
    let input = v1::DataExpression {
        kind: Some(v1::data_expression::Kind::Stream(v1::FieldReference {
            identifier: "readings".into(),
            field: "value".into(),
        })),
        ..Default::default()
    };
    let expression = v1::DataExpression {
        kind: Some(v1::data_expression::Kind::Aggregate(Box::new(v1::AggregateExpression {
            input: Some(Box::new(input)),
            kind: v1::Aggregate::Median as i32,
            ..Default::default()
        }))),
        ..Default::default()
    };
    assert_eq!(store.document_value(&store.cut("execution")?, &expression)?.kind, Some(v1::value::Kind::UnsignedInteger(u64::MAX - 1)));
    assert_eq!(store.numeric_statistics(&numeric(3))?.count, 3);
    Ok(())
}
#[tokio::test(start_paused = true)]
async fn configured_clock_flushes_without_waiting_for_wall_time() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut settings = storage_defaults();
    settings.flush_rows = NonZeroUsize::new(10).unwrap();
    let interval = settings.flush_interval;
    let store = Store::open_with(root.path(), settings)?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    store.sample_batch("execution", &fixture::samples(3))?;
    assert_eq!(store.cut("execution")?.frontier, 0);
    tokio::task::yield_now().await;
    tokio::time::advance(interval).await;
    let mut changes = store.subscribe();
    while store.cut("execution")?.frontier < 3 {
        changes.changed().await.unwrap();
    }
    Ok(())
}
