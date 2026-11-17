/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Backend Durability and Retained Reads

/* === Modules === */

mod receipt_history_reopens;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::fs;
use std::io::Read;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Arc;
use std::{error, result, time::Duration};

// Third Party Imports
use arrow_array::UInt64Array;
use harmos_domain::config::SchemaSettings;
use harmos_domain::journal::Position;
use harmos_domain::schema::{Described, graph::Schema};
use harmos_domain::{Identified, Message};
use harmos_query::columnar::{ArrowScan, Cancellation, ColumnKind, ColumnSource};
use harmos_query::index::{IndexDefinition, IndexRead};
use harmos_storage::conformance::{self, CompoundFixture, ScopeFixture};
use harmos_storage::execution::Execution;
use harmos_storage::query::projection::{ProjectionDefinition, ProjectionRead, ProjectionTransform, advance_projection};
use harmos_storage::query::{ObjectSelection, ObjectVersions, StreamSelection};
use harmos_storage::state::{RecoveryEntry, RecoverySnapshot, SnapshotPayload};
use harmos_storage::{
    FileChunk, FileMetadata, Identifier, ObjectPoint, ObjectVersion, PayloadEncoding, ReadLimits, RecoveryPosition, ScanPage, SchemaKey,
    StorageBatch, StorageCursor, StorageSchema, StoreHistory, StoredRecord, StoredRow,
};
use harmos_storage::{ObjectCatalog, ObjectDefinition, ObjectRetention, StoredObject};
use harmos_storage::{StorageFuture, StorageManager, WorkerOwner};

// Local Imports
use dokime_storage::persisted::{self, Administration, Case, Plan, Scope};
use dokime_storage::{Backend, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type StoryResult<T = ()> = Result<T, Box<dyn error::Error>>;

/* === Structs === */
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
struct Reading {
    value: u64,
    label: String,
    details: Option<Details>,
    values: Vec<i64>,
    choice: Choice,
    fraction: f32,
    bytes: Vec<u8>,
}
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
struct Details {
    enabled: bool,
    value: Option<u64>,
}

/* === Enums === */
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
enum Choice {
    Empty,
    Value(Details),
    Many(Vec<Details>),
}

/* === Traits === */

/* === Implementations === */

impl Identified for Reading {
    const ID: &'static str = "readings";
}
impl StoredObject for Reading {
    type Key = String;
    const RETENTION: ObjectRetention = ObjectRetention::All;
    fn key(&self) -> &String {
        &self.label
    }
}

struct StorageWorker(tokio::sync::Mutex<Option<tokio::task::JoinHandle<harmos_storage::Result<()>>>>);

impl WorkerOwner for StorageWorker {
    fn stop(&self) -> StorageFuture<'_, ()> {
        Box::pin(async move {
            if let Some(task) = self.0.lock().await.take() {
                task.await.map_err(|source| {
                    harmos_storage::Error::backend("Join Backend Conformance Worker", harmos_storage::BackendFailure::Permanent, source)
                })??;
            }
            Ok(())
        })
    }
}

/* === Public Functions === */

#[tokio::test]
async fn catalog_objects_can_be_scanned_before_their_first_write_and_after_reopen() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let definition = ObjectDefinition::of::<Reading>();
    let catalog = ObjectCatalog::new().entry(definition).extend(persisted::catalog());
    let (backends, lease) = Backend::open_with_execution(&store, catalog.clone(), Execution::default())?;
    let backend = backends.primary.clone();
    backend.create_session().await?;
    let at = backend.get_saved_position().await?;
    let snapshot = backend.create_snapshot(&at).await?;
    let scan = ArrowScan {
        source: ColumnSource {
            kind: ColumnKind::Objects,
            schema: definition.schema()?,
        },
        columns: vec!["payload.value".into()],
        predicate: None,
        time: None,
        start: 0,
        after: None,
        limits: limits(),
        work: limits(),
    };
    let cancel = Cancellation::default();
    let page = backend.list_column_rows(&snapshot, &scan, &cancel).await?;
    assert_eq!(page.batch.num_rows(), 0);
    assert!(page.complete);
    let administration = ArrowScan {
        source: ColumnSource {
            kind: ColumnKind::Objects,
            schema: ObjectDefinition::of::<Administration>().schema()?,
        },
        columns: vec!["payload.key".into(), "_storage.scope".into(), "payload.payload".into()],
        ..scan.clone()
    };
    let page = backend.list_column_rows(&snapshot, &administration, &cancel).await?;
    assert_eq!(page.batch.num_rows(), 0);
    assert!(page.complete);
    backend.delete_snapshot(&snapshot).await?;
    drop(lease);
    assert!(backend.get_saved_position().await.is_err());

    let (reopened, _lease) = Backend::open_with_execution(&store, catalog, Execution::default())?;
    reopened.primary.create_session().await?;
    let snapshot = reopened.primary.create_snapshot(&at).await?;
    let page = reopened.primary.list_column_rows(&snapshot, &scan, &cancel).await?;
    assert_eq!(page.batch.num_rows(), 0);
    assert!(page.complete);
    let page = reopened.primary.list_column_rows(&snapshot, &administration, &cancel).await?;
    assert_eq!(page.batch.num_rows(), 0);
    assert!(page.complete);
    reopened.primary.delete_snapshot(&snapshot).await?;
    reopened.primary.delete_session().await?;
    Ok(())
}

