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
    num::{NonZeroU64, NonZeroUsize},
    time::Duration,
};

// Third Party Imports
use harmos::storage::{
    recovery::{RecoveryBackend, RecoveryEntry},
    *,
};
use sha2::{Digest, Sha256};

// Local Imports
use dokime::storage::sqlite::{Backend, Limits, RecoveryLimits};

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

pub(crate) async fn private_file_bytes() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let prior = backend.cursor_saved().await.unwrap();
    let position = StorageCursor {
        history: prior.history.clone(),
        sequence: 1,
    };
    let coverage = RetainedCoverage {
        cut: position.clone(),
        complete_after: 0,
        retention_generation: NonZeroU64::MIN,
    };
    let identity = BootstrapId::new(&coverage).unwrap();
    backend
        .begin_bootstrap(
            &identity,
            &prior,
            StorageSettings::default().query_execution,
        )
        .await
        .unwrap();
    let digest: [u8; 32] = Sha256::digest(b"abc").into();
    let metadata = FileMetadata {
        id: harmos::Identifier::new("private.retry").unwrap(),
        scope: None,
        media_type: "text/plain".into(),
        expected_length: Some(3),
        expected_sha256: Some(digest),
    };
    let stage = backend
        .create_bootstrap_file(&identity, &metadata)
        .await
        .unwrap();
    backend
        .write_bootstrap_file(&identity, &stage, 0, b"ab")
        .await
        .unwrap();
    assert!(
        backend
            .write_bootstrap_file(&identity, &stage, 0, b"ax")
            .await
            .is_err()
    );
    assert!(
        backend
            .write_bootstrap_file(&identity, &stage, 3, b"c")
            .await
            .is_err()
    );
    backend
        .write_bootstrap_file(&identity, &stage, 0, b"ab")
        .await
        .unwrap();
    backend
        .write_bootstrap_file(&identity, &stage, 2, b"c")
        .await
        .unwrap();
    let other = BootstrapId::new(&RetainedCoverage {
        retention_generation: NonZeroU64::new(2).unwrap(),
        ..coverage
    })
    .unwrap();
    assert!(backend.abort_bootstrap_file(&other, &stage).await.is_err());
    assert!(
        backend
            .finish_bootstrap_file(&identity, &stage, &position, 3, [0; 32])
            .await
            .is_err()
    );
    let reference = backend
        .finish_bootstrap_file(&identity, &stage, &position, 3, digest)
        .await
        .unwrap();
    assert_eq!(
        reference,
        backend
            .finish_bootstrap_file(&identity, &stage, &position, 3, digest)
            .await
            .unwrap()
    );
    backend
        .write_bootstrap_file(&identity, &stage, 0, b"abc")
        .await
        .unwrap();
    assert!(
        backend
            .write_bootstrap_file(&identity, &stage, 0, b"abd")
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), prior);
    backend
        .abort_bootstrap_file(&identity, &stage)
        .await
        .unwrap();
    backend
        .abort_bootstrap_file(&identity, &stage)
        .await
        .unwrap();
    assert!(
        backend
            .finish_bootstrap_file(&identity, &stage, &position, 3, digest)
            .await
            .is_err()
    );
    backend.abort_bootstrap(&identity).await.unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap(), prior);
    backend.close().await.unwrap();
}

