/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Keeps Execution Notes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::Client;
use dokime::Import;
use dokime::Origin;
use dokime::Termination;
use dokime::client::EvidenceAnnotationOwner;
use dokime::client::EvidenceAnnotationOwnerOwner;
use dokime::client::EvidenceAnnotationSelection;
use dokime::client::EvidenceExecutionNoteOwner;
use dokime::client::EvidenceNoteWritable;
use dokime::client::EvidenceNotesCreateOptions;
use dokime::client::EvidenceNotesGetOptions;
use dokime::client::EvidenceNotesListOptions;
use dokime::client::EvidenceNotesSetOptions;
use dokime::client::ExecutionOutcome;
use dokime_protocol::host::evidence::NotesCreateRequest;
use dokime_protocol::host::evidence::NotesSetRequest;
use dokime_protocol::host::evidence::annotation_owner_owner::Kind as AnnotationOwnerOwnerKind;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_storage::Execution;
use dokime_storage::Store;
use harmos_protocol::model::PageRequest;
use std::error::Error;
use tonic::transport::Endpoint;

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
fn client_keeps_execution_notes() -> Result<(), Box<dyn Error>> {
    super::support::run(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Prepare Independent Draft Executions
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let store = Store::open(root.path())?;
            for id in ["notes-a", "notes-b"] {
                store.create(&Execution {
                    identifier: id.into(),
                    cycle: "notes-cycle".into(),
                    origin: Some(Origin {
                        kind: Some(OriginKind::Import(Import::default())),
                    }),
                    ..Default::default()
                })?;
            }

            // Create a Note and Resolve a Retried Mutation
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let notes = Client::from_channel(channel, super::support::client_settings())?.evidence.notes.clone();
            let owner = EvidenceAnnotationOwner {
                owner: Some(EvidenceAnnotationOwnerOwner {
                    kind: Some(AnnotationOwnerOwnerKind::Execution(EvidenceExecutionNoteOwner {
                        cycle_id: "notes-cycle".into(),
                        execution_id: "notes-a".into(),
                    })),
                }),
            };
            let value = EvidenceNoteWritable {
                message: uuid::Uuid::now_v7().to_string(),
            };
            let request = NotesCreateRequest {
                owner: Some(owner.clone()),
                value: Some(value.clone()),
                request_id: uuid::Uuid::now_v7().to_string(),
            };
            let receipt = notes
                .create_with(
                    ((request.clone()).owner).unwrap_or_default(),
                    ((request.clone()).value).unwrap_or_default(),
                    EvidenceNotesCreateOptions {
                        request_id: Some((request.clone()).request_id),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(
                notes
                    .create_with(
                        ((request.clone()).owner).unwrap_or_default(),
                        ((request.clone()).value).unwrap_or_default(),
                        EvidenceNotesCreateOptions {
                            request_id: Some((request.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await?,
                receipt
            );
            let mut conflict = request;
            conflict.value.as_mut().unwrap().message.push('x');
            assert!(
                notes
                    .create_with(
                        ((conflict.clone()).owner).unwrap_or_default(),
                        ((conflict.clone()).value).unwrap_or_default(),
                        EvidenceNotesCreateOptions {
                            request_id: Some((conflict.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            let name = receipt.note.unwrap().note_id;
            let first = notes.get(name.clone()).await?;
            let changed = EvidenceNoteWritable {
                message: uuid::Uuid::now_v7().to_string(),
            };
            let update = NotesSetRequest {
                name: name.clone(),
                value: Some(changed.clone()),
                expected_revision: first.revision.clone(),
                request_id: "edit".into(),
            };
            let saved = notes
                .set_with(
                    (update.clone()).name,
                    ((update.clone()).value).unwrap_or_default(),
                    (update.clone()).expected_revision,
                    EvidenceNotesSetOptions {
                        request_id: Some((update.clone()).request_id),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(saved.value, Some(changed.clone()));
            assert_eq!(
                notes
                    .set_with(
                        (update.clone()).name,
                        ((update.clone()).value).unwrap_or_default(),
                        (update.clone()).expected_revision,
                        EvidenceNotesSetOptions {
                            request_id: Some((update.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await?,
                saved
            );
            let selection = EvidenceAnnotationSelection {
                owner: Some(owner),
                include_context: true,
            };
            let page = notes
                .list_with(
                    selection.clone(),
                    EvidenceNotesListOptions {
                        page: Some(PageRequest { page_size: 1, ..Default::default() }),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(page.notes.len(), 1);
            assert_eq!(page.notes[0].scope.as_ref().unwrap().execution_id.as_deref(), Some("notes-a"));
            assert_eq!(page.notes[0].message, changed.message);
            let other = EvidenceAnnotationOwner {
                owner: Some(EvidenceAnnotationOwnerOwner {
                    kind: Some(AnnotationOwnerOwnerKind::Execution(EvidenceExecutionNoteOwner {
                        cycle_id: "notes-cycle".into(),
                        execution_id: "notes-b".into(),
                    })),
                }),
            };
            assert!(
                notes
                    .list_with(
                        EvidenceAnnotationSelection {
                            owner: Some(other),
                            include_context: false
                        },
                        EvidenceNotesListOptions {
                            at: page.resolved_at,
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            let pinned = notes
                .get_with(
                    name.clone(),
                    EvidenceNotesGetOptions {
                        at: first.resolved_at.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(pinned.value, Some(value));
            store.close("notes-a", Termination::Completed, ExecutionOutcome::default())?;
            store.seal("notes-a")?;
            let mut closed = update;
            closed.request_id = "closed".into();
            closed.expected_revision = saved.revision.clone();
            assert!(
                notes
                    .set_with(
                        (closed.clone()).name,
                        ((closed.clone()).value).unwrap_or_default(),
                        (closed.clone()).expected_revision,
                        EvidenceNotesSetOptions {
                            request_id: Some((closed.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            process.close().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let notes =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .notes
                    .clone();
            let page = notes.list(selection).await?;
            assert_eq!(page.notes[0].note_id, name);
            assert_eq!(page.notes[0].message, changed.message);
            assert!(
                notes
                    .set_with(
                        (closed.clone()).name,
                        ((closed.clone()).value).unwrap_or_default(),
                        (closed.clone()).expected_revision,
                        EvidenceNotesSetOptions {
                            request_id: Some((closed.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            process.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    })
}