/* === Private Functions === */
fn schema(history: &StoreHistory) -> StoryResult<StorageSchema> {
    Ok(StorageSchema {
        position: StorageCursor { history: history.clone(), sequence: 1 },
        key: SchemaKey {
            id: Identifier::new("readings")?,
            version: NonZeroU32::MIN,
            payload_encoding: PayloadEncoding::PostcardV1,
        },
        schema: Arc::new(Schema::from_declaration(Reading::DEFINITION, &SchemaSettings::default())?),
    })
}
fn rows(schema: &StorageSchema, first: u64, count: u64, scope: &str) -> StoryResult<Vec<StoredRow>> {
    (0..count)
        .map(|index| {
            let value = Reading {
                value: index,
                label: index.to_string(),
                details: (index % 2 == 0).then_some(Details {
                    enabled: true,
                    value: (index % 4 == 0).then_some(u64::MAX - index),
                }),
                values: vec![-(index as i64), index as i64],
                choice: match index % 3 {
                    0 => Choice::Empty,
                    1 => Choice::Value(Details { enabled: false, value: Some(index) }),
                    _ => Choice::Many(vec![Details { enabled: true, value: None }]),
                },
                fraction: index as f32 + 0.125,
                bytes: vec![0, 1, index as u8],
            };
            Ok(StoredRow {
                record: StoredRecord {
                    position: StorageCursor {
                        history: schema.position.history.clone(),
                        sequence: first + index,
                    },
                    schema: schema.key.clone(),
                    scope: Some(Identifier::new(scope)?),
                    item: Identifier::new("source")?,
                    source: None,
                    source_time: None,
                    payload: value.encode(),
                },
                stream: Identifier::new("readings")?,
                sequence: index + 1,
            })
        })
        .collect()
}
fn selection(schema: &SchemaKey, scope: &str) -> StoryResult<StreamSelection> {
    Ok(StreamSelection {
        schema: schema.clone(),
        scope: Some(Identifier::new(scope)?),
        item: None,
        stream: None,
        source_start: None,
        source_end: None,
        after: 0,
    })
}
fn limits() -> ReadLimits {
    ReadLimits {
        rows: NonZeroUsize::new(64).unwrap(),
        bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
    }
}

/* === Tests === */

#[test]
fn library_bucket_continuation_coarsens_a_stored_envelope() -> StoryResult {
    use arrow_array::{Array, ArrayRef, BinaryArray, Float64Array, Int64Array, RecordBatch, UInt64Array};
    use harmos_domain::model::storage::query::transform::statistics::BucketSummary;
    use harmos_query::transform::{FieldPath, Transform, TransformExecution};
    use std::num::NonZeroU64;
    let samples = RecordBatch::try_from_iter([
        ("time", Arc::new(Int64Array::from(vec![0, 1, 2, 3])) as ArrayRef),
        ("position", Arc::new(UInt64Array::from(vec![1, 2, 3, 4])) as ArrayRef),
        ("value", Arc::new(Float64Array::from(vec![10.0, 20.0, 30.0, 40.0])) as ArrayRef),
    ])?;
    let opening = BucketSummary {
        time: FieldPath::from("time"),
        position: FieldPath::from("position"),
        value: FieldPath::from("value"),
        width_ns: NonZeroU64::MIN,
        max_buckets: NonZeroUsize::new(4).unwrap(),
        source_cut: StorageCursor {
            history: StoreHistory {
                store: Identifier::new("preview-story")?,
                epoch: Identifier::new("execution")?,
            },
            sequence: 4,
        },
        prior: None,
    };
    let saved = Transform::BucketSummary(opening.clone()).apply(&samples, &mut || Ok(()))?;
    let continuation = saved.column_by_name("continuation").unwrap().as_any().downcast_ref::<BinaryArray>().unwrap();
    assert!(!continuation.is_null(0));
    let coarsened = Transform::BucketSummary(BucketSummary {
        max_buckets: NonZeroUsize::new(2).unwrap(),
        prior: Some(continuation.value(0).to_vec()),
        ..opening
    })
    .apply(&samples.slice(0, 0), &mut || Ok(()))?;
    let counts = coarsened.column_by_name("count").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
    let means = coarsened.column_by_name("mean").unwrap().as_any().downcast_ref::<Float64Array>().unwrap();
    assert_eq!(counts.values().as_ref(), &[2, 2]);
    assert_eq!(means.values().as_ref(), &[15.0, 35.0]);
    Ok(())
}

