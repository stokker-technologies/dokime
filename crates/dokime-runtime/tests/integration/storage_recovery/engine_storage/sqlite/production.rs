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
use super::storage::Store;
use super::storage::id;

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
fn checkpoint(metadata: FileMetadata, file: &StagedFile, generation: u64, bytes: &[u8]) -> FileCheckpoint {
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
    ReadLimits { rows: bound(8), bytes: bound(32 * 1024) }
}

/* === Tests === */

pub(crate) async fn file_retry_checks_entire_overlap_before_append() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let original = (0..200_003).map(|index| (index % 251) as u8).collect::<Vec<_>>();
    let suffix = b"-tail";
    let conflicting = backend.create_file(&metadata("retry.conflict")).await.unwrap();
    backend.write_file(&conflicting, 0, store.batch(original.clone())).await.unwrap();
    backend.write_file(&conflicting, 0, store.batch(original.clone())).await.unwrap();
    let mut changed = original.clone();
    changed[150_001] ^= 1;
    changed.extend_from_slice(suffix);
    assert!(backend.write_file(&conflicting, 0, store.batch(changed)).await.is_err());
    let first = backend
        .finish_file(
            &conflicting,
            &StorageCursor {
                history: conflicting.history.clone(),
                sequence: 1,
            },
            original.len() as u64,
            Sha256::digest(&original).into(),
        )
        .await
        .unwrap();

    let matching = backend.create_file(&metadata("retry.append")).await.unwrap();
    backend.write_file(&matching, 0, store.batch(original.clone())).await.unwrap();
    let mut retry = original[17..].to_vec();
    retry.extend_from_slice(suffix);
    backend.write_file(&matching, 17, store.batch(retry)).await.unwrap();
    let mut appended = original.clone();
    appended.extend_from_slice(suffix);
    let second = backend
        .finish_file(
            &matching,
            &StorageCursor {
                history: matching.history.clone(),
                sequence: 2,
            },
            appended.len() as u64,
            Sha256::digest(&appended).into(),
        )
        .await
        .unwrap();
    backend.write_files(store.batch(vec![first.clone(), second.clone()])).await.unwrap();
    let snapshot = backend.open_snapshot(&backend.cursor_saved().await.unwrap()).await.unwrap();
    assert_eq!(backend.read_file(&snapshot, &first, 0, bound(original.len())).await.unwrap(), original);
    assert_eq!(backend.read_file(&snapshot, &second, 0, bound(appended.len())).await.unwrap(), appended);
    backend.release_snapshot(&snapshot).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn checkpoint_recovery_fence() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let metadata = metadata("production");
    let file = backend.create_production(&metadata).await.unwrap();
    assert!(backend.delete_file(&file).await.is_err());
    backend.write_file(&file, 0, store.batch(b"abcdef".to_vec())).await.unwrap();
    let checkpoint = checkpoint(metadata.clone(), &file, 1, b"abc");
    backend.save_production(store.batch(vec![checkpoint.clone()])).await.unwrap();
    backend.save_production(store.batch(vec![checkpoint.clone()])).await.unwrap();
    backend.fence_production(&file, 1, 2).await.unwrap();
    backend.fence_production(&file, 1, 2).await.unwrap();
    assert!(backend.save_production(store.batch(vec![checkpoint.clone()])).await.is_err());
    backend.close().await.unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let recovered = backend.recover_productions(limits()).await.unwrap();
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].session.generation, 2);
    assert_eq!(recovered[0].length, 3);
    assert_eq!(backend.read_production(&file, 0, bound(2)).await.unwrap(), b"ab");
    assert_eq!(backend.file_progress(&file).await.unwrap(), FileProgress { length: 3, phase: FilePhase::Producing });
    assert_eq!(backend.recover_productions(limits()).await.unwrap(), recovered);
    assert!(backend.fence_production(&file, 1, 3).await.is_err());
    backend.abandon_file(&file).await.unwrap();
    assert_eq!(backend.file_progress(&file).await.unwrap().phase, FilePhase::Abandoned);
    assert!(backend.create_production(&metadata).await.is_err());
    assert!(backend.write_file(&file, 3, store.batch(b"d".to_vec())).await.is_err());
    backend.close().await.unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    assert_eq!(backend.read_production(&file, 1, bound(8)).await.unwrap(), b"bc");
    backend.delete_file(&file).await.unwrap();
    backend.delete_file(&file).await.unwrap();
    assert!(backend.read_production(&file, 0, bound(1)).await.is_err());
    assert!(backend.create_file(&metadata).await.is_err());
    backend.close().await.unwrap();
}