pub(crate) async fn private_file_restart() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let prior = backend.cursor_saved().await.unwrap();
    let identity = BootstrapId::new(&RetainedCoverage {
        cut: StorageCursor {
            history: prior.history.clone(),
            sequence: 1,
        },
        complete_after: 0,
        retention_generation: NonZeroU64::MIN,
    })
    .unwrap();
    let limits = StorageSettings::default().query_execution;
    let metadata = FileMetadata {
        id: harmos::Identifier::new("same.file").unwrap(),
        scope: None,
        media_type: "text/plain".into(),
        expected_length: Some(3),
        expected_sha256: None,
    };
    let ordinary = backend.create_file(&metadata).await.unwrap();
    backend.write_file(&ordinary, 0, b"old").await.unwrap();
    backend
        .begin_bootstrap(&identity, &prior, limits)
        .await
        .unwrap();
    let staged = backend
        .create_bootstrap_file(&identity, &metadata)
        .await
        .unwrap();
    let mut empty = BootstrapManifest::new(identity.clone());
    for kind in [
        ReadKind::Schemas,
        ReadKind::Objects,
        ReadKind::Records,
        ReadKind::Stream,
        ReadKind::Files,
    ] {
        empty.finish_collection(kind);
    }
    assert!(
        backend
            .finish_bootstrap(&empty)
            .await
            .unwrap_err()
            .to_string()
            .contains("private import has undeclared file stages")
    );
    assert_eq!(
        backend
            .create_bootstrap_file(&identity, &metadata)
            .await
            .unwrap(),
        staged
    );
    let mut changed = metadata.clone();
    changed.media_type = "text/html".into();
    assert!(
        backend
            .create_bootstrap_file(&identity, &changed)
            .await
            .is_err()
    );
    let stage_directory = root.path().join("staging");
    // The ordinary file's open stage shares the directory. Stage names are
    // time-ordered, and the private stage was created last; directory order
    // is the filesystem's and says nothing.
    let private = std::fs::read_dir(&stage_directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .max()
        .unwrap();
    assert_eq!(std::fs::metadata(&private).unwrap().len(), 0);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let name = uuid::Uuid::now_v7().to_string();
        let installed = stage_directory.join(&name);
        let owner: String = Sha256::digest(serde_json::to_vec(&identity).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let file_id = staged.id.to_string();
        backend
            .control()
            .await
            .unwrap()
            .call(move |connection| {
                connection.execute(
                    "INSERT INTO storage_staging VALUES (?1,?2,'sealed',?3)",
                    rusqlite::params![owner, file_id, name],
                )
            })
            .await
            .unwrap();
        std::fs::write(&installed, b"abc").unwrap();
        let permissions = std::fs::metadata(&stage_directory).unwrap().permissions();
        std::fs::set_permissions(&stage_directory, std::fs::Permissions::from_mode(0o300)).unwrap();
        // Privileged Test Hosts May Still Open A Directory Without Read Permission
        if std::fs::File::open(&stage_directory).is_err() {
            assert!(backend.abort_bootstrap(&identity).await.is_err());
            assert_eq!(std::fs::read(&installed).unwrap(), b"abc");
            assert!(private.is_file());
        }
        std::fs::set_permissions(&stage_directory, permissions).unwrap();
    }
    std::fs::remove_file(&private).unwrap();
    std::fs::create_dir(&private).unwrap();
    assert!(backend.abort_bootstrap(&identity).await.is_err());
    assert!(
        backend
            .create_bootstrap_file(&identity, &metadata)
            .await
            .is_err()
    );
    backend.close().await.unwrap();
    drop(backend);

    let reopened = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    assert_eq!(reopened.cursor_saved().await.unwrap(), prior);
    assert!(
        reopened
            .begin_bootstrap(&identity, &prior, limits)
            .await
            .is_err()
    );
    let control = reopened.control().await.unwrap();
    control
        .call(|connection| {
            let files: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_files",
                [],
                |row| row.get(0),
            )?;
            let pending: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_abort",
                [],
                |row| row.get(0),
            )?;
            assert_eq!((files, pending), (1, 1));
            Ok(())
        })
        .await
        .unwrap();
    drop(control);
    std::fs::remove_dir(&private).unwrap();
    reopened.abort_bootstrap(&identity).await.unwrap();
    reopened.abort_bootstrap(&identity).await.unwrap();
    // The Ordinary Stage With The Same Logical ID Is A Separate Owner
    reopened.write_file(&ordinary, 0, b"old").await.unwrap();
    reopened.close().await.unwrap();
}