#[tokio::test]
async fn native_backend_passes_compound_and_scoped_deletion_conformance() -> StoryResult {
    use harmos_storage::{
        MutationId, MutationItem, ObjectVersionToken, RecordInput, StorageFilesystemSettings, StoragePolicy, StorageRequestSettings,
        StorageResponseSettings, StorageSettings, StreamInput,
    };
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let catalog = ObjectCatalog::new().entry(ObjectDefinition::of::<Reading>()).extend(persisted::catalog());
    let (backends, _lease) = Backend::open_with_execution(&store, catalog.clone(), Execution::default())?;
    let capacity = NonZeroUsize::new(64).unwrap();
    let storage = StorageManager::with_catalog(
        catalog.clone(),
        backends,
        StorageSettings {
            transfer: Default::default(),
            policy: StoragePolicy::default(),
            request: StorageRequestSettings {
                max_file_productions: capacity,
                max_mutation_receipts: capacity,
                max_pending_queries: capacity,
                max_active_queries: capacity,
                max_pending_write_batches: capacity,
                max_active_file_operations: capacity,
                max_pending_write_bytes: limits().bytes,
            },
            response: StorageResponseSettings {
                batch_rows: capacity,
                batch_bytes: limits().bytes,
            },
            filesystem: StorageFilesystemSettings {
                poll_interval: Duration::from_millis(100),
                max_entries: capacity,
                event_capacity: capacity,
            },
        },
    )?;
    storage
        .start_with(Arc::new(()), |mut worker| {
            let task = tokio::spawn(async move {
                worker.run().await?;
                worker.cleanup().await
            });
            Ok(Box::new(StorageWorker(tokio::sync::Mutex::new(Some(task)))))
        })
        .await?;

    let value = |key: &str, value| -> StoryResult<_> {
        let reading = Reading {
            value,
            label: key.into(),
            details: None,
            values: Vec::new(),
            choice: Choice::Empty,
            fraction: 0.5,
            bytes: vec![1, 2],
        };
        Ok(catalog.encode(&reading, limits().bytes)?)
    };
    let first = value("compound-first", 10)?;
    let record = RecordInput {
        schema: first.schema.clone(),
        scope: None,
        item: Identifier::new("compound-record")?,
        source: None,
        source_time: None,
        payload: first.payload.clone(),
    };
    let identity = MutationId {
        window: storage.mutation_window()?.id,
        key: Identifier::new("conformance.compound.create")?,
    };
    let items = vec![MutationItem::Create(first.clone()), MutationItem::Append(record.clone())];
    let proof = CompoundFixture {
        first: first.clone(),
        replacement: value("compound-first", 20)?,
        second: value("compound-second", 30)?,
        record: record.clone(),
    };
    conformance::compound_mutations(&storage, proof).await?;
    let created = storage.mutate_once(&identity, items.clone()).await?;
    let partial = StorageCursor {
        sequence: created.position.sequence - 1,
        ..created.position.clone()
    };
    let opening = StorageCursor { sequence: 0, ..created.position.clone() };
    let frontier = storage.watch_frontier(&opening, &partial).await?;
    assert!(frontier.sequence < partial.sequence);
    let Err(harmos_storage::Error::Persistence { source }) = storage.snapshot(&partial).await else {
        panic!("Partial Compound Cut Must Be Refused");
    };
    assert!(matches!(source.as_ref(), harmos_storage::Error::InvalidInput { .. }));

    let refused_identity = MutationId {
        window: identity.window.clone(),
        key: Identifier::new("conformance.compound.conflict")?,
    };
    let refused_items = vec![
        MutationItem::Create(value("compound-second", 30)?),
        MutationItem::Append(record),
        MutationItem::Set {
            expected: ObjectVersionToken::new(created.position.clone()),
            value: value("compound-first", 20)?,
        },
    ];
    let refused = storage.mutate_once(&refused_identity, refused_items.clone()).await?;

    let scope = Identifier::new("conformance-scope")?;
    let mut object = value("scoped", 42)?;
    object.scope = Some(scope.clone());
    let record = RecordInput {
        schema: object.schema.clone(),
        scope: Some(scope.clone()),
        item: Identifier::new("scoped-record")?,
        source: None,
        source_time: None,
        payload: object.payload.clone(),
    };
    let mut mixed = Vec::new();
    for index in 0..6 {
        let payload = value("mixed", index)?;
        mixed.push(StreamInput {
            record: RecordInput {
                schema: object.schema.clone(),
                scope: (index % 2 == 1).then(|| scope.clone()),
                item: Identifier::new("mixed")?,
                source: None,
                source_time: None,
                payload: payload.payload,
            },
            stream: Identifier::new("mixed-stream")?,
            sequence: index + 1,
        });
    }
    storage.append_samples(mixed).await?;
    storage.flush().await?;
    let connection = rusqlite::Connection::open(root.path().join("storage.sqlite3"))?;
    let deleted_chunks = {
        let mut query = connection.prepare(
            "SELECT file FROM arrow_chunks c WHERE EXISTS(SELECT 1 FROM storage_rows r
             WHERE r.kind=3 AND r.scope=?1 AND r.position BETWEEN c.first AND c.last)",
        )?;
        query.query_map([scope.as_str()], |row| row.get::<_, String>(0))?.collect::<result::Result<Vec<_>, _>>()?
    };
    let proof = ScopeFixture {
        scope: scope.clone(),
        object,
        record: record.clone(),
        sample: StreamInput {
            record,
            stream: Identifier::new("scoped-stream")?,
            sequence: 1,
        },
        file: FileMetadata {
            id: Identifier::new("scoped-file")?,
            scope: Some(scope),
            media_type: "application/octet-stream".into(),
            expected_length: None,
            expected_sha256: None,
        },
        bytes: b"scoped-content".to_vec(),
    };
    conformance::scoped_deletion(&storage, proof).await?;
    let snapshot = storage.snapshot(&storage.get_saved_position()).await?;
    let mut scan = ArrowScan {
        source: ColumnSource {
            kind: ColumnKind::Stream,
            schema: ObjectDefinition::of::<Reading>().schema()?,
        },
        columns: vec!["payload.value".into()],
        predicate: None,
        time: None,
        start: 0,
        after: None,
        limits: limits(),
        work: limits(),
    };
    let mut values = Vec::new();
    loop {
        let page = snapshot.list_column_rows(&scan, &Cancellation::default()).await?;
        let column = page.batch.column(0).as_any().downcast_ref::<UInt64Array>().unwrap();
        values.extend(column.values().iter().copied());
        if page.complete {
            break;
        }
        scan.after = page.next;
    }
    assert_eq!(values, [0, 2, 4]);
    drop(snapshot);

    // Retain One Execution Scope Independently of Unrelated History.
    let retained_scope = Identifier::new("retained-execution")?;
    let mut object = value("retained-object", 87)?;
    object.scope = Some(retained_scope.clone());
    let record = RecordInput {
        schema: object.schema.clone(),
        scope: Some(retained_scope.clone()),
        item: Identifier::new("retained-record")?,
        source: None,
        source_time: None,
        payload: object.payload.clone(),
    };
    let retained = conformance::retain_scope(
        &storage,
        ScopeFixture {
            scope: retained_scope.clone(),
            object,
            record: record.clone(),
            sample: StreamInput {
                record,
                stream: Identifier::new("retained-stream")?,
                sequence: 1,
            },
            file: FileMetadata {
                id: Identifier::new("retained-file")?,
                scope: Some(retained_scope),
                media_type: "application/octet-stream".into(),
                expected_length: None,
                expected_sha256: None,
            },
            bytes: b"retained-evidence".to_vec(),
        },
    )
    .await?;
    let sample_scope = Identifier::new("columnar-execution")?;
    let frozen = storage.lookup_schema(&first.schema).await?;
    let sample_columns = harmos_storage::writes::DecodedBatch {
        destination: Identifier::new("readings")?,
        schema: first.schema.clone(),
        data: harmos_query::arrow::ArrowBatch::try_new(harmos_query::source::stream(
            &frozen,
            &rows(&frozen, 1, 5, sample_scope.as_str())?,
        )?)?,
    };
    let (sample_mutation, sample_frontier) = conformance::columnar_appends(&storage, &sample_scope, &sample_columns).await?;
    let oversized = harmos_storage::writes::DecodedBatch {
        data: harmos_query::arrow::ArrowBatch::try_new(harmos_query::source::stream(
            &frozen,
            &rows(&frozen, 1, 32_768, sample_scope.as_str())?,
        )?)?,
        ..sample_columns.clone()
    };
    conformance::columnar_admission_bounds(&storage, &sample_scope, &oversized).await?;
    let retained_samples = storage.snapshot(&sample_frontier).await?.retain(Some(&sample_scope)).await?;

    let settings = storage.settings().clone();
    storage.close().await?;
    let connection = rusqlite::Connection::open(root.path().join("storage.sqlite3"))?;
    let chunks: i64 = connection.query_row("SELECT count(*) FROM storage_file_chunks WHERE id='scoped-file'", [], |row| row.get(0))?;
    assert_eq!(chunks, 0);
    for file in deleted_chunks {
        assert!(!root.path().join("storage.arrow").join(file).exists());
    }

    // Replay the Original Receipt After Both Backend Sessions Reopen.
    let (backends, _lease) = Backend::open_with_execution(&store, catalog.clone(), Execution::default())?;
    let storage = StorageManager::with_catalog(catalog.clone(), backends, settings)?;
    storage
        .start_with(Arc::new(()), |mut worker| {
            let task = tokio::spawn(async move {
                worker.run().await?;
                worker.cleanup().await
            });
            Ok(Box::new(StorageWorker(tokio::sync::Mutex::new(Some(task)))))
        })
        .await?;
    conformance::columnar_reopened(&storage, &sample_scope, &sample_columns, &sample_mutation, &sample_frontier).await?;
    let reader = storage.open_retained_snapshot(&retained_samples).await?;
    let independent_samples = reader.retain(None).await?;
    drop(reader);
    let delete_samples = MutationId {
        window: storage.mutation_window()?.id,
        key: Identifier::new("delete-columnar-scope")?,
    };
    storage.mutate_once(&delete_samples, vec![MutationItem::DeleteScope(sample_scope.clone())]).await?;
    assert!(storage.open_retained_snapshot(&retained_samples).await.is_err());
    let current = storage.snapshot(&storage.get_saved_position()).await?;
    let mut sample_scan = ArrowScan {
        source: ColumnSource {
            kind: ColumnKind::Stream,
            schema: sample_columns.schema.clone(),
        },
        columns: vec!["_storage.scope".into(), "payload.value".into()],
        predicate: None,
        time: None,
        start: 0,
        after: None,
        limits: limits(),
        work: limits(),
    };
    loop {
        let page = current.list_column_rows(&sample_scan, &Cancellation::default()).await?;
        let scopes = page.batch.column_by_name("_storage.scope").unwrap().as_any().downcast_ref::<arrow_array::StringArray>().unwrap();
        assert!(!scopes.iter().flatten().any(|scope| scope == sample_scope.as_str()));
        if page.complete {
            break;
        }
        sample_scan.after = page.next;
    }
    drop(current);
    let independent = storage.open_retained_snapshot(&independent_samples).await?;
    sample_scan.after = None;
    let mut retained_rows = 0;
    loop {
        let page = independent.list_column_rows(&sample_scan, &Cancellation::default()).await?;
        retained_rows += page.batch.num_rows();
        if page.complete {
            break;
        }
        sample_scan.after = page.next;
    }
    assert_eq!(retained_rows, 5);
    drop(independent);
    storage.release_retained_snapshot(&independent_samples).await?;

    assert_eq!(storage.mutate_once(&identity, items).await?, created);
    assert_eq!(storage.mutate_once(&refused_identity, refused_items).await?, refused);
    conformance::check_retained_scope(&storage, &retained).await?;

    // A Retained Execution Must Not Pin Another Execution's Deleted Bytes.
    let unrelated = Identifier::new("unrelated-execution")?;
    let sample = StreamInput {
        record: RecordInput {
            schema: first.schema.clone(),
            scope: Some(unrelated.clone()),
            item: Identifier::new("unrelated-record")?,
            source: None,
            source_time: None,
            payload: first.payload.clone(),
        },
        stream: Identifier::new("unrelated-stream")?,
        sequence: 1,
    };
    storage.append_samples(vec![sample]).await?;
    let mut writer = storage
        .produce_file(FileMetadata {
            id: Identifier::new("unrelated-file")?,
            scope: Some(unrelated.clone()),
            media_type: "application/octet-stream".into(),
            expected_length: None,
            expected_sha256: None,
        })
        .await?;
    writer.write_all(b"unrelated-evidence".to_vec()).await?;
    writer.finish().await?;
    storage.flush().await?;
    let deletion = MutationId {
        window: storage.mutation_window()?.id,
        key: Identifier::new("delete-unrelated")?,
    };
    storage.mutate_once(&deletion, vec![MutationItem::DeleteScope(unrelated.clone())]).await?;
    let rows: i64 = connection.query_row("SELECT count(*) FROM storage_rows WHERE scope=?1", [unrelated.as_str()], |row| row.get(0))?;
    assert_eq!(rows, 0);
    let chunks: i64 = connection.query_row("SELECT count(*) FROM storage_file_chunks WHERE id='unrelated-file'", [], |row| row.get(0))?;
    assert_eq!(chunks, 0);
    conformance::check_retained_scope(&storage, &retained).await?;
    // Independent Reader Ownership Survives Scope Deletion and Another Restart.
    let image = storage.open_retained_snapshot(&retained.snapshot).await?;
    let point = ObjectPoint {
        schema: first.schema.id.clone(),
        item: first.item.clone(),
        version: ObjectVersion::Latest,
    };
    assert!(image.get_object(&point).await?.is_none());
    let independent = image.retain(None).await?;
    drop(image);
    let deletion = MutationId {
        window: storage.mutation_window()?.id,
        key: Identifier::new("delete-retained-owner")?,
    };
    storage.mutate_once(&deletion, vec![MutationItem::DeleteScope(retained.fixture.scope.clone())]).await?;
    assert!(storage.open_retained_snapshot(&retained.snapshot).await.is_err());
    let settings = storage.settings().clone();
    storage.close().await?;
    let sample_ranges: i64 =
        connection.query_row("SELECT count(*) FROM storage_sample_ranges WHERE scope=?1", [sample_scope.as_str()], |row| row.get(0))?;
    assert_eq!(sample_ranges, 0);
    let (backends, _lease) = Backend::open_with_execution(&store, catalog.clone(), Execution::default())?;
    let storage = StorageManager::with_catalog(catalog.clone(), backends, settings)?;
    storage
        .start_with(Arc::new(()), |mut worker| {
            let task = tokio::spawn(async move {
                worker.run().await?;
                worker.cleanup().await
            });
            Ok(Box::new(StorageWorker(tokio::sync::Mutex::new(Some(task)))))
        })
        .await?;
    let mut retained = retained;
    retained.snapshot = independent;
    conformance::check_retained_scope(&storage, &retained).await?;
    storage.release_retained_snapshot(&retained.snapshot).await?;
    assert!(storage.open_retained_snapshot(&retained.snapshot).await.is_err());

    // Publish the Frozen Plan and Cases Under Independent Exact Owners.
    let product_mutation = MutationId {
        window: storage.mutation_window()?.id,
        key: Identifier::new("product-plan")?,
    };
    let case = Case {
        key: "shared".into(),
        scope: Scope::Case("shared").identifier()?,
        cycle: "shared".into(),
        payload: vec![10, 2, 255, 0],
    };
    let related = Case {
        key: "shared/child".into(),
        scope: Scope::Case("shared/child").identifier()?,
        ..case.clone()
    };
    let plan = Plan {
        key: "product-plan".into(),
        scope: Scope::Cycle("shared").identifier()?,
        plan: vec![18, 2, 0, 255],
        cases: vec![case.clone(), related.clone()],
        window: product_mutation.window.clone(),
        mutation: product_mutation.key.clone(),
    };
    let settings = Administration {
        key: "preferences".into(),
        scope: Scope::Root("shared").identifier()?,
        payload: vec![8, 1],
    };
    let command = Administration {
        key: "execution-command/shared/first".into(),
        scope: Scope::Execution("shared").identifier()?,
        payload: vec![8, 2],
    };
    let product_items = vec![
        MutationItem::Create(catalog.encode(&plan, limits().bytes)?),
        MutationItem::Create(catalog.encode(&case, limits().bytes)?),
        MutationItem::Create(catalog.encode(&related, limits().bytes)?),
        MutationItem::Create(catalog.encode(&settings, limits().bytes)?),
        MutationItem::Create(catalog.encode(&command, limits().bytes)?),
    ];
    let product_receipt = storage.mutate_once(&product_mutation, product_items.clone()).await?;
    let before = storage.snapshot(&product_receipt.position).await?;
    let deletion = MutationId {
        window: product_mutation.window.clone(),
        key: Identifier::new("delete-product-case")?,
    };
    storage.mutate_once(&deletion, vec![MutationItem::DeleteScope(case.scope.clone())]).await?;
    assert!(storage.get_object::<Case>(&case.key, ObjectVersion::Latest).await?.is_none());
    assert_eq!(storage.get_object::<Case>(&related.key, ObjectVersion::Latest).await?.unwrap().value, related);
    assert_eq!(storage.get_object::<Plan>(&plan.key, ObjectVersion::Latest).await?.unwrap().value, plan);
    assert_eq!(storage.get_object::<Administration>(&settings.key, ObjectVersion::Latest).await?.unwrap().value, settings);
    assert_eq!(storage.get_object::<Administration>(&command.key, ObjectVersion::Latest).await?.unwrap().value, command);
    let encoded = catalog.encode(&case, limits().bytes)?;
    let point = ObjectPoint {
        schema: encoded.schema.id,
        item: encoded.item,
        version: ObjectVersion::Latest,
    };
    let retained = before.get_object(&point).await?.unwrap();
    assert_eq!(catalog.decode::<Case>(&retained.value, limits().bytes)?, case);
    assert_eq!(storage.mutate_once(&product_mutation, product_items).await?, product_receipt);
    assert!(storage.get_object::<Case>(&case.key, ObjectVersion::Latest).await?.is_none());
    drop(before);

    // Delete Execution View Bytes Without Removing Other Exact Owners.
    let mut files = Vec::new();
    for scope in [Scope::Execution("shared"), Scope::Cycle("shared"), Scope::Case("shared"), Scope::Root("shared")] {
        let scope = scope.identifier()?;
        let mut writer = storage
            .produce_file(FileMetadata {
                id: Identifier::new(format!("retained-view/{scope}"))?,
                scope: Some(scope),
                media_type: "application/x-protobuf".into(),
                expected_length: None,
                expected_sha256: None,
            })
            .await?;
        writer.write_all(vec![8, 1]).await?;
        files.push(writer.finish().await?);
    }
    storage.flush().await?;
    let image = storage.snapshot(&storage.get_saved_position()).await?;
    let deletion = MutationId {
        window: storage.mutation_window()?.id,
        key: Identifier::new("delete-execution-views")?,
    };
    storage.mutate_once(&deletion, vec![MutationItem::DeleteScope(Scope::Execution("shared").identifier()?)]).await?;
    let current = storage.snapshot(&storage.get_saved_position()).await?;
    assert!(current.get_file_bytes(&files[0], 0, NonZeroUsize::new(2).unwrap()).await.is_err());
    for file in &files {
        assert_eq!(image.get_file_bytes(file, 0, NonZeroUsize::new(2).unwrap()).await?.data, vec![8, 1]);
    }
    for file in &files[1..] {
        assert_eq!(current.get_file_bytes(file, 0, NonZeroUsize::new(2).unwrap()).await?.data, vec![8, 1]);
    }
    drop(current);
    drop(image);

    storage.close().await?;
    let chunks: i64 = connection.query_row("SELECT count(*) FROM storage_file_chunks WHERE id='retained-file'", [], |row| row.get(0))?;
    assert_eq!(chunks, 0);
    Ok(())
}

