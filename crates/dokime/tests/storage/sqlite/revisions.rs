/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "retention.rs"]
pub(crate) mod retention;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{sync::Arc, time::Duration};

// Third Party Imports
use harmos::catalog::Projection;
use harmos::{Application, DefinitionId, Identified, Runtime, state::Never, storage::*};

// Local Imports
use super::storage::write_batch;
use super::storage::{history, id, limits, position, view};
use dokime::storage::sqlite::{Backend as NativeBackend, Limits as NativeLimits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Bench;
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Charger {
    id: String,
    volts: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
struct RevisedCharger {
    id: String,
    volts: u32,
    enabled: bool,
}

/* === Enums === */
#[harmos::catalog]
enum Objects {
    Charger(Charger),
    RevisedCharger(RevisedCharger),
    RetainedCharger(retention::RetainedCharger),
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

impl Identified for Charger {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("bench.charger"), 1)
    }
}

impl Identified for RevisedCharger {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("bench.charger"), 2)
    }
}

impl StoredObject for Charger {
    type Key = String;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;

    fn key(&self) -> &Self::Key {
        &self.id
    }
}

impl StoredObject for RevisedCharger {
    type Key = String;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;

    fn key(&self) -> &Self::Key {
        &self.id
    }
}

/* === Public Functions === */

/* === Private Functions === */
// The Object Catalog Occupies The First Three Saved Positions.
fn object_position(sequence: u64) -> StorageCursor {
    position(sequence + 3)
}

async fn object_view<C: ObjectCatalog>(backend: &NativeBackend<C>, sequence: u64) -> ReadView {
    view(backend, sequence + 3).await
}

async fn admit_object_schemas(backend: &NativeBackend<Objects>) {
    let mut schemas = Vec::new();
    Objects::definitions()
        .try_for_each(|definition| {
            schemas.push(StorageSchema::new(
                position(schemas.len() as u64 + 1),
                definition.schema()?,
                definition.graph(&harmos::config::SchemaSettings::default())?,
            ));
            Ok::<(), harmos::Error>(())
        })
        .unwrap();
    assert_eq!(schemas.len(), 3);
    backend
        .write(write_batch(&[StoragePart::Schemas(&schemas)]))
        .await
        .unwrap();
}

fn entry(sequence: u64, volts: u32) -> ObjectEntry {
    ObjectEntry {
        position: object_position(sequence),
        version: version(sequence),
        value: Objects::encode(
            &Charger {
                id: "charger:0123456789".into(),
                volts,
            },
            limits(1).bytes,
        )
        .unwrap(),
    }
}

// Backend fixtures restore tokens through the same persisted wire shape used
// on replication. Production admissions obtain them from StorageManager.
fn version(sequence: u64) -> ObjectVersionToken {
    serde_json::from_value(serde_json::json!({
        "history": history(),
        "revision": sequence + 3,
    }))
    .unwrap()
}

fn object_key(value: &str) -> ObjectKey {
    ObjectKey::new(&value, limits(1).bytes).unwrap()
}

fn query(version: ObjectVersion) -> ObjectPoint {
    ObjectPoint {
        schema: id("bench.charger"),
        item: object_key("charger:0123456789"),
        version,
    }
}

/* === Tests === */

