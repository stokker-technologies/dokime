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
use std::{num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use harmos::{
    Aggregate, Application, DefinitionId, Identified, Runtime,
    schema::Described,
    state::Never,
    storage::{
        frame::{Expr, GroupBy, ScalarField, ScalarPath},
        reduce::{Agg, ScalarKind, ScalarValue},
        *,
    },
};

// Local Imports
use dokime::storage::sqlite::{Backend, Limits, StateLimits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Bench;

#[harmos::message]
#[derive(Clone, PartialEq)]
struct Sample {
    id: String,
    value: u64,
    optional: Option<u64>,
}

/* === Enums === */
#[harmos::catalog]
enum Objects {
    Sample(Sample),
}

/* === Traits === */

/* === Implementations === */
impl Application for Bench {
    type State = ();
    type Transaction = Never;
    type Record = Never;
    type Streams = ();
    type Resources = ();
}

impl Identified for Sample {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("fixture.scalar.sample"), 1)
    }
}

impl StoredObject for Sample {
    type Key = String;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;
    fn key(&self) -> &String {
        &self.id
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn scalar_failure_cursor() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(
            Backend::<Objects>::primary(root.path().to_path_buf(), Limits::default())
                .await
                .unwrap(),
        );
        let control = backend.control().await.unwrap();
        control
            .call(|connection| {
                connection.execute_batch(
                    "CREATE TRIGGER reject_scalar BEFORE INSERT ON storage_object_scalars
             BEGIN SELECT RAISE(ABORT, 'injected scalar index failure'); END;",
                )
            })
            .await
            .unwrap();
        let recovery = Arc::new(backend.state(StateLimits::default()));
        let storage = StorageManager::new(
            StorageBackends {
                primary: backend.clone(),
                state: recovery,
                replicas: vec![],
            },
            StorageSettings {
                flush_interval: Duration::from_secs(60),
                ..crate::sqlite::storage::storage_settings()
            },
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let sample = Sample {
            id: "first".into(),
            value: u64::MAX,
            optional: None,
        };
        let schema = Objects::encode(&sample, NonZeroUsize::new(4096).unwrap())
            .unwrap()
            .schema;
        let schema_position = storage
            .write_schema(&schema, &super::storage::declared::<Sample>())
            .await
            .unwrap();
        let prior = storage.flush_until(&schema_position).await.unwrap();
        let admitted = storage.write_object(&sample).await.unwrap();
        assert!(storage.flush_until(&admitted.position).await.is_err());
        assert_eq!(backend.cursor_saved().await.unwrap(), prior);
        control
            .call(|connection| {
                let objects: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM storage_data_entries WHERE kind='object'",
                    [],
                    |row| row.get(0),
                )?;
                let scalars: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM storage_object_scalars",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!((objects, scalars), (0, 0));
                connection.execute_batch("DROP TRIGGER reject_scalar")
            })
            .await
            .unwrap();
        let saved = storage.flush_until(&admitted.position).await.unwrap();
        assert_eq!(saved, admitted.position);
        let value_tag = Sample::FIELDS
            .iter()
            .find(|field| field.name == "value")
            .unwrap()
            .tag;
        let optional_tag = Sample::FIELDS
            .iter()
            .find(|field| field.name == "optional")
            .unwrap()
            .tag;
        control
            .call(move |connection| {
                let value: Vec<u8> = connection.query_row(
                    "SELECT value FROM storage_object_scalars WHERE tag=?1",
                    [value_tag],
                    |row| row.get(0),
                )?;
                let optional: Option<Vec<u8>> = connection.query_row(
                    "SELECT value FROM storage_object_scalars WHERE tag=?1",
                    [optional_tag],
                    |row| row.get(0),
                )?;
                assert_eq!(value, u64::MAX.to_be_bytes());
                assert_eq!(optional, None);
                let fields: Vec<u8> = connection.query_row(
                    "SELECT object_fields FROM storage_data_entries WHERE kind='object'",
                    [],
                    |row| row.get(0),
                )?;
                let restored: Vec<harmos::schema::graph::Field> =
                    serde_json::from_slice(&fields).unwrap();
                let declared = super::storage::declared::<Sample>();
                assert_eq!(restored, declared.definitions[declared.root]);
                let count: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM storage_object_scalars",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(count, 3);
                Ok(())
            })
            .await
            .unwrap();
        let snapshot = backend.read_view(&saved).await.unwrap();
        let view = ReadView::new(
            harmos::Identifier::new("view.scalar").unwrap(),
            saved.clone(),
            saved,
            snapshot.clone(),
        )
        .unwrap();
        let page = backend
            .read_objects(
                &view,
                &ReadSelection::new(ReadKind::Objects),
                None,
                ReadLimits {
                    rows: NonZeroUsize::new(64).unwrap(),
                    bytes: NonZeroUsize::new(4096).unwrap(),
                },
            )
            .await
            .unwrap();
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.entries[0].version, admitted.version);
        let portable = serde_json::to_vec(&page.entries[0]).unwrap().len();
        let exact = ReadLimits {
            rows: NonZeroUsize::new(64).unwrap(),
            bytes: NonZeroUsize::new(portable).unwrap(),
        };
        let selection = ReadSelection::new(ReadKind::Objects);
        let bounded = backend
            .read_objects(&view, &selection, None, exact)
            .await
            .unwrap();
        assert_eq!(bounded.entries, page.entries);
        bounded
            .check(exact, |entry| serde_json::to_vec(entry).unwrap().len())
            .unwrap();
        assert!(
            backend
                .read_objects(
                    &view,
                    &selection,
                    None,
                    ReadLimits {
                        bytes: NonZeroUsize::new(portable - 1).unwrap(),
                        ..exact
                    },
                )
                .await
                .is_err()
        );
        assert!(
            Objects::decode::<Sample>(&page.entries[0].value, NonZeroUsize::new(4096).unwrap())
                .unwrap()
                == sample
        );
        backend.release_view(&snapshot).await.unwrap();
        runtime.stop().await.unwrap();
    })
    .await
    .expect("scalar rollback/retry did not settle");
}

