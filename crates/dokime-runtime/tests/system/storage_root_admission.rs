/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exclusive Evidence Root Admission

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error;
use std::fs::OpenOptions;

// Third Party Imports
use harmos_protocol::model::FileMetadata;
use prost::Message;
use rusqlite::Connection;
use tonic::Code;
use tonic::transport::Endpoint;

// Local Imports
use dokime_client_internal::{Client, ContentArtifactsCreateOptions, EvidenceNotesCreateOptions};
use dokime_protocol::host::evidence::{
    AnnotationOwner, AnnotationOwnerOwner, AnnotationSelection, ExecutionNoteOwner, NoteWritable, annotation_owner_owner,
};
use dokime_protocol::host::{CyclePhase, CycleSummary};
use dokime_protocol::model::{ArtifactScope, Import, Origin, origin::Kind};
use dokime_runtime::Error;
use dokime_runtime::Host;
use dokime_storage::{Execution, Store};

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

#[test]
fn incompatible_content_roots_are_refused_without_resetting_evidence() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let execution = Execution {
            identifier: "evidence".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(Kind::Import(Import::default())),
            }),
            ..Default::default()
        };
        let identifier = execution.identifier.clone();
        store.create(&execution)?;
        let database = Connection::open(root.path().join("execution.sqlite3"))?;
        database.execute_batch("CREATE TABLE content_objects(id TEXT PRIMARY KEY); INSERT INTO content_objects VALUES('retired-file')")?;
        drop(database);
        drop(store);
        let refused = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await;
        assert!(matches!(refused, Err(Error::Storage(dokime_storage::Error::Incompatible(_)))));
        let database = Connection::open(root.path().join("execution.sqlite3"))?;
        assert_eq!(database.query_row("SELECT state FROM executions WHERE id=?1", [identifier], |row| row.get::<_, i32>(0))?, 0);
        assert_eq!(database.query_row("SELECT COUNT(*) FROM content_objects", [], |row| row.get::<_, i64>(0))?, 1);
        Ok(())
    }))
}

#[test]
fn hosts_refuse_occupied_roots_before_opening_evidence() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        // Refuse An External Lease Before Creating The Evidence Database
        let root = super::support::tempdir()?;
        let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.path().join("local.lock"))?;
        lease.try_lock()?;
        let refused = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await;
        assert!(matches!(refused, Err(Error::OccupiedRoot(path)) if path == root.path()));
        assert!(!root.path().join("execution.sqlite3").exists());
        drop(lease);

        // Keep The Lease While A Running Host Owns The Root
        let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        let retained = host.clone();
        assert!(
            matches!(Host::open(root.path(), super::support::settings(), super::support::service_settings()).await, Err(Error::OccupiedRoot(path)) if path == root.path())
        );

        // Release The Root At Shutdown Although A Handle Remains, And Refuse New Work Through It
        host.shutdown().await?;
        let reopened = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        assert!(reopened.executions()?.is_empty());
        assert!(retained.add_remote("unavailable", "http://127.0.0.1:1").await.is_err());
        assert!(retained.start_cycle(Vec::new()).await.is_err());
        reopened.shutdown().await?;

        // Release The Root With Its Last Owner When A Host Is Dropped Without Shutdown
        drop((host, retained, reopened));
        let abandoned = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        let second = abandoned.clone();
        drop(abandoned);
        assert!(matches!(
            Host::open(root.path(), super::support::settings(), super::support::service_settings()).await,
            Err(Error::OccupiedRoot(_))
        ));
        drop(second);
        let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
        host.shutdown().await?;
        Ok(())
    }))
}

