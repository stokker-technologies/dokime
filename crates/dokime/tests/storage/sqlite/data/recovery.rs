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
use std::{num::NonZeroUsize, path::Path};

// Third Party Imports
use harmos::{
    BackendFailure, Identifier,
    storage::{
        ReadLimits, RecoveryPosition, StorageBackend, StorageCursor, StoreHistory,
        state::{RecoveryEntry, RecoverySnapshot, StateBackend},
    },
};

// Local Imports
use dokime::storage::sqlite::{Backend, Limits, StateBackend as SQLiteStateBackend, StateLimits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn history() -> StoreHistory {
    StoreHistory {
        store: Identifier::new("bench").unwrap(),
        epoch: Identifier::new("epoch-one").unwrap(),
    }
}

fn at(sequence: u64) -> RecoveryPosition {
    RecoveryPosition {
        history: history(),
        sequence,
    }
}

fn entry(sequence: u64) -> RecoveryEntry {
    RecoveryEntry {
        required_data: StorageCursor {
            history: history(),
            sequence: 0,
        },
        position: at(sequence),
        payload: format!("transition-{sequence}").into_bytes(),
    }
}

fn page(rows: usize) -> ReadLimits {
    ReadLimits {
        rows: NonZeroUsize::new(rows).unwrap(),
        bytes: NonZeroUsize::new(16_384).unwrap(),
    }
}

async fn native(root: &Path, limits: StateLimits) -> (Backend<()>, SQLiteStateBackend) {
    let primary = Backend::<()>::replica(root.to_path_buf(), Limits::default(), history())
        .await
        .unwrap();
    let recovery = primary.state(limits);
    (primary, recovery)
}

async fn close(primary: &Backend<()>, recovery: &SQLiteStateBackend) {
    recovery.close().await.unwrap();
    primary.close().await.unwrap();
}

async fn recovery_sql<T: Send + 'static>(
    root: &Path,
    operation: impl FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error> + Send + 'static,
) -> Result<T, rusqlite::Error> {
    let path = root.join("local.sqlite");
    tokio::task::spawn_blocking(move || {
        let connection = rusqlite::Connection::open(path)?;
        operation(&connection)
    })
    .await
    .unwrap()
}

/* === Tests === */

pub(crate) async fn durable_prefix_rollback() {
    let root = tempfile::tempdir().unwrap();
    let (primary, backend) = native(root.path(), StateLimits::default()).await;
    assert_eq!(backend.open().await.unwrap(), at(0));
    backend.write_entries(&[entry(1), entry(2)]).await.unwrap();
    // Acknowledged SQLite Writes Are Already Durable
    assert_eq!(backend.flush(&at(1)).await.unwrap(), at(2));
    recovery_sql(root.path(), |connection| {
        connection.execute_batch(
            "CREATE TRIGGER refuse_four BEFORE INSERT ON storage_recovery_entries
         WHEN NEW.position='0000000000000004' BEGIN SELECT RAISE(ABORT,'fixture'); END;",
        )
    })
    .await
    .unwrap();
    assert!(backend.write_entries(&[entry(3), entry(4)]).await.is_err());
    assert_eq!(backend.flush(&at(2)).await.unwrap(), at(2));
    close(&primary, &backend).await;

    let (primary, reopened) = native(root.path(), StateLimits::default()).await;
    assert_eq!(reopened.open().await.unwrap(), at(2));
    assert_eq!(
        reopened
            .read_entries(&at(0), &at(2), page(4))
            .await
            .unwrap()
            .entries,
        [entry(1), entry(2)]
    );
    recovery_sql(root.path(), |connection| {
        connection.execute_batch("DROP TRIGGER refuse_four")
    })
    .await
    .unwrap();
    reopened.write_entries(&[entry(3)]).await.unwrap();
    assert_eq!(reopened.flush(&at(3)).await.unwrap(), at(3));
    close(&primary, &reopened).await;
}

