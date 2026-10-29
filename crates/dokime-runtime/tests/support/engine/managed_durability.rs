/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compact Checkpoints, Evidence Dependencies And Uncertain Publication Recovery

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    num::NonZeroUsize,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

// Third Party Imports
use harmos::storage::state::{RecoveryEntry, RecoverySnapshot, StateBackend, file::FileBackend};
use harmos::storage::{ReadLimits, RecoveryPosition, StorageBackend, StorageBackends, StorageBatch, StorageManager, StorageSchema};

// Local Imports
use crate::evidence_support::{announce_and_wait, assert_counts, position, record_batch, record_schema};
use dokime_runtime::storage::{records, sqlite::Backend};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn managed_durability(days: usize, rows_per_day: usize) {
    assert!(rows_per_day > 1);
    let root = tempfile::tempdir().unwrap();
    let settings = settings();
    let (mut journal_baseline, mut peak_snapshot) = (None, 0);
    for day in 0..days {
        let store = data(root.path(), day == 0).await;
        let first = day * rows_per_day;
        let count = first + rows_per_day;
        store.write_records(StorageBatch::new(record_batch(&store, first, rows_per_day))).await.unwrap();
        let fired = Arc::new(AtomicU64::new(0));
        let armed = Arc::new(AtomicBool::new(false));
        let journal_bytes = Arc::new(AtomicU64::new(0));
        let (fault, failures, retained) = (armed.clone(), fired.clone(), journal_bytes.clone());
        let state = FileBackend::with_journal_opener(root.path().to_path_buf(), store.history().clone(), settings, move |path| {
            let file = std::fs::OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path)?;
            retained.store(file.metadata()?.len(), Ordering::Relaxed);
            if fault.swap(false, Ordering::AcqRel) {
                failures.fetch_add(1, Ordering::Relaxed);
                return Err(std::io::Error::other("fixture"));
            }
            Ok(file)
        });
        state.open().await.unwrap();
        state.write_entries(StorageBatch::new(entries(&store, &state, first, rows_per_day))).await.unwrap();
        let through = recovery(&state, count);
        state.flush(&through).await.unwrap();
        let previous = state.read_snapshot(settings.max_checkpoint_bytes).await.unwrap();
        let expected = checkpoint(&store, &state, count - 1);

        // Refusal Before Publication Leaves The Previous Checkpoint And Complete Tail.
        let mut oversized = expected.clone();
        oversized.payload = vec![0; settings.max_checkpoint_bytes.get()];
        assert!(state.write_snapshot(StorageBatch::new(vec![oversized])).await.is_err());
        assert_eq!(state.read_snapshot(settings.max_checkpoint_bytes).await.unwrap(), previous);
        let after = previous.as_ref().map_or_else(|| recovery(&state, 0), |snapshot| snapshot.through.clone());
        let mut after = after;
        let mut next = after.sequence + 1;
        loop {
            let page = state.read_entries(&after, &through, limits(64)).await.unwrap();
            assert!(page.entries.len() <= 64);
            for entry in &page.entries {
                assert_eq!(entry.position.sequence, next);
                assert_eq!(entry.required_data, position(&store, next + 1));
                assert_eq!(entry.payload, serde_json::to_vec(&(next as usize)).unwrap());
                next += 1;
            }
            let Some(position) = page.next else { break };
            after = position;
        }
        assert_eq!(next, count as u64 + 1);

        // The Opener Fails Only After Durable Checkpoint And Journal Replacement.
        armed.store(true, Ordering::Release);
        assert!(state.write_snapshot(StorageBatch::new(vec![expected.clone()])).await.is_err());
        assert_eq!(fired.load(Ordering::Relaxed), 1);
        let bytes = journal_bytes.load(Ordering::Relaxed);
        if let Some(first) = journal_baseline {
            assert!(bytes <= first + 64);
        } else {
            journal_baseline = Some(bytes);
        }
        drop(state);

        // Reopen Recovers The Published Checkpoint And Exactly One Uncovered Entry.
        let state = Arc::new(FileBackend::new(root.path().to_path_buf(), store.history().clone(), settings));
        assert_eq!(state.open().await.unwrap(), recovery(&state, count));
        let actual = state.read_snapshot(settings.max_checkpoint_bytes).await.unwrap().unwrap();
        assert_eq!(actual, expected);
        peak_snapshot = peak_snapshot.max(actual.payload.len());
        let maximum = serde_json::to_vec(&(usize::MAX, position(&store, u64::MAX))).unwrap().len();
        assert!(peak_snapshot <= maximum);
        let tail = state.read_entries(&actual.through, &through, limits(1)).await.unwrap();
        assert_eq!(tail.entries, entries(&store, &state, count - 1, 1));
        assert!(tail.next.is_none());
        state.write_snapshot(StorageBatch::new(vec![actual.clone()])).await.unwrap();
        assert_eq!(state.read_snapshot(settings.max_checkpoint_bytes).await.unwrap(), Some(actual.clone()));

        // The Consumer Refuses A Checkpoint That Would Refer To Unsaved Evidence.
        let manager = StorageManager::<()>::new(
            StorageBackends {
                primary: store.clone(),
                state: state.clone(),
            },
            dokime_runtime::config::storage_defaults(),
        )
        .unwrap();
        manager.acquire().await.unwrap();
        assert_eq!(&**manager.read_snapshot(settings.max_checkpoint_bytes).await.unwrap().as_ref().unwrap(), &actual);
        let mut future = actual.clone();
        future.required_data = position(&store, count as u64 + 2);
        assert!(matches!(manager.write_snapshot(future).await, Err(harmos::storage::Error::InvalidInput { .. })));
        assert_eq!(&**manager.read_snapshot(settings.max_checkpoint_bytes).await.unwrap().as_ref().unwrap(), &actual);
        assert_counts(&store, count).await;
        if day + 1 == days {
            // A Corrupt Stored Dependency Cannot Be Used To Restore Managed State.
            let mut invalid = checkpoint(&store, &state, count);
            invalid.required_data = position(&store, count as u64 + 2);
            state.write_snapshot(StorageBatch::new(vec![invalid])).await.unwrap();
            assert!(matches!(manager.read_snapshot(settings.max_checkpoint_bytes).await, Err(harmos::storage::Error::InvalidInput { .. })));
        }
        manager.close().await.unwrap();
    }
}

