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

/* === Re-Exports === */

/* === Imports === */

use harmos_domain::Identified;
use harmos_storage::{ObjectCatalog, ObjectDefinition, ObjectRetention, StoredObject};

// Standard Library Imports

// Third Party Imports

// Local Imports
use arrow_array::UInt64Array;
use dokime_storage::{Backend, Store};
use harmos_domain::Message;
use harmos_domain::config::SchemaSettings;
use harmos_domain::journal::Position;
use harmos_domain::schema::{Described, graph::Schema};
use harmos_query::columnar::{ArrowScan, Cancellation, ColumnKind, ColumnSource};
use harmos_storage::query::StreamSelection;
use harmos_storage::state::{RecoveryEntry, RecoverySnapshot, SnapshotPayload};
use harmos_storage::{
    FileChunk, FileMetadata, Identifier, PayloadEncoding, ReadLimits, RecoveryPosition, ScanPage, SchemaKey, StorageBatch, StorageCursor,
    StorageSchema, StoreHistory, StoredRecord, StoredRow,
};
use std::io::Read;
use std::num::{NonZeroU32, NonZeroUsize};
use std::sync::Arc;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type StoryResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

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

/* === Public Functions === */

#[tokio::test]
async fn catalog_objects_can_be_scanned_before_their_first_write_and_after_reopen() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let definition = ObjectDefinition::of::<Reading>();
    let catalog = ObjectCatalog::new().entry(definition);
    let (backends, lease) = Backend::open(&store, catalog.clone())?;
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
    backend.delete_snapshot(&snapshot).await?;
    drop(lease);
    assert!(backend.get_saved_position().await.is_err());

    let (reopened, _lease) = Backend::open(&store, catalog)?;
    reopened.primary.create_session().await?;
    let snapshot = reopened.primary.create_snapshot(&at).await?;
    let page = reopened.primary.list_column_rows(&snapshot, &scan, &cancel).await?;
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
#[tokio::test]
async fn pinned_native_arrow_pages_keep_their_saved_cut() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (reopened, _reopened_lease) = Backend::open(&store, ObjectCatalog::new())?;
    assert_eq!(reopened.primary.get_history(), &history);
    reopened.primary.create_session().await?;
    let snapshot = reopened.primary.create_snapshot(&saved).await?;
    assert_eq!(reopened.primary.list_samples(&snapshot, &selection(&schema.key, "first")?, page).await?, first);
    reopened.primary.delete_snapshot(&snapshot).await?;
    reopened.primary.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn recovery_snapshot_and_entries_survive_reopening_the_root() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (reopened, _reopened_lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (backends, _lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (backends, _lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (reopened, _reopened_lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (backends, _lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (reopened, _reopened_lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    use harmos_query::index::{IndexDefinition, IndexRead};
    use harmos_storage::query::projection::{ProjectionDefinition, ProjectionRead, ProjectionTransform, advance_projection};
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let read = IndexRead {
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
    let page = backend.list_index_entries(&snapshot, &read, &cancel).await?;
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
    let (reopened, _reopened_lease) = Backend::open(&store, ObjectCatalog::new())?;
    reopened.primary.create_session().await?;
    assert_eq!(reopened.primary.get_index_definition(&index.name).await?.0, index);
    assert_eq!(reopened.primary.get_projection_definition(&projection.name, Some(1)).await?.0, projection);
    assert_eq!(reopened.primary.list_projection_rows(&read).await?.batch.num_rows(), 4);
    reopened.primary.delete_session().await?;
    Ok(())
}

#[tokio::test]
async fn recovery_reopens_at_its_saved_frontier_and_replaces_only_the_unflushed_tail() -> StoryResult {
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    let (backends, _lease) = Backend::open(&store, ObjectCatalog::new())?;
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
    let (reopened, _reopened_lease) = Backend::open(&store, ObjectCatalog::new())?;
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
