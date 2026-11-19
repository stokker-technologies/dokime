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
    collections::BTreeMap,
    fs,
    io::{Cursor, Read, Write},
    num::NonZeroUsize,
    result,
};

// Third Party Imports
use arrow_array::{Array, StructArray, UInt64Array};
use harmos_domain::message::postcard::Value;
use prost::Message;
use prost_types::{
    DescriptorProto, FieldDescriptorProto, MessageOptions,
    field_descriptor_proto::{Label, Type},
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use zip::ZipArchive;

// Local Imports
use super::fixture::{self, Row};

use dokime_protocol::model;
use dokime_protocol::model::DataExpression;
use dokime_protocol::model::Downsample;
use dokime_protocol::model::FftDetrend;
use dokime_protocol::model::FftWindow;
use dokime_protocol::model::FieldReference;
use dokime_protocol::model::FilterOperator;
use dokime_protocol::model::GridAxisEntry;
use dokime_protocol::model::GridDefinition;
use dokime_protocol::model::GridHighlight;
use dokime_protocol::model::PlotFftOptions;
use dokime_protocol::model::Reduction;
use dokime_storage::{
    ArchivedContent, ArchivedNativeFile, BatchSelection, Error, Evidence, ExecutionCut, ExecutionState, KindWindow, NumericWindow, Result,
    Sample, SampleWindow, Store, StoredArtifact, StoredExecutionFile, storage_defaults,
};

use dokime_protocol::host::UnevaluatedOutcome;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::{NotesCreateResponse, OperatorNote};
use dokime_protocol::host::execution_outcome;
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::host::{CyclePhase, CycleSummary, ExecutionOutcome};
use dokime_protocol::model::Aggregate;
use dokime_protocol::model::AggregateExpression;
use dokime_protocol::model::ArtifactMetadata;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::ExactNumber;
use dokime_protocol::model::Metadata;
use dokime_protocol::model::PlotFilter;
use dokime_protocol::model::RecordContract;
use dokime_protocol::model::RecordContracts;
use dokime_protocol::model::Termination;
use dokime_protocol::model::TransformExpression;
use dokime_protocol::model::data_expression;
use dokime_protocol::model::transform_expression;
use dokime_protocol::model::value;
use dokime_storage::OutcomeCounts;
use harmos_protocol::model::Declarations;
use harmos_protocol::model::StreamDeclaration;
use harmos_protocol::model::{FileMetadata, FileReference};

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
    counts: BTreeMap<String, u64>,

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
fn cartesian_display_budget_counts_vertices_without_time_bucket_reduction() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    let mut samples = fixture::samples(41);
    for (index, sample) in samples.iter_mut().enumerate() {
        sample.source_ns = (index * index) as i64;
    }
    store.sample_batch("execution", &samples)?;
    store.flush_samples()?;
    for limit in [1, 2, 3, 4, 20, 41, 80] {
        let (frame, count, groups) = store.cartesian_window(KindWindow {
            window: NumericWindow {
                bins: NonZeroUsize::new(1).unwrap(),
                ..numeric(41)
            },
            x: Some("value"),
            z: None,
            group: None,
            limit,
        })?;
        assert_eq!(count, 41);
        assert_eq!(groups, vec![(String::new(), 41)]);
        assert_eq!(frame.point_count, limit.min(41) as u64);
        assert_eq!(frame.series[0].y.len(), limit.min(41));
        if limit >= 41 {
            assert_eq!(
                frame.series[0].y.iter().map(|point| point.value).collect::<Vec<_>>(),
                (0..41).map(|value| Some(value as f64)).collect::<Vec<_>>()
            );
        }
    }
    let grouped = KindWindow {
        window: numeric(41),
        x: Some("value"),
        z: None,
        group: Some("label"),
        limit: 2,
    };
    assert!(store.cartesian_window(grouped).is_err());
    for limit in [3, 4, 20, 41] {
        let grouped = KindWindow {
            window: numeric(41),
            x: Some("value"),
            z: None,
            group: Some("label"),
            limit,
        };
        let (frame, count, groups) = store.cartesian_window(grouped)?;
        assert_eq!(count, 41);
        assert_eq!(frame.point_count, limit as u64);
        assert_eq!(frame.series.len(), 3);
        assert_eq!(groups.iter().map(|(_, count)| count).sum::<u64>(), count);
    }
    Ok(())
}

#[test]
fn time_series_preserves_all_samples_within_budget_for_every_technique() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    store.sample_batch("execution", &fixture::samples(41))?;
    store.flush_samples()?;
    for technique in [Downsample::Unspecified, Downsample::Auto, Downsample::Raw, Downsample::MinMax, Downsample::Lttb] {
        for count in [1, 2, 41] {
            for limit in [count, count + 39] {
                let (column, summary, selected) = store.sampled_time_series_window(
                    NumericWindow {
                        bins: NonZeroUsize::new(1).unwrap(),
                        ..numeric(count as u64)
                    },
                    technique,
                    limit,
                )?;
                assert_eq!(selected, count as u64);
                assert_eq!(summary.count, selected);
                assert_eq!(
                    column.y.iter().map(|point| point.value).collect::<Vec<_>>(),
                    (0..count).map(|value| Some(value as f64)).collect::<Vec<_>>()
                );
                assert_eq!(column.x.len(), count);
            }
        }
    }
    Ok(())
}

