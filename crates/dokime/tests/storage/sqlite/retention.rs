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

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(super) struct RetainedCharger {
    id: String,
    volts: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Identified for RetainedCharger {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("bench.retained_charger"), 1)
    }
}

impl StoredObject for RetainedCharger {
    type Key = String;
    const RETENTION: ObjectRetention =
        ObjectRetention::Versions(std::num::NonZeroUsize::new(2).unwrap());

    fn key(&self) -> &Self::Key {
        &self.id
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn pruning_pins_coverage() {
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
    let mut updates = vec![entry(1, 230), entry(2, 240), entry(3, 250)];
    // Retention is shared by codec versions of the same logical key.
    updates[2].value = Objects::encode(
        &RevisedCharger {
            id: "charger:0123456789".into(),
            volts: 250,
            enabled: true,
        },
        limits(1).bytes,
    )
    .unwrap();
    backend
        .write(write_batch(&[StoragePart::Objects(&updates)]))
        .await
        .unwrap();
    let old = object_view(&backend, 3).await;
    assert!(
        !backend
            .prune_objects(&object_position(3), 1.try_into().unwrap())
            .await
            .unwrap()
    );
    let intermediate = object_view(&backend, 3).await;
    let mut retained = ReadSelection::new(ReadKind::Objects);
    retained.history = ReadHistory::Retained;
    assert_eq!(
        backend
            .read_objects(&intermediate, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[1..]
    );
    assert!(
        !backend
            .prune_objects(&object_position(3), 1.try_into().unwrap())
            .await
            .unwrap()
    );
    assert!(
        backend
            .prune_objects(&object_position(3), 1.try_into().unwrap())
            .await
            .unwrap()
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), object_position(3));
    assert_eq!(
        backend
            .read_objects(
                &old,
                &ReadSelection::new(ReadKind::Objects),
                None,
                limits(8)
            )
            .await
            .unwrap()
            .entries,
        updates
    );
    // A pin between passes keeps exactly its snapshot as well.
    assert_eq!(
        backend
            .read_objects(&intermediate, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[1..]
    );
    let current = object_view(&backend, 3).await;
    assert!(matches!(
        backend
            .read_objects(
                &current,
                &ReadSelection::new(ReadKind::Objects),
                None,
                limits(8)
            )
            .await,
        Err(harmos::Error::Gap {
            requested: 1,
            available: 6
        })
    ));
    assert_eq!(
        backend
            .read_objects(&current, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[2..]
    );
    assert!(
        backend
            .read_object(
                &current,
                &query(ObjectVersion::Exact(version(1))),
                limits(1).bytes
            )
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        backend
            .read_object(
                &old,
                &query(ObjectVersion::Exact(version(1))),
                limits(1).bytes
            )
            .await
            .unwrap(),
        Some(updates[0].clone())
    );
    assert!(
        backend
            .write(write_batch(&[StoragePart::Objects(&updates[..1])]))
            .await
            .is_err()
    );
    assert!(
        backend
            .prune_objects(&object_position(4), 1.try_into().unwrap())
            .await
            .is_err()
    );
    let mut foreign = object_position(3);
    foreign.history.epoch = id("different");
    assert!(
        backend
            .prune_objects(&foreign, 1.try_into().unwrap())
            .await
            .is_err()
    );
    for pinned in [&old, &intermediate, &current] {
        backend.release_view(pinned.snapshot()).await.unwrap();
    }
    backend.close().await.unwrap();

    // Reading durable coverage needs no installed producer or object catalog.
    let reopened = NativeBackend::<()>::replica(
        root.path().to_path_buf(),
        NativeLimits::default(),
        history(),
    )
    .await
    .unwrap();
    assert_eq!(reopened.open().await.unwrap(), object_position(3));
    let current = object_view(&reopened, 3).await;
    assert!(matches!(
        reopened
            .read_objects(
                &current,
                &ReadSelection::new(ReadKind::Objects),
                None,
                limits(8)
            )
            .await,
        Err(harmos::Error::Gap { .. })
    ));
    assert_eq!(
        reopened
            .read_objects(&current, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[2..]
    );
    reopened.release_view(current.snapshot()).await.unwrap();
    reopened.close().await.unwrap();
}

pub(crate) async fn bounded_retention_prefix() {
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
    let updates = (1..=6)
        .map(|sequence| ObjectEntry {
            position: object_position(sequence),
            version: version(sequence),
            value: Objects::encode(
                &RetainedCharger {
                    id: "charger:0123456789".into(),
                    volts: sequence as u32,
                },
                limits(1).bytes,
            )
            .unwrap(),
        })
        .collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Objects(&updates)]))
        .await
        .unwrap();
    assert!(
        !backend
            .prune_objects(&object_position(4), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    assert!(
        backend
            .prune_objects(&object_position(4), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    let prefix = object_view(&backend, 6).await;
    let mut retained = ReadSelection::new(ReadKind::Objects);
    retained.history = ReadHistory::Retained;
    assert_eq!(
        backend
            .read_objects(&prefix, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[2..]
    );
    assert!(
        !backend
            .prune_objects(&object_position(6), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    assert!(
        backend
            .prune_objects(&object_position(6), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    let current = object_view(&backend, 6).await;
    assert_eq!(
        backend
            .read_objects(&current, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[4..]
    );
    let mut complete_tail = ReadSelection::new(ReadKind::Objects);
    complete_tail.after = object_position(4).sequence;
    assert_eq!(
        backend
            .read_objects(&current, &complete_tail, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[4..]
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), object_position(6));
    for pinned in [&prefix, &current] {
        backend.release_view(pinned.snapshot()).await.unwrap();
    }
    backend.close().await.unwrap();
    assert_eq!(backend.open().await.unwrap(), object_position(6));
    backend.close().await.unwrap();
}

pub(crate) async fn pruning_records_loss() {
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
    let mut record_schema = super::super::storage::schema();
    record_schema.position = object_position(1);
    let mut retained_record = super::super::storage::record(2);
    retained_record.position = object_position(2);
    backend
        .write(write_batch(&[
            StoragePart::Schemas(&[record_schema.clone()]),
            StoragePart::Records(&[retained_record.clone()]),
            StoragePart::Objects(&[entry(3, 230), entry(4, 240), entry(5, 250)]),
        ]))
        .await
        .unwrap();
    assert!(
        !backend
            .prune_objects(&object_position(5), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    assert!(
        backend
            .prune_objects(&object_position(5), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    let saved = object_view(&backend, 5).await;
    assert_eq!(
        backend
            .read_records(
                &saved,
                &ReadSelection::new(ReadKind::Records),
                None,
                limits(8)
            )
            .await
            .unwrap()
            .entries,
        [retained_record]
    );
    assert_eq!(
        backend
            .read_schemas(
                &saved,
                &{
                    let mut selected = ReadSelection::new(ReadKind::Schemas);
                    selected.schema = Some(record_schema.key.clone());
                    selected
                },
                None,
                limits(8)
            )
            .await
            .unwrap()
            .entries,
        [record_schema]
    );
    backend.release_view(saved.snapshot()).await.unwrap();
    backend.close().await.unwrap();
    assert_eq!(backend.open().await.unwrap(), object_position(5));
    backend.close().await.unwrap();

    // An unrelated missing record must not hide behind valid pruning coverage.
    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    assert_eq!(
        connection
            .execute(
                "DELETE FROM storage_data_entries WHERE position='0000000000000005'",
                [],
            )
            .unwrap(),
        1
    );
    drop(connection);
    assert!(backend.open().await.is_err());
}

pub(crate) async fn cancelled_pruning_writes() {
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
    let updates = [entry(1, 230), entry(2, 240), entry(3, 250)];
    backend
        .write(write_batch(&[StoragePart::Objects(&updates)]))
        .await
        .unwrap();
    let old = object_view(&backend, 3).await;
    let through = object_position(3);
    let mut pruning = backend.prune_objects(&through, 8.try_into().unwrap());
    assert!(futures_util::poll!(pruning.as_mut()).is_pending());
    drop(pruning);
    assert_eq!(backend.cursor_saved().await.unwrap(), through);
    assert!(
        backend
            .prune_objects(&through, 8.try_into().unwrap())
            .await
            .unwrap()
    );
    backend
        .write(write_batch(&[StoragePart::Objects(&[
            entry(4, 260),
            entry(5, 270),
        ])]))
        .await
        .unwrap();
    assert_eq!(
        backend
            .read_objects(
                &old,
                &ReadSelection::new(ReadKind::Objects),
                None,
                limits(8)
            )
            .await
            .unwrap()
            .entries,
        updates
    );
    backend.release_view(old.snapshot()).await.unwrap();
    backend.close().await.unwrap();
    assert_eq!(backend.open().await.unwrap(), object_position(5));
    let current = object_view(&backend, 5).await;
    let mut retained = ReadSelection::new(ReadKind::Objects);
    retained.history = ReadHistory::Retained;
    assert_eq!(
        backend
            .read_objects(&current, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        [entry(3, 250), entry(4, 260), entry(5, 270)]
    );
    backend.release_view(current.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn released_pin_page() {
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
    let updates = (1..=6)
        .map(|sequence| ObjectEntry {
            position: object_position(sequence),
            version: version(sequence),
            value: Objects::encode(
                &RetainedCharger {
                    id: "charger:0123456789".into(),
                    volts: sequence as u32,
                },
                limits(1).bytes,
            )
            .unwrap(),
        })
        .collect::<Vec<_>>();
    backend
        .write(write_batch(&[StoragePart::Objects(&updates)]))
        .await
        .unwrap();
    let pinned = object_view(&backend, 6).await;
    assert!(
        !backend
            .prune_objects(&object_position(6), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    assert!(
        backend
            .prune_objects(&object_position(6), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    let original = backend
        .read_objects(
            &pinned,
            &ReadSelection::new(ReadKind::Objects),
            None,
            limits(8),
        )
        .await
        .unwrap();
    assert_eq!(original.entries, updates);
    backend.release_view(pinned.snapshot()).await.unwrap();
    let connection = rusqlite::Connection::open(root.path().join("local.sqlite")).unwrap();
    for remaining in (2..6).rev() {
        assert!(
            !backend
                .prune_objects(&object_position(6), 1.try_into().unwrap())
                .await
                .unwrap()
        );
        let physical = connection
            .query_row(
                "SELECT COUNT(*) FROM storage_data_entries WHERE kind='object'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(physical, remaining);
        assert_eq!(backend.cursor_saved().await.unwrap(), object_position(6));
    }
    assert!(
        backend
            .prune_objects(&object_position(6), 1.try_into().unwrap())
            .await
            .unwrap()
    );
    let reclaimed = connection
        .query_row(
            "SELECT COUNT(*) FROM storage_data_inventory WHERE kind='object' AND reclaimed=1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap();
    assert_eq!(reclaimed, 4);
    let current = object_view(&backend, 6).await;
    let mut retained = ReadSelection::new(ReadKind::Objects);
    retained.history = ReadHistory::Retained;
    assert_eq!(
        backend
            .read_objects(&current, &retained, None, limits(8))
            .await
            .unwrap()
            .entries,
        updates[4..]
    );
    backend.release_view(current.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn new_versions_remain() {
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
    let update = |sequence, key: &str| {
        let mut value = entry(sequence, sequence as u32);
        value.value = Objects::encode(
            &Charger {
                id: key.into(),
                volts: sequence as u32,
            },
            limits(1).bytes,
        )
        .unwrap();
        value
    };
    let original = [update(1, "charger:left"), update(2, "charger:right")];
    backend
        .write(write_batch(&[StoragePart::Objects(&original)]))
        .await
        .unwrap();
    assert!(
        backend
            .prune_objects(&object_position(2), 8.try_into().unwrap())
            .await
            .unwrap()
    );
    let pinned = object_view(&backend, 2).await;
    let mut retained = ReadSelection::new(ReadKind::Objects);
    retained.history = ReadHistory::Retained;
    for first in [3, 5] {
        let latest = [
            update(first, "charger:left"),
            update(first + 1, "charger:right"),
        ];
        backend
            .write(write_batch(&[StoragePart::Objects(&latest)]))
            .await
            .unwrap();
        let through = object_position(first + 1);
        for _ in 0..3 {
            if backend
                .prune_objects(&through, 1.try_into().unwrap())
                .await
                .unwrap()
            {
                break;
            }
        }
        assert!(
            backend
                .prune_objects(&through, 1.try_into().unwrap())
                .await
                .unwrap()
        );
        let current = object_view(&backend, first + 1).await;
        assert_eq!(
            backend
                .read_objects(&current, &retained, None, limits(8))
                .await
                .unwrap()
                .entries,
            latest
        );
        backend.release_view(current.snapshot()).await.unwrap();
        assert_eq!(
            backend
                .read_objects(
                    &pinned,
                    &ReadSelection::new(ReadKind::Objects),
                    None,
                    limits(8)
                )
                .await
                .unwrap()
                .entries,
            original
        );
    }
    backend.release_view(pinned.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}
