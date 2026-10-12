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
    num::NonZeroUsize,
    time::{Duration, SystemTime},
};

// Third Party Imports
use harmos::storage::*;
use sha2::{Digest, Sha256, digest::common::hazmat::SerializableState};

// Local Imports
use super::storage::id;
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
fn metadata(name: &str) -> FileMetadata {
    FileMetadata {
        id: id(name),
        scope: None,
        media_type: "text/plain".into(),
        expected_length: None,
        expected_sha256: None,
    }
}

fn bound(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}
fn checkpoint(
    metadata: FileMetadata,
    file: &StagedFile,
    generation: u64,
    bytes: &[u8],
) -> FileCheckpoint {
    let hash = Sha256::new().chain_update(bytes);
    FileCheckpoint {
        metadata,
        session: FileSession {
            history: file.history.clone(),
            id: file.id.clone(),
            generation,
        },
        length: bytes.len() as u64,
        hash_format: 1,
        hash_state: hash.serialize().to_vec(),
    }
}

fn limits() -> ReadLimits {
    ReadLimits {
        rows: bound(8),
        bytes: bound(32 * 1024),
    }
}

/* === Tests === */

pub(crate) async fn checkpoint_recovery_fence() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let metadata = metadata("production");
    let file = backend.create_production(&metadata).await.unwrap();
    assert!(backend.delete_file(&file).await.is_err());
    backend.write_file(&file, 0, b"abcdef").await.unwrap();
    let checkpoint = checkpoint(metadata.clone(), &file, 1, b"abc");
    backend.save_production(&checkpoint).await.unwrap();
    backend.save_production(&checkpoint).await.unwrap();
    backend.fence_production(&file, 1, 2).await.unwrap();
    backend.fence_production(&file, 1, 2).await.unwrap();
    assert!(backend.save_production(&checkpoint).await.is_err());
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let recovered = backend.recover_productions(limits()).await.unwrap();
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].session.generation, 2);
    assert_eq!(recovered[0].length, 3);
    assert_eq!(
        backend.read_production(&file, 0, bound(2)).await.unwrap(),
        b"ab"
    );
    assert_eq!(
        backend.file_progress(&file).await.unwrap(),
        FileProgress {
            length: 3,
            phase: FilePhase::Producing
        }
    );
    assert_eq!(
        backend.recover_productions(limits()).await.unwrap(),
        recovered
    );
    assert!(backend.fence_production(&file, 1, 3).await.is_err());
    backend.abandon_file(&file).await.unwrap();
    assert_eq!(
        backend.file_progress(&file).await.unwrap().phase,
        FilePhase::Abandoned
    );
    assert!(backend.create_production(&metadata).await.is_err());
    assert!(backend.write_file(&file, 3, b"d").await.is_err());
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    assert_eq!(
        backend.read_production(&file, 1, bound(8)).await.unwrap(),
        b"bc"
    );
    backend.delete_file(&file).await.unwrap();
    backend.delete_file(&file).await.unwrap();
    assert!(backend.read_production(&file, 0, bound(1)).await.is_err());
    assert!(backend.create_file(&metadata).await.is_err());
    backend.close().await.unwrap();
}