pub(crate) async fn query_page_drop() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(
            Backend::<Objects>::primary(root.path().to_path_buf(), Limits::default())
                .await
                .unwrap(),
        );
        let storage = StorageManager::new(
            StorageBackends {
                primary: backend.clone(),
                state: Arc::new(backend.state(StateLimits::default())),
                replicas: vec![],
            },
            StorageSettings {
                query_decoded_bytes: NonZeroUsize::new(4096).unwrap(),
                flush_interval: Duration::from_secs(60),
                ..crate::sqlite::storage::storage_settings()
            },
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let sample = Sample {
            id: "leased".into(),
            value: u64::MAX,
            optional: None,
        };
        let schema = Objects::encode(&sample, NonZeroUsize::new(4096).unwrap())
            .unwrap()
            .schema;
        storage
            .write_schema(&schema, &super::storage::declared::<Sample>())
            .await
            .unwrap();
        let admitted = storage.write_object(&sample).await.unwrap();
        storage.flush_until(&admitted.position).await.unwrap();
        let mut reader = storage.query(ObjectQuery::<Sample>::new()).await.unwrap();
        let page = reader.next_page().await.unwrap().unwrap();
        assert_eq!(page.len(), 1);
        assert!(page[0].value == sample);
        assert_eq!(page[0].version, admitted.version);
        assert!(reader.next_page().await.unwrap().is_none());
        drop(reader);
        let projection = || {
            ObjectQuery::<Sample>::new()
                .project(vec![
                    harmos::Identifier::new("value").unwrap(),
                    harmos::Identifier::new("optional").unwrap(),
                ])
                .unwrap()
        };
        let mut oversized_query = projection();
        oversized_query.options.decoded_bytes = Some(NonZeroUsize::MIN);
        let mut oversized = storage.query(oversized_query).await.unwrap();
        for _ in 0..2 {
            let refused = oversized.next_page().await;
            assert!(
                matches!(&refused, Err(harmos::Error::ResourceExhausted { .. })),
                "{refused:?}"
            );
        }
        drop(oversized);
        let mut projected = projection();
        projected.options.page = Some(ReadLimits {
            rows: NonZeroUsize::MIN,
            bytes: NonZeroUsize::new(4096).unwrap(),
        });
        let mut metadata = storage.query(projected).await.unwrap();
        let selected = metadata.next_page().await.unwrap().unwrap();
        assert_eq!(
            selected[0].values,
            [ObjectScalar::Unsigned(u64::MAX), ObjectScalar::Null]
        );
        assert_eq!(selected[0].version, admitted.version);
        assert!(metadata.next_page().await.unwrap().is_none());
        drop(metadata);
        runtime.stop().await.unwrap();
        let retained = storage.status().read_bytes;
        assert!(
            retained > 0,
            "the returned page retains its original shared read charge"
        );
        assert_eq!(page[0].value.value, u64::MAX);
        drop(page);
        assert!(
            storage.status().read_bytes > 0,
            "the independent scalar page remains charged"
        );
        assert_eq!(selected[0].values[0], ObjectScalar::Unsigned(u64::MAX));
        drop(selected);
        assert_eq!(storage.status().read_bytes, 0);
    })
    .await
    .expect("native query page ownership did not settle");
}

