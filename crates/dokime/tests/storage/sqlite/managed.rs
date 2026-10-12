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
use std::{
    num::{NonZeroU64, NonZeroUsize},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use harmos::{
    Application, Never, Runtime, state::SnapshotPolicy, storage::recovery::RecoveryBackend,
    storage::*,
};
use prost::Message;

// Local Imports
use super::storage::{history, id, key, limits, read_all, record, schema, view};
use dokime::storage::sqlite::{Backend, Limits as BackendLimits, RecoveryLimits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Bench;
struct Charger;
struct Capture;

#[harmos::record(id = "charger.selected", version = 1)]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Selected {
    volts: u32,
}

#[derive(Clone, PartialEq, Message)]
struct Measurement {
    #[prost(uint64, tag = "1")]
    counter: u64,
}

#[harmos::transaction(id = "charger.setpoint", version = 1, irreversible)]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Setpoint {
    volts: u32,
}

/* === Enums === */
#[harmos::transactions(state = u32, error = String)]
#[derive(Clone)]
enum Transactions {
    Setpoint(Setpoint),
}

#[harmos::records]
#[derive(Clone)]
enum Records {
    Selected(Selected),
}

/* === Traits === */

/* === Implementations === */
impl Application for Charger {
    type State = u32;
    type Transaction = Transactions;
    type Record = Records;
    type Streams = ();
    type Resources = ();
}

impl harmos::Transaction<u32> for Setpoint {
    type Error = String;
    fn check(&self, _: &u32) -> Result<(), String> {
        Ok(())
    }
    fn apply(&self, state: &mut u32) {
        *state = self.volts;
    }

    fn records(&self, _: &u32, records: &mut harmos::Records) {
        records.push(Selected { volts: self.volts }).unwrap();
    }
}

impl RecordProjection for Capture {
    fn project<'a>(
        &'a self,
        entry: &'a harmos::Envelope<&serde_json::Value>,
        output: &'a mut ProjectedRecords<'_>,
    ) -> harmos::storage::StorageFuture<'a, ()> {
        Box::pin(async move {
            if entry.id.as_str() != "charger.selected" {
                return Ok(());
            }

            let selected: Selected = serde_json::from_value(entry.payload.clone()).unwrap();
            let payload = Measurement {
                counter: u64::from(selected.volts),
            }
            .encode_to_vec();
            output
                .push(RecordWrite {
                    schema: &key(),
                    scope: Some(&id("execution")),
                    item: &id(&format!("measurement-{}", entry.position.0)),
                    source: Some(&id("charger")),
                    source_time: None,
                    payload: &payload,
                })
                .await
        })
    }
}

impl Application for Bench {
    type State = ();
    type Transaction = Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

/* === Public Functions === */

/* === Private Functions === */
fn record_write(record: &StoredRecord) -> RecordWrite<'_> {
    RecordWrite {
        schema: &record.schema,
        scope: record.scope.as_ref(),
        item: &record.item,
        source: record.source.as_ref(),
        source_time: record.source_time,
        payload: &record.payload,
    }
}

fn policy() -> StorageSettings {
    StorageSettings {
        flush_interval: Duration::from_secs(60),
        max_retries: 0,
        query_page: limits(1),
        ..StorageSettings::default()
    }
}

fn records_query() -> RecordQuery<Vec<u8>> {
    RecordQuery::new(key(), |bytes, _| Ok(bytes.to_vec()))
}

fn snapshot_policy() -> SnapshotPolicy {
    SnapshotPolicy {
        schema: SchemaKey {
            payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
            id: id("charger.state"),
            version: 1.try_into().unwrap(),
        },
        every: NonZeroU64::new(2).unwrap(),
        max_bytes: NonZeroUsize::new(64 * 1024).unwrap(),
    }
}

