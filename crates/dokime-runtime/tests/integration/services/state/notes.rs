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
use harmos::state::Transaction;
// Local Imports
use dokime_runtime::core::{
    CaseId, CommandId, NoteId, RecordScope, SessionId, TestCycleId, TestExecutionId, Timestamp,
};
use dokime_runtime::execution::{CycleView, OperatorNote};
use dokime_runtime::execution::notes::*;
use dokime_runtime::state::RuntimeState;
use dokime_runtime::transactions::{AddOperatorNote, CreateOperatorNote};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn fixture() -> (RuntimeState, NoteAdmission) {
    let mut state = RuntimeState::new(dokime_runtime::model::identity::new_id::<SessionId>());
    dokime_runtime::transactions::MarkRuntimeStarted {
        session_id: state.session_id.clone(),
        record_namespace: dokime_runtime::model::identity::new_id::<SessionId>(),
    }
    .apply(&mut state);
    let cycle_id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    let case_id = dokime_runtime::model::identity::new_id::<CaseId>();
    let cycle:CycleView=serde_json::from_value(serde_json::json!({
        "test_cycle_id":cycle_id,"metadata":{"identifier":"fixture.cycle","label":"Fixture"},
        "created_at":dokime_runtime::model::time::now_utc(),"updated_at":dokime_runtime::model::time::now_utc(),
        "test_executions":[{
            "test_cycle_id": cycle_id, "test_execution_id": dokime_runtime::model::identity::new_id::<TestExecutionId>(), "test_case_id": case_id, "status":"skipped", "scope":{},
            "snapshot":dokime_runtime::execution::ExecutionSnapshot::materialize(
                &serde_json::from_value(serde_json::json!({"suite_id":"fixture.registration","test_case_id":case_id, "test_case_template_id":dokime_runtime::model::identity::new_id::<dokime_runtime::core::TemplateId>(),"metadata":{"identifier":"fixture.case","label":"Case"}})).unwrap(),
                None, Default::default(), None, Default::default(),
            ).unwrap()
        }]
    })).unwrap();
    state
        .replace_cycle(cycle)
        .expect("cycle projection has valid ownership");
    let owner = NoteOwner::Case { cycle_id, case_id };
    let mutation_id = dokime_runtime::model::identity::new_id::<CommandId>();
    let admission = NoteAdmission {
        mutation_id: mutation_id.clone(),
        owner: owner.clone(),
        note: OperatorNote {
            note_id: note_identity(state.process_epoch(), "operator", &mutation_id),
            scope: state.note_owner_scope(&owner).unwrap(),
            author: Some("operator".into()),
            message: "inspected connector".into(),
            created_at: dokime_runtime::model::time::now_utc(),
        },
        checkpoint_targets: Vec::new(),
    };
    (state, admission)
}

/* === Tests === */

#[test]
fn note_mutations_restarts() {
    let (mut state, admission) = fixture();
    let mut create = CreateOperatorNote { admission };
    create.check(&state).unwrap();
    let scope = create.admission.note.scope.clone();

    dokime_runtime::transactions::MarkRuntimeStarted {
        session_id: state.session_id.clone(),
        record_namespace: dokime_runtime::model::identity::new_id::<SessionId>(),
    }
    .apply(&mut state);
    assert!(create.check(&state).unwrap_err().contains("epoch/actor"));

    create.admission.note.note_id = note_identity(
        state.process_epoch(),
        "operator",
        &create.admission.mutation_id,
    );
    create.check(&state).unwrap();
    assert_eq!(create.admission.note.scope, scope);
}

#[test]
fn retry_identity_retirement() {
    let (mut state, admission) = fixture();
    let create = CreateOperatorNote {
        admission: admission.clone(),
    };
    create.check(&state).unwrap();
    create.apply(&mut state);
    assert_eq!(state.iter_notes().count(), 1);
    let mut retry = create.clone();
    retry.admission.note.created_at = dokime_runtime::model::time::now_utc();
    retry.check(&state).unwrap();
    retry.apply(&mut state);
    assert_eq!(state.iter_notes().count(), 1);
    assert_eq!(state.note_admissions[&admission.note.note_id], admission);
    state.cycles.clear();
    retry.check(&state).unwrap();
    retry.apply(&mut state);
    assert!(state.cycles.is_empty());
    assert_eq!(state.note_admissions.len(), 1);
    retry.admission.note.message.push('!');
    assert!(retry.check(&state).is_err());
    retry = create.clone();
    retry.admission.owner = NoteOwner::Cycle {
        cycle_id: dokime_runtime::model::identity::new_id::<TestCycleId>(),
    };
    assert!(retry.check(&state).is_err());
    let old_id = create.admission.note.note_id;
    assert_ne!(
        old_id,
        note_identity(&dokime_runtime::model::identity::new_id::<SessionId>(), "operator", &admission.mutation_id)
    );
    assert_ne!(
        old_id,
        note_identity(
            state.process_epoch(),
            "another operator",
            &admission.mutation_id
        )
    );
}