pub(crate) async fn stream_frames_charge() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(
            Backend::<Objects>::primary(root.path().to_path_buf(), Limits::default())
                .await
                .unwrap(),
        );
        let storage = StorageManager::new(
            StorageBackends {
                primary: backend.clone(),
                state: Arc::new(backend.state(StateLimits::default())),
                replicas: vec![],
            },
            crate::sqlite::storage::storage_settings(),
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let schema = Objects::encode(
            &Sample {
                id: "fixture".into(),
                value: 0,
                optional: None,
            },
            NonZeroUsize::new(4096).unwrap(),
        )
        .unwrap()
        .schema;
        storage
            .write_schema(&schema, &super::storage::declared::<Sample>())
            .await
            .unwrap();
        let stream = harmos::Identifier::new("fixture.samples").unwrap();
        for (index, value) in [u64::MAX - 2, u64::MAX, 1, 3].into_iter().enumerate() {
            let item = harmos::Identifier::new(format!("sample.{index}")).unwrap();
            let encoded = Objects::encode(
                &Sample {
                    id: item.to_string(),
                    value,
                    optional: None,
                },
                NonZeroUsize::new(4096).unwrap(),
            )
            .unwrap();
            assert_eq!(encoded.schema, schema);
            let source_time = if index < 2 {
                std::time::UNIX_EPOCH - Duration::from_nanos(100)
            } else {
                std::time::UNIX_EPOCH + Duration::from_nanos(100)
            };
            let represented = if index < 2 {
                std::time::UNIX_EPOCH.duration_since(source_time).unwrap()
            } else {
                source_time.duration_since(std::time::UNIX_EPOCH).unwrap()
            };
            assert_eq!(represented, Duration::from_nanos(100));
            storage
                .write_stream(&[StreamWrite {
                    stream: &stream,
                    sequence: index as u64 + 1,
                    record: RecordWrite {
                        schema: &schema,
                        scope: None,
                        item: &item,
                        source: None,
                        source_time: Some(source_time),
                        payload: &encoded.payload,
                    },
                }])
                .await
                .unwrap();
        }
        storage.flush().await.unwrap();
        let mut selection = ReadSelection::new(ReadKind::Stream);
        selection.schema = Some(schema);
        let source_range = SourceTimeRange {
            start: std::time::UNIX_EPOCH - Duration::from_nanos(1000),
            end: std::time::UNIX_EPOCH + Duration::from_nanos(1000),
        };
        assert_eq!(
            std::time::UNIX_EPOCH
                .duration_since(source_range.start)
                .unwrap(),
            Duration::from_nanos(1000),
        );
        assert_eq!(
            source_range
                .end
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap(),
            Duration::from_nanos(1000),
        );
        assert!(source_range.start < source_range.end);
        selection.source_time = Some(source_range);
        let mut frame = Frame::new(NonZeroUsize::new(16).unwrap(), NonZeroUsize::MIN);
        let scan = frame.scan(
            selection,
            vec![ScalarField {
                path: ScalarPath::Payload(vec![harmos::Identifier::new("value").unwrap()]),
                kind: ScalarKind::Unsigned,
            }],
        );
        frame.group_by(
            [
                Agg::Of(Aggregate::Count),
                Agg::Of(Aggregate::Min),
                Agg::Of(Aggregate::Max),
                Agg::Of(Aggregate::Median),
            ]
            .into_iter()
            .fold(GroupBy::new(scan), |group_by, agg| {
                group_by.agg(Expr::Col(0), agg)
            }),
        );
        let query = FrameQuery {
            frame,
            options: QueryOptions {
                page: Some(ReadLimits {
                    rows: NonZeroUsize::MIN,
                    bytes: NonZeroUsize::new(4096).unwrap(),
                }),
                ..QueryOptions::default()
            },
        };
        let mut reader = storage.query(query).await.unwrap();
        let collected = reader.read().await.unwrap();
        // The Exact Center Of 3 And u64::MAX - 2 Is Integral.
        let median = ScalarValue::Unsigned(1 << 63);
        assert_eq!(
            collected.outputs[0].groups[0].values,
            [
                ScalarValue::Unsigned(4),
                ScalarValue::Unsigned(1),
                ScalarValue::Unsigned(u64::MAX),
                median,
            ]
        );
        drop(reader);
        runtime.stop().await.unwrap();
        assert!(storage.status().read_bytes > 0);
        drop(collected);
        assert_eq!(storage.status().read_bytes, 0);
    })
    .await
    .expect("stream frame ownership did not settle");
}

