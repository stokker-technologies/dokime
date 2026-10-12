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

// Third Party Imports

// Local Imports
use super::*;
use dokime_app::capabilities::evidence::OperatorNoteRequest;
use dokime_app::core::{RecordScope, TestCycleId};
use dokime_app::execution::HealthSeverity;
use dokime_app::runtime::command::CommandContext;
use dokime_app::transactions::CycleDefinition;
use dokime_protocol::v1::{health_client::HealthClient, notes_client::NotesClient};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn cycle(runtime: &dokime_app::RuntimeHandle) -> TestCycleId {
    let suite = common::fixture_suite(&["passed"]);
    common::register_fixture_execution(runtime, suite.clone()).await;
    runtime
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: common::metadata("notes.fixture", "Notes"),
                cases: common::materialize_cases(suite.metadata.identifier.as_str(), &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap()
}

fn selection(cycle: &TestCycleId, context: bool) -> Option<v1::AnnotationSelection> {
    Some(v1::AnnotationSelection {
        owner: Some(v1::AnnotationOwner {
            owner: Some(v1::annotation_owner::Owner::CycleId(cycle.to_string())),
        }),
        include_context: context,
    })
}

pub(crate) async fn scoped_annotation_workflow() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let first = cycle(&handle).await;
    let second = cycle(&handle).await;
    let epoch = handle.process_epoch().to_string();
    handle
        .add_operator_note(OperatorNoteRequest {
            scope: None,
            author: None,
            message: "session context".into(),
        })
        .await
        .unwrap();
    handle
        .add_health_item(
            RecordScope::default().with_cycle(first.clone()),
            HealthSeverity::Warning,
            "connector warning".into(),
            Some("inspect connector".into()),
        )
        .await
        .unwrap();
    handle
        .add_health_item(
            RecordScope::default().with_cycle(second.clone()),
            HealthSeverity::Error,
            "unrelated warning".into(),
            None,
        )
        .await
        .unwrap();
    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![
                grant(Role::Control),
                BearerGrant::new(
                    "other-operator".into(),
                    Role::Read,
                    "other-operator-token-for-notes-tests",
                )
                .unwrap(),
            ])
            .unwrap(),
    )
    .await
    .unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut notes = NotesClient::new(stream.clone());
    let mut health = HealthClient::new(stream);
    assert_eq!(
        notes
            .list_notes(v1::ListAnnotationsRequest::default())
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let create = v1::CreateNoteRequest {
        epoch: epoch.clone(),
        mutation_id: uuid::Uuid::now_v7().to_string(),
        owner: selection(&first, false).unwrap().owner,
        message: "x".repeat(16 * 1024),
    };
    let receipt = notes
        .create_note(authenticated(create.clone()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(receipt.epoch, epoch);
    assert_eq!(
        receipt.publication(),
        v1::NotePublication::NoExistingDatasets
    );
    assert_eq!(
        receipt.note.as_ref().unwrap().author.as_deref(),
        Some("fixture")
    );
    assert_eq!(
        receipt,
        notes
            .create_note(authenticated(create.clone()))
            .await
            .unwrap()
            .into_inner()
    );
    let mut changed = create.clone();
    changed.message.replace_range(..1, "y");
    assert_eq!(
        notes
            .create_note(authenticated(changed))
            .await
            .unwrap_err()
            .code(),
        Code::AlreadyExists
    );
    let mut wrong_epoch = create.clone();
    wrong_epoch.epoch = uuid::Uuid::now_v7().to_string();
    assert_eq!(
        notes
            .create_note(authenticated(wrong_epoch))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    let page = notes
        .list_notes(authenticated(v1::ListAnnotationsRequest {
            selection: selection(&first, false),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.notes.len(), 1);
    let contextual = notes
        .list_notes(authenticated(v1::ListAnnotationsRequest {
            selection: selection(&first, true),
            limit: 1,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(!contextual.next_after_id.is_empty());
    let next = notes
        .list_notes(authenticated(v1::ListAnnotationsRequest {
            selection: selection(&first, true),
            limit: 1,
            after_id: contextual.next_after_id.clone(),
            cursor: contextual.cursor.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(next.notes.len(), 1);
    assert_ne!(next.notes[0].note_id, contextual.notes[0].note_id);
    assert_eq!(
        notes
            .list_notes(authenticated(v1::ListAnnotationsRequest {
                selection: selection(&second, true),
                limit: 1,
                after_id: contextual.next_after_id.clone(),
                cursor: contextual.cursor.clone()
            }))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    let health = health
        .list_health(authenticated(v1::ListAnnotationsRequest {
            selection: selection(&first, false),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(health.items.len(), 2); // Native cycle-ready observation plus the warning.
    assert!(
        health
            .items
            .iter()
            .all(|item| item.scope.as_ref().unwrap().cycle_id.as_deref() == Some(first.as_str()))
    );
    let warning = health
        .items
        .iter()
        .find(|item| item.message == "connector warning")
        .unwrap();
    assert_eq!(warning.severity(), v1::Severity::Warn);
    assert_eq!(warning.detail.as_deref(), Some("inspect connector"));
    assert!(
        health
            .items
            .iter()
            .any(|item| item.message == "Cycle ready for execution.")
    );
    let mut another = create.clone();
    another.mutation_id = uuid::Uuid::now_v7().to_string();
    notes.create_note(authenticated(another)).await.unwrap();
    assert_eq!(
        notes
            .list_notes(authenticated(v1::ListAnnotationsRequest {
                selection: selection(&first, true),
                limit: 1,
                after_id: contextual.next_after_id,
                cursor: contextual.cursor
            }))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    let case_id = handle
        .watch_state()
        .latest()
        .state()
        .cycle_view(&first)
        .unwrap()
        .cases()
        .next()
        .unwrap()
        .test_case_id
        .to_string();
    let case_owner = v1::AnnotationOwner {
        owner: Some(v1::annotation_owner::Owner::PlannedCase(
            v1::PlannedCaseNoteOwner {
                cycle_id: first.to_string(),
                case_id: case_id.clone(),
            },
        )),
    };
    let mut case_request = create.clone();
    case_request.owner = Some(case_owner.clone());
    case_request.mutation_id = uuid::Uuid::now_v7().to_string();
    case_request.message = "planned case note".into();
    let case_receipt = notes
        .create_note(authenticated(case_request))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        case_receipt
            .note
            .as_ref()
            .unwrap()
            .scope
            .as_ref()
            .unwrap()
            .case_id
            .as_deref(),
        Some(case_id.as_str())
    );
    let case_notes = notes
        .list_notes(authenticated(v1::ListAnnotationsRequest {
            selection: Some(v1::AnnotationSelection {
                owner: Some(case_owner),
                include_context: false,
            }),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(case_notes.notes, vec![case_receipt.note.unwrap()]);
    let mut oversized = create;
    oversized.mutation_id = uuid::Uuid::now_v7().to_string();
    oversized.message = "x".repeat(16 * 1024 + 1);
    assert_eq!(
        notes
            .create_note(authenticated(oversized))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