pub(crate) async fn retries_conflicts_explicit() {
    let root = tempfile::tempdir().unwrap();
    let (primary, backend) = native(root.path(), StateLimits::default()).await;
    backend.open().await.unwrap();
    backend.write_entries(&[entry(1)]).await.unwrap();
    backend.write_entries(&[entry(1)]).await.unwrap();
    let mut changed = entry(1);
    changed.payload.push(0);
    assert!(matches!(
        backend.write_entries(&[changed]).await,
        Err(harmos::Error::Backend {
            kind: BackendFailure::Conflict,
            ..
        })
    ));
    assert!(matches!(
        backend.write_entries(&[entry(3)]).await,
        Err(harmos::Error::InvalidInput { .. })
    ));
    let mut foreign = entry(2);
    foreign.position.history.epoch = Identifier::new("epoch-two").unwrap();
    assert!(matches!(
        backend.write_entries(&[foreign]).await,
        Err(harmos::Error::DifferentHistory)
    ));
    backend.write_entries(&[entry(2), entry(3)]).await.unwrap();
    let first = backend.read_entries(&at(0), &at(3), page(2)).await.unwrap();
    assert_eq!(first.entries, [entry(1), entry(2)]);
    assert_eq!(first.next, Some(at(2)));
    first
        .check(&at(0), &at(3), page(2), |entry| entry.payload.len())
        .unwrap();
    let last = backend
        .read_entries(&first.next.unwrap(), &at(3), page(2))
        .await
        .unwrap();
    assert_eq!(last.entries, [entry(3)]);
    assert_eq!(last.next, None);
    let tiny = ReadLimits {
        bytes: NonZeroUsize::MIN,
        ..page(2)
    };
    assert!(matches!(
        backend.read_entries(&at(0), &at(3), tiny).await,
        Err(harmos::Error::InvalidInput { .. })
    ));
    close(&primary, &backend).await;
}

pub(crate) async fn snapshot_transaction_tail() {
    let root = tempfile::tempdir().unwrap();
    let (primary, backend) = native(root.path(), StateLimits::default()).await;
    backend.open().await.unwrap();
    backend
        .write_entries(&(1..=5).map(entry).collect::<Vec<_>>())
        .await
        .unwrap();
    let snapshot = RecoverySnapshot {
        required_data: StorageCursor {
            history: history(),
            sequence: 0,
        },
        through: at(2),
        payload: b"logical state".to_vec(),
    };
    backend.write_snapshot(&snapshot).await.unwrap();
    backend.write_snapshot(&snapshot).await.unwrap();
    let mut conflict = snapshot.clone();
    conflict.payload.push(0);
    assert!(matches!(
        backend.write_snapshot(&conflict).await,
        Err(harmos::Error::Backend {
            kind: BackendFailure::Conflict,
            ..
        })
    ));
    assert_eq!(
        backend
            .read_snapshot(NonZeroUsize::new(4096).unwrap())
            .await
            .unwrap(),
        Some(snapshot.clone())
    );
    assert!(matches!(
        backend.read_entries(&at(0), &at(5), page(4)).await,
        Err(harmos::Error::Gap { .. })
    ));
    assert_eq!(
        backend
            .read_entries(&at(2), &at(5), page(4))
            .await
            .unwrap()
            .entries,
        [entry(3), entry(4), entry(5)]
    );
    close(&primary, &backend).await;

    let (primary, reopened) = native(root.path(), StateLimits::default()).await;
    assert_eq!(reopened.open().await.unwrap(), at(5));
    assert_eq!(
        reopened
            .read_snapshot(NonZeroUsize::new(4096).unwrap())
            .await
            .unwrap(),
        Some(snapshot)
    );
    assert_eq!(
        reopened
            .read_entries(&at(2), &at(5), page(4))
            .await
            .unwrap()
            .entries,
        [entry(3), entry(4), entry(5)]
    );
    reopened.write_entries(&[entry(6)]).await.unwrap();
    close(&primary, &reopened).await;
}