#[test]
fn superseded_plan_tables_are_refused_without_resetting_the_root() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        for table in ["cases", "registrations"] {
            // Preserve the Incompatible Journal Alongside Existing Execution Evidence.
            let root = tempfile::tempdir()?;
            let store = Store::open(root.path())?;
            store.create(&Execution {
                identifier: "retained-execution".into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(Kind::Import(Import::default())),
                }),
                ..Default::default()
            })?;
            drop(store);
            let database = Connection::open(root.path().join("execution.sqlite3"))?;
            database.execute_batch(&format!("CREATE TABLE {table}(payload BLOB); INSERT INTO {table} VALUES(X'010203')"))?;
            drop(database);

            // Admission Refuses the Old Shape Without Migrating or Deleting It.
            let refused = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await;
            assert!(matches!(refused, Err(Error::Storage(dokime_storage::Error::Incompatible(_)))));
            let database = Connection::open(root.path().join("execution.sqlite3"))?;
            let execution: String = database.query_row("SELECT id FROM executions", [], |row| row.get(0))?;
            assert_eq!(execution, "retained-execution");
            let payload: Vec<u8> = database.query_row(&format!("SELECT payload FROM {table}"), [], |row| row.get(0))?;
            assert_eq!(payload, vec![1, 2, 3]);
        }
        Ok(())
    }))
}

