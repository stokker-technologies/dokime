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
use dokime_protocol::client::content::{ExecutionFileMetadata, ExecutionFileState};
use dokime_protocol::v1;
use dokime_protocol::v1::{FftDetrend, FftWindow, GridAxisEntry, GridDefinition, PlotFftOptions, Reduction};
use dokime_storage::{BatchSelection, ExecutionState, KindWindow, NumericWindow, Result, SampleWindow, Store, storage_defaults};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct ContainerRow {
    #[prost(uint64, tag = "1")]
    value: u64,

    #[prost(string, tag = "2")]
    label: String,

    #[prost(btree_map = "string,uint64", tag = "3")]
    counts: std::collections::BTreeMap<String, u64>,

    #[prost(uint64, repeated, tag = "4")]
    series: Vec<u64>,
}

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
fn frequency_selects_latest_source_samples_across_long_history_and_restart() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut settings = storage_defaults();
    settings.flush_rows = NonZeroUsize::new(32).unwrap();
    let store = Store::open_with(root.path(), settings)?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    let mut samples = fixture::samples(2048);
    for (index, sample) in samples.iter_mut().enumerate() {
        sample.source_ns = if index < 2032 { index as i64 * 120_000_000_000 } else { (259_200 + index as i64 - 2032) * 1_000_000_000 };
        sample.row = Row {
            value: [10, 11, 10, 9][index % 4],
            label: String::new(),
        }
        .encode_to_vec();
    }
    for batch in samples.chunks(32) {
        store.sample_batch("execution", batch)?;
    }
    store.flush_samples()?;
    let fixed = store.cut("execution")?.frontier;
    let mut late = samples[0].clone();
    late.sequence = 2049;
    late.row = Row { value: 999, label: String::new() }.encode_to_vec();
    store.sample_batch("execution", &[late])?;
    store.flush_samples()?;
    let options = PlotFftOptions {
        input_samples: Some(16),
        sample_rate_hz: Some(1.0),
        window: FftWindow::Rectangular as i32,
        detrend: FftDetrend::Mean as i32,
        ..Default::default()
    };
    let read = |store: &Store, through| {
        store.spectrum_window(
            KindWindow {
                window: numeric(through),
                x: None,
                z: None,
                group: None,
                limit: 32,
            },
            &options,
        )
    };
    let (column, count) = read(&store, fixed)?;
    assert_eq!(count, 16);
    let peak = column.y.iter().enumerate().max_by(|(_, a), (_, b)| a.value.unwrap().total_cmp(&b.value.unwrap())).unwrap().0;
    assert_eq!(column.x[peak].kind, Some(v1::value::Kind::Number(0.25)));
    assert!((column.y[peak].value.unwrap() - 1.0).abs() < 1e-10);
    assert_eq!(read(&store, store.cut("execution")?.frontier)?.0, column);
    let axes = [9, 10, 11].map(|value| GridAxisEntry {
        key: value.to_string(),
        label: value.to_string(),
        ..Default::default()
    });
    let (grid, count) = store.grid_window(
        KindWindow {
            window: numeric(fixed),
            x: Some("value"),
            z: Some("value"),
            group: None,
            limit: 18,
        },
        &GridDefinition {
            rows: axes.to_vec(),
            columns: axes.to_vec(),
            ..Default::default()
        },
        Reduction::Mean,
    )?;
    assert_eq!(count, 2048);
    assert_eq!(
        grid.frames.iter().map(|frame| frame.instant.as_str()).collect::<Vec<_>>(),
        ["1970-01-04T00:00:14Z", "1970-01-04T00:00:15Z"]
    );
    assert_eq!(grid.frames[1].values.iter().copied().filter(|value| *value != 0.0).collect::<Vec<_>>(), [9.0]);
    let refused = store.spectrum_window(
        KindWindow {
            window: numeric(fixed),
            x: None,
            z: None,
            group: None,
            limit: 8,
        },
        &options,
    );
    assert!(refused.is_err());
    store.close_streams("execution")?;
    drop(store);
    let store = Store::open(root.path())?;
    assert_eq!(read(&store, fixed)?.0, column);
    Ok(())
}

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
    db.execute_batch("DROP TRIGGER refuse_batch; CREATE TABLE fault_parent(id INTEGER PRIMARY KEY); CREATE TABLE fault_commit(id INTEGER REFERENCES fault_parent(id) DEFERRABLE INITIALLY DEFERRED); CREATE TRIGGER refuse_commit AFTER INSERT ON telemetry_batches BEGIN INSERT INTO fault_commit VALUES(1); END;")?;
    assert!(store.flush_samples().is_err());
    assert_eq!(store.cut("execution")?.frontier, 0);
    db.execute_batch("DROP TRIGGER refuse_commit; DROP TABLE fault_commit; DROP TABLE fault_parent;")?;
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
    store.sample_batch("execution", &fixture::samples(3))?;
    store.register_execution_file(
        "capture",
        "execution",
        ExecutionFileMetadata {
            relative_path: "capture.bin".into(),
            state: ExecutionFileState::Streaming as i32,
            ..Default::default()
        },
    )?;
    db.execute_batch(
        "CREATE TRIGGER refuse_barrier BEFORE INSERT ON telemetry_batches BEGIN SELECT RAISE(ABORT,'injected barrier failure'); END;",
    )?;
    assert!(store.finish_execution_file("capture", false).is_err());
    assert_eq!(store.cut("execution")?.frontier, 1034);
    db.execute_batch("DROP TRIGGER refuse_barrier;")?;
    store.finish_execution_file("capture", false)?;
    assert_eq!(store.cut("execution")?.frontier, 1038);
    db.execute_batch(
        "CREATE TRIGGER refuse_closure BEFORE UPDATE OF closed ON streams BEGIN SELECT RAISE(ABORT,'injected closure failure'); END;",
    )?;
    assert!(store.close_streams("execution").is_err());
    assert!(!store.streams_closed("execution")?);
    db.execute_batch("DROP TRIGGER refuse_closure;")?;
    store.close_streams("execution")?;
    assert!(store.streams_closed("execution")?);
    let saved = store.cut("execution")?.frontier;
    assert!(store.sample_batch("execution", &fixture::samples(1)).is_err());
    drop(finite);
    drop(store);
    let reopened = Store::open(root.path())?;
    assert_eq!(reopened.cut("execution")?.frontier, saved);
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
    let statistics = store.numeric_statistics(&numeric(3))?;
    assert_eq!(statistics.count, 3);
    assert!((statistics.stddev - (2.0f64 / 3.0).sqrt()).abs() < 1e-12);
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
    store.shutdown_samples().await?;
    Ok(())
}

