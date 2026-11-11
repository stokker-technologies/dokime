/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Persistent State Compatibility

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::storage::{self, sqlite::State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn obsolete_evidence_digest_does_not_invalidate_a_root() {
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("storage-format.json");
    let bytes = br#"{"format":"old-evidence-codec","writer_build":"old-build"}"#;
    std::fs::write(&marker, bytes).unwrap();
    let state = State::open(root.path()).await.unwrap();
    state.close().await.unwrap();
    let state = State::open(root.path()).await.unwrap();
    state.close().await.unwrap();
    assert_eq!(std::fs::read(marker).unwrap(), bytes);
}

#[tokio::test]
async fn persistent_state_keeps_exclusive_root_ownership() {
    let root = tempfile::tempdir().unwrap();
    let first = State::open(root.path()).await.unwrap();
    assert!(matches!(State::open(root.path()).await, Err(storage::Error::Owned)));
    first.close().await.unwrap();
    State::open(root.path()).await.unwrap().close().await.unwrap();
}

/* === Tests === */