pub(crate) async fn compacted_snapshots_receipts() {
    let root = tempfile::tempdir().unwrap();
    let request = harmos::RequestKey::new("charger.power-up");
    let mut first_receipt = None;
    for pass in 0..3 {
        let data = Arc::new(
            Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
                .await
                .unwrap(),
        );
        let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
        let storage = StorageManager::new(
            StorageBackends {
                primary: data.clone(),
                recovery: recovery.clone(),
                replicas: vec![],
            },
            policy(),
        )
        .unwrap();
        let runtime = Runtime::<Charger>::builder(0, ())
            .register(Transactions::catalog())
            .register(Records::catalog())
            .managed_state(limits(1))
            .managed_snapshots(snapshot_policy())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let before = runtime.state.state().read(|state| *state).await.unwrap();
        assert_eq!(before.value, if pass == 0 { 0 } else { 229 + pass });
        let receipt = runtime
            .state
            .commit_keyed(request.clone(), Setpoint { volts: 230 })
            .await
            .unwrap();
        if let Some(first) = &first_receipt {
            assert_eq!(&receipt, first);
            let unchanged = runtime.state.state().read(|state| *state).await.unwrap();
            assert_eq!(unchanged.value, before.value);
            assert_eq!(unchanged.position, before.position);
        } else {
            first_receipt = Some(receipt);
        }
        runtime
            .state
            .commit(Setpoint { volts: 230 + pass })
            .await
            .unwrap();
        let applied = runtime.state.applied();
        runtime.stop().await.unwrap();

        // The physical backend compacts journal entries through the snapshot.
        // Reboot must recover their state and deduplication index from that snapshot.
        data.open().await.unwrap();
        let saved = recovery.open().await.unwrap();
        let snapshot = recovery
            .read_snapshot(NonZeroUsize::new(64 * 1024).unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(snapshot.through, saved);
        assert!(saved.sequence > 0);
        assert_eq!(applied.0, (2 + u64::from(pass)) * 2);
        let origin = RecoveryPosition {
            history: saved.history.clone(),
            sequence: 0,
        };
        assert!(matches!(
            recovery.read_entries(&origin, &saved, limits(1)).await,
            Err(harmos::Error::Gap { .. })
        ));
        recovery.close().await.unwrap();
        data.close().await.unwrap();
    }
}

pub(crate) async fn primary_history_close() {
    let root = tempfile::tempdir().unwrap();
    let first = Backend::<()>::primary(root.path().to_path_buf(), BackendLimits::default())
        .await
        .unwrap();
    let history = first.history().clone();
    assert_eq!(first.open().await.unwrap().history, history);
    assert!(
        Backend::<()>::primary(root.path().to_path_buf(), BackendLimits::default())
            .await
            .is_err()
    );
    first.close().await.unwrap();
    let reopened = Backend::<()>::primary(root.path().to_path_buf(), BackendLimits::default())
        .await
        .unwrap();
    assert_eq!(reopened.history(), &history);
    reopened.close().await.unwrap();
    let other_root = tempfile::tempdir().unwrap();
    let other = Backend::<()>::primary(other_root.path().to_path_buf(), BackendLimits::default())
        .await
        .unwrap();
    assert_ne!(other.history(), &history);
    other.close().await.unwrap();
}

pub(crate) fn cancelled_backend_instance() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .unwrap()
        .block_on(async {
            let root = tempfile::tempdir().unwrap();
            let data = Backend::<()>::replica(
                root.path().join("data"),
                BackendLimits::default(),
                history(),
            )
            .await
            .unwrap();
            data.close().await.unwrap();
            // Hold The Only Physical Worker Through Cancelled Reopening
            let (release, blocked) = std::sync::mpsc::channel::<()>();
            let (started, ready) = tokio::sync::oneshot::channel();
            let worker = tokio::task::spawn_blocking(move || {
                let _ = started.send(());
                let _ = blocked.recv();
            });
            ready.await.unwrap();
            let mut opening = data.open();
            assert!(futures_util::poll!(opening.as_mut()).is_pending());
            drop(opening);
            release.send(()).unwrap();
            worker.await.unwrap();
            assert_eq!(data.open().await.unwrap().sequence, 0);
            assert!(
                Backend::<()>::primary(root.path().join("data"), BackendLimits::default(),)
                    .await
                    .is_err()
            );

            let recovery = data.recovery(RecoveryLimits::default());
            let (release, blocked) = std::sync::mpsc::channel::<()>();
            let (started, ready) = tokio::sync::oneshot::channel();
            let worker = tokio::task::spawn_blocking(move || {
                let _ = started.send(());
                let _ = blocked.recv();
            });
            ready.await.unwrap();
            let mut opening = recovery.open();
            assert!(futures_util::poll!(opening.as_mut()).is_pending());
            drop(opening);
            release.send(()).unwrap();
            worker.await.unwrap();
            assert_eq!(recovery.open().await.unwrap().sequence, 0);
            assert!(
                Backend::<()>::primary(root.path().join("data"), BackendLimits::default(),)
                    .await
                    .is_err()
            );
            data.close().await.unwrap();
            recovery.close().await.unwrap();
        });
}