pub(crate) async fn shared_sha_identity() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
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
        backend.write_file(file, 0, store.batch(b"same".to_vec())).await.unwrap();
        let position = StorageCursor {
            history: file.history.clone(),
            sequence: index as u64 + 1,
        };
        let append = MutationReceipt {
            mutation: MutationId {
                window: id("append"),
                key: file.id.clone(),
            },
            fingerprint: [index as u8; 32],
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            value: MutationValue::Producer(FileProducer {
                session: FileSession {
                    history: file.history.clone(),
                    id: file.id.clone(),
                    generation: 1,
                },
                length: 4,
            }),
        };
        assert!(backend.save_mutations(store.batch(vec![append.clone()])).await.is_err());
        let reference = backend.finish_file(file, &position, 4, hash).await.unwrap();
        backend.save_mutations(store.batch(vec![append.clone()])).await.unwrap();
        for (window, length, generation) in [("past-end", 5, 1), ("future-owner", 4, 2)] {
            let mut invalid = append.clone();
            invalid.mutation.window = id(window);
            let MutationValue::Producer(producer) = &mut invalid.value else { unreachable!() };
            producer.length = length;
            producer.session.generation = generation;
            assert!(backend.save_mutations(store.batch(vec![invalid])).await.is_err());
        }
        let receipt = MutationReceipt {
            mutation: MutationId {
                window: id("capture"),
                key: file.id.clone(),
            },
            fingerprint: [index as u8; 32],
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            value: MutationValue::Reference(Box::new(reference.clone())),
        };
        assert!(backend.save_mutations(store.batch(vec![receipt.clone()])).await.is_err());
        backend.write_files(store.batch(vec![reference.clone()])).await.unwrap();
        backend.save_mutations(store.batch(vec![receipt.clone()])).await.unwrap();
        refs.push(reference);
    }
    let physical = std::fs::read_dir(root.path().join("files")).unwrap().map(|value| value.unwrap().path()).collect::<Vec<_>>();
    assert_eq!(physical.len(), 1);
    let name = physical[0].file_name().unwrap().to_string_lossy();
    assert_eq!(name.len(), 64);
    assert!(name.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    let fixed = backend.open_snapshot(&backend.cursor_saved().await.unwrap()).await.unwrap();
    backend.delete_file(&a).await.unwrap();
    assert!(physical[0].is_file());
    assert!(backend.read_file(&fixed, &refs[0], 0, bound(4)).await.is_err());
    assert_eq!(backend.read_file(&fixed, &refs[1], 0, bound(4)).await.unwrap(), b"same");
    let page = backend
        .read_files(
            &fixed,
            &query::FileSelection { scope: None, item: None },
            ScanPage {
                after: 0,
                rows: bound(8),
                bytes: bound(32 * 1024),
            },
        )
        .await
        .unwrap();
    assert_eq!(page, [refs[1].clone()]);
    backend.release_snapshot(&fixed).await.unwrap();
    backend.delete_file(&b).await.unwrap();
    assert!(!physical[0].exists());
    backend.close().await.unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    assert!(backend.create_file(&first).await.is_err());
    backend.close().await.unwrap();
}