#[test]
fn native_notes_preserve_owner_finality_replay_and_restart() -> Result<(), Box<dyn error::Error>> {
    super::support::run(Box::pin(async {
        // Admit a Frozen Import and Cycle Without Any Plugin Fixture.
        let root = super::support::tempdir()?;
        let store = Store::open(root.path())?;
        store.create(&Execution {
            identifier: "evidence".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(Kind::Import(Import::default())),
            }),
            ..Default::default()
        })?;
        let mut cycle = CycleSummary {
            cycle_id: "cycle".into(),
            ..Default::default()
        };
        store.save_cycle(&cycle)?;
        let server = dokime_runtime::services::serve(
            "127.0.0.1:0".parse()?,
            root.path(),
            super::support::settings(),
            super::support::service_settings(),
        )
        .await?;
        let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;
        let execution = AnnotationOwner {
            owner: Some(AnnotationOwnerOwner {
                kind: Some(annotation_owner_owner::Kind::Execution(ExecutionNoteOwner {
                    cycle_id: "cycle".into(),
                    execution_id: "evidence".into(),
                })),
            }),
        };
        let cycle_owner = AnnotationOwner {
            owner: Some(AnnotationOwnerOwner {
                kind: Some(annotation_owner_owner::Kind::CycleId("cycle".into())),
            }),
        };

        // Refuse Missing Owners and Cross-Cycle Execution Ownership.
        let wrong = AnnotationOwner {
            owner: Some(AnnotationOwnerOwner {
                kind: Some(annotation_owner_owner::Kind::Execution(ExecutionNoteOwner {
                    cycle_id: "other".into(),
                    execution_id: "evidence".into(),
                })),
            }),
        };
        assert!(client.evidence.notes.create(wrong, NoteWritable { message: "invalid".into() }).await.is_err());
        let missing = AnnotationOwner {
            owner: Some(AnnotationOwnerOwner {
                kind: Some(annotation_owner_owner::Kind::CycleId("missing".into())),
            }),
        };
        assert!(client.evidence.notes.create(missing, NoteWritable { message: "invalid".into() }).await.is_err());
        let missing = AnnotationOwner {
            owner: Some(AnnotationOwnerOwner {
                kind: Some(annotation_owner_owner::Kind::Execution(ExecutionNoteOwner {
                    cycle_id: "cycle".into(),
                    execution_id: "missing".into(),
                })),
            }),
        };
        let refused = client.evidence.notes.create(missing, NoteWritable { message: "invalid".into() }).await.unwrap_err();
        assert_eq!(refused.code(), Code::NotFound);

        // Persist and Edit Native Notes While Their Owner Is Open.
        let value = NoteWritable { message: "captured".into() };
        let created = client
            .evidence
            .notes
            .create_with(
                execution.clone(),
                value.clone(),
                EvidenceNotesCreateOptions {
                    request_id: Some("native-note-before".into()),
                    ..Default::default()
                },
            )
            .await?;
        let note = created.note.as_ref().ok_or("Create Note: Return Its Durable Value")?;
        let reviewed = NoteWritable { message: "reviewed".into() };
        let updated = client.evidence.notes.set(note.note_id.clone(), reviewed.clone(), created.revision.clone()).await?;
        assert_ne!(updated.revision, created.revision);
        let file = super::support::native_file(&client, "source", "evidence", b"execution-source").await?;
        let scope = ArtifactScope {
            cycle_id: Some("cycle".into()),
            execution_id: Some("evidence".into()),
            ..Default::default()
        };
        client
            .content
            .artifacts
            .create_with(
                file.clone(),
                "source.bin",
                "application/octet-stream",
                ContentArtifactsCreateOptions {
                    scope: Some(scope.clone()),
                    ..Default::default()
                },
            )
            .await?;

        // Closing the Execution Refuses New Writes and Preserves Original Admission Replay.
        store.seal_unstarted("evidence", Default::default())?;
        let replay = client
            .evidence
            .notes
            .create_with(
                execution.clone(),
                value.clone(),
                EvidenceNotesCreateOptions {
                    request_id: Some("native-note-before".into()),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(replay.encode_to_vec(), created.encode_to_vec());
        assert!(client.evidence.notes.create(execution.clone(), value.clone()).await.is_err());
        assert!(client.evidence.notes.set(note.note_id.clone(), value.clone(), updated.revision).await.is_err());
        assert!(
            client
                .files
                .uploads
                .create(FileMetadata {
                    id: "late".into(),
                    scope: Some("execution/evidence".into()),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(
            client
                .content
                .artifacts
                .create_with(
                    file,
                    "late.bin",
                    "application/octet-stream",
                    ContentArtifactsCreateOptions { scope: Some(scope), ..Default::default() }
                )
                .await
                .is_err()
        );
        let contextual = client.evidence.notes.create(cycle_owner.clone(), value.clone()).await?;

        // Sealing Fences Cycle Writes; a Failed Seal Allows Repair Evidence.
        cycle.phase = CyclePhase::Sealing as i32;
        store.save_cycle(&cycle)?;
        assert!(client.evidence.notes.create(cycle_owner.clone(), value.clone()).await.is_err());
        cycle.phase = CyclePhase::SealFailed as i32;
        store.save_cycle(&cycle)?;
        let repaired = client.evidence.notes.create(cycle_owner.clone(), value.clone()).await?;
        cycle.sealed = true;
        cycle.phase = CyclePhase::Sealed as i32;
        store.save_cycle(&cycle)?;
        assert!(client.evidence.notes.create(cycle_owner.clone(), value).await.is_err());
        assert!(
            client
                .files
                .uploads
                .create(FileMetadata {
                    id: "late-cycle".into(),
                    scope: Some("cycle/cycle".into()),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let selected = AnnotationSelection {
            owner: Some(execution),
            include_context: true,
        };
        let notes = client.evidence.notes.list(selected.clone()).await?.notes;
        assert_eq!(notes.len(), 3);
        assert!(notes.iter().any(|saved| saved.note_id == note.note_id && saved.message == reviewed.message));
        assert!(notes.iter().any(|saved| contextual.note.as_ref() == Some(saved)));
        assert!(notes.iter().any(|saved| repaired.note.as_ref() == Some(saved)));
        drop(client);
        server.close().await?;
        drop(store);

        // Reopen the Native History Without Any SQL Note Table.
        let server = dokime_runtime::services::serve(
            "127.0.0.1:0".parse()?,
            root.path(),
            super::support::settings(),
            super::support::service_settings(),
        )
        .await?;
        let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
        let client = Client::from_channel(channel, super::support::client_settings())?;
        assert_eq!(client.evidence.notes.list(selected).await?.notes, notes);
        drop(client);
        server.close().await?;
        Ok(())
    }))
}