#[test]
fn request_source_blocks_replay_exact_windows_and_cuts() -> Result<()> {
    // Save One Immutable Source Block.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    store.sample_batch("execution", &fixture::samples(6))?;
    store.flush_samples()?;
    let cut = store.cut("execution")?;
    let (mut reader, cancellation) = store.read_session()?;
    reader.reuse_sample_reads()?;
    let mut expected = Vec::new();
    store.read_batches(selection(6), &["value".into()], |batch| {
        expected.push(batch);
        Ok(true)
    })?;
    let mut actual = Vec::new();
    reader.read_batches(selection(6), &["value".into()], |batch| {
        actual.push(batch);
        Ok(true)
    })?;
    assert_eq!(actual, expected);

    // Different Projections and Windows Reuse the Original Bytes, Even If the Source Becomes Unavailable.
    let source = fs::read_dir(root.path().join("telemetry"))?.next().unwrap()?.path();
    let retained = source.with_extension("retained");
    fs::rename(&source, &retained)?;
    let expression = FieldReference::new("readings").field("label");
    let value = reader.document_value(&cut, &expression)?;
    assert_eq!(value.kind, Some(value::Kind::Text("group2".into())));
    let values = reader.sample_window(SampleWindow {
        execution: "execution",
        streams: &["readings".into()],
        after: &[("readings".into(), 1)],
        through: 4,
        start_ns: i64::MIN,
        end_ns: i64::MAX,
        latest: false,
        limit: NonZeroUsize::new(6).unwrap(),
    })?;
    assert_eq!(values.iter().map(|sample| sample.position).collect::<Vec<_>>(), vec![2, 3, 4]);
    let earlier = reader.sample_page("execution", "readings", 0, 3, NonZeroUsize::new(6).unwrap())?;
    assert_eq!(earlier.iter().map(|sample| sample.position).collect::<Vec<_>>(), vec![1, 2, 3]);

    // A New Request Cannot Borrow Another Request's Source Blocks.
    let (mut fresh, _fresh_cancellation) = store.read_session()?;
    fresh.reuse_sample_reads()?;
    assert!(fresh.document_value(&cut, &expression).is_err());
    fs::rename(&retained, &source)?;
    assert_eq!(fresh.document_value(&cut, &expression)?, value);

    // Cancellation Still Applies to Retained Bytes.
    drop(cancellation);
    assert!(reader.document_value(&cut, &expression).is_err());
    Ok(())
}