pub(crate) async fn manager_queries_evidence() {
    let root = tempfile::tempdir().unwrap();
    let data = Arc::new(
        Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
            .await
            .unwrap(),
    );
    let history = data.history().clone();
    let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
    let storage = StorageManager::new(
        StorageBackends {
            primary: data.clone(),
            recovery: recovery.clone(),
            replicas: vec![],
        },
        policy(),
    )
    .unwrap();
    let runtime = Runtime::<Bench>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    let handle = runtime.handle();
    assert!(handle.storage().is_some());
    let key = key();
    storage.write_schema(&key, &schema().fields).await.unwrap();
    let first = id("first");
    let scope = id("execution");
    let boundary = storage
        .write_records(&[RecordWrite {
            schema: &key,
            scope: Some(&scope),
            item: &first,
            source: None,
            source_time: None,
            payload: &[8, 42],
        }])
        .await
        .unwrap();
    let second = id("second");
    let stream = id("voltage");
    storage
        .write_stream(&[StreamWrite {
            stream: &stream,
            sequence: u64::MAX,
            record: RecordWrite {
                schema: &key,
                scope: Some(&scope),
                item: &second,
                source: None,
                source_time: None,
                payload: &[8, 43],
            },
        }])
        .await
        .unwrap();
    assert_eq!(storage.cursor_accepted().sequence, 3);
    assert_eq!(storage.flush_until(&boundary).await.unwrap(), boundary);
    assert_eq!(storage.cursor_saved().sequence, 2);
    let captured = storage
        .view_with_limits(policy().query_execution)
        .await
        .unwrap();
    assert_eq!(captured.saved().unwrap().sequence, 2);
    assert_eq!(captured.accepted().unwrap().sequence, 3);
    let mut records = storage.query_in(&captured, records_query()).unwrap();
    assert_eq!(
        records.next_page().await.unwrap().unwrap()[0].payload,
        vec![8, 42]
    );
    assert!(records.next_page().await.unwrap().is_none());
    let mut rows = storage
        .query_in(
            &captured,
            StreamQuery::new(key.clone(), |bytes, _| Ok(bytes.to_vec())),
        )
        .unwrap();
    assert!(rows.next_page().await.unwrap().is_none());
    storage.flush().await.unwrap();
    let mut same_view = storage
        .query_in(
            &captured,
            StreamQuery::new(key.clone(), |bytes, _| Ok(bytes.to_vec())),
        )
        .unwrap();
    assert!(same_view.next_page().await.unwrap().is_none());
    let mut refreshed = storage
        .query(StreamQuery::new(key.clone(), |bytes, _| Ok(bytes.to_vec())))
        .await
        .unwrap();
    let row = refreshed.next_page().await.unwrap().unwrap();
    assert_eq!(row[0].sequence, u64::MAX);
    assert_eq!(row[0].record.payload, vec![8, 43]);
    assert!(refreshed.next_page().await.unwrap().is_none());
    let position = storage.write_recovery(&[b"working-state"]).await.unwrap();
    assert_eq!(storage.status().recovery_saved.sequence, 0);
    let bytes = b"charger trace";
    let file = storage
        .write_file(
            FileMetadata {
                id: id("trace"),
                scope: Some(scope),
                media_type: "application/octet-stream".into(),
                expected_length: Some(bytes.len() as u64),
                expected_sha256: None,
            },
            bytes,
        )
        .await
        .unwrap();
    let mut reader = storage.read_file(&file).await.unwrap();
    let mut actual = Vec::new();
    while let Some(chunk) = reader
        .next_chunk(NonZeroUsize::new(3).unwrap())
        .await
        .unwrap()
    {
        actual.extend_from_slice(chunk.as_bytes());
    }
    assert_eq!(actual, bytes);
    drop(reader);
    drop(captured);

    // Final evidence is admitted during participant shutdown, while the runtime
    // keeps its storage workers alive. Stop owns both durability barriers.
    let mut participant = runtime.stop_participant();
    let final_storage = handle.storage().unwrap().clone();
    let final_key = key.clone();
    let final_write = tokio::spawn(async move {
        participant.stopping().await;
        let item = id("shutdown");
        final_storage
            .write_records(&[RecordWrite {
                schema: &final_key,
                scope: None,
                item: &item,
                source: None,
                source_time: None,
                payload: &[8, 44],
            }])
            .await
            .unwrap();
        participant.finish();
    });
    runtime.stop().await.unwrap();
    final_write.await.unwrap();
    assert_eq!(storage.status().recovery_saved, position);
    assert!(
        storage
            .view_with_limits(policy().query_execution)
            .await
            .is_err()
    );
    runtime.stop().await.unwrap();
    drop(storage);
    drop(data);
    drop(recovery);

    let data = Arc::new(
        Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
            .await
            .unwrap(),
    );
    assert_eq!(data.history(), &history);
    let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
    let storage = StorageManager::new(
        StorageBackends {
            primary: data,
            recovery,
            replicas: vec![],
        },
        policy(),
    )
    .unwrap();
    let runtime = Runtime::<Bench>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    assert_eq!(storage.status().saved.sequence, 5);
    assert_eq!(storage.status().recovery_saved, position);
    let mut saved_records = storage
        .query({
            let mut query = records_query();
            query.options.page = Some(limits(2));
            query
        })
        .await
        .unwrap();
    let records = saved_records.next_page().await.unwrap().unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].item, id("shutdown"));
    assert_eq!(records[1].payload, vec![8, 44]);
    assert!(saved_records.next_page().await.unwrap().is_none());
    drop(saved_records);
    let after = RecoveryPosition {
        history: position.history.clone(),
        sequence: 0,
    };
    let recovered = storage
        .read_recovery(&after, &position, limits(1))
        .await
        .unwrap();
    assert_eq!(recovered.entries[0].payload, b"working-state");
    assert!(recovered.next.is_none());
    let mut schemas = storage.query(SchemaQuery::new()).await.unwrap();
    assert_eq!(
        schemas.next_page().await.unwrap().unwrap()[0].fields,
        schema().fields
    );
    let mut files = storage.query(FileQuery::new()).await.unwrap();
    assert_eq!(&*files.next_page().await.unwrap().unwrap(), &[file]);
    runtime.stop().await.unwrap();
}

