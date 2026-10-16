/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage exhaustion Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::*;

// Local Imports
use super::sqlite::storage::{history, id, position, read_all, record, schema, view, write_batch};
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
async fn exhaustion() {
    let root = tempfile::tempdir().unwrap();
    let settings = Limits {
        entry_bytes: NonZeroUsize::new(1024).unwrap(),
        batch_bytes: NonZeroUsize::new(4096).unwrap(),
    };
    let backend = Backend::<()>::replica(root.path().to_owned(), settings, history())
        .await
        .unwrap();
    backend.open().await.unwrap();
    backend
        .write(write_batch(&[StoragePart::Schemas(&[schema()])]))
        .await
        .unwrap();

    // Reject The Whole Batch Before Any Valid Prefix Becomes Durable
    let mut oversized = record(3);
    oversized.payload = vec![0; 2048];
    let rejected = [record(2), oversized];
    assert!(
        backend
            .write(write_batch(&[StoragePart::Records(&rejected)]))
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), position(1));
    let accepted = [record(2), record(3)];
    backend
        .write(write_batch(&[StoragePart::Records(&accepted)]))
        .await
        .unwrap();

    // Independent Views Preserve The Same Saved Frontier
    let saved = backend.cursor_saved().await.unwrap();
    let snapshot = backend.read_view(&saved).await.unwrap();
    let parallel = backend.read_view(&saved).await.unwrap();
    backend.release_view(&parallel).await.unwrap();
    let current = ReadView::new(id("exhaustion"), saved.clone(), saved.clone(), snapshot).unwrap();
    let tiny = ReadLimits {
        rows: NonZeroUsize::MIN,
        bytes: NonZeroUsize::MIN,
    };
    assert!(
        backend
            .read_records(&current, &ReadSelection::new(ReadKind::Records), None, tiny)
            .await
            .is_err()
    );
    assert_eq!(read_all(&backend, &current).await, accepted);
    backend.release_view(current.snapshot()).await.unwrap();
    let replacement = backend.read_view(&saved).await.unwrap();
    backend.release_view(&replacement).await.unwrap();
    backend.close().await.unwrap();

    let backend = Backend::<()>::replica(root.path().to_owned(), settings, history())
        .await
        .unwrap();
    assert_eq!(backend.open().await.unwrap(), saved);
    let current = view(&backend, 3).await;
    assert_eq!(read_all(&backend, &current).await, accepted);
    backend.release_view(current.snapshot()).await.unwrap();
    backend.close().await.unwrap();
}