pub(crate) async fn mutation_receipts_outcomes() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let receipt = MutationReceipt {
        mutation: MutationId { window: id("window"), key: id("save") },
        fingerprint: [3; 32],
        expires_at: SystemTime::now() + Duration::from_secs(60),
        value: MutationValue::Cursor(backend.cursor_saved().await.unwrap()),
    };
    backend.save_mutations(store.batch(vec![receipt.clone()])).await.unwrap();
    backend.save_mutations(store.batch(vec![receipt.clone()])).await.unwrap();
    let mut changed = receipt.clone();
    changed.fingerprint[0] = 4;
    assert!(backend.save_mutations(store.batch(vec![changed])).await.is_err());
    let newer = MutationReceipt {
        mutation: MutationId { window: id("window"), key: id("later") },
        expires_at: receipt.expires_at + Duration::from_secs(60),
        ..receipt.clone()
    };
    backend.save_mutations(store.batch(vec![newer.clone()])).await.unwrap();
    // Recovery Keeps The Newest Receipts That Fit Its Bounds, Omitting Older Ones.
    assert!(backend.recover_mutations(ReadLimits { bytes: bound(1), ..limits() }).await.unwrap().is_empty());
    assert_eq!(backend.recover_mutations(ReadLimits { rows: bound(1), ..limits() }).await.unwrap(), std::slice::from_ref(&newer));
    backend.close().await.unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    assert_eq!(backend.recover_mutations(limits()).await.unwrap(), [newer, receipt]);
    backend.close().await.unwrap();
}

pub(crate) async fn mutation_receipts_admit_large_values_and_preserve_atomicity() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let saved = MutationReceipt {
        mutation: MutationId { window: id("window"), key: id("saved") },
        fingerprint: [3; 32],
        expires_at: SystemTime::now() + Duration::from_secs(60),
        value: MutationValue::Cursor(backend.cursor_saved().await.unwrap()),
    };
    backend.save_mutations(store.batch(vec![saved.clone()])).await.unwrap();

    let mut metadata = metadata("large.receipt");
    metadata.media_type = "x".repeat(32 * 1024);
    let file = backend.create_production(&metadata).await.unwrap();
    let position = StorageCursor {
        history: file.history.clone(),
        sequence: 1,
    };
    let reference = backend.finish_file(&file, &position, 0, Sha256::digest([]).into()).await.unwrap();
    backend.write_files(store.batch(vec![reference.clone()])).await.unwrap();
    let large = MutationReceipt {
        mutation: MutationId { window: id("window"), key: id("large") },
        fingerprint: [4; 32],
        expires_at: SystemTime::now() + Duration::from_secs(120),
        value: MutationValue::Reference(Box::new(reference)),
    };
    backend.save_mutations(store.batch(vec![saved.clone(), large.clone()])).await.unwrap();
    backend.save_mutations(store.batch(vec![large.clone()])).await.unwrap();

    // A Later Conflict Rolls Back The Earlier New Receipt In The Same Batch.
    let pending = MutationReceipt {
        mutation: MutationId { window: id("window"), key: id("pending") },
        ..saved.clone()
    };
    let mut conflict = large.clone();
    conflict.fingerprint[0] ^= 1;
    assert!(backend.save_mutations(store.batch(vec![pending, conflict])).await.is_err());
    let recovered = backend.recover_mutations(ReadLimits { rows: bound(8), bytes: bound(128 * 1024) }).await.unwrap();
    let expected = [large, saved];
    assert_eq!(recovered, expected);
    backend.close().await.unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    assert!(backend.recover_mutations(limits()).await.unwrap().is_empty());
    assert_eq!(backend.recover_mutations(ReadLimits { rows: bound(8), bytes: bound(128 * 1024) }).await.unwrap(), expected);
    backend.close().await.unwrap();
}