pub(crate) async fn file_query_offset() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(
            Backend::<Objects>::primary(root.path().to_path_buf(), Limits::default())
                .await
                .unwrap(),
        );
        let storage = StorageManager::new(
            StorageBackends {
                primary: backend.clone(),
                state: Arc::new(backend.state(StateLimits::default())),
                replicas: vec![],
            },
            crate::sqlite::storage::storage_settings(),
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let reference = storage
            .write_file(
                FileMetadata {
                    id: harmos::Identifier::new("report").unwrap(),
                    scope: None,
                    media_type: "text/plain".into(),
                    expected_length: Some(6),
                    expected_sha256: None,
                },
                b"abcdef",
            )
            .await
            .unwrap();
        storage.flush().await.unwrap();
        let view = storage
            .view_with_limits(crate::sqlite::storage::storage_settings().query_execution)
            .await
            .unwrap();
        let mut metadata = storage.query_in(&view, FileQuery::new()).unwrap();
        let page = metadata.next_page().await.unwrap().unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0], reference);
        let mut reader = view.read_file(&reference).await.unwrap();
        assert_eq!(
            reader
                .next_chunk(NonZeroUsize::new(3).unwrap())
                .await
                .unwrap()
                .unwrap()
                .as_bytes(),
            b"abc"
        );
        assert_eq!(reader.offset(), 3);

        // Corrupt The Unread Suffix Without Changing Its Immutable Length
        let payload = std::fs::read_dir(root.path().join("files"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        std::fs::write(&payload, b"abcXYZ").unwrap();
        let error = reader
            .next_chunk(NonZeroUsize::new(3).unwrap())
            .await
            .unwrap_err();
        assert!(error.to_string().contains("file digest mismatch"));
        assert_eq!(reader.offset(), 3);
        std::fs::write(&payload, b"abcdef").unwrap();
        assert_eq!(
            reader
                .next_chunk(NonZeroUsize::new(3).unwrap())
                .await
                .unwrap()
                .unwrap()
                .as_bytes(),
            b"def"
        );
        assert_eq!(reader.offset(), 6);
        assert!(
            reader
                .next_chunk(NonZeroUsize::new(3).unwrap())
                .await
                .unwrap()
                .is_none()
        );
        drop(reader);

        storage
            .write_file(
                FileMetadata {
                    id: harmos::Identifier::new("later").unwrap(),
                    scope: None,
                    media_type: "text/plain".into(),
                    expected_length: Some(1),
                    expected_sha256: None,
                },
                b"x",
            )
            .await
            .unwrap();
        storage.flush().await.unwrap();
        assert!(metadata.next_page().await.unwrap().is_none());
        drop(metadata);
        drop(view);
        runtime.stop().await.unwrap();
        assert!(storage.status().read_bytes > 0);
        assert_eq!(page[0], reference);
        drop(page);
        assert_eq!(storage.status().read_bytes, 0);
    })
    .await
    .expect("native file query did not settle");
}

