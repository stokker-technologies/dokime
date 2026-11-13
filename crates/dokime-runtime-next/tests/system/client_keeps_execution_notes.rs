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
use dokime::v1;
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

#[tokio::test]
async fn client_keeps_execution_notes() -> Result<(), Box<dyn std::error::Error>> {
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
                origin: Some(v1::Origin {
                    kind: Some(v1::origin::Kind::Import(v1::Import::default())),
                }),
                ..Default::default()
            })?;
        }

        // Create a Note and Resolve a Retried Mutation
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut notes = v1::notes_client::NotesClient::new(channel);
        let epoch = notes.list_notes(v1::NotesListNotesRequest::default()).await?.into_inner().cursor.unwrap().epoch;
        let owner = v1::AnnotationOwner {
            owner: Some(v1::AnnotationOwnerOwner {
                kind: Some(v1::annotation_owner_owner::Kind::Execution(v1::ExecutionNoteOwner {
                    cycle_id: "notes-cycle".into(),
                    execution_id: "notes-a".into(),
                })),
            }),
        };
        let request = v1::NotesCreateNoteRequest {
            epoch: epoch.clone(),
            mutation_id: uuid::Uuid::now_v7().to_string(),
            owner: Some(owner.clone()),
            message: "Inspect the connector".into(),
        };
        let receipt = notes.create_note(request.clone()).await?.into_inner();
        assert_eq!(notes.create_note(request.clone()).await?.into_inner(), receipt);
        let mut conflict = request;
        conflict.message = "Different mutation".into();
        assert_eq!(notes.create_note(conflict).await.unwrap_err().code(), tonic::Code::AlreadyExists);

        // Edit Through the Run Envelope and Page Only This Owner
        let id = receipt.note.unwrap().note_id;
        let input = v1::NotesEditNoteRunInput {
            value: Some(v1::notes_edit_note_run_input::Value::Request(v1::NotesEditNoteRequest {
                epoch: epoch.clone(),
                note_id: id.clone(),
                message: "Connector inspected".into(),
            })),
        };
        let mut edit = notes.run_edit_note(tokio_stream::iter([input])).await?.into_inner();
        assert!(matches!(edit.message().await?.unwrap().value,
            Some(v1::notes_edit_note_run_output::Value::Response(reply)) if reply.note.as_ref().unwrap().message == "Connector inspected"));
        assert!(edit.message().await?.is_none());
        let selection = Some(v1::AnnotationSelection {
            owner: Some(owner),
            include_context: true,
        });
        let page = notes
            .list_notes(v1::NotesListNotesRequest {
                selection: selection.clone(),
                limit: 1,
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(page.notes.len(), 1);
        assert_eq!(page.notes[0].scope.as_ref().unwrap().execution_id.as_deref(), Some("notes-a"));
        assert_eq!(page.notes[0].message, "Connector inspected");
        let other = v1::AnnotationOwner {
            owner: Some(v1::AnnotationOwnerOwner {
                kind: Some(v1::annotation_owner_owner::Kind::Execution(v1::ExecutionNoteOwner {
                    cycle_id: "notes-cycle".into(),
                    execution_id: "notes-b".into(),
                })),
            }),
        };
        assert!(
            notes
                .list_notes(v1::NotesListNotesRequest {
                    selection: Some(v1::AnnotationSelection {
                        owner: Some(other),
                        include_context: false
                    }),
                    cursor: page.cursor.clone(),
                    ..Default::default()
                })
                .await
                .is_err()
        );

        // Refuse Edits Once the Owner Closes
        store.close("notes-a", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("notes-a")?;
        assert!(
            notes
                .edit_note(v1::NotesEditNoteRequest {
                    epoch: epoch.clone(),
                    note_id: id.clone(),
                    message: "Too late".into()
                })
                .await
                .is_err()
        );

        // Reopen Durable Notes and Refuse Stale Epoch Mutations
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut notes = v1::notes_client::NotesClient::new(channel);
        let page = notes.list_notes(v1::NotesListNotesRequest { selection, ..Default::default() }).await?.into_inner();
        assert_eq!(page.notes[0].note_id, id);
        assert_eq!(page.notes[0].message, "Connector inspected");
        assert_eq!(
            notes
                .edit_note(v1::NotesEditNoteRequest {
                    epoch,
                    note_id: id,
                    message: "Stale".into()
                })
                .await
                .unwrap_err()
                .code(),
            tonic::Code::FailedPrecondition
        );
        process.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
