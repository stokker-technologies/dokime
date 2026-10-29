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

// Third Party Imports
use harmos::storage::*;

// Local Imports
use super::sqlite::storage::Store;

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
pub(crate) async fn cancellation() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    backend.write_schemas(store.batch(vec![store.schema()])).await.unwrap();

    // Hold The Real SQLite Worker At A Deterministic Boundary
    let control = backend.local_state();
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
    let records = (2..66).map(|sequence| store.record(sequence)).collect::<Vec<_>>();
    let batch = store.batch(records.clone());
    let mut writing = backend.write_records(batch.clone());
    assert!(futures_util::poll!(writing.as_mut()).is_pending());
    drop(writing);
    release_tx.send(()).unwrap();
    blocker.await.unwrap();

    // Cancellation Before Dispatch Leaves No Hidden Write To Overtake A Retry
    assert_eq!(backend.cursor_saved().await.unwrap(), store.at(1));
    assert!(store.records(1).await.is_empty());
    let saved = backend.write_records(batch).await.unwrap();
    assert_eq!(saved, store.at(65));
    assert_eq!(store.records(65).await, records);
    backend.close().await.unwrap();
    drop(store);
    let store = Store::<()>::open(root.path()).await;
    assert_eq!(store.backend.cursor_saved().await.unwrap(), saved);
    assert_eq!(store.records(65).await, records);
    store.backend.close().await.unwrap();
}