pub(crate) async fn shared_sha_identity() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let first = metadata("first");
    let second = metadata("second");
    let a = backend.create_production(&first).await.unwrap();
    let b = backend.create_production(&second).await.unwrap();
    let names = std::fs::read_dir(root.path().join("staging"))
        .unwrap()
        .map(|value| value.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 2);
    assert!(names.iter().all(|name| uuid::Uuid::parse_str(name).is_ok()));
    let hash = Sha256::digest(b"same").into();
    let mut refs = Vec::new();
    for (index, file) in [&a, &b].into_iter().enumerate() {
        backend.write_file(file, 0, b"same").await.unwrap();
        let position = StorageCursor {
            history: file.history.clone(),
            sequence: index as u64 + 1,
        };
        let reference = backend.finish_file(file, &position, 4, hash).await.unwrap();
        let receipt = MutationReceipt {
            mutation: MutationId {
                window: id("capture"),
                key: file.id.clone(),
            },
            fingerprint: [index as u8; 32],
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            value: MutationValue::Reference(Box::new(reference.clone())),
        };
        assert!(
            backend
                .save_mutations(std::slice::from_ref(&receipt))
                .await
                .is_err()
        );
        backend
            .write(
                StorageBatch::new(
                    &[StoragePart::Files(std::slice::from_ref(&reference))],
                    bound(4096),
                )
                .unwrap(),
            )
            .await
            .unwrap();
        backend
            .save_mutations(std::slice::from_ref(&receipt))
            .await
            .unwrap();
        refs.push(reference);
    }
    let physical = std::fs::read_dir(root.path().join("files"))
        .unwrap()
        .map(|value| value.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(physical.len(), 1);
    let name = physical[0].file_name().unwrap().to_string_lossy();
    assert_eq!(name.len(), 64);
    assert!(
        name.bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    let snapshot = backend
        .read_view(&backend.cursor_saved().await.unwrap())
        .await
        .unwrap();
    let fixed = ReadView::new(
        id("fixed"),
        snapshot.saved.clone(),
        snapshot.saved.clone(),
        snapshot.clone(),
    )
    .unwrap();
    backend.delete_file(&a).await.unwrap();
    assert!(physical[0].is_file());
    assert!(
        backend
            .read_file(&fixed, &refs[0], 0, bound(4))
            .await
            .is_err()
    );
    assert_eq!(
        backend
            .read_file(&fixed, &refs[1], 0, bound(4))
            .await
            .unwrap(),
        b"same"
    );
    let page = backend
        .read_files(&fixed, &ReadSelection::new(ReadKind::Files), None, limits())
        .await
        .unwrap();
    assert_eq!(page.entries, [refs[1].clone()]);
    backend.release_view(&snapshot).await.unwrap();
    let retained = backend
        .read_retained_view(
            &backend.cursor_saved().await.unwrap(),
            StorageSettings::default().query_execution,
        )
        .await
        .unwrap();
    assert!(retained.coverage.complete_after >= 1);
    backend.release_view(&retained.snapshot).await.unwrap();
    backend.delete_file(&b).await.unwrap();
    assert!(!physical[0].exists());
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    assert!(backend.create_file(&first).await.is_err());
    backend.close().await.unwrap();
}

pub(crate) async fn mutation_receipts_outcomes() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let receipt = MutationReceipt {
        mutation: MutationId {
            window: id("window"),
            key: id("save"),
        },
        fingerprint: [3; 32],
        expires_at: SystemTime::now() + Duration::from_secs(60),
        value: MutationValue::Cursor(backend.cursor_saved().await.unwrap()),
    };
    backend
        .save_mutations(std::slice::from_ref(&receipt))
        .await
        .unwrap();
    backend
        .save_mutations(std::slice::from_ref(&receipt))
        .await
        .unwrap();
    let mut changed = receipt.clone();
    changed.fingerprint[0] = 4;
    assert!(backend.save_mutations(&[changed]).await.is_err());
    assert!(
        backend
            .recover_mutations(ReadLimits {
                bytes: bound(1),
                ..limits()
            })
            .await
            .is_err()
    );
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    assert_eq!(
        backend.recover_mutations(limits()).await.unwrap(),
        [receipt]
    );
    backend.close().await.unwrap();
}

pub(crate) async fn checkpoints_advance_boundaries() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let metadata = metadata("boundaries");
    let file = backend.create_production(&metadata).await.unwrap();
    let bytes = (0..129).collect::<Vec<u8>>();
    backend.write_file(&file, 0, &bytes).await.unwrap();
    for length in [63, 64, 65, 129] {
        let value = checkpoint(metadata.clone(), &file, 1, &bytes[..length]);
        backend.save_production(&value).await.unwrap();
        backend.save_production(&value).await.unwrap();
    }
    backend.close().await.unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let recovered = backend.recover_productions(limits()).await.unwrap();
    assert_eq!(recovered, [checkpoint(metadata, &file, 1, &bytes)]);
    assert_eq!(
        backend.read_production(&file, 0, bound(129)).await.unwrap(),
        bytes
    );
    backend.close().await.unwrap();
}

pub(crate) async fn recovery_charges_limits() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(
        root.path().to_path_buf(),
        Limits {
            memory_bytes: bound(4 * 1024 * 1024),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let limits = ReadLimits {
        rows: bound(8),
        bytes: bound(32 * 1024 * 1024),
    };
    assert!(
        backend
            .recover_productions(limits)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(backend.recover_mutations(limits).await.unwrap().is_empty());
    let metadata = metadata("small");
    let file = backend.create_production(&metadata).await.unwrap();
    let receipt = MutationReceipt {
        mutation: MutationId {
            window: id("window"),
            key: id("small"),
        },
        fingerprint: [1; 32],
        expires_at: SystemTime::now() + Duration::from_secs(60),
        value: MutationValue::Producer(FileProducer {
            session: FileSession {
                history: file.history.clone(),
                id: file.id.clone(),
                generation: 1,
            },
            length: 0,
        }),
    };
    backend
        .save_mutations(std::slice::from_ref(&receipt))
        .await
        .unwrap();
    assert_eq!(backend.recover_mutations(limits).await.unwrap(), [receipt]);
    assert_eq!(
        backend.recover_productions(limits).await.unwrap(),
        [checkpoint(metadata, &file, 1, b"")]
    );
    backend.close().await.unwrap();
}