pub(crate) async fn managed_journal_backends() {
    let root = tempfile::tempdir().unwrap();
    for pass in 0..2 {
        let data = Arc::new(
            Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
                .await
                .unwrap(),
        );
        let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
        let storage = StorageManager::new(
            StorageBackends {
                primary: data,
                recovery,
                replicas: vec![],
            },
            policy(),
        )
        .unwrap();
        let runtime = Runtime::<Charger>::builder(0, ())
            .register(Transactions::catalog())
            .register(Records::catalog())
            .managed_state(limits(1))
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let restored = runtime.state.state().read(|state| *state).await.unwrap();
        if pass == 0 {
            assert_eq!(restored.value, 0);
            for volts in [200, 300, 400] {
                runtime.state.commit(Setpoint { volts }).await.unwrap();
            }
            assert_eq!(
                runtime
                    .state
                    .state()
                    .read(|state| *state)
                    .await
                    .unwrap()
                    .value,
                400
            );
        } else {
            assert_eq!(restored.value, 400);
            assert_eq!(restored.position, harmos::Position(6));
            runtime.state.commit(Setpoint { volts: 450 }).await.unwrap();
        }
        runtime.stop().await.unwrap();
        assert_eq!(
            storage.status().recovery_saved.sequence,
            if pass == 0 { 3 } else { 4 }
        );
    }
}

