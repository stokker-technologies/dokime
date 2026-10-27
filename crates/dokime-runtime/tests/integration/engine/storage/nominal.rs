/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Round Trip Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    f64::consts::TAU,
    num::NonZeroUsize,
    time::{Duration, UNIX_EPOCH},
};

// Third Party Imports
use harmos::storage::operations::Query;
use harmos::storage::query::{
    ObjectSelection, ObjectVersions, QueryArrow, RecordSelection, RowSlice, Source, StreamSelection,
    scalar::ScalarValue,
    transform::{
        FieldPath, FieldPathExecution, Transform,
        aggregate::{Aggregate, Measure},
        sort::{Sort, SortKey},
    },
};
use harmos::{Application, DefinitionId, Identified, Message, Runtime, state::Never, storage::*};
use sha2::{Digest, Sha256};

// Local Imports
use super::sqlite::storage::{Store, declared, id, page, rows};
use dokime_runtime::storage::sqlite::Backend;
use {
    dokime_domain::model::telemetry::evidence::TelemetrySample, dokime_runtime::SessionId, dokime_runtime::Timestamp,
    dokime_runtime::Value, dokime_runtime::storage::telemetry::TelemetryEntry, dokime_runtime::storage::telemetry::TypedCodec,
};

/* === Constants === */
const MAX_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct App;
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Waveform {
    a: f64,
    b: f64,
    c: f64,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
struct Details {
    label: String,
    samples: Vec<f64>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct TypedWave {
    rms: f64,
    details: Details,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
struct Subject {
    id: String,
    stopped: bool,
}

/* === Enums === */
#[harmos::catalog]
enum Objects {
    Subject(Subject),
}

/* === Traits === */

/* === Implementations === */
impl Application for App {
    type State = ();
    type Transaction = Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

impl Identified for Subject {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("story.subject"), 1)
    }
}

impl StoredObject for Subject {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::UntilReleased;

    fn key(&self) -> &str {
        &self.id
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn retention(schema: &SchemaKey) -> harmos::domain::Result<ObjectRetention> {
    let definition = harmos::domain::model::storage::catalog::definition::<Objects>(schema)?;
    Ok(definition.retention())
}

/* === Tests === */
#[tokio::test]
async fn nominal() {
    // Configure And Start The Production SQLite Adapter
    let root = tempfile::tempdir().unwrap();
    let store = Store::<Objects>::open(root.path()).await;
    let backend = &store.backend;
    let history = backend.history().clone();
    let at = |sequence| StorageCursor { history: history.clone(), sequence };
    let waveform_schema =
        SchemaKey { id: id("story.waveform"), version: std::num::NonZeroU32::MIN, payload_encoding: PayloadEncoding::PostcardV1 };
    let subject_schema =
        Objects::encode(&Subject { id: "execution".into(), stopped: false }, NonZeroUsize::new(MAX_BYTES).unwrap()).unwrap().schema;
    let typed_stream = id("typed-voltage");
    let codec = TypedCodec::new(&typed_stream, 1, &declared::<TypedWave>(), &harmos::config::SchemaSettings::default()).unwrap();
    let schemas = [
        StorageSchema::new(at(1), waveform_schema.clone(), declared::<Waveform>()),
        StorageSchema::new(at(2), subject_schema.clone(), declared::<Subject>()),
        StorageSchema::new(at(3), codec.schema().clone(), codec.graph().clone()),
    ];
    backend.write_schemas(store.batch(schemas.to_vec())).await.unwrap();

    // Generate Three Balanced Phases Over An Integer Number Of Periods
    let samples = 256usize;
    let amplitude = 325.0;
    let mut waves = Vec::with_capacity(samples);
    for index in 0..samples {
        let phase = TAU * index as f64 / 64.0;
        let waveform =
            Waveform { a: amplitude * phase.sin(), b: amplitude * (phase - TAU / 3.0).sin(), c: amplitude * (phase + TAU / 3.0).sin() };
        waves.push(StoredRow {
            record: StoredRecord {
                position: at(index as u64 + 4),
                schema: waveform_schema.clone(),
                scope: Some(id("execution")),
                item: id(&format!("sample-{index}")),
                source: Some(id("three-phase-meter")),
                source_time: Some(UNIX_EPOCH + Duration::from_millis(index as u64)),
                payload: waveform.encode_to_vec(),
            },
            stream: id("voltage"),
            sequence: index as u64,
        });
    }
    backend.write_stream(store.batch(waves.clone())).await.unwrap();
    let record = StoredRecord {
        position: at(samples as u64 + 4),
        schema: waveform_schema.clone(),
        scope: Some(id("execution")),
        item: id("calibration"),
        source: None,
        source_time: None,
        payload: Waveform { a: amplitude, b: amplitude, c: amplitude }.encode_to_vec(),
    };
    backend.write_records(store.batch(vec![record.clone()])).await.unwrap();
    let object_position = samples as u64 + 5;
    let subject = ObjectEntry {
        position: at(object_position),
        version: serde_json::from_value(serde_json::json!({"history": history, "revision": object_position})).unwrap(),
        value: Objects::encode(&Subject { id: "execution".into(), stopped: false }, NonZeroUsize::new(MAX_BYTES).unwrap()).unwrap(),
    };
    backend.write_objects(store.batch(vec![subject.clone()])).await.unwrap();

    let stopped = ObjectEntry {
        position: at(object_position + 1),
        version: serde_json::from_value(serde_json::json!({"history": history, "revision": object_position + 1})).unwrap(),
        value: Objects::encode(&Subject { id: "execution".into(), stopped: true }, NonZeroUsize::new(MAX_BYTES).unwrap()).unwrap(),
    };
    backend.write_objects(store.batch(vec![stopped.clone()])).await.unwrap();

    // Persist Original Typed Rows With Nested Schema And Complete Sample Metadata
    let typed = TypedWave { rms: amplitude / 2.0f64.sqrt(), details: Details { label: "meter-A".into(), samples: vec![1.0, 2.0, 3.0] } };
    let mut sample = TelemetrySample::new(
        dokime_runtime::model::identity::new_id(),
        typed_stream.clone(),
        0,
        Timestamp::new("2026-10-15T10:00:00Z"),
        "typed-meter",
    )
    .with_storage_payload(typed.encode_to_vec());
    sample.summary.insert("health".into(), Value::Text("ok".into()));
    sample.metadata.insert("operator".into(), Value::Text("lab".into()));
    let typed_entry = TelemetryEntry { epoch: SessionId::from_seed("nominal"), ordinal: 0, sample };
    let typed_row = StoredRow {
        record: StoredRecord {
            position: at(object_position + 2),
            schema: codec.schema().clone(),
            scope: None,
            item: id(typed_entry.sample.sample_id.as_str()),
            source: Some(id(typed_entry.epoch.as_str())),
            source_time: Some(
                UNIX_EPOCH
                    + Duration::from_nanos(
                        u64::try_from(typed_entry.sample.timestamp.to_datetime().unwrap().unix_timestamp_nanos()).unwrap(),
                    ),
            ),
            payload: codec.encode(&typed_entry, MAX_BYTES).unwrap(),
        },
        stream: typed_stream.clone(),
        sequence: 0,
    };
    backend.write_stream(store.batch(vec![typed_row.clone()])).await.unwrap();

    // Publish Deterministic Empty, Small And Multi-Batch Finished Files
    let mut files = Vec::new();
    for (index, size) in [0usize, 17, 150_000].into_iter().enumerate() {
        let bytes = (0..size).map(|offset| (offset % 251) as u8).collect::<Vec<_>>();
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        let metadata = FileMetadata {
            id: id(&format!("artifact-{index}")),
            scope: Some(id("execution")),
            media_type: "application/octet-stream".into(),
            expected_length: Some(size as u64),
            expected_sha256: Some(digest),
        };
        let stage = backend.create_file(&metadata).await.unwrap();
        for (batch, bytes) in bytes.chunks(128 * 1024).enumerate() {
            backend.write_file(&stage, (batch * 128 * 1024) as u64, store.batch(bytes.to_vec())).await.unwrap();
        }
        let reference = backend.finish_file(&stage, &at(object_position + index as u64 + 3), size as u64, digest).await.unwrap();
        backend.write_files(store.batch(vec![reference.clone()])).await.unwrap();
        files.push((reference, bytes));
    }
    let durable = backend.cursor_saved().await.unwrap();
    assert!(backend.prune_objects(&durable, NonZeroUsize::new(64).unwrap(), retention).await.unwrap());
    backend.close().await.unwrap();

    // Reopen And Read All Evidence At The Durable Boundary
    let backend = std::sync::Arc::new(Backend::<Objects>::primary(root.path().to_owned()).await.unwrap());
    let storage = StorageManager::<Objects>::new(
        StorageBackends {
            primary: backend.clone(),
            state: std::sync::Arc::new(harmos::storage::state::file::FileBackend::new(
                root.path().to_owned(),
                backend.history().clone(),
                dokime_runtime::config::storage_state_defaults(),
            )),
        },
        super::sqlite::storage::storage_settings(),
    )
    .unwrap();
    let runtime = Runtime::<App>::builder((), ()).runtime_with_storage(&storage).await.unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap(), durable);
    let view = backend.open_snapshot(&durable).await.unwrap();
    let selection = StreamSelection {
        schema: waveform_schema.clone(),
        scope: None,
        item: None,
        stream: Some(id("voltage")),
        source_start: None,
        source_end: None,
        after: 0,
    };
    let mut after = 0;
    let mut actual = Vec::new();
    loop {
        let page = backend.read_stream(&view, &selection, page(after, rows(31))).await.unwrap();
        let Some(last) = page.last() else {
            break;
        };
        after = last.record.position.sequence;
        actual.extend(page);
    }
    assert_eq!(actual, waves);
    let mut sums = [0.0f64; 3];
    let mut squares = [0.0f64; 3];
    for row in &actual {
        let wave = Waveform::decode(&row.record.payload).unwrap();
        for (phase, value) in [wave.a, wave.b, wave.c].into_iter().enumerate() {
            sums[phase] += value;
            squares[phase] += value * value;
        }
    }
    for phase in 0..3 {
        assert!((sums[phase] / samples as f64).abs() < 1e-10);
        let rms = (squares[phase] / samples as f64).sqrt();
        assert!((rms - amplitude / 2.0f64.sqrt()).abs() < 1e-10);
    }
    let typed_selection = StreamSelection {
        schema: codec.schema().clone(),
        scope: None,
        item: None,
        stream: None,
        source_start: None,
        source_end: None,
        after: 0,
    };
    let typed_rows = backend.read_stream(&view, &typed_selection, page(0, rows(8))).await.unwrap();
    assert_eq!(typed_rows, [typed_row]);
    let decoded = codec.decode(&typed_rows[0].record.payload, MAX_BYTES).unwrap();
    assert_eq!(decoded.epoch, typed_entry.epoch);
    assert_eq!(decoded.sample.sample_id, typed_entry.sample.sample_id);
    assert_eq!(decoded.sample.summary, typed_entry.sample.summary);
    assert_eq!(decoded.sample.metadata, typed_entry.sample.metadata);
    assert_eq!(decoded.sample.values.get("timestamp"), typed_entry.sample.values.get("timestamp"));
    assert_eq!(decoded.sample.storage_payload, typed_entry.sample.storage_payload);
    assert_eq!(decoded.sample.values.get("rms"), Some(&Value::Number(typed.rms)));
    assert!(matches!(decoded.sample.values.get("details"), Some(Value::Object(details))
            if details.get("label") == Some(&Value::Text("meter-A".into()))));

    let mut typed_query = storage
        .query(Query {
            at: durable.clone(),
            source: Source::Stream(StreamSelection {
                schema: codec.schema().clone(),
                scope: None,
                item: None,
                stream: Some(typed_stream.clone()),
                source_start: None,
                source_end: None,
                after: 0,
            }),
            transforms: vec![],
            slice: RowSlice::default(),
        })
        .await
        .unwrap();
    let mut typed_count = 0;
    while let Some(output) = typed_query.next().await.unwrap() {
        if let QueryArrow::Batch(batch) = output {
            let entries = dokime_runtime::storage::telemetry::decode_positioned(&codec, batch).unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].1.sample.storage_payload, typed_entry.sample.storage_payload);
            typed_count += entries.len();
        }
    }
    assert_eq!(typed_count, 1);
    drop(typed_query);

    // Query One Complete Period And Compare Backend RMS To Its Analytical Value
    let mut reader = storage
        .query(Query {
            at: durable.clone(),
            source: Source::Stream(StreamSelection {
                schema: waveform_schema.clone(),
                scope: Some(id("execution")),
                item: None,
                stream: Some(id("voltage")),
                source_start: Some(UNIX_EPOCH + Duration::from_millis(64)),
                source_end: Some(UNIX_EPOCH + Duration::from_millis(128)),
                after: 0,
            }),
            transforms: vec![Transform::Aggregate(Aggregate {
                groups: vec![],
                measures: ["a", "b", "c"]
                    .into_iter()
                    .map(|phase| Measure {
                        field: format!("payload.{phase}").as_str().into(),
                        statistic: harmos::Aggregate::Rms,
                        name: phase.into(),
                    })
                    .collect(),
            })],
            slice: RowSlice::default(),
        })
        .await
        .unwrap();
    let mut schema_seen = false;
    let mut aggregates = 0;
    while let Some(output) = reader.next().await.unwrap() {
        match output {
            QueryArrow::Schema(schema) => {
                assert!(!schema_seen);
                assert_eq!(schema.as_schema().fields().len(), 3);
                schema_seen = true;
            }
            QueryArrow::Batch(batch) => {
                assert!(schema_seen);
                assert_eq!(batch.as_record_batch().num_rows(), 1);
                for phase in ["a", "b", "c"] {
                    let rms = FieldPath::from(phase).value(batch.as_record_batch(), 0).unwrap();
                    match rms {
                        ScalarValue::Float(rms) => {
                            assert!((rms - amplitude / 2.0f64.sqrt()).abs() < 1e-10)
                        }
                        _ => panic!("unexpected RMS type"),
                    }
                }
                aggregates += 1;
            }
        }
    }
    assert_eq!(aggregates, 1);
    assert_eq!(reader.completion().unwrap().rows, 1);
    drop(reader);

    let mut expected = waves.iter().map(|row| Waveform::decode(&row.record.payload).unwrap().a).collect::<Vec<_>>();
    expected.sort_by(|left, right| right.total_cmp(left));
    let expected = expected[5..22].to_vec();
    let mut sorted = storage
        .query(Query {
            at: durable.clone(),
            source: Source::Stream(StreamSelection {
                schema: waveform_schema.clone(),
                scope: Some(id("execution")),
                item: None,
                stream: Some(id("voltage")),
                source_start: None,
                source_end: None,
                after: 0,
            }),
            transforms: vec![Transform::Sort(Sort {
                keys: vec![SortKey { field: "payload.a".into(), descending: true, nulls_first: false }],
            })],
            slice: RowSlice { offset: 5, limit: Some(17) },
        })
        .await
        .unwrap();
    let mut values = Vec::new();
    while let Some(output) = sorted.next().await.unwrap() {
        if let QueryArrow::Batch(batch) = output {
            for row in 0..batch.as_record_batch().num_rows() {
                match FieldPath::from("payload.a").value(batch.as_record_batch(), row).unwrap() {
                    ScalarValue::Float(value) => values.push(value),
                    _ => panic!("unexpected sorted waveform type"),
                }
            }
        }
    }
    assert_eq!(values, expected);
    assert_eq!(sorted.completion().unwrap().rows, 17);
    drop(sorted);

    let records = backend
        .read_records(
            &view,
            &RecordSelection { schema: waveform_schema.clone(), scope: None, item: None, source_start: None, source_end: None },
            page(0, rows(8)),
        )
        .await
        .unwrap();
    assert_eq!(records, [record]);
    let revisions = ObjectSelection { schema: None, key: None, versions: ObjectVersions::Revisions };
    let objects = backend.read_objects(&view, &revisions, page(0, rows(8))).await.unwrap();
    assert_eq!(objects, [subject.clone(), stopped.clone()]);
    for (file, expected) in files {
        let mut actual = Vec::new();
        while actual.len() < expected.len() {
            let bytes = backend.read_file(&view, &file, actual.len() as u64, NonZeroUsize::new(MAX_BYTES).unwrap()).await.unwrap();
            assert!(!bytes.is_empty());
            actual.extend(bytes);
        }
        assert_eq!(actual, expected);
        assert!(backend.read_file(&view, &file, file.length, NonZeroUsize::new(MAX_BYTES).unwrap()).await.unwrap().is_empty());
        assert!(backend.read_file(&view, &file, file.length + 1, NonZeroUsize::new(MAX_BYTES).unwrap()).await.is_err());
    }
    // Release Only After The Subject Is Sealed; Existing Reads Keep Their History
    backend.release_object_history(&subject_schema, &subject.value.item, &durable).await.unwrap();
    while !backend.prune_objects(&durable, NonZeroUsize::new(64).unwrap(), retention).await.unwrap() {}
    let pinned = backend.read_objects(&view, &revisions, page(0, rows(8))).await.unwrap();
    assert_eq!(pinned, [subject, stopped.clone()], "a pinned snapshot keeps history");
    backend.release_snapshot(&view).await.unwrap();
    runtime.stop().await.unwrap();

    // The Lifecycle Release Survives Restart And Reclaims Unpinned Revisions
    let backend = std::sync::Arc::new(Backend::<Objects>::primary(root.path().to_owned()).await.unwrap());
    while !backend.prune_objects(&durable, NonZeroUsize::new(64).unwrap(), retention).await.unwrap() {}
    let current = backend.open_snapshot(&durable).await.unwrap();
    let objects = backend.read_objects(&current, &revisions, page(0, rows(8))).await.unwrap();
    assert_eq!(objects, [stopped]);
    backend.release_snapshot(&current).await.unwrap();
    let releases = backend
        .local_state()
        .call(|connection| connection.query_row("SELECT COUNT(*) FROM storage_object_releases", [], |row| row.get::<_, i64>(0)))
        .await
        .unwrap();
    assert_eq!(releases, 1, "the lifecycle release survives restart");
    backend.close().await.unwrap();
}