#[test]
fn finite_numeric_reads_keep_selection_and_request_boundaries() -> Result<()> {
    // Save and Reopen One Stream.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    store.sample_batch("execution", &fixture::samples(6))?;
    store.flush_samples()?;
    drop(store);
    let store = Store::open(root.path())?;
    let cut = store.cut("execution")?;
    let (reader, cancellation) = store.read_session()?;

    // Share the Full Selection Across Preview, Statistics, RMS and Nested Document Reads.
    let (_, summary, count) = reader.time_series_window(numeric(6))?;
    assert_eq!(count, 6);
    assert_eq!(
        summary.mean.unwrap().kind,
        Some(value::Kind::Exact(ExactNumber {
            negative: false,
            numerator: "5".into(),
            denominator: "2".into(),
        }))
    );
    let statistics = reader.numeric_statistics(&numeric(6))?;
    assert_eq!(statistics.count, 6);
    assert_eq!(statistics.sum.unwrap().kind, Some(value::Kind::UnsignedInteger(15)));
    assert_eq!(reader.numeric_rms(&numeric(6))?, (55.0_f64 / 6.0).sqrt());
    let expression = DataExpression {
        kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
            input: Some(Box::new(FieldReference::new("readings").field("value"))),
            kind: Aggregate::Sum as i32,
            percentile: 0.0,
        }))),
        ..Default::default()
    };
    assert_eq!(reader.document_value(&cut, &expression)?.kind, Some(value::Kind::UnsignedInteger(15)));

    // Retained Positions, Source Windows and Filters Must Not Reuse the Full Selection.
    let (_, _, count) = reader.time_series_window(numeric(3))?;
    assert_eq!(count, 3);
    let (_, summary, count) = reader.time_series_window(NumericWindow { after: 3, ..numeric(6) })?;
    assert_eq!(count, 3);
    assert_eq!(summary.mean.unwrap().kind, Some(value::Kind::UnsignedInteger(4)));
    let (_, summary, count) = reader.time_series_window(NumericWindow { start_ns: 2, end_ns: 4, ..numeric(6) })?;
    assert_eq!(count, 3);
    assert_eq!(summary.mean.unwrap().kind, Some(value::Kind::UnsignedInteger(3)));
    let request = PlotRequest {
        stream: "readings".into(),
        y_keys: vec!["value".into()],
        filters: vec![PlotFilter {
            expression: Some(FieldReference::new("readings").field("value")),
            operator: FilterOperator::Ge as i32,
            value: Some(model::Value {
                kind: Some(value::Kind::UnsignedInteger(4)),
            }),
        }],
        ..Default::default()
    };
    reader.plot_selection(&cut, &request)?;
    let filtered = reader.numeric_statistics(&numeric(6))?;
    assert_eq!(filtered.count, 2);
    assert_eq!(filtered.sum.unwrap().kind, Some(value::Kind::UnsignedInteger(9)));
    reader.plot_selection(&cut, &PlotRequest { filters: Vec::new(), ..request })?;
    assert_eq!(reader.numeric_statistics(&numeric(6))?.count, 6);

    // Each Selected Group Has Its Own Numeric Values.
    reader.plot_selection(
        &cut,
        &PlotRequest {
            stream: "readings".into(),
            series_by: Some("label".into()),
            ..Default::default()
        },
    )?;
    let groups = reader.plot_group_values(numeric(6), "label", 3)?;
    assert_eq!(groups.len(), 3);
    for (group, sum) in groups.iter().zip([3, 5, 7]) {
        reader.select_plot_group(Some(group))?;
        let statistics = reader.numeric_statistics(&numeric(6))?;
        assert_eq!(statistics.count, 2);
        assert_eq!(statistics.sum.unwrap().kind, Some(value::Kind::UnsignedInteger(sum)));
    }
    reader.select_plot_group(None)?;
    assert_eq!(reader.numeric_statistics(&numeric(6))?.count, 6);

    // Cancellation Applies Even After a Result Was Computed; Another Request Has Its Own Saved Image.
    drop(cancellation);
    assert!(reader.numeric_rms(&numeric(6)).is_err());
    drop(reader);
    store.sample_batch("execution", &fixture::samples(2))?;
    store.flush_samples()?;
    let (reader, _cancellation) = store.read_session()?;
    assert_eq!(reader.numeric_statistics(&numeric(8))?.count, 8);
    assert_eq!(reader.numeric_statistics(&numeric(6))?.count, 6);
    Ok(())
}

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
    assert_eq!(column.x[peak].kind, Some(value::Kind::Number(0.25)));
    assert!((column.y[peak].value.unwrap() - 1.0).abs() < 1e-10);
    assert_eq!(read(&store, store.cut("execution")?.frontier)?.0, column);
    let axes = [9, 10, 11].map(|value| GridAxisEntry {
        key: value.to_string(),
        label: value.to_string(),
        ..Default::default()
    });
    let cell = DataExpression {
        kind: Some(data_expression::Kind::Stream(FieldReference {
            identifier: "readings".into(),
            field: "value".into(),
        })),
        ..Default::default()
    };
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
            highlights: vec![GridHighlight {
                row: Some(cell.clone()),
                column: Some(cell),
                label: "Selected Cell".into(),
            }],
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
    assert_eq!(
        grid.frames.iter().flat_map(|frame| frame.highlights.iter().map(|cell| (cell.row, cell.column))).collect::<Vec<_>>(),
        [(1, 1), (0, 0)]
    );
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
    let db = Connection::open(root.path().join("execution.sqlite3"))?;
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
    db.execute_batch(
        "CREATE TRIGGER refuse_barrier BEFORE INSERT ON telemetry_batches BEGIN SELECT RAISE(ABORT,'injected barrier failure'); END;",
    )?;
    assert!(store.flush_samples().is_err());
    assert_eq!(store.cut("execution")?.frontier, 1034);
    db.execute_batch("DROP TRIGGER refuse_barrier;")?;
    store.flush_samples()?;
    assert_eq!(store.cut("execution")?.frontier, 1037);
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
fn execution_archive_orders_native_content_deterministically() -> Result<()> {
    // Seal Source Evidence.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.close(
        "execution",
        Termination::Completed,
        ExecutionOutcome {
            attribution: Some(execution_outcome::Attribution::NotEvaluated(UnevaluatedOutcome::default())),
        },
    )?;
    store.seal("execution", Default::default())?;

    // Export Opposite Source Inventory Orders.
    let mut archives = Vec::new();
    for names in [["first", "second"], ["second", "first"]] {
        let mut content = ArchivedContent::default();
        for name in names {
            let data = name.as_bytes();
            let digest = Sha256::digest(data).to_vec();
            let reference = FileReference {
                metadata: Some(FileMetadata {
                    id: name.into(),
                    scope: Some("execution/execution".into()),
                    media_type: "application/octet-stream".into(),
                    expected_length: Some(data.len() as u64),
                    expected_sha256: Some(digest.clone()),
                }),
                length: data.len() as u64,
                sha256: digest,
                ..Default::default()
            };
            let mut file = NamedTempFile::new()?;
            file.write_all(data)?;
            content.files.push(ArchivedNativeFile {
                reference: reference.clone(),
                path: file.into_temp_path(),
            });
            let artifact = StoredArtifact {
                artifact: Some(ArtifactMetadata {
                    artifact_id: name.into(),
                    ..Default::default()
                }),
                ..Default::default()
            };
            content.artifacts.push(artifact.encode_to_vec());
            content.execution_files.push(StoredExecutionFile {
                file: Some(reference),
                ..Default::default()
            });
        }
        let mut archive = Vec::new();
        store.write_execution_archive("execution", &mut archive, content)?;
        archives.push(archive);
    }
    assert_eq!(archives[0], archives[1]);

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
        Termination::Completed,
        ExecutionOutcome {
            attribution: Some(execution_outcome::Attribution::NotEvaluated(UnevaluatedOutcome::default())),
        },
    )?;
    store.seal("execution", Default::default())?;
    assert_eq!(store.cut("execution")?.state, ExecutionState::Sealed);
    let mut archive = Vec::new();
    store.write_execution_archive("execution", &mut archive, Default::default())?;
    let mut zip = ZipArchive::new(Cursor::new(archive))?;
    let mut copied = Vec::new();
    zip.by_name("telemetry/readings.arrow")?.read_to_end(&mut copied)?;
    let file = fs::read_dir(root.path().join("telemetry"))?.next().unwrap()?.path();
    let source = fs::read(file)?;
    assert_eq!(copied, source);
    let manifest: serde_json::Value = serde_json::from_reader(zip.by_name("manifest.json")?)?;
    assert_eq!(
        manifest["entries"].as_array().unwrap().iter().find(|entry| entry["path"] == "telemetry/readings.arrow").unwrap()["size"],
        source.len()
    );
    assert_eq!(
        manifest["entries"].as_array().unwrap().iter().find(|entry| entry["path"] == "telemetry/readings.arrow").unwrap()["sha256"],
        Sha256::digest(&source).iter().map(|b| format!("{b:02x}")).collect::<String>()
    );
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
fn integer_document_medians_preserve_exact_values() -> Result<()> {
    let base = 1_152_921_504_606_847_975u64;
    for signed in [false, true] {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let mut execution = fixture::execution();
        if signed {
            let mut catalog = Declarations::decode(execution.catalog.as_slice())?;
            catalog.types.as_mut().unwrap().file[0].message_type[0].field[0].r#type = Some(Type::Int64 as i32);
            execution.catalog = catalog.encode_to_vec();
        }
        store.create(&execution)?;
        store.open_stream("execution", "readings")?;
        let mut samples = fixture::samples(3);
        for (sample, magnitude) in samples.iter_mut().zip([base + 1, base, base + 2]) {
            let value = if signed { (-(magnitude as i64)) as u64 } else { magnitude };
            sample.row = Row { value, label: "large".into() }.encode_to_vec();
        }
        store.sample_batch("execution", &samples)?;
        store.flush_samples()?;
        let input = FieldReference::new("readings").field("value");
        let expression = DataExpression {
            kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                input: Some(Box::new(input)),
                kind: Aggregate::Median as i32,
                ..Default::default()
            }))),
            ..Default::default()
        };
        let cut = store.cut("execution")?;
        let expected = if signed { value::Kind::Integer(-((base + 1) as i64)) } else { value::Kind::UnsignedInteger(base + 1) };
        assert_eq!(store.document_value(&cut, &expression)?.kind, Some(expected));

        // An Even Selection Keeps Its Fractional Midpoint Exact.
        let held = ExecutionCut { frontier: 2, ..cut };
        let expected = value::Kind::Exact(ExactNumber {
            negative: signed,
            numerator: (u128::from(base) * 2 + 1).to_string(),
            denominator: "2".into(),
        });
        assert_eq!(store.document_value(&held, &expression)?.kind, Some(expected));

        // Literal Expressions Use the Same Native Median.
        let expected = if signed { value::Kind::Integer(-(base as i64)) } else { value::Kind::UnsignedInteger(base) };
        let input = DataExpression {
            kind: Some(data_expression::Kind::Literal(model::Value { kind: Some(expected.clone()) })),
            ..Default::default()
        };
        let expression = DataExpression {
            kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                input: Some(Box::new(input)),
                kind: Aggregate::Median as i32,
                ..Default::default()
            }))),
            ..Default::default()
        };
        assert_eq!(store.document_value(&held, &expression)?.kind, Some(expected));
    }
    Ok(())
}
#[test]
fn histogram_medians_match_native_integer_medians() -> Result<()> {
    let base = (1u64 << 60) + 487;
    for signed in [false, true] {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let mut execution = fixture::execution();
        if signed {
            let mut catalog = Declarations::decode(execution.catalog.as_slice())?;
            catalog.types.as_mut().unwrap().file[0].message_type[0].field[0].r#type = Some(Type::Int64 as i32);
            execution.catalog = catalog.encode_to_vec();
        }
        store.create(&execution)?;
        store.open_stream("execution", "readings")?;
        let mut samples = fixture::samples(3);
        for (sample, magnitude) in samples.iter_mut().zip([base, base + 4097, base + 8194]) {
            let value = if signed { (-(magnitude as i64)) as u64 } else { magnitude };
            sample.row = Row { value, label: "large".into() }.encode_to_vec();
        }
        store.sample_batch("execution", &samples)?;
        store.flush_samples()?;
        let expression = DataExpression {
            kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                input: Some(Box::new(FieldReference::new("readings").field("value"))),
                kind: Aggregate::Median as i32,
                ..Default::default()
            }))),
            ..Default::default()
        };
        for through in [2, 3] {
            let cut = ExecutionCut {
                frontier: through,
                ..store.cut("execution")?
            };
            let expected = store.document_value(&cut, &expression)?;
            let histogram = store.histogram_window(numeric(through), 2)?;
            assert_eq!(histogram.statistics.unwrap().median, Some(expected.clone()));
            assert_eq!(store.numeric_statistics(&numeric(through))?.median, Some(expected));
        }
    }
    Ok(())
}