pub(crate) async fn pinned_page_waiting() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(
            Backend::<Objects>::primary(root.path().to_path_buf(), Limits::default())
                .await
                .unwrap(),
        );
        let mut settings = StorageSettings {
            budget: StorageBudget {
                read_memory_bytes: NonZeroUsize::new(3 * 1024 * 1024).unwrap(),
            },
            query_decoded_bytes: NonZeroUsize::new(4096).unwrap(),
            ..crate::sqlite::storage::storage_settings()
        };
        settings.query_execution.memory_bytes = NonZeroUsize::new(2 * 1024 * 1024).unwrap();
        let execution = settings.query_execution;
        let storage = StorageManager::new(
            StorageBackends {
                primary: backend.clone(),
                state: Arc::new(backend.state(StateLimits::default())),
                replicas: vec![],
            },
            settings,
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let sample = Sample {
            id: "pinned".into(),
            value: u64::MAX,
            optional: None,
        };
        let schema = Objects::encode(&sample, NonZeroUsize::new(4096).unwrap())
            .unwrap()
            .schema;
        storage
            .write_schema(&schema, &super::storage::declared::<Sample>())
            .await
            .unwrap();
        let admitted = storage.write_object(&sample).await.unwrap();
        storage.flush_until(&admitted.position).await.unwrap();
        let view = storage.view_with_limits(execution).await.unwrap();
        let small = || {
            let mut query = ObjectQuery::<Sample>::new();
            query.options.page = Some(ReadLimits {
                rows: NonZeroUsize::MIN,
                bytes: NonZeroUsize::new(4096).unwrap(),
            });
            query
        };
        let mut first = storage.query_in(&view, small()).unwrap();
        let retained = first.next_page().await.unwrap().unwrap();
        assert!(first.next_page().await.unwrap().is_none());
        drop(first);
        let before = storage.status().read_bytes;
        // The Requested Page Exceeds The Pool While An Earlier Page Remains Retained.
        let mut oversized = small();
        oversized.options.page.as_mut().unwrap().bytes =
            NonZeroUsize::new(4 * 1024 * 1024).unwrap();
        let mut refused = storage.query_in(&view, oversized).unwrap();
        for _ in 0..2 {
            let result = tokio::time::timeout(Duration::from_secs(1), refused.next_page())
                .await
                .expect("shared memory refusal waited for its own retained page");
            assert!(matches!(
                result,
                Err(harmos::Error::InvalidInput {
                    field: "storage read",
                    reason: "page exceeds read memory budget"
                })
            ));
            assert_eq!(storage.status().read_bytes, before);
            assert_eq!(retained[0].version, admitted.version);
        }
        drop(refused);
        let mut retry = storage.query_in(&view, small()).unwrap();
        let second = retry.next_page().await.unwrap().unwrap();
        assert_eq!(second[0].value, sample);
        assert_eq!(second[0].version, admitted.version);
        assert!(retry.next_page().await.unwrap().is_none());
        drop(retry);
        drop(view);
        runtime.stop().await.unwrap();
        drop(retained);
        assert!(storage.status().read_bytes > 0);
        drop(second);
        assert_eq!(storage.status().read_bytes, 0);
    })
    .await
    .expect("native shared memory refusal did not settle");
}