pub(crate) async fn object_revisions_latest() {
    let root = tempfile::tempdir().unwrap();
    let backend = NativeBackend::<Objects>::replica(
        root.path().to_path_buf(),
        NativeLimits::default(),
        history(),
    )
    .await
    .unwrap();
    backend.open().await.unwrap();
    admit_object_schemas(&backend).await;
    let first = entry(1, 230);
    backend
        .write(write_batch(&[StoragePart::Objects(std::slice::from_ref(
            &first,
        ))]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    let old = object_view(&backend, 1).await;

    let unrelated = ObjectEntry {
        position: object_position(2),
        version: version(2),
        value: Objects::encode(
            &Charger {
                id: "obc-2".into(),
                volts: 220,
            },
            limits(1).bytes,
        )
        .unwrap(),
    };
    backend
        .write(write_batch(&[StoragePart::Objects(&[unrelated])]))
        .await
        .unwrap();
    let second = ObjectEntry {
        position: object_position(3),
        version: version(3),
        value: Objects::encode(
            &RevisedCharger {
                id: "charger:0123456789".into(),
                volts: 240,
                enabled: true,
            },
            limits(1).bytes,
        )
        .unwrap(),
    };
    backend
        .write(write_batch(&[StoragePart::Objects(std::slice::from_ref(
            &second,
        ))]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    let current = object_view(&backend, 3).await;
    assert_eq!(
        backend
            .read_object(&old, &query(ObjectVersion::Latest), limits(1).bytes)
            .await
            .unwrap(),
        Some(first.clone())
    );
    assert_eq!(
        backend
            .read_object(&current, &query(ObjectVersion::Latest), limits(1).bytes)
            .await
            .unwrap(),
        Some(second.clone())
    );
    assert_eq!(
        backend
            .read_object(
                &current,
                &query(ObjectVersion::Exact(version(1))),
                limits(1).bytes
            )
            .await
            .unwrap(),
        Some(first.clone())
    );
    assert!(
        backend
            .read_object(
                &current,
                &query(ObjectVersion::Exact(version(2))),
                limits(1).bytes,
            )
            .await
            .unwrap()
            .is_none()
    );
    let missing = ObjectPoint {
        item: object_key("absent"),
        ..query(ObjectVersion::Exact(version(1)))
    };
    assert!(
        backend
            .read_object(&current, &missing, limits(1).bytes)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        backend
            .read_object(
                &old,
                &query(ObjectVersion::Exact(second.version.clone())),
                limits(1).bytes
            )
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        backend
            .read_object(
                &current,
                &query(ObjectVersion::Latest),
                1.try_into().unwrap()
            )
            .await
            .is_err()
    );

    let mut selection = ReadSelection::new(ReadKind::Objects);
    selection.object_key = Some(object_key("charger:0123456789"));
    let page = backend
        .read_objects(&current, &selection, None, limits(1))
        .await
        .unwrap();
    assert_eq!(page.entries, [first]);
    let page = backend
        .read_objects(&current, &selection, page.next.as_ref(), limits(1))
        .await
        .unwrap();
    assert_eq!(page.entries.as_slice(), std::slice::from_ref(&second));
    assert!(page.next.is_none());

    selection.object_key = Some(object_key("charger:absent/é' OR true --"));
    let missing = backend
        .read_objects(&current, &selection, None, limits(1))
        .await
        .unwrap();
    assert!(missing.entries.is_empty());
    assert!(missing.next.is_none());

    backend.release_view(old.snapshot()).await.unwrap();
    backend.release_view(current.snapshot()).await.unwrap();
    backend.close().await.unwrap();

    // Retained field descriptors, not a loaded producer catalog, restore saved values.
    let reader = NativeBackend::<()>::replica(
        root.path().to_path_buf(),
        NativeLimits::default(),
        history(),
    )
    .await
    .unwrap();
    assert_eq!(reader.open().await.unwrap(), object_position(3));
    let saved = object_view(&reader, 3).await;
    let restored = reader
        .read_object(&saved, &query(ObjectVersion::Latest), limits(1).bytes)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(restored, second);
    assert_eq!(
        <RevisedCharger as harmos::Message>::decode(restored.value.payload.as_slice())
            .unwrap()
            .volts,
        240
    );
    reader.release_view(saved.snapshot()).await.unwrap();
    reader.close().await.unwrap();
}

pub(crate) async fn object_batches_identity() {
    let root = tempfile::tempdir().unwrap();
    let backend = NativeBackend::<Objects>::replica(
        root.path().to_path_buf(),
        NativeLimits::default(),
        history(),
    )
    .await
    .unwrap();
    backend.open().await.unwrap();
    admit_object_schemas(&backend).await;
    let updates = vec![entry(1, 230), entry(2, 240), entry(3, 250)];
    backend
        .write(write_batch(&[StoragePart::Objects(&updates)]))
        .await
        .unwrap();
    backend.cursor_saved().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Objects(&updates)]))
        .await
        .unwrap();
    assert!(
        backend
            .write(write_batch(&[StoragePart::Objects(&[entry(2, 260)])]))
            .await
            .is_err()
    );
    let mut invalid = entry(2, 240);
    invalid.value.item = object_key("wrong-id");
    assert!(
        backend
            .write(write_batch(&[StoragePart::Objects(&[invalid])]))
            .await
            .is_err()
    );
    let saved = object_view(&backend, 3).await;
    let mut selection = ReadSelection::new(ReadKind::Objects);
    selection.order = ReadOrder::Descending;
    let page = backend
        .read_objects(&saved, &selection, None, limits(4))
        .await
        .unwrap();
    assert_eq!(page.entries, updates.into_iter().rev().collect::<Vec<_>>());
    assert!(page.next.is_none());
    assert!(
        backend
            .read_records(&saved, &selection, None, limits(1))
            .await
            .is_err()
    );
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn manager_replicates_collections() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        NativeBackend::<Objects>::replica(
            root.path().join("primary"),
            NativeLimits::default(),
            history(),
        )
        .await
        .unwrap(),
    );
    let replica = Arc::new(
        NativeBackend::<Objects>::replica(
            root.path().join("replica"),
            NativeLimits::default(),
            history(),
        )
        .await
        .unwrap(),
    );
    let recovery = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        primary.history().clone(),
        &crate::sqlite::storage::storage_settings(),
    ));
    let storage = StorageManager::new(
        StorageBackends {
            primary,
            state: recovery,
            replicas: vec![ReplicaTarget {
                id: id("secondary"),
                backend: replica.clone(),
            }],
        },
        StorageSettings {
            flush_interval: Duration::from_secs(60),
            replication_shutdown: Duration::from_secs(5),
            ..crate::sqlite::storage::storage_settings()
        },
    )
    .unwrap();
    let runtime = Runtime::<Bench>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    storage
        .write_schema(&super::storage::key(), &super::storage::schema().schema)
        .await
        .unwrap();
    let first = storage
        .write_object(&Charger {
            id: "charger:0123456789".into(),
            volts: 230,
        })
        .await
        .unwrap();
    let key = super::storage::key();
    let record_id = id("measurement");
    storage
        .write_records(&[RecordWrite {
            schema: &key,
            scope: None,
            item: &record_id,
            source: None,
            source_time: None,
            payload: &[8, 42],
        }])
        .await
        .unwrap();
    storage
        .write_stream(&[StreamWrite {
            stream: &id("voltage"),
            sequence: 1,
            record: RecordWrite {
                schema: &key,
                scope: None,
                item: &id("sample"),
                source: None,
                source_time: None,
                payload: &[8, 43],
            },
        }])
        .await
        .unwrap();
    storage
        .write_file(
            FileMetadata {
                id: id("trace"),
                scope: None,
                media_type: "text/plain".into(),
                expected_length: Some(2),
                expected_sha256: None,
            },
            b"ok",
        )
        .await
        .unwrap();
    let last = storage
        .write_object(&Charger {
            id: "charger:0123456789".into(),
            volts: 240,
        })
        .await
        .unwrap();
    assert_eq!((first.position.sequence, last.position.sequence), (3, 8));
    storage.flush().await.unwrap();
    let latest = storage
        .read_object::<Charger>(
            &"charger:0123456789".to_owned(),
            ObjectVersion::Latest,
            limits(1).bytes,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.position, last.position);
    assert_eq!(latest.version, last.version);
    assert_eq!(latest.value.volts, 240);

    let restored_token =
        serde_json::from_slice(&serde_json::to_vec(&last.version).unwrap()).unwrap();
    let exact = storage
        .read_object::<Charger>(
            &"charger:0123456789".to_owned(),
            ObjectVersion::Exact(restored_token),
            limits(1).bytes,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(exact.position, last.position);
    assert_eq!(exact.version, last.version);
    assert_eq!(exact.value.volts, 240);
    runtime.stop().await.unwrap();

    assert_eq!(replica.open().await.unwrap(), position(8));
    let saved = ReadView::new(
        id("replica.saved-six"),
        position(8),
        position(8),
        replica.read_view(&position(8)).await.unwrap(),
    )
    .unwrap();
    let replicated = replica
        .read_object(
            &saved,
            &query(ObjectVersion::Exact(first.version.clone())),
            limits(1).bytes,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(replicated.version, first.version);
    assert_eq!(replicated.position, first.position);
    assert_eq!(
        Objects::decode::<Charger>(&replicated.value, limits(1).bytes)
            .unwrap()
            .volts,
        230
    );

    let objects = replica
        .read_objects(
            &saved,
            &ReadSelection::new(ReadKind::Objects),
            None,
            limits(8),
        )
        .await
        .unwrap();
    let mut expected_first = entry(0, 230);
    expected_first.position = first.position.clone();
    expected_first.version = first.version.clone();
    let mut expected_last = entry(0, 240);
    expected_last.position = last.position.clone();
    expected_last.version = last.version.clone();
    assert_eq!(objects.entries, [expected_first, expected_last]);
    let mut positions = objects
        .entries
        .iter()
        .map(|entry| entry.position.sequence)
        .collect::<Vec<_>>();
    let mut cursor = None;
    loop {
        let page = replica
            .read_schemas(
                &saved,
                &ReadSelection::new(ReadKind::Schemas),
                cursor.as_ref(),
                limits(8),
            )
            .await
            .unwrap();
        positions.extend(page.entries.iter().map(|entry| entry.position.sequence));
        cursor = page.next;
        if cursor.is_none() {
            break;
        }
    }
    positions.extend(
        replica
            .read_records(
                &saved,
                &ReadSelection::new(ReadKind::Records),
                None,
                limits(8),
            )
            .await
            .unwrap()
            .entries
            .iter()
            .map(|entry| entry.position.sequence),
    );
    positions.extend(
        replica
            .read_stream(
                &saved,
                &ReadSelection::new(ReadKind::Stream),
                None,
                limits(8),
            )
            .await
            .unwrap()
            .entries
            .iter()
            .map(|entry| entry.record.position.sequence),
    );
    let files = replica
        .read_files(
            &saved,
            &ReadSelection::new(ReadKind::Files),
            None,
            limits(8),
        )
        .await
        .unwrap();
    assert_eq!(
        replica
            .read_file(&saved, &files.entries[0], 0, limits(1).bytes)
            .await
            .unwrap(),
        b"ok"
    );
    positions.extend(files.entries.iter().map(|entry| entry.position.sequence));
    positions.sort_unstable();
    assert_eq!(positions, [1, 2, 3, 4, 5, 6, 7, 8]);
    replica.release_view(saved.snapshot()).await.unwrap();
    replica.close().await.unwrap();
}