pub(crate) async fn saved_corruption_rows() {
    for missing in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let (primary, backend) = native(root.path(), StateLimits::default()).await;
        backend.open().await.unwrap();
        backend.write_entries(&[entry(1), entry(2)]).await.unwrap();
        recovery_sql(root.path(), move |connection| {
            connection.execute(if missing {
                "DELETE FROM storage_recovery_entries WHERE position='0000000000000001'"
            } else {
                "UPDATE storage_recovery_entries SET payload=x'00' WHERE position='0000000000000001'"
            }, [])
        }).await.unwrap();
        close(&primary, &backend).await;

        let (primary, reopened) = native(root.path(), StateLimits::default()).await;
        assert_eq!(reopened.open().await.unwrap(), at(2));
        assert!(matches!(
            reopened.read_entries(&at(0), &at(2), page(2)).await,
            Err(harmos::Error::Backend {
                kind: BackendFailure::Corrupt,
                ..
            })
        ));
        let rows: i64 = recovery_sql(root.path(), |connection| {
            connection.query_row("SELECT COUNT(*) FROM storage_recovery_entries", [], |row| {
                row.get(0)
            })
        })
        .await
        .unwrap();
        assert_eq!(rows, if missing { 1 } else { 2 });
        close(&primary, &reopened).await;
    }
}

pub(crate) async fn primary_recovery_owner() {
    let root = tempfile::tempdir().unwrap();
    let (primary, backend) = native(root.path(), StateLimits::default()).await;
    backend.open().await.unwrap();
    assert!(matches!(
        Backend::<()>::primary(root.path().to_path_buf(), Limits::default()).await,
        Err(harmos::Error::Backend {
            kind: BackendFailure::Conflict,
            ..
        })
    ));
    // Neither Adapter Alone Can Release The Shared Physical Owner
    primary.close().await.unwrap();
    assert!(
        Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
            .await
            .is_err()
    );
    backend.close().await.unwrap();
    let mut other = history();
    other.store = Identifier::new("other").unwrap();
    assert!(matches!(
        Backend::<()>::replica(root.path().to_path_buf(), Limits::default(), other).await,
        Err(harmos::Error::DifferentHistory)
    ));
    let (primary, reopened) = native(root.path(), StateLimits::default()).await;
    assert_eq!(reopened.open().await.unwrap(), at(0));
    close(&primary, &reopened).await;
}

pub(crate) async fn input_snapshot_bounded() {
    let root = tempfile::tempdir().unwrap();
    let limits = StateLimits {
        entry_bytes: NonZeroUsize::new(512).unwrap(),
        batch_bytes: NonZeroUsize::new(2048).unwrap(),
        snapshot_bytes: NonZeroUsize::new(1024).unwrap(),
    };
    let (primary, backend) = native(root.path(), limits).await;
    backend.open().await.unwrap();
    let mut oversized = entry(1);
    oversized.payload = vec![0; 513];
    assert!(matches!(
        backend.write_entries(&[oversized]).await,
        Err(harmos::Error::InvalidInput { .. })
    ));
    backend.write_entries(&[entry(1)]).await.unwrap();
    let snapshot = RecoverySnapshot {
        required_data: StorageCursor {
            history: history(),
            sequence: 0,
        },
        through: at(1),
        payload: b"state".to_vec(),
    };
    backend.write_snapshot(&snapshot).await.unwrap();
    close(&primary, &backend).await;
    let (primary, reopened) = native(root.path(), limits).await;
    reopened.open().await.unwrap();
    assert!(matches!(
        reopened.read_snapshot(NonZeroUsize::MIN).await,
        Err(harmos::Error::InvalidInput { .. })
    ));
    assert_eq!(
        reopened
            .read_snapshot(NonZeroUsize::new(1024).unwrap())
            .await
            .unwrap(),
        Some(snapshot)
    );
    close(&primary, &reopened).await;
}

