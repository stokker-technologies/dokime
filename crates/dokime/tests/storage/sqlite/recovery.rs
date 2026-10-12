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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::{recovery::*, *};

// Local Imports
use dokime::storage::sqlite::{Backend, Limits, RecoveryLimits, State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

fn bound(bytes: usize) -> NonZeroUsize {
    NonZeroUsize::new(bytes).unwrap()
}

pub(crate) async fn primary_close_until() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let recovery = backend.recovery(RecoveryLimits::default());
    RecoveryBackend::open(&recovery).await.unwrap();
    let at = RecoveryPosition {
        history: backend.history().clone(),
        sequence: 1,
    };
    recovery
        .write_entries(&[RecoveryEntry {
            position: at.clone(),
            payload: vec![7],
        }])
        .await
        .unwrap();
    StorageBackend::close(&backend).await.unwrap();
    assert!(backend.cursor_saved().await.is_err());
    assert_eq!(recovery.flush(&at).await.unwrap(), at);
    assert!(State::open(root.path()).await.is_err());
    RecoveryBackend::close(&recovery).await.unwrap();
    let owner = State::open(root.path()).await.unwrap();
    owner.close().await.unwrap();
    backend.open().await.unwrap();
    assert_eq!(RecoveryBackend::open(&recovery).await.unwrap(), at);
    StorageBackend::close(&backend).await.unwrap();
    RecoveryBackend::close(&recovery).await.unwrap();
}

pub(crate) async fn snapshot_tail_frontier() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let recovery = backend.recovery(RecoveryLimits::default());
    RecoveryBackend::open(&recovery).await.unwrap();
    let history = backend.history().clone();
    let at = |sequence| RecoveryPosition {
        history: history.clone(),
        sequence,
    };
    let entries = (1..=3)
        .map(|sequence| RecoveryEntry {
            position: at(sequence),
            payload: vec![sequence as u8],
        })
        .collect::<Vec<_>>();
    recovery.write_entries(&entries).await.unwrap();
    recovery.write_entries(&entries).await.unwrap();
    let snapshot = RecoverySnapshot {
        through: at(2),
        payload: b"snapshot".to_vec(),
    };
    recovery.write_snapshot(&snapshot).await.unwrap();
    recovery.write_snapshot(&snapshot).await.unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 0);
    assert_eq!(recovery.flush(&at(2)).await.unwrap().sequence, 3);
    let limits = ReadLimits {
        rows: bound(64),
        bytes: bound(4096),
    };
    assert!(recovery.read_entries(&at(0), &at(3), limits).await.is_err());
    let tail = recovery.read_entries(&at(2), &at(3), limits).await.unwrap();
    assert_eq!(tail.entries, entries[2..]);
    assert!(tail.next.is_none());
    StorageBackend::close(&backend).await.unwrap();
    RecoveryBackend::close(&recovery).await.unwrap();
    backend.open().await.unwrap();
    assert_eq!(RecoveryBackend::open(&recovery).await.unwrap().sequence, 3);
    assert_eq!(
        recovery.read_snapshot(bound(4096)).await.unwrap(),
        Some(snapshot)
    );
    assert_eq!(backend.cursor_saved().await.unwrap().sequence, 0);
    StorageBackend::close(&backend).await.unwrap();
    RecoveryBackend::close(&recovery).await.unwrap();
}
