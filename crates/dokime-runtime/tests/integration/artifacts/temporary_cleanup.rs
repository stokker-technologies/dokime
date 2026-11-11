/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Artifact Scratch Ownership Across Clones And Unwinding

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime_runtime::{
    artifacts::{ArtifactSink, Artifacts},
    transport::ArtifactChunk,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Sink;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ArtifactSink for Sink {
    fn push(&self, _: ArtifactChunk) {}
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn temporary_artifacts_cleanup_without_sweep() {
    for failure in [false, true] {
        let artifacts = Artifacts::new("temporary-owner", Arc::new(Sink));
        let root = artifacts.root().unwrap();
        let area = root.parent().unwrap().to_owned();
        std::fs::write(root.join("evidence.bin"), b"captured evidence").unwrap();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _owner = artifacts;
            assert!(!failure, "injected artifact fixture failure");
        }));
        assert_eq!(outcome.is_err(), failure);
        assert!(!area.exists(), "artifact scratch survived owner teardown: {}", area.display());
    }
}

#[test]
fn temporary_artifacts_follow_the_last_clone_and_isolate_equal_tokens() {
    let first = Artifacts::new("same-token", Arc::new(Sink));
    let second = Artifacts::new("same-token", Arc::new(Sink));
    let first_root = first.root().unwrap();
    let second_root = second.root().unwrap();
    assert_ne!(first_root, second_root);
    let retained = first.clone();
    drop(first);
    assert!(first_root.exists());
    drop(retained);
    assert!(!first_root.exists());
    assert!(second_root.exists());
    drop(second);
    assert!(!second_root.exists());
}