#[test]
fn large_integer_numeric_statistics_use_library_statistics() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    let mut samples = fixture::samples(3);
    for (sample, value) in samples.iter_mut().zip([u64::MAX, u64::MAX - 2, u64::MAX - 1]) {
        sample.row = Row { value, label: "large".into() }.encode_to_vec();
    }
    store.sample_batch("execution", &samples)?;
    store.flush_samples()?;
    let statistics = store.numeric_statistics(&numeric(3))?;
    assert_eq!(statistics.count, 3);
    assert_eq!(statistics.stddev, 0.0);
    assert_eq!(store.numeric_rms(&numeric(3))?, (u64::MAX - 1) as f64);
    Ok(())
}
#[test]
fn integer_statistics_keep_exact_values_and_float_ranks_across_reopen() -> Result<()> {
    let root = tempfile::tempdir()?;
    let base = 1_152_921_504_606_847_975u64;
    let input = FieldReference::new("readings").field("value");
    {
        let store = Store::open(root.path())?;
        store.create(&fixture::execution())?;
        store.open_stream("execution", "readings")?;
        let mut samples = fixture::samples(101);
        for (index, sample) in samples.iter_mut().enumerate() {
            sample.row = Row {
                value: base + index as u64 * 100,
                label: "large".into(),
            }
            .encode_to_vec();
        }
        store.sample_batch("execution", &samples)?;
        store.flush_samples()?;
    }
    let store = Store::open(root.path())?;
    for kind in [value::Kind::Integer(-(base as i64)), value::Kind::UnsignedInteger(base)] {
        let input = DataExpression {
            kind: Some(data_expression::Kind::Literal(model::Value { kind: Some(kind.clone()) })),
            ..Default::default()
        };
        let expression = DataExpression {
            kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                input: Some(Box::new(input)),
                kind: Aggregate::Percentile as i32,
                percentile: 95.0,
            }))),
            ..Default::default()
        };
        let answer = store.document_value(&store.cut("execution")?, &expression)?;
        let Some(value::Kind::Number(actual)) = answer.kind else {
            panic!("Library Ranks Return Floating-Point Values");
        };
        let expected = match kind {
            value::Kind::Integer(value) => value as f64,
            value::Kind::UnsignedInteger(value) => value as f64,
            _ => unreachable!(),
        };
        assert!((actual - expected).abs() <= expected.abs() * f64::EPSILON);
    }
    for (start_ns, end_ns, count, minimum, mean, maximum) in
        [(0, 100, 101, base, base + 5_000, base + 10_000), (20, 60, 41, base + 2_000, base + 4_000, base + 6_000)]
    {
        let (_, summary, selected) = store.time_series_window(NumericWindow { start_ns, end_ns, ..numeric(101) })?;
        assert_eq!(selected, count);
        for (value, expected) in [(&summary.min, minimum), (&summary.mean, mean), (&summary.max, maximum)] {
            assert_eq!(value.as_ref().unwrap().kind, Some(value::Kind::UnsignedInteger(expected)));
        }
    }
    let statistics = store.numeric_statistics(&numeric(101))?;
    for (value, expected) in [
        (&statistics.min, base),
        (&statistics.max, base + 10_000),
        (&statistics.first, base),
        (&statistics.last, base + 10_000),
        (&statistics.mean, base + 5_000),
    ] {
        assert_eq!(value.as_ref().unwrap().kind, Some(value::Kind::UnsignedInteger(expected)));
    }
    for (value, expected) in [(&statistics.p05, base + 500), (&statistics.p95, base + 9_500), (&statistics.p99, base + 9_900)] {
        let Some(value::Kind::Number(actual)) = value.as_ref().unwrap().kind else {
            panic!("Library Ranks Return Floating-Point Values");
        };
        assert!((actual - expected as f64).abs() <= expected as f64 * f64::EPSILON);
    }
    for (kind, expected) in [
        (Aggregate::Min, statistics.min.as_ref().unwrap()),
        (Aggregate::Max, statistics.max.as_ref().unwrap()),
        (Aggregate::Mean, statistics.mean.as_ref().unwrap()),
        (Aggregate::Sum, statistics.sum.as_ref().unwrap()),
    ] {
        let (frame, count, _) = store.category_window(
            KindWindow {
                window: numeric(101),
                x: Some("label"),
                z: None,
                group: None,
                limit: 8,
            },
            Some(kind),
            true,
        )?;
        assert_eq!(count, 101);
        assert_eq!(&frame.series[0].totals[0], expected);
    }
    let sum = statistics.sum.as_ref().unwrap();
    assert_eq!(
        sum.kind,
        Some(value::Kind::Exact(ExactNumber {
            negative: false,
            numerator: ((base as u128 + 5_000) * 101).to_string(),
            denominator: "1".into(),
        }))
    );
    for (kind, percentile, expected) in [
        (Aggregate::Percentile, 5.0, base + 500),
        (Aggregate::Percentile, 95.0, base + 9_500),
        (Aggregate::Min, 0.0, base),
        (Aggregate::Max, 0.0, base + 10_000),
        (Aggregate::First, 0.0, base),
        (Aggregate::Last, 0.0, base + 10_000),
        (Aggregate::Mean, 0.0, base + 5_000),
    ] {
        let expression = DataExpression {
            kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                input: Some(Box::new(input.clone())),
                kind: kind as i32,
                percentile,
            }))),
            ..Default::default()
        };
        let answer = store.document_value(&store.cut("execution")?, &expression)?;
        if kind == Aggregate::Percentile {
            let Some(value::Kind::Number(actual)) = answer.kind else {
                panic!("Library Ranks Return Floating-Point Values");
            };
            assert!((actual - expected as f64).abs() <= expected as f64 * f64::EPSILON);
        } else {
            assert_eq!(answer.kind, Some(value::Kind::UnsignedInteger(expected)));
        }
    }
    let expression = DataExpression {
        kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
            input: Some(Box::new(input)),
            kind: Aggregate::Sum as i32,
            percentile: 0.0,
        }))),
        ..Default::default()
    };
    assert_eq!(&store.document_value(&store.cut("execution")?, &expression)?, sum);
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