pub async fn checkpoint_crash_writer(root: std::path::PathBuf) {
    let store = data(&root, true).await;
    store.write_records(StorageBatch::new(record_batch(&store, 0, 3))).await.unwrap();
    let armed = Arc::new(AtomicBool::new(false));
    let fault = armed.clone();
    let state = FileBackend::with_journal_opener(root, store.history().clone(), settings(), move |path| {
        if fault.load(Ordering::Acquire) {
            announce_and_wait();
        }
        std::fs::OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path)
    });
    state.open().await.unwrap();
    state.write_entries(StorageBatch::new(entries(&store, &state, 0, 3))).await.unwrap();
    state.flush(&recovery(&state, 3)).await.unwrap();
    armed.store(true, Ordering::Release);
    state.write_snapshot(StorageBatch::new(vec![checkpoint(&store, &state, 2)])).await.unwrap();
    unreachable!();
}

pub async fn checkpoint_recovered(root: std::path::PathBuf) {
    let store = data(&root, false).await;
    assert_counts(&store, 3).await;
    assert_eq!(store.cursor_saved().await.unwrap(), position(&store, 4));
    let state = FileBackend::new(root, store.history().clone(), settings());
    let through = state.open().await.unwrap();
    assert_eq!(through, recovery(&state, 3));
    let snapshot = state.read_snapshot(settings().max_checkpoint_bytes).await.unwrap().unwrap();
    assert_eq!(snapshot, checkpoint(&store, &state, 2));
    let tail = state.read_entries(&snapshot.through, &through, limits(1)).await.unwrap();
    assert_eq!(tail.entries, entries(&store, &state, 2, 1));
    assert!(tail.next.is_none());
    state.write_snapshot(StorageBatch::new(vec![snapshot.clone()])).await.unwrap();
    assert_eq!(state.read_snapshot(settings().max_checkpoint_bytes).await.unwrap(), Some(snapshot));
    state.close().await.unwrap();
    store.close().await.unwrap();
}

/* === Private Functions === */

fn settings() -> harmos::config::StorageStateSettings {
    let mut settings = dokime_runtime::config::storage_state_defaults();
    settings.max_checkpoint_bytes = NonZeroUsize::new(16 * 1024).unwrap();
    settings
}

fn limits(rows: usize) -> ReadLimits {
    ReadLimits {
        rows: NonZeroUsize::new(rows).unwrap(),
        bytes: dokime_runtime::config::managed_storage_defaults().replay.bytes,
    }
}

async fn data(root: &Path, initialize: bool) -> Arc<Backend<()>> {
    let store = Arc::new(Backend::<()>::primary(root.to_path_buf()).await.unwrap());
    store.open().await.unwrap();
    if initialize {
        let schema = StorageSchema::new(position(&store, 1), records::schema(), record_schema());
        store.write_schemas(StorageBatch::new(vec![schema])).await.unwrap();
    }
    store
}

fn recovery(state: &FileBackend, sequence: usize) -> RecoveryPosition {
    RecoveryPosition {
        history: state.history().clone(),
        sequence: sequence as u64,
    }
}

fn checkpoint(store: &Backend<()>, state: &FileBackend, records: usize) -> RecoverySnapshot {
    let required_data = position(store, records as u64 + 1);
    let payload = serde_json::to_vec(&(records, &required_data)).unwrap();
    RecoverySnapshot {
        required_data,
        through: recovery(state, records),
        payload,
    }
}

fn entries(store: &Backend<()>, state: &FileBackend, first: usize, count: usize) -> Vec<RecoveryEntry> {
    (first + 1..=first + count)
        .map(|records| RecoveryEntry {
            required_data: position(store, records as u64 + 1),
            position: recovery(state, records),
            payload: serde_json::to_vec(&records).unwrap(),
        })
        .collect()
}

/* === Tests === */