#[test]
fn rejects_missing_owners() {
    let (mut state, admission) = fixture();
    let cycle_id = admission.owner.cycle_id().clone();
    let mut note = admission.note;
    note.scope.test_execution_id = Some(dokime_runtime::model::identity::new_id::<TestExecutionId>());
    let transaction = AddOperatorNote { note: note.clone() };
    assert!(transaction.check(&state).is_err());
    assert!(state.cycles[&cycle_id].records.notes().is_empty());
    note.scope.test_execution_id = None;
    note.scope.session_id = Some(dokime_runtime::model::identity::new_id::<SessionId>());
    assert!(
        AddOperatorNote { note: note.clone() }
            .check(&state)
            .is_err()
    );
    note.scope = state.note_owner_scope(&admission.owner).unwrap();
    let removed = std::mem::take(&mut state.cycles.get_mut(&cycle_id).unwrap().execution_ids);
    for id in removed {
        state.executions.remove(&id);
    }
    assert!(AddOperatorNote { note }.check(&state).is_err());
}

#[test]
fn capacity_preserves_receipts() {
    let (mut state, admission) = fixture();
    for _ in 0..NOTE_RECEIPT_LIMIT {
        let mut item = admission.clone();
        item.mutation_id = dokime_runtime::model::identity::new_id::<CommandId>();
        item.note.note_id = note_identity(state.process_epoch(), "operator", &item.mutation_id);
        state
            .note_admissions
            .insert(item.note.note_id.clone(), item);
    }
    let existing = state.note_admissions.values().next().unwrap().clone();
    assert!(
        CreateOperatorNote {
            admission: admission.clone()
        }
        .check(&state)
        .unwrap_err()
        .contains("capacity")
    );
    CreateOperatorNote {
        admission: existing.clone(),
    }
    .check(&state)
    .unwrap();
    assert_eq!(state.note_admissions.len(), NOTE_RECEIPT_LIMIT);
    state.note_admissions.clear();
    for _ in 0..600 {
        let mut item = admission.clone();
        item.note.note_id = dokime_runtime::model::identity::new_id::<NoteId>();
        item.note.message = "x".repeat(NOTE_TEXT_BYTES);
        state
            .note_admissions
            .insert(item.note.note_id.clone(), item);
    }
    assert!(
        CreateOperatorNote { admission }
            .check(&state)
            .unwrap_err()
            .contains("capacity")
    );
}

#[test]
fn scoped_read_bounds() {
    let (mut state, admission) = fixture();
    let cycle = state.cycle_views().next().unwrap();
    for _ in 0..NOTE_SCAN_LIMIT {
        let mut cycle = cycle.clone();
        cycle.test_cycle_id = dokime_runtime::model::identity::new_id::<TestCycleId>();
        for execution in &mut cycle.test_executions {
            execution.test_cycle_id = cycle.test_cycle_id.clone();
            execution.test_execution_id = dokime_runtime::model::identity::new_id::<dokime_runtime::core::TestExecutionId>();
        }
        state
            .replace_cycle(cycle)
            .expect("cycle projection has valid ownership");
    }
    assert!(state.annotation_sets(None, false).is_err());
    assert!(state.annotation_sets(Some(&admission.owner), false).is_ok());
    state.session_records_mut().push_note(OperatorNote {
        note_id: dokime_runtime::model::identity::new_id::<NoteId>(),
        scope: RecordScope::default(),
        author: None,
        message: "x".repeat(NOTE_TEXT_BYTES + 1),
        created_at: dokime_runtime::model::time::now_utc(),
    });
    let invalid = &state.session_records().notes()[0];
    assert!(state.validate_operator_note(invalid).is_err());
}