pub(crate) async fn failed_snapshot_retry() {
    let root = tempfile::tempdir().unwrap();
    let (primary, backend) = native(root.path(), StateLimits::default()).await;
    backend.open().await.unwrap();
    backend.write_entries(&[entry(1), entry(2)]).await.unwrap();
    recovery_sql(root.path(), |connection| {
        connection.execute_batch(
            "CREATE TRIGGER refuse_snapshot BEFORE UPDATE OF snapshot ON storage_recovery_head
         BEGIN SELECT RAISE(ABORT,'fixture'); END;",
        )
    })
    .await
    .unwrap();
    let snapshot = RecoverySnapshot {
        required_data: StorageCursor {
            history: history(),
            sequence: 0,
        },
        through: at(1),
        payload: b"state".to_vec(),
    };
    assert!(backend.write_snapshot(&snapshot).await.is_err());
    assert!(
        backend
            .read_snapshot(NonZeroUsize::new(4096).unwrap())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        backend
            .read_entries(&at(0), &at(2), page(2))
            .await
            .unwrap()
            .entries,
        [entry(1), entry(2)]
    );
    assert!(
        Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
            .await
            .is_err()
    );
    recovery_sql(root.path(), |connection| {
        connection.execute_batch("DROP TRIGGER refuse_snapshot")
    })
    .await
    .unwrap();
    backend.write_snapshot(&snapshot).await.unwrap();
    close(&primary, &backend).await;
    let (primary, reopened) = native(root.path(), StateLimits::default()).await;
    assert_eq!(reopened.open().await.unwrap(), at(2));
    assert_eq!(
        reopened
            .read_snapshot(NonZeroUsize::new(4096).unwrap())
            .await
            .unwrap(),
        Some(snapshot)
    );
    assert_eq!(
        reopened
            .read_entries(&at(1), &at(2), page(2))
            .await
            .unwrap()
            .entries,
        [entry(2)]
    );
    close(&primary, &reopened).await;
}

pub(crate) async fn snapshot_requires_frontier() {
    let root = tempfile::tempdir().unwrap();
    let (primary, backend) = native(root.path(), StateLimits::default()).await;
    backend.open().await.unwrap();
    backend.write_entries(&[entry(1)]).await.unwrap();
    let snapshot = RecoverySnapshot {
        required_data: StorageCursor {
            history: history(),
            sequence: 0,
        },
        through: at(2),
        payload: b"state at two".to_vec(),
    };
    assert!(matches!(
        backend.write_snapshot(&snapshot).await,
        Err(harmos::Error::InvalidInput { .. })
    ));
    assert!(
        backend
            .read_snapshot(NonZeroUsize::new(4096).unwrap())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(backend.flush(&at(1)).await.unwrap(), at(1));
    close(&primary, &backend).await;
    let (primary, reopened) = native(root.path(), StateLimits::default()).await;
    assert_eq!(reopened.open().await.unwrap(), at(1));
    assert!(
        reopened
            .read_snapshot(NonZeroUsize::new(4096).unwrap())
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        reopened
            .read_entries(&at(0), &at(1), page(2))
            .await
            .unwrap()
            .entries,
        [entry(1)]
    );
    reopened.write_entries(&[entry(2)]).await.unwrap();
    reopened.write_snapshot(&snapshot).await.unwrap();
    close(&primary, &reopened).await;
}

pub(crate) async fn cancelled_waiter_owned() {
    let root = tempfile::tempdir().unwrap();
    let (primary, backend) = native(root.path(), StateLimits::default()).await;
    backend.open().await.unwrap();
    let entries = [entry(1)];
    let mut write = backend.write_entries(&entries);
    // One Poll Admits The Original Owned SQLite Operation
    if let std::task::Poll::Ready(result) = futures_util::poll!(write.as_mut()) {
        result.unwrap();
    }
    drop(write);
    close(&primary, &backend).await;
    let (primary, reopened) = native(root.path(), StateLimits::default()).await;
    assert_eq!(reopened.open().await.unwrap(), at(1));
    assert_eq!(
        reopened
            .read_entries(&at(0), &at(1), page(1))
            .await
            .unwrap()
            .entries,
        entries
    );
    close(&primary, &reopened).await;
}