#[tokio::test]
async fn pinned_native_arrow_pages_keep_their_saved_cut() -> Result<()> {
    use harmos_storage::query::columnar::{ArrowScan, Cancellation, ColumnKind, ColumnSource};
    use harmos_storage::{PayloadEncoding, ReadLimits, SchemaKey, StorageBackend};
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    store.sample_batch("execution", &fixture::samples(23))?;
    store.flush_samples()?;
    let backend = dokime_storage::Backend::for_execution(store.clone(), "execution")?;
    let saved = backend.get_saved_position().await?;
    let snapshot = backend.create_snapshot(&saved).await?;
    store.sample_batch("execution", &fixture::samples(10))?;
    store.flush_samples()?;
    let mut scan = ArrowScan {
        source: ColumnSource {
            kind: ColumnKind::Stream,
            schema: SchemaKey {
                id: harmos_storage::Identifier::new("readings").unwrap(),
                version: 1.try_into().unwrap(),
                payload_encoding: PayloadEncoding::PostcardV1,
            },
        },
        columns: vec!["values.value".into()],
        predicate: None,
        time: None,
        start: 0,
        after: None,
        limits: ReadLimits {
            rows: NonZeroUsize::new(7).unwrap(),
            bytes: NonZeroUsize::new(4096).unwrap(),
        },
        work: ReadLimits {
            rows: NonZeroUsize::new(10).unwrap(),
            bytes: NonZeroUsize::new(4096).unwrap(),
        },
    };
    let cancel = Cancellation::default();
    let version = scan.source.schema.version;
    scan.source.schema.version = (version.get() + 1).try_into().unwrap();
    assert!(backend.list_column_rows(&snapshot, &scan, &cancel).await.is_err());
    scan.source.schema.version = version;
    let mut changed = snapshot.clone();
    changed.at.sequence += 1;
    assert!(backend.list_column_rows(&changed, &scan, &cancel).await.is_err());
    assert!(backend.delete_snapshot(&changed).await.is_err());
    let mut values = Vec::new();
    loop {
        let page = backend.list_column_rows(&snapshot, &scan, &cancel).await?;
        let column = page.batch.column(0).as_any().downcast_ref::<UInt64Array>().unwrap();
        values.extend(column.values().iter().copied());
        if page.complete {
            break;
        }
        scan.after = page.next;
    }
    assert_eq!(values, (0..23).collect::<Vec<_>>());
    store.close_streams("execution")?;
    store.close("execution", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
    store.seal("execution")?;
    let seal = store.cut("execution")?.seal;
    let response = dokime_protocol::client::ExecutionsDeleteResponse::default();
    assert!(store.delete_sealed_execution("delete", "execution", &seal, &response).is_err());
    backend.delete_snapshot(&snapshot).await?;
    store.delete_sealed_execution("delete", "execution", &seal, &response)?;
    assert_eq!(std::fs::read_dir(root.path().join("telemetry"))?.count(), 0);
    assert!(backend.list_column_rows(&snapshot, &scan, &cancel).await.is_err());
    backend.delete_session().await?;
    Ok(())
}
#[test]
fn rejected_rebuilt_root_preserves_its_database() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("execution.sqlite3");
    let reference = tempfile::tempdir()?;
    drop(Store::open(reference.path())?);
    let identity: i64 =
        rusqlite::Connection::open(reference.path().join("execution.sqlite3"))?.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    let old = rusqlite::Connection::open(&path)?;
    old.execute_batch(&format!("PRAGMA application_id={identity}; PRAGMA user_version=1; CREATE TABLE samples(payload BLOB); INSERT INTO samples VALUES(X'010203');"))?;
    drop(old);
    let before = std::fs::read(&path)?;
    assert!(matches!(Store::open(root.path()), Err(dokime_storage::Error::Incompatible(_))));
    assert_eq!(std::fs::read(&path)?, before);
    Ok(())
}

