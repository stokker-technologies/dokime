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
        let mut notes = dokime_protocol::client::evidence::notes_client::NotesClient::new(channel);
        let owner = v1::AnnotationOwner {
            owner: Some(v1::AnnotationOwnerOwner {
                kind: Some(v1::annotation_owner_owner::Kind::Execution(v1::ExecutionNoteOwner {
                    cycle_id: "notes-cycle".into(),
                    execution_id: "notes-a".into(),
                })),
            }),
        };
        let value = dokime_protocol::client::evidence::NoteWritable {
            message: uuid::Uuid::now_v7().to_string(),
        };
        let request = dokime_protocol::client::evidence::NotesCreateRequest {
            owner: Some(owner.clone()),
            value: Some(value.clone()),
            request_id: uuid::Uuid::now_v7().to_string(),
        };
        let receipt = notes.create(request.clone()).await?.into_inner();
        assert_eq!(notes.create(request.clone()).await?.into_inner(), receipt);
        let mut conflict = request;
        conflict.value.as_mut().unwrap().message.push('x');
        assert!(notes.create(conflict).await.is_err());
        let name = receipt.note.unwrap().note_id;
        let first =
            notes.get(dokime_protocol::client::evidence::NotesGetRequest { name: name.clone(), ..Default::default() }).await?.into_inner();
        let changed = dokime_protocol::client::evidence::NoteWritable {
            message: uuid::Uuid::now_v7().to_string(),
        };
        let update = dokime_protocol::client::evidence::NotesSetRequest {
            name: name.clone(),
            value: Some(changed.clone()),
            expected_revision: first.revision.clone(),
            request_id: "edit".into(),
        };
        let saved = notes.set(update.clone()).await?.into_inner();
        assert_eq!(saved.value, Some(changed.clone()));
        assert_eq!(notes.set(update.clone()).await?.into_inner(), saved);
        let selection = Some(v1::AnnotationSelection {
            owner: Some(owner),
            include_context: true,
        });
        let page = notes
            .list(dokime_protocol::client::evidence::NotesListRequest {
                selection: selection.clone(),
                page: Some(harmos_protocol::v1::PageRequest { page_size: 1, ..Default::default() }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(page.notes.len(), 1);
        assert_eq!(page.notes[0].scope.as_ref().unwrap().execution_id.as_deref(), Some("notes-a"));
        assert_eq!(page.notes[0].message, changed.message);
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
                .list(dokime_protocol::client::evidence::NotesListRequest {
                    selection: Some(v1::AnnotationSelection {
                        owner: Some(other),
                        include_context: false
                    }),
                    at: page.resolved_at,
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let pinned = notes
            .get(dokime_protocol::client::evidence::NotesGetRequest {
                name: name.clone(),
                at: first.resolved_at.clone(),
            })
            .await?
            .into_inner();
        assert_eq!(pinned.value, Some(value));
        store.close("notes-a", v1::Termination::Completed, v1::ExecutionOutcome::default())?;
        store.seal("notes-a")?;
        let mut closed = update;
        closed.request_id = "closed".into();
        closed.expected_revision = saved.revision.clone();
        assert!(notes.set(closed.clone()).await.is_err());
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut notes = dokime_protocol::client::evidence::notes_client::NotesClient::connect(process.url.clone()).await?;
        let page = notes.list(dokime_protocol::client::evidence::NotesListRequest { selection, ..Default::default() }).await?.into_inner();
        assert_eq!(page.notes[0].note_id, name);
        assert_eq!(page.notes[0].message, changed.message);
        assert!(notes.set(closed).await.is_err());
        process.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