pub(crate) async fn private_object_version() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let source = Arc::new(
            Backend::<Objects>::primary(root.path().join("source"), Limits::default())
                .await
                .unwrap(),
        );
        let storage = StorageManager::new(
            StorageBackends {
                primary: source.clone(),
                state: Arc::new(source.state(StateLimits::default())),
                replicas: vec![],
            },
            crate::sqlite::storage::storage_settings(),
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let sample = Sample {
            id: "historical".into(),
            value: u64::MAX,
            optional: None,
        };
        let admitted = storage.write_object(&sample).await.unwrap();
        storage.flush_until(&admitted.position).await.unwrap();
        let snapshot = source.read_view(&admitted.position).await.unwrap();
        let view = ReadView::new(
            harmos::Identifier::new("view.original.object").unwrap(),
            admitted.position.clone(),
            admitted.position.clone(),
            snapshot.clone(),
        )
        .unwrap();
        let limits = ReadLimits {
            rows: NonZeroUsize::new(64).unwrap(),
            bytes: NonZeroUsize::new(4096).unwrap(),
        };
        let schemas = source
            .read_schemas(&view, &ReadSelection::new(ReadKind::Schemas), None, limits)
            .await
            .unwrap();
        let objects = source
            .read_objects(&view, &ReadSelection::new(ReadKind::Objects), None, limits)
            .await
            .unwrap();
        assert_eq!(schemas.entries.len(), 1);
        assert_eq!(objects.entries.len(), 1);
        assert_eq!(objects.entries[0].version, admitted.version);
        let dangling = Backend::<()>::replica(
            root.path().join("dangling"),
            Limits::default(),
            source.history().clone(),
        )
        .await
        .unwrap();
        let mut unrelated = schemas.entries.clone();
        unrelated[0].key.id = harmos::Identifier::new("fixture.unrelated").unwrap();
        let prefix = [StoragePart::Schemas(&unrelated)];
        dangling
            .write(StorageBatch::new(&prefix, limits.bytes).unwrap())
            .await
            .unwrap();
        let before = dangling.cursor_saved().await.unwrap();
        let object_only = [StoragePart::Objects(&objects.entries)];
        assert!(
            dangling
                .write(StorageBatch::new(&object_only, limits.bytes).unwrap())
                .await
                .unwrap_err()
                .to_string()
                .contains("indexed row requires its preceding admitted schema")
        );
        assert_eq!(dangling.cursor_saved().await.unwrap(), before);
        dangling.close().await.unwrap();

        let incremental = Backend::<()>::replica(
            root.path().join("incremental"),
            Limits::default(),
            source.history().clone(),
        )
        .await
        .unwrap();
        let copied = [
            StoragePart::Schemas(&schemas.entries),
            StoragePart::Objects(&objects.entries),
        ];
        let copy = StorageBatch::new(&copied, limits.bytes).unwrap();
        let control = incremental.control().await.unwrap();
        control
            .call(|connection| {
                connection.execute_batch(
                    "CREATE TRIGGER reject_copy BEFORE INSERT ON storage_object_scalars
             BEGIN SELECT RAISE(ABORT, 'injected replica index failure'); END;",
                )
            })
            .await
            .unwrap();
        assert!(incremental.write(copy).await.is_err());
        assert_eq!(incremental.cursor_saved().await.unwrap().sequence, 0);
        control
            .call(|connection| {
                let rows: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM storage_data_entries",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(rows, 0);
                connection.execute_batch("DROP TRIGGER reject_copy")
            })
            .await
            .unwrap();
        assert_eq!(incremental.write(copy).await.unwrap(), admitted.position);
        assert_eq!(incremental.write(copy).await.unwrap(), admitted.position);
        let expected = objects.entries[0].clone();
        control
            .call(move |connection| {
                let (body, fields): (Vec<u8>, Vec<u8>) = connection.query_row(
                    "SELECT body,object_fields FROM storage_data_entries WHERE kind='object'",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                let metadata = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
                let end = 4 + metadata;
                let mut actual: ObjectEntry = serde_json::from_slice(&body[4..end]).unwrap();
                assert!(actual.value.payload.is_empty());
                actual.value.payload.extend_from_slice(&body[end..]);
                assert_eq!(actual, expected);
                let fields: Vec<harmos::schema::graph::Field> =
                    serde_json::from_slice(&fields).unwrap();
                let declared = super::storage::declared::<Sample>();
                assert_eq!(fields, declared.definitions[declared.root]);
                let scalars: i64 = connection.query_row(
                    "SELECT COUNT(*) FROM storage_object_scalars",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(scalars, 3);
                Ok(())
            })
            .await
            .unwrap();
        drop(control);
        incremental.close().await.unwrap();
        let target = Backend::<()>::replica(
            root.path().join("target"),
            Limits::default(),
            source.history().clone(),
        )
        .await
        .unwrap();
        let prior = target.cursor_saved().await.unwrap();
        let identity = BootstrapId::new(&RetainedCoverage {
            cut: admitted.position.clone(),
            complete_after: 0,
            retention_generation: std::num::NonZeroU64::MIN,
        })
        .unwrap();
        target
            .begin_bootstrap(
                &identity,
                &prior,
                crate::sqlite::storage::storage_settings().query_execution,
            )
            .await
            .unwrap();
        target
            .write_bootstrap(
                RetainedBatch::new(&identity, StoragePart::Schemas(&schemas.entries), limits)
                    .unwrap(),
            )
            .await
            .unwrap();
        let batch =
            RetainedBatch::new(&identity, StoragePart::Objects(&objects.entries), limits).unwrap();
        target.write_bootstrap(batch).await.unwrap();
        target.write_bootstrap(batch).await.unwrap();
        let mut manifest = BootstrapManifest::new(identity.clone());
        manifest
            .record(
                RetainedBatch::new(&identity, StoragePart::Schemas(&schemas.entries), limits)
                    .unwrap(),
            )
            .unwrap();
        manifest.record(batch).unwrap();
        for kind in [
            ReadKind::Schemas,
            ReadKind::Objects,
            ReadKind::Records,
            ReadKind::Stream,
            ReadKind::Files,
        ] {
            manifest.finish_collection(kind);
        }
        manifest.finish_releases();
        target.finish_bootstrap(&manifest).await.unwrap();
        assert_eq!(target.cursor_saved().await.unwrap(), admitted.position);
        let target_snapshot = target.read_view(&admitted.position).await.unwrap();
        let target_view = ReadView::new(
            harmos::Identifier::new("imported.object").unwrap(),
            admitted.position.clone(),
            admitted.position.clone(),
            target_snapshot.clone(),
        )
        .unwrap();
        let imported = target
            .read_objects(
                &target_view,
                &ReadSelection::new(ReadKind::Objects),
                None,
                limits,
            )
            .await
            .unwrap();
        assert_eq!(imported.entries, objects.entries);
        target.release_view(&target_snapshot).await.unwrap();
        target.close().await.unwrap();
        source.release_view(&snapshot).await.unwrap();
        runtime.stop().await.unwrap();
    })
    .await
    .expect("original logical object import did not settle");
}