#[test]
fn record_intervals_anchor_before_aligning_to_later_samples() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let mut execution = fixture::execution();
    let mut catalog = harmos_protocol::v1::Catalog::decode(execution.catalog.as_slice())?;
    catalog.types.as_mut().unwrap().file[0].message_type.push(prost_types::DescriptorProto {
        name: Some("Marker".into()),
        ..Default::default()
    });
    let contracts = v1::RecordContracts {
        records: ["start", "finish"]
            .into_iter()
            .map(|id| v1::RecordContract {
                metadata: Some(v1::Metadata {
                    identifier: id.into(),
                    ..Default::default()
                }),
                message: "fixture.Marker".into(),
            })
            .collect(),
    };
    catalog.sections.push(Default::default());
    let section = catalog.sections.last_mut().unwrap();
    section.r#type = "dokime.v1.RecordContracts".into();
    section.payload = contracts.encode_to_vec();
    execution.catalog = catalog.encode_to_vec();
    store.create(&execution)?;
    store.open_stream("execution", "readings")?;
    for (record, time) in [("start", 2_000_000_000), ("finish", 4_000_000_000), ("start", 6_000_000_000)] {
        let payload = dokime_protocol::context::evidence::RecordsAppendRequest {
            payload: Vec::new(),
            ..Default::default()
        };
        store.append(
            "execution",
            dokime_storage::Evidence {
                kind: format!("record:{record}"),
                source_ns: time,
                payload: payload.encode_to_vec(),
                ..Default::default()
            },
        )?;
    }
    let mut rows = fixture::samples(3);
    for (row, time) in rows.iter_mut().zip([1_000_000_000, 5_000_000_000, 9_000_000_000]) {
        row.source_ns = time;
    }
    store.sample_batch("execution", &rows)?;
    store.flush_samples()?;
    let expression = v1::DataExpression {
        kind: Some(v1::data_expression::Kind::Transform(Box::new(v1::TransformExpression {
            input: Some(Box::new(v1::DataExpression {
                kind: Some(v1::data_expression::Kind::Record(v1::FieldReference {
                    identifier: "finish".into(),
                    field: String::new(),
                })),
                ..Default::default()
            })),
            kind: Some(v1::transform_expression::Kind::TimeBetween("start".into())),
        }))),
        ..Default::default()
    };
    let (reader, _cancel) = store.read_session()?;
    let cut = reader.cut("execution")?;
    let selected = reader.plot_selection(
        &cut,
        &v1::PlotRequest {
            stream: "readings".into(),
            y_expressions: vec![expression],
            ..Default::default()
        },
    )?;
    let Some(v1::data_expression::Kind::Stream(field)) = &selected.request.y_expressions[0].kind else { unreachable!() };
    let statistics = reader.numeric_statistics(&NumericWindow {
        field: &field.field,
        ..numeric(cut.frontier)
    })?;
    assert_eq!(statistics.count, 2);
    assert_eq!(statistics.mean, 2.0);
    Ok(())
}