pub(crate) async fn checkpoints_advance_boundaries() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let metadata = metadata("boundaries");
    let file = backend.create_production(&metadata).await.unwrap();
    let bytes = (0..129).collect::<Vec<u8>>();
    let mut offset = 0;
    for length in [63, 64, 65, 129] {
        backend.write_file(&file, offset as u64, store.batch(bytes[offset..length].to_vec())).await.unwrap();
        let value = checkpoint(metadata.clone(), &file, 1, &bytes[..length]);
        backend.save_production(store.batch(vec![value.clone()])).await.unwrap();
        backend.save_production(store.batch(vec![value.clone()])).await.unwrap();
        offset = length;
    }
    backend.write_file(&file, bytes.len() as u64, store.batch(b"unconfirmed tail".to_vec())).await.unwrap();
    backend.close().await.unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let recovered = backend.recover_productions(limits()).await.unwrap();
    assert_eq!(recovered, [checkpoint(metadata, &file, 1, &bytes)]);
    assert_eq!(backend.read_production(&file, 0, bound(129)).await.unwrap(), bytes);
    backend.close().await.unwrap();
}

pub(crate) async fn late_checkpoints_settle() {
    let root = tempfile::tempdir().unwrap();
    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let bytes = b"abcdef";

    let metadata = metadata("late-longer");
    let file = backend.create_production(&metadata).await.unwrap();
    backend.write_file(&file, 0, store.batch(bytes.to_vec())).await.unwrap();
    let longer = checkpoint(metadata.clone(), &file, 1, bytes);
    let shorter = checkpoint(metadata.clone(), &file, 1, &bytes[..3]);
    backend.save_production(store.batch(vec![longer.clone()])).await.unwrap();
    backend.save_production(store.batch(vec![shorter.clone()])).await.unwrap();
    let other = checkpoint(self::metadata("late-other"), &file, 1, bytes);
    assert!(backend.save_production(store.batch(vec![other.clone()])).await.is_err());

    let finished_metadata = self::metadata("late-finished");
    let finished = backend.create_production(&finished_metadata).await.unwrap();
    backend.write_file(&finished, 0, store.batch(bytes.to_vec())).await.unwrap();
    let staged = checkpoint(finished_metadata.clone(), &finished, 1, &bytes[..3]);
    let position = StorageCursor {
        history: finished.history.clone(),
        sequence: 1,
    };
    backend.finish_file(&finished, &position, 6, Sha256::digest(bytes).into()).await.unwrap();
    backend.save_production(store.batch(vec![staged.clone()])).await.unwrap();
    backend.save_production(store.batch(vec![checkpoint(finished_metadata.clone(), &finished, 1, bytes)])).await.unwrap();
    let beyond = checkpoint(finished_metadata.clone(), &finished, 1, b"abcdefg");
    assert!(backend.save_production(store.batch(vec![beyond.clone()])).await.is_err());
    let fenced = checkpoint(finished_metadata, &finished, 2, &bytes[..3]);
    assert!(backend.save_production(store.batch(vec![fenced.clone()])).await.is_err());

    let abandoned_metadata = self::metadata("late-abandoned");
    let abandoned = backend.create_production(&abandoned_metadata).await.unwrap();
    backend.write_file(&abandoned, 0, store.batch(bytes.to_vec())).await.unwrap();
    backend.abandon_file(&abandoned).await.unwrap();
    let staged = checkpoint(abandoned_metadata, &abandoned, 1, bytes);
    backend.save_production(store.batch(vec![staged.clone()])).await.unwrap();
    backend.delete_file(&abandoned).await.unwrap();
    backend.save_production(store.batch(vec![staged.clone()])).await.unwrap();
    backend.close().await.unwrap();

    let store = Store::<()>::open(root.path()).await;
    let backend = &store.backend;
    let recovered = backend.recover_productions(limits()).await.unwrap();
    assert_eq!(recovered, [longer]);
    backend.close().await.unwrap();
}
