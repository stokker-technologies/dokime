/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage cancellation Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::*;

// Local Imports
use super::sqlite::storage::{history, position, read_all, record, schema, view, write_batch};
use dokime::storage::sqlite::{Backend, Limits};

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
#[tokio::test]
async fn cancellation() {
    let root = tempfile::tempdir().unwrap();
    let settings = Limits {
        entry_bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
        batch_bytes: NonZeroUsize::new(16 * 1024 * 1024).unwrap(),

        memory_bytes: NonZeroUsize::new(32 * 1024 * 1024).unwrap(),
    };
    let backend = Backend::<()>::replica(root.path().to_owned(), settings, history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();

    // Hold The Real SQLite Worker At A Deterministic Boundary
    let control = backend.control().await.unwrap();
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let blocker = tokio::spawn(async move {
        control
            .call(move |_| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
            .await
            .unwrap();
    });
    entered_rx.await.unwrap();
    let records = (2..66).map(record).collect::<Vec<_>>();
    let parts = [StoragePart::Records(&records)];
    let mut writing = backend.write(write_batch(&parts));
    assert!(futures_util::poll!(writing.as_mut()).is_pending());
    drop(writing);
    release_tx.send(()).unwrap();
    blocker.await.unwrap();

    // Cancellation Before Dispatch Leaves No Hidden Write To Overtake A Retry
    assert_eq!(backend.cursor_saved().await.unwrap(), position(1));
    let unchanged = view(&backend, 1).await;
    assert!(read_all(&backend, &unchanged).await.is_empty());
    backend.release_view(unchanged.snapshot()).await.unwrap();
    let saved = backend.write(write_batch(&parts)).await.unwrap();
    assert_eq!(saved, position(65));
    let current = view(&backend, 65).await;
    assert_eq!(read_all(&backend, &current).await, records);
    backend.release_view(current.snapshot()).await.unwrap();
    backend.close().await.unwrap();
    let backend = Backend::<()>::replica(root.path().to_owned(), settings, history())
        .await
        .unwrap();
    assert_eq!(backend.open().await.unwrap(), saved);
    let current = view(&backend, 65).await;
    assert_eq!(read_all(&backend, &current).await, records);
    backend.release_view(current.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}