#[tokio::test]
async fn superseded_backend_roots_are_refused_without_changing_their_values() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    backends.primary.create_session().await?;
    backends.primary.delete_session().await?;
    let connection = rusqlite::Connection::open(root.path().join("storage.sqlite3"))?;
    let prior = vec![1u8, 2, 3, 4];
    connection.execute("UPDATE storage_meta SET value=?1 WHERE name='format'", ["dokime.scoped-objects.v1"])?;
    connection.execute("INSERT INTO storage_meta VALUES('preserved',?1)", [&prior])?;
    assert!(Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default()).is_err());
    let execution_bytes = fs::read(root.path().join("execution.sqlite3"))?;
    assert!(matches!(Store::open(root.path()), Err(dokime_storage::Error::Incompatible(_))));
    assert_eq!(fs::read(root.path().join("execution.sqlite3"))?, execution_bytes);
    let saved: Vec<u8> = connection.query_row("SELECT value FROM storage_meta WHERE name='preserved'", [], |row| row.get(0))?;
    assert_eq!(saved, prior);
    Ok(())
}

#[tokio::test]
async fn object_identities_stay_unique_when_their_scope_changes() -> StoryResult {
    use harmos_storage::{IdentityKind, IdentitySelection, ObjectEntry, ObjectKey, ObjectValue, ObjectVersionToken, StoredIdentity};
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    let backend = &backends.primary;
    backend.create_session().await?;
    let schema = schema(backend.get_history())?;
    backend.append_schemas(StorageBatch::new(vec![schema.clone()]).into()).await?;
    let key = ObjectKey::from_owned("object".into())?;
    let mut entries = Vec::new();
    for (sequence, scope) in [(2, "first"), (3, "second")] {
        let position = StorageCursor {
            sequence,
            history: backend.get_history().clone(),
        };
        entries.push(ObjectEntry {
            version: ObjectVersionToken::new(position.clone()),
            position,
            value: ObjectValue {
                schema: schema.key.clone(),
                item: key.clone(),
                scope: Some(Identifier::new(scope)?),
                payload: rows(&schema, sequence, 1, scope)?.remove(0).record.payload,
            },
        });
    }
    let at = backend.append_object_revisions(StorageBatch::new(entries)).await?;
    let snapshot = backend.create_snapshot(&at).await?;
    let selection = IdentitySelection {
        kind: IdentityKind::Objects,
        schema: Some(schema.key.id.clone()),
    };
    let page = ScanPage {
        after: 0,
        rows: limits().rows,
        bytes: limits().bytes,
    };
    let identities = backend.list_identities(&snapshot, &selection, page).await?;
    assert!(matches!(identities.as_slice(), [StoredIdentity::Object { key: saved, .. }] if saved == &key));
    // A Scoped Image Uses the Current Object Owner at Its Cut.
    for (scope, count) in [("first", 0), ("second", 1)] {
        let scope = Identifier::new(scope)?;
        let retained = backend.retain_snapshot(&snapshot, Some(&scope)).await?;
        let image = backend.open_retained_snapshot(&retained).await?;
        let selected = ObjectSelection {
            schema: Some(schema.key.clone()),
            key: None,
            versions: ObjectVersions::Revisions,
        };
        let objects = backend.list_object_revisions(&image, &selected, page).await?;
        assert_eq!(objects.len(), count);
        let identities = backend.list_identities(&image, &selection, page).await?;
        assert_eq!(identities.len(), count);
        let point = ObjectPoint {
            schema: schema.key.id.clone(),
            item: key.clone(),
            version: ObjectVersion::Latest,
        };
        assert_eq!(usize::from(backend.get_object(&image, &point).await?.is_some()), count);
        let foreign = Identifier::new("other")?;
        assert!(backend.retain_snapshot(&image, Some(&foreign)).await.is_err());
        backend.release_retained_snapshot(&retained).await?;
        backend.delete_snapshot(&image).await?;
    }
    backend.delete_snapshot(&snapshot).await?;
    backend.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn pinned_native_arrow_pages_keep_their_saved_cut() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    let backend = &backends.primary;
    let history = backend.get_history().clone();
    assert_eq!(&history, backends.state.get_history());
    backend.create_session().await?;
    let schema = schema(&history)?;
    backend.append_schemas(StorageBatch::new(vec![schema.clone()]).into()).await?;
    let first = rows(&schema, 2, 23, "first")?;
    backend.append_samples(StorageBatch::new(first.clone())).await?;
    let saved = backend.get_saved_position().await?;
    let snapshot = backend.create_snapshot(&saved).await?;
    backend.append_samples(StorageBatch::new(rows(&schema, 25, 10, "second")?)).await?;
    let mut scan = ArrowScan {
        source: ColumnSource {
            kind: ColumnKind::Stream,
            schema: schema.key.clone(),
        },
        columns: vec!["payload.value".into()],
        predicate: None,
        time: None,
        start: 0,
        after: None,
        limits: ReadLimits {
            rows: NonZeroUsize::new(7).unwrap(),
            ..limits()
        },
        work: limits(),
    };
    let cancel = Cancellation::default();
    let version = scan.source.schema.version;
    scan.source.schema.version = version.checked_add(1).unwrap();
    assert!(backend.list_column_rows(&snapshot, &scan, &cancel).await.is_err(), "Refuse a Schema Version the Cut Does Not Hold");
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
    let page = ScanPage {
        after: 0,
        rows: limits().rows,
        bytes: limits().bytes,
    };
    assert_eq!(backend.list_samples(&snapshot, &selection(&schema.key, "first")?, page).await?, first);
    assert!(backend.list_samples(&snapshot, &selection(&schema.key, "second")?, page).await?.is_empty());
    backend.delete_snapshot(&snapshot).await?;
    assert!(backend.list_column_rows(&snapshot, &scan, &cancel).await.is_err());
    backend.delete_session().await?;
    drop(backends);
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    assert_eq!(reopened.primary.get_history(), &history);
    reopened.primary.create_session().await?;
    let snapshot = reopened.primary.create_snapshot(&saved).await?;
    assert_eq!(reopened.primary.list_samples(&snapshot, &selection(&schema.key, "first")?, page).await?, first);
    reopened.primary.delete_snapshot(&snapshot).await?;
    reopened.primary.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn native_sample_pages_preserve_valid_rows_before_an_oversized_payload() -> StoryResult {
    // Store a Small Row and a Larger Row in the Same Native Arrow Batch.
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    let backend = &backends.primary;
    backend.create_session().await?;
    let schema = schema(backend.get_history())?;
    backend.append_schemas(StorageBatch::new(vec![schema.clone()]).into()).await?;
    let mut expected = rows(&schema, 2, 2, "execution")?;
    expected[1].record.payload = Reading {
        value: 1,
        label: "large".repeat(1024),
        details: None,
        values: vec![i64::MIN, i64::MAX],
        choice: Choice::Empty,
        fraction: 1.25,
        bytes: vec![0, 255],
    }
    .encode();
    let at = backend.append_samples(StorageBatch::new(expected.clone())).await?;
    let snapshot = backend.create_snapshot(&at).await?;
    let selection = selection(&schema.key, "execution")?;

    // Return the First Page Before Refusing an Individually Oversized Row.
    let mut page = ScanPage {
        after: 0,
        rows: limits().rows,
        bytes: NonZeroUsize::new(1024).unwrap(),
    };
    let first = backend.list_samples(&snapshot, &selection, page).await?;
    assert_eq!(first, expected[..1]);
    page.after = first.last().unwrap().record.position.sequence;
    let refused = backend.list_samples(&snapshot, &selection, page).await;
    assert!(matches!(refused, Err(harmos_storage::Error::ResourceExhausted { .. })));

    // A Larger Bound Returns the Exact Remaining Native Payload.
    page.bytes = limits().bytes;
    let remaining = backend.list_samples(&snapshot, &selection, page).await?;
    assert_eq!(remaining, expected[1..]);
    backend.delete_snapshot(&snapshot).await?;
    backend.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn recovery_snapshot_and_entries_survive_reopening_the_root() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    backends.primary.create_session().await?;
    let saved = backends.primary.get_saved_position().await?;
    backends.state.create_recovery_session().await?;
    let through = RecoveryPosition {
        history: saved.history.clone(),
        sequence: 1,
    };
    let entry = RecoveryEntry {
        required_data: saved.clone(),
        position: through.clone(),
        payload: vec![1, 2, 3],
    };
    backends.state.append_recovery_entries(StorageBatch::new(vec![entry.clone()])).await?;
    let payload = SnapshotPayload::from_bytes(&[5, 6, 7, 8, 9], NonZeroUsize::new(2).unwrap(), Position(1))?;
    backends
        .state
        .set_recovery_snapshot(StorageBatch::new(vec![RecoverySnapshot {
            required_data: saved.clone(),
            through: through.clone(),
            payload,
        }]))
        .await?;
    backends.state.flush(&through).await?;
    backends.state.delete_recovery_session().await?;
    backends.primary.delete_session().await?;
    drop(backends);
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    assert_eq!(reopened.state.create_recovery_session().await?, through);
    let snapshot = reopened.state.get_recovery_snapshot(NonZeroUsize::new(2).unwrap()).await?.unwrap();
    let mut bytes = Vec::new();
    snapshot.payload.reader()?.read_to_end(&mut bytes)?;
    assert_eq!(bytes, vec![5, 6, 7, 8, 9]);
    let origin = RecoveryPosition { history: saved.history, sequence: 0 };
    let page = reopened.state.list_recovery_entries(&origin, &through, limits()).await?;
    assert_eq!(page.entries, vec![entry]);
    reopened.state.delete_recovery_session().await?;
    Ok(())
}

#[tokio::test]
async fn deleting_a_file_keeps_existing_readers_and_refuses_new_readers() -> StoryResult {
    use sha2::{Digest, Sha256};
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    let backend = &backends.primary;
    backend.create_session().await?;
    let metadata = FileMetadata {
        id: Identifier::new("artifact")?,
        scope: None,
        media_type: "application/octet-stream".into(),
        expected_length: Some(3),
        expected_sha256: None,
    };
    let staging = backend.create_file_staging(&metadata).await?;
    backend.append_file_bytes(&staging, 0, FileChunk::new(vec![1, 2, 3])).await?;
    let at = StorageCursor {
        history: backend.get_history().clone(),
        sequence: 1,
    };
    let reference = backend.publish_file_production(&staging, &at, 3, Sha256::digest([1, 2, 3]).into()).await?;
    backend.append_file_references(StorageBatch::new(vec![reference.clone()])).await?;
    let old = backend.create_snapshot(&at).await?;
    backend.delete_file(&staging).await?;
    assert_eq!(backend.get_file_bytes(&old, &reference, 0, NonZeroUsize::new(3).unwrap()).await?, vec![1, 2, 3]);
    let current = backend.create_snapshot(&at).await?;
    assert!(backend.get_file_bytes(&current, &reference, 0, NonZeroUsize::new(3).unwrap()).await.is_err());
    backend.delete_snapshot(&old).await?;
    backend.delete_snapshot(&current).await?;
    backend.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn failed_native_catalog_commit_does_not_publish_arrow_rows() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    backends.primary.create_session().await?;
    let schema = schema(backends.primary.get_history())?;
    backends.primary.append_schemas(StorageBatch::new(vec![schema.clone()]).into()).await?;
    let catalog = rusqlite::Connection::open(root.path().join("storage.sqlite3"))?;
    catalog.execute_batch(
        "CREATE TRIGGER reject_arrow BEFORE INSERT ON arrow_chunks BEGIN SELECT RAISE(ABORT,'injected commit failure'); END;",
    )?;
    let expected = rows(&schema, 2, 4, "first")?;
    assert!(backends.primary.append_samples(StorageBatch::new(expected.clone())).await.is_err());
    assert_eq!(backends.primary.get_saved_position().await?.sequence, 1);
    catalog.execute_batch("DROP TRIGGER reject_arrow")?;
    backends.primary.append_samples(StorageBatch::new(expected.clone())).await?;
    let saved = backends.primary.get_saved_position().await?;
    backends.primary.delete_session().await?;
    drop(backends);
    drop(catalog);
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    reopened.primary.create_session().await?;
    let snapshot = reopened.primary.create_snapshot(&saved).await?;
    let page = ScanPage {
        after: 0,
        rows: limits().rows,
        bytes: limits().bytes,
    };
    assert_eq!(reopened.primary.list_samples(&snapshot, &selection(&schema.key, "first")?, page).await?, expected);
    let files = std::fs::read_dir(root.path().join("storage.arrow"))?.collect::<std::io::Result<Vec<_>>>()?;
    assert_eq!(files.len(), 1);
    reopened.primary.delete_snapshot(&snapshot).await?;
    reopened.primary.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn conditional_objects_replay_the_original_applied_and_conflict_outcomes() -> StoryResult {
    use harmos_storage::{CompareExchange, ObjectEntry, ObjectExchange, ObjectKey, ObjectValue, ObjectVersionToken};
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    let backend = &backends.primary;
    backend.create_session().await?;
    let schema = schema(backend.get_history())?;
    let position = StorageCursor {
        history: backend.get_history().clone(),
        sequence: 2,
    };
    let initial = ObjectExchange {
        expected: None,
        schema: schema.clone(),
        value: ObjectEntry {
            position: position.clone(),
            version: ObjectVersionToken::new(position),
            value: ObjectValue {
                scope: None,
                schema: schema.key.clone(),
                item: ObjectKey::from_owned("object".to_owned())?,
                payload: rows(&schema, 2, 1, "first")?.remove(0).record.payload,
            },
        },
    };
    let applied = backend.set_object(&initial).await?;
    let CompareExchange::Applied(receipt) = &applied else {
        panic!("initial object creation failed");
    };
    let mut conflicting = initial.clone();
    conflicting.schema.position.sequence = 3;
    conflicting.value.position.sequence = 4;
    conflicting.value.version = ObjectVersionToken::new(conflicting.value.position.clone());
    let refused = backend.set_object(&conflicting).await?;
    assert_eq!(refused, CompareExchange::Conflict { current: Some(receipt.version.clone()) });
    let mut replacement = conflicting.clone();
    replacement.schema.position.sequence = 5;
    replacement.value.position.sequence = 6;
    replacement.value.version = ObjectVersionToken::new(replacement.value.position.clone());
    replacement.expected = Some(receipt.version.clone());
    assert!(matches!(backend.set_object(&replacement).await?, CompareExchange::Applied(_)));
    assert_eq!(backend.set_object(&initial).await?, applied);
    assert_eq!(backend.set_object(&conflicting).await?, refused);
    backend.delete_session().await?;
    drop(backends);
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    reopened.primary.create_session().await?;
    assert_eq!(reopened.primary.set_object(&initial).await?, applied);
    assert_eq!(reopened.primary.set_object(&conflicting).await?, refused);
    let mut changed = initial.clone();
    changed.value.value.payload.push(0);
    assert!(reopened.primary.set_object(&changed).await.is_err());
    reopened.primary.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn native_indexes_and_shared_projections_retain_published_results_after_restart() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    let backend = &backends.primary;
    backend.create_session().await?;
    let schema = schema(backend.get_history())?;
    backend.append_schemas(StorageBatch::new(vec![schema.clone()]).into()).await?;
    backend.append_samples(StorageBatch::new(rows(&schema, 2, 4, "first")?)).await?;
    let at = backend.get_saved_position().await?;
    let snapshot = backend.create_snapshot(&at).await?;
    let source = ColumnSource {
        kind: ColumnKind::Stream,
        schema: schema.key.clone(),
    };
    let index = IndexDefinition {
        name: Identifier::new("values")?,
        version: 1,
        source: source.clone(),
        columns: vec!["payload.value".into()],
    };
    let cancel = Cancellation::default();
    backend.set_index_definition(&index, None, false, limits(), &cancel, None).await?;
    let index_read = IndexRead {
        name: index.name.clone(),
        revision: Some(1),
        lower: None,
        upper: None,
        prefix: Vec::new(),
        descending: false,
        after: None,
        limits: limits(),
        work: limits(),
    };
    let page = backend.list_index_entries(&snapshot, &index_read, &cancel).await?;
    assert_eq!(page.entries.iter().map(|entry| entry.position).collect::<Vec<_>>(), vec![2, 3, 4, 5]);
    let projection = ProjectionDefinition {
        name: Identifier::new("selected")?,
        version: 1,
        source,
        transform: ProjectionTransform::Select(vec!["payload.value".into()]),
    };
    backend.set_projection_definition(&projection, None, false, None).await?;
    advance_projection(backend.as_ref(), &snapshot, &projection.name, Some(1), limits(), &cancel).await?;
    let read = ProjectionRead {
        name: projection.name.clone(),
        revision: 1,
        frontier: at.clone(),
        after: 0,
        limits: limits(),
        work: limits(),
    };
    let page = backend.list_projection_rows(&read).await?;
    assert_eq!(page.batch.column(0).as_any().downcast_ref::<UInt64Array>().unwrap().values().as_ref(), &[0, 1, 2, 3]);
    backend.delete_snapshot(&snapshot).await?;
    backend.delete_session().await?;
    drop(backends);
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    reopened.primary.create_session().await?;
    assert_eq!(reopened.primary.get_index_definition(&index.name).await?.0, index);
    let snapshot = reopened.primary.create_snapshot(&at).await?;
    let page = reopened.primary.list_index_entries(&snapshot, &index_read, &cancel).await?;
    assert_eq!(page.entries.iter().map(|entry| entry.position).collect::<Vec<_>>(), vec![2, 3, 4, 5]);
    reopened.primary.delete_snapshot(&snapshot).await?;
    assert_eq!(reopened.primary.get_projection_definition(&projection.name, Some(1)).await?.0, projection);
    assert_eq!(reopened.primary.list_projection_rows(&read).await?.batch.num_rows(), 4);
    reopened.primary.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn recovery_reopens_at_its_saved_frontier_and_replaces_only_the_unflushed_tail() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    backends.primary.create_session().await?;
    backends.state.create_recovery_session().await?;
    let data = backends.primary.get_saved_position().await?;
    let mut entry = RecoveryEntry {
        required_data: data.clone(),
        position: RecoveryPosition {
            history: data.history.clone(),
            sequence: 1,
        },
        payload: vec![1],
    };
    backends.state.append_recovery_entries(StorageBatch::new(vec![entry.clone()])).await?;
    backends.state.flush(&entry.position).await?;
    let saved = entry.position.clone();
    entry.position.sequence = 2;
    entry.payload = vec![2];
    backends.state.append_recovery_entries(StorageBatch::new(vec![entry.clone()])).await?;
    backends.state.delete_recovery_session().await?;
    backends.primary.delete_session().await?;
    drop(backends);
    let (reopened, _reopened_lease) = Backend::open_with_execution(&store, ObjectCatalog::new(), Execution::default())?;
    reopened.primary.create_session().await?;
    assert_eq!(reopened.state.create_recovery_session().await?, saved);
    entry.payload = vec![3];
    reopened.state.append_recovery_entries(StorageBatch::new(vec![entry.clone()])).await?;
    reopened.state.flush(&entry.position).await?;
    let page = reopened.state.list_recovery_entries(&saved, &entry.position, limits()).await?;
    assert_eq!(page.entries, vec![entry]);
    reopened.state.delete_recovery_session().await?;
    reopened.primary.delete_session().await?;
    Ok(())
}