pub(crate) async fn private_import_ownership() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let prior = backend.cursor_saved().await.unwrap();
    let position = StorageCursor {
        history: prior.history.clone(),
        sequence: 1,
    };
    let identity = BootstrapId::new(&RetainedCoverage {
        cut: position.clone(),
        complete_after: 0,
        retention_generation: NonZeroU64::MIN,
    })
    .unwrap();
    let limits = StorageSettings::default().query_execution;
    let metadata = FileMetadata {
        id: harmos::Identifier::new("ordinary.owner").unwrap(),
        scope: None,
        media_type: "text/plain".into(),
        expected_length: Some(3),
        expected_sha256: None,
    };
    let ordinary = backend.create_file(&metadata).await.unwrap();
    backend.write_file(&ordinary, 0, b"old").await.unwrap();

    for (bytes, completes) in [(b"new", false), (b"old", true)] {
        let saved = backend.cursor_saved().await.unwrap();
        backend
            .begin_bootstrap(&identity, &saved, limits)
            .await
            .unwrap();
        let private = backend
            .create_bootstrap_file(&identity, &metadata)
            .await
            .unwrap();
        backend
            .write_bootstrap_file(&identity, &private, 0, bytes)
            .await
            .unwrap();
        let reference = backend
            .finish_bootstrap_file(
                &identity,
                &private,
                &position,
                3,
                Sha256::digest(bytes).into(),
            )
            .await
            .unwrap();
        let files = [reference];
        let part = StoragePart::Files(&files);
        let pages = ReadLimits {
            rows: NonZeroUsize::MIN,
            bytes: NonZeroUsize::new(4096).unwrap(),
        };
        let batch = RetainedBatch::new(&identity, part, pages).unwrap();
        backend.write_bootstrap(batch).await.unwrap();
        let mut manifest = BootstrapManifest::new(identity.clone());
        manifest.record(batch).unwrap();
        for kind in [
            ReadKind::Schemas,
            ReadKind::Objects,
            ReadKind::Records,
            ReadKind::Stream,
            ReadKind::Files,
        ] {
            manifest.finish_collection(kind);
        }
        if completes {
            // Exact Saved Receipts May Be Reused Without Replacing Their Owner
            assert_eq!(backend.finish_bootstrap(&manifest).await.unwrap(), position);
            assert_eq!(backend.finish_bootstrap(&manifest).await.unwrap(), position);
        } else {
            let error = backend.finish_bootstrap(&manifest).await.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("private import differs from a prior file owner")
            );
            assert_eq!(backend.cursor_saved().await.unwrap(), prior);
            backend.abort_bootstrap(&identity).await.unwrap();
            // The Refused Import Cannot Adopt Or Remove Ordinary Producer Bytes
            backend.write_file(&ordinary, 0, b"old").await.unwrap();
            let reference = backend
                .finish_file(&ordinary, &position, 3, Sha256::digest(b"old").into())
                .await
                .unwrap();
            let files = [reference];
            let parts = [StoragePart::Files(&files)];
            let batch = StorageBatch::new(&parts, NonZeroUsize::new(4096).unwrap()).unwrap();
            assert_eq!(backend.write(batch).await.unwrap(), position);
        }
    }
    let snapshot = backend.read_view(&position).await.unwrap();
    let read = ReadView::new(
        harmos::Identifier::new("ordinary.owner.read").unwrap(),
        position.clone(),
        position.clone(),
        snapshot.clone(),
    )
    .unwrap();
    let files = backend
        .read_files(
            &read,
            &ReadSelection::new(ReadKind::Files),
            None,
            ReadLimits {
                rows: NonZeroUsize::MIN,
                bytes: NonZeroUsize::new(4096).unwrap(),
            },
        )
        .await
        .unwrap();
    assert_eq!(files.entries.len(), 1);
    assert_eq!(
        backend
            .read_file(&read, &files.entries[0], 0, NonZeroUsize::new(3).unwrap())
            .await
            .unwrap(),
        b"old"
    );
    backend.release_view(&snapshot).await.unwrap();
    backend.close().await.unwrap();
}