pub(crate) async fn captured_records_duplicates() {
    let root = tempfile::tempdir().unwrap();
    for pass in 0..3 {
        let data = Arc::new(
            Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
                .await
                .unwrap(),
        );
        let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
        let storage = StorageManager::new(
            StorageBackends {
                primary: data,
                recovery,
                replicas: vec![],
            },
            policy(),
        )
        .unwrap();
        let started = Arc::new(tokio::sync::Notify::new());
        let signal = started.clone();
        let saved = storage.clone();
        let runtime = Runtime::<Charger>::builder(0, ())
            .register(Transactions::catalog())
            .register(Records::catalog())
            .managed_state(limits(1))
            .managed_snapshots(snapshot_policy())
            .store_records(limits(1), Capture)
            .reconcile(move |state_manager, storage| async move {
                let mut selection = SchemaQuery::new();
                selection.schema = Some(key());
                let mut schemas = storage.query(selection).await?;
                let existing = schemas.next_page().await?;
                drop(schemas);
                if let Some(existing) = existing {
                    assert_eq!(existing[0].fields, schema().fields);
                } else {
                    storage.write_schema(&key(), &schema().fields).await?;
                }
                let restored = state_manager.state().read(|state| *state).await?.value;
                assert_eq!(restored, if pass == 1 { 230 } else { 0 });
                if restored != 0 {
                    // Reconciliation seals logical work before a new run can start.
                    // It must not repeat a physical charger action.
                    state_manager.commit(Setpoint { volts: 0 }).await.unwrap();
                }
                Ok(())
            })
            .service("charger.ready", move |context: harmos::Service<Charger>| {
                let signal = signal.clone();
                let saved = saved.clone();
                async move {
                    assert_eq!(
                        context
                            .state()
                            .state()
                            .read(|state| *state)
                            .await
                            .unwrap()
                            .value,
                        0
                    );
                    assert_eq!(saved.status().accepted, saved.status().saved);
                    assert_eq!(
                        saved.status().recovery_accepted,
                        saved.status().recovery_saved
                    );
                    context.ready();
                    signal.notify_one();
                    Ok::<(), harmos::Fault<String>>(())
                }
            })
            .runtime_with_storage(&storage)
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(5), started.notified())
            .await
            .unwrap();

        assert_eq!(
            runtime
                .state
                .state()
                .read(|state| *state)
                .await
                .unwrap()
                .value,
            0
        );
        assert_eq!(storage.status().accepted, storage.status().saved);
        assert_eq!(
            storage.status().recovery_accepted,
            storage.status().recovery_saved
        );
        if pass == 0 {
            runtime.state.commit(Setpoint { volts: 230 }).await.unwrap();
        }

        let mut selection = records_query();
        selection.scope = Some(id("execution"));
        storage.flush().await.unwrap();
        let mut reader = storage.query(selection).await.unwrap();
        let first = reader.next_page().await.unwrap().unwrap();
        assert_eq!(first[0].item, id("measurement-2"));
        assert_eq!(
            Measurement::decode(first[0].payload.as_slice())
                .unwrap()
                .counter,
            230
        );
        if pass > 0 {
            let second = reader.next_page().await.unwrap().unwrap();
            assert_eq!(second[0].item, id("measurement-4"));
            assert_eq!(
                Measurement::decode(second[0].payload.as_slice())
                    .unwrap()
                    .counter,
                0
            );
        }
        assert!(reader.next_page().await.unwrap().is_none());
        drop(reader);
        runtime.stop().await.unwrap();
    }
}