#[test]
fn rejected_rebuilt_root_preserves_its_database() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("execution.sqlite3");
    let reference = tempfile::tempdir()?;
    drop(Store::open(reference.path())?);
    let identity: i64 =
        Connection::open(reference.path().join("execution.sqlite3"))?.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    let old = Connection::open(&path)?;
    old.execute_batch(&format!("PRAGMA application_id={identity}; PRAGMA user_version=1; CREATE TABLE samples(payload BLOB); INSERT INTO samples VALUES(X'010203');"))?;
    drop(old);
    let before = fs::read(&path)?;
    assert!(matches!(Store::open(root.path()), Err(Error::Incompatible(_))));
    assert_eq!(fs::read(&path)?, before);
    Ok(())
}

#[test]
fn record_intervals_anchor_before_aligning_to_later_samples() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let mut execution = fixture::execution();
    let mut catalog = Declarations::decode(execution.catalog.as_slice())?;
    catalog.types.as_mut().unwrap().file[0].message_type.push(DescriptorProto {
        name: Some("Marker".into()),
        ..Default::default()
    });
    let contracts = RecordContracts {
        records: ["start", "finish"]
            .into_iter()
            .map(|id| RecordContract {
                metadata: Some(Metadata {
                    identifier: id.into(),
                    ..Default::default()
                }),
                message: "fixture.Marker".into(),
            })
            .collect(),
    };
    catalog.sections.push(Default::default());
    let section = catalog.sections.last_mut().unwrap();
    section.r#type = "dokime.model.v1.RecordContracts".into();
    section.payload = contracts.encode_to_vec();
    execution.catalog = catalog.encode_to_vec();
    store.create(&execution)?;
    store.open_stream("execution", "readings")?;
    for (record, time) in [("start", 2_000_000_000), ("finish", 4_000_000_000), ("start", 6_000_000_000)] {
        let payload = RecordsAppendRequest {
            payload: Vec::new(),
            ..Default::default()
        };
        store.append(
            "execution",
            Evidence {
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
    let expression = DataExpression {
        kind: Some(data_expression::Kind::Transform(Box::new(TransformExpression {
            input: Some(Box::new(DataExpression {
                kind: Some(data_expression::Kind::Record(FieldReference {
                    identifier: "finish".into(),
                    field: String::new(),
                })),
                ..Default::default()
            })),
            kind: Some(transform_expression::Kind::TimeBetween("start".into())),
        }))),
        ..Default::default()
    };
    let (reader, _cancel) = store.read_session()?;
    let cut = reader.cut("execution")?;
    let selected = reader.plot_selection(
        &cut,
        &PlotRequest {
            stream: "readings".into(),
            y_expressions: vec![expression],
            ..Default::default()
        },
    )?;
    let Some(data_expression::Kind::Stream(field)) = &selected.request.y_expressions[0].kind else { unreachable!() };
    let statistics = reader.numeric_statistics(&NumericWindow {
        field: &field.field,
        ..numeric(cut.frontier)
    })?;
    assert_eq!(statistics.count, 2);
    assert_eq!(statistics.mean.unwrap().kind, Some(value::Kind::Number(2.0)));
    Ok(())
}

#[test]
fn typed_map_and_list_columns_survive_finalization_and_restart() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let mut definition = fixture::execution();
    let mut catalog = Declarations::decode(definition.catalog.as_slice())?;
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
            counts: BTreeMap::from([("large".into(), u64::MAX)]),
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
    let actual = rows.iter().map(|sample| ContainerRow::decode(sample.row.as_slice())).collect::<result::Result<Vec<_>, _>>()?;
    assert_eq!(actual, values);
    Ok(())
}

/// The Host's Stored `harmos.batch` Encoding of `fixture.Row` Columns, Field by Field.
fn interval(start_ns: i64, period_ns: u64, values: &[u64], labels: &[&str]) -> Vec<u8> {
    let mut payload = Vec::new();
    start_ns.write(&mut payload);
    period_ns.write(&mut payload);
    (values.len() as u64).write(&mut payload);
    1_u8.write(&mut payload);
    values.len().write(&mut payload);
    for value in values {
        value.write(&mut payload);
    }
    labels.len().write(&mut payload);
    for label in labels {
        label.to_string().write(&mut payload);
    }
    payload
}

#[test]
fn interval_batches_store_one_row_per_sample_on_their_grid() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.open_stream("execution", "readings")?;
    let window = |through| SampleWindow {
        execution: "execution",
        streams: &[],
        after: &[],
        through,
        start_ns: i64::MIN,
        end_ns: i64::MAX,
        latest: false,
        limit: NonZeroUsize::new(64).unwrap(),
    };
    let observed = |samples: Vec<Sample>| {
        samples
            .into_iter()
            .map(|sample| {
                let row = Row::decode(sample.row.as_slice()).unwrap();
                (sample.position, sample.sequence, sample.source_ns, row.value, row.label)
            })
            .collect::<Vec<_>>()
    };

    // One Batch Row Becomes Three Accepted Samples at Its Start Plus Whole Periods.
    let first = interval(1_000, 10, &[7, 8, u64::MAX], &["a", "b", "c"]);
    assert_eq!(store.sample_interval("execution", "readings", &first, 1)?, 3);
    let expected = vec![(1, 1, 1_000, 7, "a".to_owned()), (2, 2, 1_010, 8, "b".to_owned()), (3, 3, 1_020, u64::MAX, "c".to_owned())];
    assert_eq!(observed(store.live_sample_window(window(0))?), expected);
    assert_eq!(store.cut("execution")?.frontier, 0);

    // A Malformed Batch Admits Nothing; the Next Batch Continues Positions and Sequence.
    let short = interval(1_030, 10, &[9, 10], &["d"]);
    assert!(store.sample_interval("execution", "readings", &short, 4).is_err());
    assert_eq!(store.accepted_frontier("execution")?, 3);
    let second = interval(1_030, 10, &[9, 10], &["d", "e"]);
    assert_eq!(store.sample_interval("execution", "readings", &second, 4)?, 2);

    // Flushed Rows Read Back Unchanged Across Reopening.
    store.flush_samples()?;
    let saved = store.cut("execution")?.frontier;
    assert_eq!(saved, 5);
    drop(store);
    let reopened = Store::open(root.path())?;
    let mut expected = expected;
    expected.push((4, 4, 1_030, 9, "d".to_owned()));
    expected.push((5, 5, 1_040, 10, "e".to_owned()));
    assert_eq!(observed(reopened.sample_window(window(saved))?), expected);
    Ok(())
}