pub(crate) async fn private_import_abort() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let root = tempfile::tempdir().unwrap();
        let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
            .await
            .unwrap();
        let prior = backend.cursor_saved().await.unwrap();
        let coverage = RetainedCoverage {
            cut: StorageCursor {
                history: prior.history.clone(),
                sequence: 5,
            },
            complete_after: 3,
            retention_generation: NonZeroU64::MIN,
        };
        let identity = BootstrapId::new(&coverage).unwrap();
        let mut other = coverage.clone();
        other.retention_generation = NonZeroU64::new(2).unwrap();
        let other = BootstrapId::new(&other).unwrap();
        let limits = StorageSettings::default().query_execution;
        let metadata = FileMetadata {
            id: harmos::Identifier::new("old.stage").unwrap(),
            scope: None,
            media_type: "text/plain".into(),
            expected_length: Some(3),
            expected_sha256: None,
        };
        let staging = backend.create_file(&metadata).await.unwrap();
        backend.write_file(&staging, 0, b"old").await.unwrap();
        backend
            .begin_bootstrap(&identity, &prior, limits)
            .await
            .unwrap();
        backend
            .begin_bootstrap(&identity, &prior, limits)
            .await
            .unwrap();
        assert!(
            backend
                .begin_bootstrap(&other, &prior, limits)
                .await
                .is_err()
        );
        let mut changed = limits;
        changed.memory_bytes = NonZeroUsize::new(8 * 1024 * 1024).unwrap();
        assert!(
            backend
                .begin_bootstrap(&identity, &prior, changed)
                .await
                .is_err()
        );
        assert_eq!(backend.cursor_saved().await.unwrap(), prior);
        let recovery = backend.recovery(RecoveryLimits::default());
        RecoveryBackend::open(&recovery).await.unwrap();
        let recovered = RecoveryPosition {
            history: prior.history.clone(),
            sequence: 1,
        };
        recovery
            .write_entries(&[RecoveryEntry {
                position: recovered.clone(),
                payload: vec![7],
            }])
            .await
            .unwrap();
        assert_eq!(recovery.flush(&recovered).await.unwrap(), recovered);
        RecoveryBackend::close(&recovery).await.unwrap();
        assert_eq!(backend.cursor_saved().await.unwrap(), prior);
        let schema = StorageSchema::new(
            StorageCursor {
                history: prior.history.clone(),
                sequence: 1,
            },
            SchemaKey {
                payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
                id: harmos::Identifier::new("fixture.import").unwrap(),
                version: std::num::NonZeroU32::MIN,
            },
            vec![],
            harmos::storage::SchemaEncoding::Inline,
        );
        let schemas = [schema];
        let parts = [StoragePart::Schemas(&schemas)];
        let batch = StorageBatch::new(&parts, NonZeroUsize::new(4096).unwrap()).unwrap();
        assert!(backend.write(batch).await.is_err());
        assert!(backend.create_file(&metadata).await.is_err());
        assert!(backend.write_file(&staging, 0, b"old").await.is_err());
        assert!(
            backend
                .finish_file(
                    &staging,
                    &schemas[0].position,
                    3,
                    Sha256::digest(b"old").into()
                )
                .await
                .is_err()
        );
        assert!(backend.abort_file(&staging).await.is_err());
        assert!(
            backend
                .prune_objects(&prior, NonZeroUsize::MIN)
                .await
                .is_err()
        );
        assert!(backend.abort_bootstrap(&other).await.is_err());
        let snapshot = backend.read_view(&prior).await.unwrap();
        backend.release_view(&snapshot).await.unwrap();
        backend.close().await.unwrap();
        drop(backend);

        let reopened = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
            .await
            .unwrap();
        assert_eq!(reopened.cursor_saved().await.unwrap(), prior);
        let recovery = reopened.recovery(RecoveryLimits::default());
        assert_eq!(RecoveryBackend::open(&recovery).await.unwrap(), recovered);
        RecoveryBackend::close(&recovery).await.unwrap();
        assert!(reopened.write(batch).await.is_err());
        reopened
            .begin_bootstrap(&identity, &prior, limits)
            .await
            .unwrap();
        assert!(reopened.abort_bootstrap(&other).await.is_err());
        let control = reopened.control().await.unwrap();
        control
            .call(|connection| {
                connection.execute_batch(
                    "CREATE TRIGGER reject_abort BEFORE DELETE ON storage_bootstrap
             BEGIN SELECT RAISE(ABORT, 'injected private import abort failure'); END;",
                )
            })
            .await
            .unwrap();
        assert!(reopened.abort_bootstrap(&identity).await.is_err());
        assert!(reopened.write(batch).await.is_err());
        assert_eq!(reopened.cursor_saved().await.unwrap(), prior);
        control
            .call(|connection| connection.execute_batch("DROP TRIGGER reject_abort"))
            .await
            .unwrap();
        drop(control);
        reopened.abort_bootstrap(&identity).await.unwrap();
        reopened.abort_bootstrap(&identity).await.unwrap();
        let saved = reopened.write(batch).await.unwrap();
        assert_eq!(saved, schemas[0].position);
        assert!(
            reopened
                .begin_bootstrap(&identity, &prior, limits)
                .await
                .is_err()
        );
        reopened
            .begin_bootstrap(&identity, &saved, limits)
            .await
            .unwrap();
        reopened.abort_bootstrap(&identity).await.unwrap();
        reopened.abort_file(&staging).await.unwrap();
        reopened.close().await.unwrap();
    })
    .await
    .expect("private import fencing did not settle");
}

pub(crate) async fn failed_private_fence() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let prior = backend.cursor_saved().await.unwrap();
    let identity = BootstrapId::new(&RetainedCoverage {
        cut: StorageCursor {
            history: prior.history.clone(),
            sequence: 5,
        },
        complete_after: 3,
        retention_generation: NonZeroU64::MIN,
    })
    .unwrap();
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            connection.execute_batch(
                "CREATE TRIGGER reject_import BEFORE INSERT ON storage_bootstrap
         BEGIN SELECT RAISE(ABORT, 'injected private import creation failure'); END;",
            )
        })
        .await
        .unwrap();
    let limits = StorageSettings::default().query_execution;
    assert!(
        backend
            .begin_bootstrap(&identity, &prior, limits)
            .await
            .is_err()
    );
    assert_eq!(backend.cursor_saved().await.unwrap(), prior);
    control
        .call(|connection| {
            let count: i64 =
                connection.query_row("SELECT COUNT(*) FROM storage_bootstrap", [], |row| {
                    row.get(0)
                })?;
            assert_eq!(count, 0);
            connection.execute_batch("DROP TRIGGER reject_import")
        })
        .await
        .unwrap();
    let schemas = [StorageSchema::new(
        StorageCursor {
            history: prior.history.clone(),
            sequence: 1,
        },
        SchemaKey {
            payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
            id: harmos::Identifier::new("fixture.import").unwrap(),
            version: std::num::NonZeroU32::MIN,
        },
        vec![],
        harmos::storage::SchemaEncoding::Inline,
    )];
    let parts = [StoragePart::Schemas(&schemas)];
    let batch = StorageBatch::new(&parts, NonZeroUsize::new(4096).unwrap()).unwrap();
    let saved = backend.write(batch).await.unwrap();
    assert_eq!(saved, schemas[0].position);
    backend
        .begin_bootstrap(&identity, &saved, limits)
        .await
        .unwrap();
    backend.abort_bootstrap(&identity).await.unwrap();
    drop(control);
    backend.close().await.unwrap();
}