pub(crate) async fn cancelled_boot_storage() {
    let root = tempfile::tempdir().unwrap();
    let data = Arc::new(
        Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
            .await
            .unwrap(),
    );
    let history = data.history().clone();
    let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
    let storage = StorageManager::new(
        StorageBackends {
            primary: data,
            recovery,
            replicas: vec![],
        },
        policy(),
    )
    .unwrap();
    let mut opening = Box::pin(Runtime::<Bench>::builder((), ()).runtime_with_storage(&storage));
    assert!(futures_util::poll!(opening.as_mut()).is_pending());
    drop(opening);
    tokio::time::timeout(Duration::from_secs(5), async {
        while !storage.status().closed {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let data = Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
        .await
        .unwrap();
    assert_eq!(data.history(), &history);
    let recovery = data.recovery(RecoveryLimits::default());
    recovery.open().await.unwrap();
    recovery.close().await.unwrap();
    data.close().await.unwrap();
}

/* === Tests === */

pub(crate) async fn oversized_admission_order() {
    let root = tempfile::tempdir().unwrap();
    let primary = Arc::new(
        Backend::<()>::replica(
            root.path().join("primary"),
            BackendLimits::default(),
            history(),
        )
        .await
        .unwrap(),
    );
    let replica = Arc::new(
        Backend::<()>::replica(
            root.path().join("replica"),
            BackendLimits::default(),
            history(),
        )
        .await
        .unwrap(),
    );
    let recovery = Arc::new(primary.recovery(RecoveryLimits::default()));
    let storage = StorageManager::new(
        StorageBackends {
            primary: primary.clone(),
            recovery,
            replicas: vec![ReplicaTarget {
                id: id("secondary"),
                backend: replica.clone(),
            }],
        },
        StorageSettings {
            target_batch_bytes: NonZeroUsize::new(1024).unwrap(),
            max_batch_bytes: NonZeroUsize::new(1024).unwrap(),
            replication_shutdown: Duration::from_secs(5),
            ..policy()
        },
    )
    .unwrap();
    let runtime = Runtime::<Bench>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    let key = key();
    let schema_cursor = storage.write_schema(&key, &schema().fields).await.unwrap();
    storage.flush_until(&schema_cursor).await.unwrap();
    let before = storage
        .view_with_limits(policy().query_execution)
        .await
        .unwrap();
    let control = primary.control().await.unwrap();
    control.call(|connection| connection.execute_batch(
        "CREATE TABLE fixture_data_commits (saved TEXT NOT NULL);
         CREATE TRIGGER observe_data_commit AFTER UPDATE OF saved ON storage_data_head
         WHEN NEW.saved != OLD.saved BEGIN INSERT INTO fixture_data_commits VALUES (NEW.saved); END;"
    )).await.unwrap();
    let expected = (2..42).map(record).collect::<Vec<_>>();
    let parts = [StoragePart::Records(&expected)];
    assert!(StorageBatch::new(&parts, NonZeroUsize::new(1024).unwrap()).is_err());
    let writes = expected
        .iter()
        .map(|row| RecordWrite {
            schema: &row.schema,
            scope: row.scope.as_ref(),
            item: &row.item,
            source: row.source.as_ref(),
            source_time: row.source_time,
            payload: &row.payload,
        })
        .collect::<Vec<_>>();

    // One admitted operation exceeds the physical byte cap; it must retain
    // one stable cursor range while the manager sends several bounded writes.
    let admitted = storage.write_records(&writes).await.unwrap();
    assert_eq!(admitted.sequence, 41);
    assert_eq!(storage.flush_until(&admitted).await.unwrap(), admitted);
    assert_eq!(primary.cursor_saved().await.unwrap(), admitted);
    let commits: i64 = control
        .call(|connection| {
            connection.query_row("SELECT COUNT(*) FROM fixture_data_commits", [], |row| {
                row.get(0)
            })
        })
        .await
        .unwrap();
    assert!(commits > 1);
    let mut old = storage.query_in(&before, records_query()).unwrap();
    assert!(old.next_page().await.unwrap().is_none());
    drop(old);
    drop(before);
    runtime.stop().await.unwrap();

    for backend in [&primary, &replica] {
        assert_eq!(backend.open().await.unwrap(), admitted);
        let saved = view(backend, admitted.sequence).await;
        assert_eq!(read_all(backend, &saved).await, expected);
        backend.release_view(saved.snapshot()).await.unwrap();
        backend.close().await.unwrap();
    }
}

pub(crate) async fn follower_bridges_admissions() {
    let root = tempfile::tempdir().unwrap();
    let data = Arc::new(
        Backend::<()>::replica(
            root.path().join("data"),
            BackendLimits::default(),
            history(),
        )
        .await
        .unwrap(),
    );
    let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
    let storage = StorageManager::new(
        StorageBackends {
            primary: data,
            recovery,
            replicas: vec![],
        },
        policy(),
    )
    .unwrap();
    let runtime = Runtime::<Bench>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    let key = key();
    storage.write_schema(&key, &schema().fields).await.unwrap();
    let first = record(2);
    let first_cursor = storage
        .write_records(&[RecordWrite {
            schema: &first.schema,
            scope: first.scope.as_ref(),
            item: &first.item,
            source: first.source.as_ref(),
            source_time: first.source_time,
            payload: &first.payload,
        }])
        .await
        .unwrap();
    storage.flush_until(&first_cursor).await.unwrap();
    let mut selection = records_query();
    selection.schema = key;
    selection.scope = first.scope.clone();
    let mut follower = storage.follow(selection).await.unwrap();
    assert_eq!(follower.saved_boundary(), &first_cursor);
    assert_eq!(follower.accepted_boundary(), &first_cursor);
    assert_eq!(&*follower.next_page().await.unwrap(), &[first]);

    // Poll a caught-up reader once, then cancel it without an elapsed-time race.
    tokio::select! {
        biased;
        result = follower.next_page() => panic!("unexpected live result: {result:?}"),
        () = std::future::ready(()) => {},
    }
    let second = record(3);
    let admitted = storage
        .write_records(&[RecordWrite {
            schema: &second.schema,
            scope: second.scope.as_ref(),
            item: &second.item,
            source: second.source.as_ref(),
            source_time: second.source_time,
            payload: &second.payload,
        }])
        .await
        .unwrap();
    // Saving must not remove a follower's unconsumed admission or replay it twice.
    storage.flush_until(&admitted).await.unwrap();
    assert_eq!(&*follower.next_page().await.unwrap(), &[second]);
    tokio::select! {
        biased;
        result = follower.next_page() => panic!("duplicate live result: {result:?}"),
        () = std::future::ready(()) => {},
    }
    drop(follower);
    runtime.stop().await.unwrap();
}

pub(crate) async fn replay_tail_saving() {
    let root = tempfile::tempdir().unwrap();
    let data = Arc::new(
        Backend::<()>::replica(
            root.path().join("data"),
            BackendLimits::default(),
            history(),
        )
        .await
        .unwrap(),
    );
    let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
    let storage = StorageManager::new(
        StorageBackends {
            primary: data,
            recovery,
            replicas: vec![],
        },
        policy(),
    )
    .unwrap();
    let runtime = Runtime::<Bench>::builder((), ())
        .runtime_with_storage(&storage)
        .await
        .unwrap();
    let key = key();
    storage.write_schema(&key, &schema().fields).await.unwrap();
    let first = record(2);
    let first_cursor = storage
        .write_records(&[record_write(&first)])
        .await
        .unwrap();
    storage.flush_until(&first_cursor).await.unwrap();
    let second = record(3);
    storage
        .write_records(&[record_write(&second)])
        .await
        .unwrap();
    let tail_schema = SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: id("other.schema"),
        version: key.version,
    };
    let boundary = storage
        .write_schema(&tail_schema, &schema().fields)
        .await
        .unwrap();
    let old = storage
        .view_with_limits(policy().query_execution)
        .await
        .unwrap();
    assert_eq!(old.saved().unwrap(), first_cursor);
    assert_eq!(old.accepted().unwrap(), boundary);

    // Dokime reads its saved prefix separately. The writer can save the tail
    // before replay registers; this must neither drop it nor duplicate the prefix.
    storage.flush_until(&boundary).await.unwrap();
    let third = record(5);
    storage
        .write_records(&[record_write(&third)])
        .await
        .unwrap();
    let mut selection = records_query();
    selection.schema = key;
    selection.scope = second.scope.clone();
    selection.options.after = old.saved().unwrap().sequence;
    selection.options.through = Some(old.accepted().unwrap().sequence);
    let mut absent_selection = records_query();
    absent_selection.schema = selection.schema.clone();
    absent_selection.options = selection.options.clone();
    absent_selection.scope = Some(id("absent"));
    let mut tail = storage.replay(selection).await.unwrap();
    assert_eq!(tail.boundary(), &boundary);
    assert_eq!(&*tail.next_page().await.unwrap().unwrap(), &[second]);
    assert!(tail.next_page().await.unwrap().is_none());

    let mut prefix = storage.query_in(&old, records_query()).unwrap();
    assert_eq!(&*prefix.next_page().await.unwrap().unwrap(), &[first]);
    assert!(prefix.next_page().await.unwrap().is_none());
    let mut absent = storage.replay(absent_selection).await.unwrap();
    assert!(absent.next_page().await.unwrap().is_none());
    drop(prefix);
    drop(old);
    drop(tail);
    drop(absent);
    runtime.stop().await.unwrap();
}

pub(crate) async fn managed_producer_restart() {
    let root = tempfile::tempdir().unwrap();
    let mut retry_window = None;
    let metadata = FileMetadata {
        id: id("managed.producer"),
        scope: None,
        media_type: "text/plain".into(),
        expected_length: None,
        expected_sha256: None,
    };
    let mut original = None;
    for pass in 0..2 {
        let data = Arc::new(
            Backend::<()>::primary(root.path().join("data"), BackendLimits::default())
                .await
                .unwrap(),
        );
        let recovery = Arc::new(data.recovery(RecoveryLimits::default()));
        let storage = StorageManager::new(
            StorageBackends {
                primary: data,
                recovery,
                replicas: vec![],
            },
            policy(),
        )
        .unwrap();
        let runtime = Runtime::<Bench>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let window = storage.mutation_window().unwrap().id;
        let retry_window = retry_window.get_or_insert_with(|| window.clone());
        let mutation = |key: &str| MutationId {
            window: retry_window.clone(),
            key: id(key),
        };
        let current = |key: &str| MutationId {
            window: window.clone(),
            key: id(key),
        };
        let created = storage
            .create_file_once(&mutation("create"), &metadata)
            .await
            .unwrap();
        let appended = storage
            .append_file_once(&mutation("append"), &created.session, 0, b"saved prefix")
            .await
            .unwrap();
        assert_eq!(appended.length, 12);
        if let Some(previous) = &original {
            assert_eq!(&appended, previous);
        } else {
            original = Some(appended.clone());
        }
        if pass == 1 {
            let resumed = storage
                .resume_file_once(&current("resume"), &appended.session)
                .await
                .unwrap();
            assert_eq!(resumed.length, 12);
            assert_eq!(resumed.session.generation, appended.session.generation + 1);
            let reference = storage
                .finish_file_once(&current("finish"), &resumed.session)
                .await
                .unwrap();
            assert_eq!(reference.length, 12);
            assert_eq!(
                storage
                    .finish_file_once(&current("finish"), &resumed.session)
                    .await
                    .unwrap(),
                reference
            );
            let staged = StagedFile {
                history: resumed.session.history.clone(),
                id: resumed.session.id.clone(),
            };
            let mut follower = storage.follow_file(&staged, 0).await.unwrap();
            assert_eq!(
                follower
                    .next_chunk(NonZeroUsize::new(64).unwrap())
                    .await
                    .unwrap()
                    .unwrap()
                    .as_bytes(),
                b"saved prefix"
            );
            assert!(
                follower
                    .next_chunk(NonZeroUsize::MIN)
                    .await
                    .unwrap()
                    .is_none()
            );
            assert_eq!(follower.progress().phase, FilePhase::Finished);
            drop(follower);
            storage
                .delete_file_once(&current("delete"), &staged)
                .await
                .unwrap();
            assert_eq!(
                storage
                    .delete_file_once(&current("delete"), &staged)
                    .await
                    .unwrap(),
                staged
            );
            assert!(storage.follow_file(&staged, 0).await.is_err());
        }
        runtime.stop().await.unwrap();
    }
}
