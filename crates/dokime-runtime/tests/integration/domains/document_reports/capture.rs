/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::service_support::prelude::*;
use dokime_domain::model;
use dokime_domain::model::identity::CommandId;
use dokime_runtime::domains::evidence::notes::{CreateNote, EditNote, NoteError};
use model::execution::notes::NoteOwner;

// operator notes are editable by their author until the seal freezes them.
pub(super) async fn notes_until_seal() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, handle, id, _executor) = blocked_cycle(root.path()).await;
    let running = wait_for_cycle_lifecycle(&handle, &id, CycleStatus::Running).await;
    let execution = &running.test_executions[0];
    let request = CreateNote {
        epoch: handle.process_epoch(),
        mutation_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        owner: NoteOwner::Execution {
            cycle_id: id.clone(),
            execution_id: execution.test_execution_id.clone(),
        },
        actor: "operator".into(),
        message: "Fan **audible** at start.".into(),
    };
    let filed = handle.create_current_note(request.clone()).await.unwrap().admission.note;
    assert_eq!(filed.author.as_deref(), Some("operator"));
    assert_eq!(filed.scope.test_execution_id, Some(execution.test_execution_id.clone()));
    let edit = EditNote {
        epoch: handle.process_epoch(),
        note_id: filed.note_id.clone(),
        actor: "operator".into(),
        message: "Fan **audible** at start; quiet after warm-up.".into(),
    };
    assert!(matches!(
        handle
            .edit_current_note(EditNote {
                actor: "someone-else".into(),
                ..edit.clone()
            })
            .await,
        Err(NoteError::NotAuthor)
    ));
    let edited = handle.edit_current_note(edit.clone()).await.unwrap();
    assert_eq!(edited.note_id, filed.note_id);
    assert_eq!(edited.created_at, filed.created_at);
    assert_eq!(edited.message, edit.message);

    let cycle_note = handle
        .create_current_note(CreateNote {
            mutation_id: dokime_runtime::model::identity::new_id::<CommandId>(),
            owner: NoteOwner::Cycle { cycle_id: id.clone() },
            message: "Bench ambient 23 °C.".into(),
            ..request.clone()
        })
        .await
        .unwrap()
        .admission
        .note;

    handle.seal_cycle(id.clone(), CommandContext::default()).await.unwrap();
    let stopped = wait_for_cycle_lifecycle(&handle, &id, CycleStatus::Stopped).await;
    let seal = stopped.saved.as_ref().expect("stopping finalizes the cycle report");
    let payload = crate::service_support::reports::payload(handle.storage(), &seal.report).await;
    let notes = payload["snapshot"]["notes"].as_array().expect("the sealed report carries its notes");
    assert_eq!(
        notes
            .iter()
            .filter(|note| note["note_id"] == filed.note_id.as_str())
            .map(|note| (note["author"].as_str(), note["text"].as_str()))
            .collect::<Vec<_>>(),
        [(Some("operator"), Some(edit.message.as_str()))],
        "the sealed report keeps the note as last edited"
    );

    let late = CreateNote {
        mutation_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        message: "Filed after the seal.".into(),
        ..request
    };
    assert!(matches!(handle.create_current_note(late.clone()).await, Err(NoteError::Sealed)));
    assert!(matches!(
        handle
            .create_current_note(CreateNote {
                owner: NoteOwner::Cycle { cycle_id: id.clone() },
                ..late
            })
            .await,
        Err(NoteError::Sealed)
    ));
    assert!(matches!(
        handle
            .edit_current_note(EditNote {
                note_id: cycle_note.note_id,
                message: "Bench ambient 24 °C.".into(),
                ..edit.clone()
            })
            .await,
        Err(NoteError::Sealed)
    ));
    assert!(matches!(handle.edit_current_note(edit).await, Err(NoteError::Sealed)));
    owner.shutdown().await.unwrap();
}

// an undeclared record type that reaches finalization settles the run, naming it.
#[cfg(feature = "fault-injection")]
pub(super) async fn undeclared_records_settle() {
    const UNDECLARED: &str = "operator.note_deleted";
    let root = tempfile::tempdir().unwrap();
    let (mut owner, handle, id, _executor) = blocked_cycle(root.path()).await;
    let running = wait_for_cycle_lifecycle(&handle, &id, CycleStatus::Running).await;
    let execution = running.test_executions[0].clone();
    let mut request = RecordPublishRequest::new(Identifier::new(UNDECLARED).unwrap(), RecordSeverity::Info);
    request.scope = Some(execution.scope.clone());

    // The publish boundary refuses a type no contract declares, naming it.
    let refused = handle.publish_record(request.clone()).await.expect_err("an undeclared record type is refused at publication");
    assert!(refused.to_string().contains(UNDECLARED), "the refusal names the type: {refused}");

    // One that reaches finalization anyway settles the run, naming the type,
    // instead of retrying a report that can never be sealed.
    handle.publish_undeclared_record(request).await.unwrap();
    // Stop freezes the already-persisted boundary, so the record must be saved first.
    wait_for_record_type(&handle, UNDECLARED).await;
    handle.storage().flush().await.unwrap();
    let sealing = handle.seal_cycle(id.clone(), CommandContext::default()).await;
    assert!(sealing.is_err(), "failed evidence durability cannot acknowledge a sealed cycle");
    // The Run Fails Publication; The Container Retains Its Truthful Outcome.
    wait_for_record_type(&handle, "test.cycle.finalization.failed").await;
    let settled = wait_for_cycle_lifecycle(&handle, &id, CycleStatus::Stopped).await;
    assert_eq!(settled.phase, dokime_domain::model::execution::result::CyclePhase::Sealing);
    assert!(settled.saved.is_none(), "an unsealable cycle claims no report");
    let evidence = handle.evidence_snapshot().await.unwrap();
    for (record_type, scoped) in
        [("test.execution.finalization.failed", Some(&execution.test_execution_id)), ("test.cycle.finalization.failed", None)]
    {
        let record = evidence
            .records
            .iter()
            .find(|record| {
                record.record_type.as_str() == record_type
                    && record.scope.as_ref().is_some_and(|scope| scope.test_cycle_id.as_ref() == Some(&id))
            })
            .unwrap_or_else(|| panic!("{record_type} is recorded as evidence"));
        assert_eq!(record.severity, RecordSeverity::Error);
        assert_eq!(record.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()), scoped, "{record_type} scope");
        assert!(
            record.message.as_deref().is_some_and(|message| message.contains(UNDECLARED)),
            "{record_type} names the undeclared type: {record:?}"
        );
    }
    // The Stop Leaves The Failed Seal Where It Is; The Next Start Retries It.
    owner.shutdown().await.unwrap();
}