pub(crate) async fn private_rows_atomically() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let prior = backend.cursor_saved().await.unwrap();
    let coverage = RetainedCoverage {
        cut: StorageCursor {
            history: prior.history.clone(),
            sequence: 10,
        },
        complete_after: 6,
        retention_generation: NonZeroU64::MIN,
    };
    let identity = BootstrapId::new(&coverage).unwrap();
    let limits = ReadLimits {
        rows: NonZeroUsize::new(64).unwrap(),
        bytes: NonZeroUsize::new(4096).unwrap(),
    };
    let key = SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: harmos::Identifier::new("fixture.private").unwrap(),
        version: std::num::NonZeroU32::MIN,
    };
    let schemas = [StorageSchema::new(
        StorageCursor {
            history: prior.history.clone(),
            sequence: 1,
        },
        key.clone(),
        vec![],
        harmos::storage::SchemaEncoding::Inline,
    )];
    let records = [StoredRecord {
        position: StorageCursor {
            history: prior.history.clone(),
            sequence: 7,
        },
        schema: key.clone(),
        item: harmos::Identifier::new("record.one").unwrap(),
        scope: None,
        source: None,
        source_time: None,
        payload: vec![9, 8],
    }];
    let streams = [StoredRow {
        stream: harmos::Identifier::new("samples").unwrap(),
        sequence: 42,
        record: StoredRecord {
            position: StorageCursor {
                history: prior.history.clone(),
                sequence: 8,
            },
            item: harmos::Identifier::new("stream.one").unwrap(),
            ..records[0].clone()
        },
    }];
    let files = [FileReference {
        position: StorageCursor {
            history: prior.history.clone(),
            sequence: 9,
        },
        metadata: FileMetadata {
            id: harmos::Identifier::new("report").unwrap(),
            scope: None,
            media_type: "text/plain".into(),
            expected_length: Some(3),
            expected_sha256: None,
        },
        length: 3,
        sha256: Sha256::digest(b"abc").into(),
    }];
    let parts = [
        StoragePart::Schemas(&schemas),
        StoragePart::Records(&records),
        StoragePart::Stream(&streams),
        StoragePart::Files(&files),
    ];
    let batch = RetainedBatch::new(&identity, parts[1], limits).unwrap();
    assert!(backend.write_bootstrap(batch).await.is_err());
    backend
        .begin_bootstrap(
            &identity,
            &prior,
            StorageSettings::default().query_execution,
        )
        .await
        .unwrap();
    let mut manifest = BootstrapManifest::new(identity.clone());
    for part in parts {
        let batch = RetainedBatch::new(&identity, part, limits).unwrap();
        backend.write_bootstrap(batch).await.unwrap();
        backend.write_bootstrap(batch).await.unwrap();
        manifest.record(batch).unwrap();
    }
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            let count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_entries",
                [],
                |row| row.get(0),
            )?;
            let visible: i64 =
                connection.query_row("SELECT COUNT(*) FROM storage_data_entries", [], |row| {
                    row.get(0)
                })?;
            assert_eq!((count, visible), (4, 0));
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(backend.cursor_saved().await.unwrap(), prior);
    let snapshot = backend.read_view(&prior).await.unwrap();
    let view = ReadView::new(
        harmos::Identifier::new("view.before.import").unwrap(),
        prior.clone(),
        prior.clone(),
        snapshot.clone(),
    )
    .unwrap();
    assert!(
        backend
            .read_schemas(&view, &ReadSelection::new(ReadKind::Schemas), None, limits)
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    assert!(
        backend
            .read_records(&view, &ReadSelection::new(ReadKind::Records), None, limits)
            .await
            .unwrap()
            .entries
            .is_empty()
    );
    backend.release_view(&snapshot).await.unwrap();
    let mut changed = records[0].clone();
    changed.payload.push(7);
    let prefix = StoredRecord {
        position: StorageCursor {
            history: prior.history.clone(),
            sequence: 6,
        },
        item: harmos::Identifier::new("record.new").unwrap(),
        ..records[0].clone()
    };
    let conflict = [prefix, changed];
    assert!(
        backend
            .write_bootstrap(
                RetainedBatch::new(&identity, StoragePart::Records(&conflict), limits).unwrap()
            )
            .await
            .is_err()
    );
    let collision = [StoredRecord {
        position: schemas[0].position.clone(),
        ..records[0].clone()
    }];
    assert!(
        backend
            .write_bootstrap(
                RetainedBatch::new(&identity, StoragePart::Records(&collision), limits).unwrap()
            )
            .await
            .is_err()
    );
    for kind in [
        ReadKind::Schemas,
        ReadKind::Objects,
        ReadKind::Records,
        ReadKind::Stream,
        ReadKind::Files,
    ] {
        manifest.finish_collection(kind);
    }
    assert!(
        backend
            .finish_bootstrap(&manifest)
            .await
            .unwrap_err()
            .to_string()
            .contains("private file reference has no staged bytes")
    );
    control
        .call(|connection| {
            connection.execute(
                "UPDATE storage_bootstrap_schemas SET position='0000000000000002'",
                [],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    assert!(
        backend
            .finish_bootstrap(&manifest)
            .await
            .unwrap_err()
            .to_string()
            .contains("private import schema index differs")
    );
    control
        .call(|connection| {
            connection.execute(
                "UPDATE storage_bootstrap_schemas SET position='0000000000000001'",
                [],
            )?;
            connection.execute(
                "INSERT INTO storage_bootstrap_schemas VALUES ('extra','000000000000000a')",
                [],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    assert!(
        backend
            .finish_bootstrap(&manifest)
            .await
            .unwrap_err()
            .to_string()
            .contains("private import schema index has extra entries")
    );
    control
        .call(|connection| {
            connection.execute(
                "DELETE FROM storage_bootstrap_schemas WHERE schema_key='extra'",
                [],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let mut repeated = schemas[0].clone();
    repeated.position.sequence = 2;
    let repeated_schema = [repeated];
    backend
        .write_bootstrap(
            RetainedBatch::new(&identity, StoragePart::Schemas(&repeated_schema), limits).unwrap(),
        )
        .await
        .unwrap();
    // Restore The Original Manifest Before Independent Corruption Checks
    control
        .call(|connection| {
            connection.execute(
                "DELETE FROM storage_bootstrap_schemas WHERE position='0000000000000002'",
                [],
            )?;
            connection.execute(
                "DELETE FROM storage_bootstrap_entries WHERE position='0000000000000002'",
                [],
            )?;
            Ok(())
        })
        .await
        .unwrap();
    let mut empty = BootstrapManifest::new(identity.clone());
    for kind in [
        ReadKind::Schemas,
        ReadKind::Objects,
        ReadKind::Records,
        ReadKind::Stream,
        ReadKind::Files,
    ] {
        empty.finish_collection(kind);
    }
    let error = backend.finish_bootstrap(&empty).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("private import collection manifest differs")
    );
    let original = serde_json::to_vec(&records[0]).unwrap();
    let tampered = serde_json::to_vec(&StoredRecord {
        payload: vec![3, 2, 1],
        ..records[0].clone()
    })
    .unwrap();
    control.call(move |connection| {
        connection.execute("UPDATE storage_bootstrap_entries SET body=?1,digest=?2 WHERE position='0000000000000007'", rusqlite::params![tampered, Sha256::digest(&tampered).as_slice()])?;
        Ok(())
    }).await.unwrap();
    let error = backend.finish_bootstrap(&manifest).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("private import collection manifest differs")
    );
    let mut whitespace = original.clone();
    whitespace.push(b' ');
    control.call(move |connection| {
        connection.execute("UPDATE storage_bootstrap_entries SET body=?1,digest=?2 WHERE position='0000000000000007'", rusqlite::params![whitespace, Sha256::digest(&whitespace).as_slice()])?;
        Ok(())
    }).await.unwrap();
    let error = backend.finish_bootstrap(&manifest).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("private import entry is not canonical")
    );
    control.call(move |connection| {
        connection.execute("UPDATE storage_bootstrap_entries SET body=?1,digest=?2,position='0000000000000006' WHERE position='0000000000000007'", rusqlite::params![original, Sha256::digest(&original).as_slice()])?;
        Ok(())
    }).await.unwrap();
    let error = backend.finish_bootstrap(&manifest).await.unwrap_err();
    assert!(
        error
            .to_string()
            .contains("private import position index differs")
    );
    control.call(|connection| {
        connection.execute("UPDATE storage_bootstrap_entries SET position='0000000000000007' WHERE position='0000000000000006'", [])?;
        Ok(())
    }).await.unwrap();
    assert!(
        backend
            .finish_bootstrap(&manifest)
            .await
            .unwrap_err()
            .to_string()
            .contains("private file reference has no staged bytes")
    );

    assert_eq!(backend.cursor_saved().await.unwrap(), prior);
    drop(control);
    backend.close().await.unwrap();
    drop(backend);

    let reopened = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    assert_eq!(reopened.cursor_saved().await.unwrap(), prior);
    reopened.write_bootstrap(batch).await.unwrap();
    let control = reopened.control().await.unwrap();
    control.call(|connection| {
        let body: Vec<u8> = connection.query_row("SELECT body FROM storage_bootstrap_entries WHERE position='0000000000000007'", [], |row| row.get(0))?;
        let actual: StoredRecord = serde_json::from_slice(&body).unwrap();
        assert_eq!(actual.payload, [9, 8]);
        let count: i64 = connection.query_row("SELECT COUNT(*) FROM storage_bootstrap_entries", [], |row| row.get(0))?;
        assert_eq!(count, 4, "a conflicting batch must not commit its earlier prefix");
        connection.execute_batch("CREATE TRIGGER reject_row_abort BEFORE DELETE ON storage_bootstrap BEGIN SELECT RAISE(ABORT, 'injected abort failure'); END;")
    }).await.unwrap();
    assert!(reopened.abort_bootstrap(&identity).await.is_err());
    assert!(
        reopened
            .write_bootstrap(batch)
            .await
            .unwrap_err()
            .to_string()
            .contains("private import abort is pending")
    );
    assert!(
        reopened
            .begin_bootstrap(
                &identity,
                &prior,
                StorageSettings::default().query_execution
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("private import abort is pending")
    );
    control
        .call(|connection| {
            let count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_entries",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 4, "failed abort retains every private row");
            let schemas: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_schemas",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(schemas, 1, "failed abort retains the schema index");
            let aborts: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_abort",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(aborts, 1, "failed cleanup retains its durable intent");
            connection.execute_batch("DROP TRIGGER reject_row_abort")
        })
        .await
        .unwrap();
    reopened.abort_bootstrap(&identity).await.unwrap();
    control
        .call(|connection| {
            let count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_entries",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 0);
            let schemas: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_schemas",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(schemas, 0);
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(reopened.cursor_saved().await.unwrap(), prior);
    drop(control);
    reopened.close().await.unwrap();
}

pub(crate) async fn private_row_unchanged() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
        .await
        .unwrap();
    let prior = backend.cursor_saved().await.unwrap();
    let identity = BootstrapId::new(&RetainedCoverage {
        cut: StorageCursor {
            history: prior.history.clone(),
            sequence: 5,
        },
        complete_after: 3,
        retention_generation: NonZeroU64::MIN,
    })
    .unwrap();
    let execution = QueryLimits {
        memory_bytes: NonZeroUsize::new(2 * 1024 * 1024).unwrap(),
        ..StorageSettings::default().query_execution
    };
    backend
        .begin_bootstrap(&identity, &prior, execution)
        .await
        .unwrap();
    let records = [StoredRecord {
        position: StorageCursor {
            history: prior.history.clone(),
            sequence: 1,
        },
        schema: SchemaKey {
            payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
            id: harmos::Identifier::new("fixture.private").unwrap(),
            version: std::num::NonZeroU32::MIN,
        },
        scope: None,
        item: harmos::Identifier::new("large").unwrap(),
        source: None,
        source_time: None,
        payload: vec![0; 500_000],
    }];
    let limits = ReadLimits {
        rows: NonZeroUsize::MIN,
        bytes: NonZeroUsize::new(1024 * 1024).unwrap(),
    };
    let batch = RetainedBatch::new(&identity, StoragePart::Records(&records), limits).unwrap();
    for _ in 0..2 {
        assert!(matches!(
            backend.write_bootstrap(batch).await,
            Err(harmos::Error::InvalidInput {
                reason: "owned query state exceeds memory bound",
                ..
            })
        ));
        assert_eq!(backend.cursor_saved().await.unwrap(), prior);
    }
    let small = [StoredRecord {
        payload: vec![1],
        ..records[0].clone()
    }];
    backend
        .write_bootstrap(
            RetainedBatch::new(&identity, StoragePart::Records(&small), limits).unwrap(),
        )
        .await
        .unwrap();
    let control = backend.control().await.unwrap();
    control
        .call(|connection| {
            let count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM storage_bootstrap_entries",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(count, 1);
            Ok(())
        })
        .await
        .unwrap();
    backend.abort_bootstrap(&identity).await.unwrap();
    drop(control);
    backend.close().await.unwrap();
}

pub(crate) async fn private_manifest_schema() {
    for stream_only in [false, true] {
        for (schema_position, present, valid, expected) in [
            (1, true, true, "private completeness suffix has a gap"),
            (1, false, true, "no preceding saved schema"),
            (10, true, true, "no preceding saved schema"),
            (1, true, false, "duplicate name"),
        ] {
            let root = tempfile::tempdir().unwrap();
            let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
                .await
                .unwrap();
            let prior = backend.cursor_saved().await.unwrap();
            let identity = BootstrapId::new(&RetainedCoverage {
                cut: StorageCursor {
                    history: prior.history.clone(),
                    sequence: 10,
                },
                complete_after: 6,
                retention_generation: NonZeroU64::MIN,
            })
            .unwrap();
            let key = SchemaKey {
                payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
                id: harmos::Identifier::new("fixture.dependency").unwrap(),
                version: std::num::NonZeroU32::MIN,
            };
            let field = harmos::schema::Field {
                name: "value".into(),
                tag: 1,
                kind: harmos::schema::Kind::U64,
                cardinality: harmos::schema::Cardinality::Required,
            };
            let schemas = [StorageSchema::new(
                StorageCursor {
                    history: prior.history.clone(),
                    sequence: schema_position,
                },
                key.clone(),
                if valid {
                    vec![field]
                } else {
                    vec![field.clone(), field]
                },
                harmos::storage::SchemaEncoding::Inline,
            )];
            let records = [StoredRecord {
                position: StorageCursor {
                    history: prior.history.clone(),
                    sequence: 7,
                },
                schema: key,
                item: harmos::Identifier::new("fixture.record").unwrap(),
                scope: None,
                source: None,
                source_time: None,
                payload: vec![8, 1],
            }];
            let streams = [StoredRow {
                stream: harmos::Identifier::new("fixture.stream").unwrap(),
                sequence: 1,
                record: StoredRecord {
                    position: StorageCursor {
                        history: prior.history.clone(),
                        sequence: 9,
                    },
                    ..records[0].clone()
                },
            }];
            backend
                .begin_bootstrap(
                    &identity,
                    &prior,
                    StorageSettings::default().query_execution,
                )
                .await
                .unwrap();
            let mut manifest = BootstrapManifest::new(identity.clone());
            let limits = ReadLimits {
                rows: NonZeroUsize::new(16).unwrap(),
                bytes: NonZeroUsize::new(4096).unwrap(),
            };
            for part in [
                StoragePart::Schemas(if present { &schemas } else { &[] }),
                StoragePart::Records(if stream_only { &[] } else { &records }),
                StoragePart::Stream(&streams),
            ] {
                if part.entries().next().is_none() {
                    continue;
                }
                let batch = RetainedBatch::new(&identity, part, limits).unwrap();
                backend.write_bootstrap(batch).await.unwrap();
                manifest.record(batch).unwrap();
            }
            for kind in [
                ReadKind::Schemas,
                ReadKind::Objects,
                ReadKind::Records,
                ReadKind::Stream,
                ReadKind::Files,
            ] {
                manifest.finish_collection(kind);
            }
            let error = backend.finish_bootstrap(&manifest).await.unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
            assert_eq!(backend.cursor_saved().await.unwrap(), prior);
            backend.abort_bootstrap(&identity).await.unwrap();
            backend.close().await.unwrap();
        }
    }
}

pub(crate) async fn private_import_floor() {
    for floor in [0, 2] {
        let root = tempfile::tempdir().unwrap();
        let backend = Backend::<()>::primary(root.path().to_path_buf(), Limits::default())
            .await
            .unwrap();
        let prior = backend.cursor_saved().await.unwrap();
        let identity = BootstrapId::new(&RetainedCoverage {
            cut: StorageCursor {
                history: prior.history.clone(),
                sequence: 3,
            },
            complete_after: floor,
            retention_generation: NonZeroU64::MIN,
        })
        .unwrap();
        backend
            .begin_bootstrap(
                &identity,
                &prior,
                StorageSettings::default().query_execution,
            )
            .await
            .unwrap();
        let key = SchemaKey {
            payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
            id: harmos::Identifier::new("floor.schema").unwrap(),
            version: std::num::NonZeroU32::MIN,
        };
        let schemas = [StorageSchema::new(
            StorageCursor {
                history: prior.history.clone(),
                sequence: 1,
            },
            key.clone(),
            vec![],
            harmos::storage::SchemaEncoding::Inline,
        )];
        let records = [StoredRecord {
            position: identity.cut().clone(),
            schema: key,
            scope: None,
            item: harmos::Identifier::new("floor.record").unwrap(),
            source: None,
            source_time: None,
            payload: vec![],
        }];
        let limits = ReadLimits {
            rows: NonZeroUsize::MIN,
            bytes: NonZeroUsize::new(4096).unwrap(),
        };
        let mut manifest = BootstrapManifest::new(identity.clone());
        for part in [
            StoragePart::Schemas(&schemas),
            StoragePart::Records(&records),
        ] {
            let batch = RetainedBatch::new(&identity, part, limits).unwrap();
            backend.write_bootstrap(batch).await.unwrap();
            manifest.record(batch).unwrap();
        }
        for kind in [
            ReadKind::Schemas,
            ReadKind::Objects,
            ReadKind::Records,
            ReadKind::Stream,
            ReadKind::Files,
        ] {
            manifest.finish_collection(kind);
        }
        if floor == 0 {
            let failure = backend
                .finish_bootstrap(&manifest)
                .await
                .unwrap_err()
                .to_string();
            assert!(failure.contains("private completeness suffix has a gap"));
            assert_eq!(backend.cursor_saved().await.unwrap(), prior);
        } else {
            assert_eq!(
                backend.finish_bootstrap(&manifest).await.unwrap(),
                *identity.cut()
            );
            assert_eq!(backend.cursor_saved().await.unwrap(), *identity.cut());
            assert_eq!(
                backend.finish_bootstrap(&manifest).await.unwrap(),
                *identity.cut()
            );
        }
        backend.abort_bootstrap(&identity).await.unwrap();
        if floor != 0 {
            assert_eq!(backend.cursor_saved().await.unwrap(), *identity.cut());
        }
        backend.close().await.unwrap();
    }
}