#[test]
fn typed_map_and_list_columns_survive_finalization_and_restart() -> Result<()> {
    use prost_types::{
        DescriptorProto, FieldDescriptorProto, MessageOptions,
        field_descriptor_proto::{Label, Type},
    };
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let mut definition = fixture::execution();
    let mut catalog = harmos_protocol::v1::Catalog::decode(definition.catalog.as_slice())?;
    let row = &mut catalog.types.as_mut().unwrap().file[0].message_type[0];
    row.nested_type.push(DescriptorProto {
        name: Some("CountsEntry".into()),
        options: Some(MessageOptions {
            map_entry: Some(true),
            ..Default::default()
        }),
        field: [("key", Type::String), ("value", Type::Uint64)]
            .into_iter()
            .enumerate()
            .map(|(i, (name, kind))| FieldDescriptorProto {
                name: Some(name.into()),
                number: Some(i as i32 + 1),
                label: Some(Label::Optional as i32),
                r#type: Some(kind as i32),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    });
    row.field.push(FieldDescriptorProto {
        name: Some("counts".into()),
        number: Some(3),
        label: Some(Label::Repeated as i32),
        r#type: Some(Type::Message as i32),
        type_name: Some(".fixture.Row.CountsEntry".into()),
        ..Default::default()
    });
    row.field.push(FieldDescriptorProto {
        name: Some("series".into()),
        number: Some(4),
        label: Some(Label::Repeated as i32),
        r#type: Some(Type::Uint64 as i32),
        ..Default::default()
    });
    definition.catalog = catalog.encode_to_vec();
    store.create(&definition)?;
    store.open_stream("execution", "readings")?;
    let values = [
        ContainerRow {
            value: 1,
            label: "first".into(),
            counts: std::collections::BTreeMap::from([("large".into(), u64::MAX)]),
            series: vec![0, u64::MAX],
        },
        ContainerRow {
            value: 2,
            label: "empty".into(),
            counts: Default::default(),
            series: Vec::new(),
        },
    ];
    let mut rows = fixture::samples(2);
    for (sample, value) in rows.iter_mut().zip(&values) {
        sample.row = value.encode_to_vec();
    }
    store.sample_batch("execution", &rows)?;
    store.close_streams("execution")?;
    drop(store);
    let store = Store::open(root.path())?;
    let rows = store.sample_page("execution", "readings", 0, 2, NonZeroUsize::new(2).unwrap())?;
    let actual = rows.iter().map(|sample| ContainerRow::decode(sample.row.as_slice())).collect::<std::result::Result<Vec<_>, _>>()?;
    assert_eq!(actual, values);
    Ok(())
}