#[test]
fn interval_batches_of_many_streams_survive_pressure_flushes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let mut execution = fixture::execution();
    let mut catalog = Declarations::decode(execution.catalog.as_slice())?;
    let row = catalog.streams[0].row.clone();
    catalog.streams = (0..128)
        .map(|index| StreamDeclaration {
            id: format!("level.stream_{index:03}"),
            row: row.clone(),
            ..Default::default()
        })
        .collect();
    execution.catalog = catalog.encode_to_vec();
    store.create(&execution)?;
    let labels = vec![""; 20_000];
    let values = (0..20_000).collect::<Vec<u64>>();
    for index in 0..128 {
        let stream = format!("level.stream_{index:03}");
        store.open_stream("execution", &stream)?;
        let payload = interval(index * 1_000, 1_000, &values, &labels);
        assert_eq!(store.sample_interval("execution", &stream, &payload, 1)?, 20_000);
    }
    store.flush_samples()?;
    assert_eq!(store.read("execution")?.samples.len(), 128 * 20_000);
    Ok(())
}

#[test]
fn cycle_finality_seals_unstarted_without_inventing_a_verdict() -> Result<()> {
    // Admit Evidence Before Its Run.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    let mut cycle = CycleSummary {
        cycle_id: "cycle".into(),
        ..Default::default()
    };
    store.save_cycle(&cycle)?;
    store.require_evidence_owner(None, Some("execution"), "Attach File")?;
    let note = NotesCreateResponse {
        note: Some(OperatorNote {
            note_id: "before".into(),
            scope: Some(EvidenceScope {
                cycle_id: Some("cycle".into()),
                execution_id: Some("execution".into()),
                ..Default::default()
            }),
            message: "captured".into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    store.create_note(note.clone())?;

    // Seal the Never-Started Execution and Preserve Its Original Note Replay.
    store.seal_unstarted("execution", Default::default())?;
    let sealed = store.read("execution")?;
    assert_eq!(sealed.state, ExecutionState::Sealed);
    assert_eq!(sealed.execution.termination, Termination::NotRun as i32);
    let counts = OutcomeCounts::from_outcomes(sealed.execution.outcome.iter());
    assert_eq!(counts.evaluated, 0);
    assert_eq!(counts.verdict, None);
    assert_eq!(store.create_note(note.clone())?, note);
    assert!(store.require_evidence_owner(None, Some("execution"), "Attach File").is_err());
    let mut after = OperatorNote {
        note_id: "after".into(),
        scope: Some(EvidenceScope {
            cycle_id: Some("cycle".into()),
            execution_id: Some("execution".into()),
            ..Default::default()
        }),
        message: "later".into(),
        ..Default::default()
    };
    assert!(
        store
            .create_note(NotesCreateResponse {
                note: Some(after.clone()),
                ..Default::default()
            })
            .is_err()
    );

    // The Cycle Remains Writable After Every Execution Seals.
    after.scope.as_mut().unwrap().execution_id = None;
    store.create_note(NotesCreateResponse {
        note: Some(after.clone()),
        ..Default::default()
    })?;
    store.require_evidence_owner(Some("cycle"), None, "Attach File")?;
    cycle.phase = CyclePhase::Sealing as i32;
    store.save_cycle(&cycle)?;
    assert!(store.require_evidence_owner(Some("cycle"), None, "Attach File").is_err());
    cycle.phase = CyclePhase::SealFailed as i32;
    store.save_cycle(&cycle)?;
    store.require_evidence_owner(Some("cycle"), None, "Attach File")?;
    cycle.sealed = true;
    cycle.phase = CyclePhase::Sealed as i32;
    store.save_cycle(&cycle)?;
    after.note_id = "sealed-cycle".into();
    assert!(store.create_note(NotesCreateResponse { note: Some(after), ..Default::default() }).is_err());
    assert!(store.require_evidence_owner(Some("cycle"), None, "Attach File").is_err());
    Ok(())
}

#[test]
fn cycle_archive_contains_report_notes_and_execution_folders() -> Result<()> {
    // Retain Both Owners and the Canonical Report Bytes.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.save_cycle(&CycleSummary {
        cycle_id: "cycle".into(),
        updated_at: Some(Default::default()),
        ..Default::default()
    })?;
    store.seal_unstarted("execution", Default::default())?;
    let report = b"<html>frozen</html>";
    let note = OperatorNote {
        note_id: "cycle-note".into(),
        message: "evidence".into(),
        ..Default::default()
    };
    let mut exports = Vec::new();
    for _ in 0..2 {
        let mut file = NamedTempFile::new()?;
        file.write_all(report)?;
        let content = ArchivedContent {
            notes: vec![note.clone()],
            report: Some(ArchivedNativeFile {
                path: file.into_temp_path(),
                reference: FileReference {
                    length: report.len() as u64,
                    sha256: Sha256::digest(report).to_vec(),
                    ..Default::default()
                },
            }),
            ..Default::default()
        };
        let mut bytes = Vec::new();
        store.write_cycle_archive("cycle", NonZeroUsize::new(1).unwrap(), &mut bytes, &mut Default::default(), content)?;
        exports.push(bytes);
    }
    assert_eq!(Sha256::digest(&exports[0]), Sha256::digest(&exports[1]));
    let mut archive = ZipArchive::new(Cursor::new(&exports[0]))?;
    let mut retained = Vec::new();
    archive.by_name("report.html")?.read_to_end(&mut retained)?;
    assert_eq!(retained, report);
    let metadata: serde_json::Value = serde_json::from_reader(archive.by_name("data/cycle.json")?)?;
    assert_eq!(metadata["notes"][0]["note_id"], note.note_id);
    assert_eq!(metadata["cycle"]["sealed"], true);
    let execution: serde_json::Value = serde_json::from_reader(archive.by_name("executions/01 execution/data/execution.json")?)?;
    assert_eq!(execution[0]["termination"], Termination::NotRun as i32);
    Ok(())
}

#[test]
fn sealing_as_not_run_refuses_started_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    store.append(
        "execution",
        Evidence {
            kind: "started".into(),
            ..Default::default()
        },
    )?;
    assert!(store.seal_unstarted("execution", Default::default()).is_err());
    assert_eq!(store.cut("execution")?.state, ExecutionState::Open);
    Ok(())
}
