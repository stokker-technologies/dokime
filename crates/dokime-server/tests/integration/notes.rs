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
use dokime_domain::model;
use super::*;
use dokime_runtime::RuntimeConfig;
use dokime_runtime::{RecordScope, TestCycleId};
use dokime_runtime::domains::evidence::OperatorNoteRequest;
use dokime_runtime::execution::HealthSeverity;
use dokime_runtime::runtime::command::CommandContext;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn cycle(runtime: &dokime_runtime::RuntimeHandle) -> TestCycleId {
    let suite = support::fixture_suite(&["passed"]);
    support::register_fixture_execution(runtime, suite.clone()).await;
    support::cycle_with_cases(
        runtime,
        support::metadata("notes.fixture", "Notes"),
        support::materialize_cases(suite.metadata.identifier.as_str(), &suite),
    )
    .await
}

fn selection(cycle: &TestCycleId, context: bool) -> Option<model::notes::wire::AnnotationSelection> {
    Some(model::notes::wire::AnnotationSelection {
        owner: Some(model::notes::wire::AnnotationOwner {
            owner: Some(model::notes::wire::AnnotationOwnerOwner::CycleId(
                cycle.to_string(),
            )),
        }),
        include_context: context,
    })
}

#[tokio::test]
async fn scoped_annotation_workflow() {
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
    let notes = WireClient::new(stream.clone());
    let health = WireClient::new(stream);
    assert_eq!(
        notes
            .unary::<contracts::notes::ListNotes>(
                contracts::notes::ListNotes::default()
            )
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let create = contracts::notes::CreateNote {
        epoch: epoch.clone(),
        mutation_id: uuid::Uuid::now_v7().to_string(),
        owner: selection(&first, false).unwrap().owner,
        message: "x".repeat(16 * 1024),
    };
    let receipt = notes
        .unary::<contracts::notes::CreateNote>(authenticated(create.clone()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(receipt.epoch, epoch);
    assert_eq!(
        receipt.publication,
        model::notes::wire::NotePublication::NoExistingDatasets
    );
    assert_eq!(
        receipt.note.as_ref().unwrap().author.as_deref(),
        Some("fixture")
    );
    assert_eq!(
        receipt,
        notes
            .unary::<contracts::notes::CreateNote>(authenticated(create.clone()))
            .await
            .unwrap()
            .into_inner()
    );
    let mut changed = create.clone();
    changed.message.replace_range(..1, "y");
    assert_eq!(
        notes
            .unary::<contracts::notes::CreateNote>(authenticated(changed))
            .await
            .unwrap_err()
            .code(),
        Code::AlreadyExists
    );
    let mut wrong_epoch = create.clone();
    wrong_epoch.epoch = uuid::Uuid::now_v7().to_string();
    assert_eq!(
        notes
            .unary::<contracts::notes::CreateNote>(authenticated(wrong_epoch))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    let page = notes
        .unary::<contracts::notes::ListNotes>(authenticated(
            contracts::notes::ListNotes {
                selection: selection(&first, false),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.notes.len(), 1);
    let contextual = notes
        .unary::<contracts::notes::ListNotes>(authenticated(
            contracts::notes::ListNotes {
                selection: selection(&first, true),
                limit: 1,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(!contextual.next_after_id.is_empty());
    let next = notes
        .unary::<contracts::notes::ListNotes>(authenticated(
            contracts::notes::ListNotes {
                selection: selection(&first, true),
                limit: 1,
                after_id: contextual.next_after_id.clone(),
                cursor: contextual.cursor.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(next.notes.len(), 1);
    assert_ne!(next.notes[0].note_id, contextual.notes[0].note_id);
    assert_eq!(
        notes
            .unary::<contracts::notes::ListNotes>(authenticated(
                contracts::notes::ListNotes {
                    selection: selection(&second, true),
                    limit: 1,
                    after_id: contextual.next_after_id.clone(),
                    cursor: contextual.cursor.clone()
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    let health = health
        .unary::<contracts::notes::ListHealth>(authenticated(
            contracts::notes::ListHealth {
                selection: selection(&first, false),
                ..Default::default()
            },
        ))
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
    assert_eq!(warning.severity, model::identity::wire::Severity::Warn);
    assert_eq!(warning.detail.as_deref(), Some("inspect connector"));
    assert!(
        health
            .items
            .iter()
            .any(|item| item.message == "Cycle ready for execution.")
    );
    let mut another = create.clone();
    another.mutation_id = uuid::Uuid::now_v7().to_string();
    notes
        .unary::<contracts::notes::CreateNote>(authenticated(another))
        .await
        .unwrap();
    assert_eq!(
        notes
            .unary::<contracts::notes::ListNotes>(authenticated(
                contracts::notes::ListNotes {
                    selection: selection(&first, true),
                    limit: 1,
                    after_id: contextual.next_after_id,
                    cursor: contextual.cursor
                }
            ))
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
    let case_owner = model::notes::wire::AnnotationOwner {
        owner: Some(model::notes::wire::AnnotationOwnerOwner::PlannedCase(
            model::notes::wire::PlannedCaseNoteOwner {
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
        .unary::<contracts::notes::CreateNote>(authenticated(case_request))
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
        .unary::<contracts::notes::ListNotes>(authenticated(
            contracts::notes::ListNotes {
                selection: Some(model::notes::wire::AnnotationSelection {
                    owner: Some(case_owner),
                    include_context: false,
                }),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(case_notes.notes, vec![case_receipt.note.clone().unwrap()]);
    let edit = contracts::notes::EditNote {
        epoch: epoch.clone(),
        note_id: case_receipt.note.as_ref().unwrap().note_id.clone(),
        message: "planned case note, **revised**".into(),
    };
    let edited = notes
        .unary::<contracts::notes::EditNote>(authenticated(edit.clone()))
        .await
        .unwrap()
        .into_inner()
        .note
        .unwrap();
    assert_eq!(edited.message, "planned case note, **revised**");
    assert_eq!(edited.note_id, edit.note_id);
    assert_eq!(
        edited.created_at,
        case_receipt.note.as_ref().unwrap().created_at
    );
    let mut absent = edit.clone();
    absent.note_id = uuid::Uuid::now_v7().to_string();
    assert_eq!(
        notes
            .unary::<contracts::notes::EditNote>(authenticated(absent))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let mut oversized = create;
    oversized.mutation_id = uuid::Uuid::now_v7().to_string();
    oversized.message = "x".repeat(16 * 1024 + 1);
    assert_eq!(
        notes
            .unary::<contracts::notes::CreateNote>(authenticated(oversized))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn sealed_refuses_notes() {
    let root = tempfile::tempdir().unwrap();
    let gate = std::sync::Arc::new(Gate::reporting(|cycle| {
        support::fixture_provider_outcome(cycle, false)
    }));
    let mut owner = Runtime::with_execution_fixture(
        RuntimeConfig::new(root.path(), support::metadata("grpc.notes", "Notes")),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    let handle = owner.handle();
    let id = cycle(&handle).await;
    let server = Server::start(
        handle.clone(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let notes = WireClient::connect(server.local_endpoint()).await.unwrap();
    let create = contracts::notes::CreateNote {
        epoch: handle.process_epoch().to_string(),
        mutation_id: uuid::Uuid::now_v7().to_string(),
        owner: selection(&id, false).unwrap().owner,
        message: "before the seal".into(),
    };
    let filed = notes
        .unary::<contracts::notes::CreateNote>(authenticated(create.clone()))
        .await
        .unwrap()
        .into_inner()
        .note
        .unwrap();
    handle
        .run_cycle_cases(id.clone(), CommandContext::default())
        .await;
    gate.started.notified().await;
    gate.release.notify_one();
    handle
        .watch_state()
        .wait_until(|snapshot| {
            snapshot
                .state()
                .cycle_view(&id)
                .is_some_and(|cycle| cycle.saved.is_some())
        })
        .await
        .expect("the runtime state watch stays open");
    let mut late = create;
    late.mutation_id = uuid::Uuid::now_v7().to_string();
    late.message = "after the seal".into();
    assert_eq!(
        notes
            .unary::<contracts::notes::CreateNote>(authenticated(late))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    assert_eq!(
        notes
            .unary::<contracts::notes::EditNote>(authenticated(contracts::notes::EditNote {
                epoch: handle.process_epoch().to_string(),
                note_id: filed.note_id,
                message: "rewritten after the seal".into(),
            }))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
