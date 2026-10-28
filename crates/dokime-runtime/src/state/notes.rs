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
use super::RuntimeState;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthItem;
use dokime_domain::model::execution::notes::NOTE_OWNER_SEALED;
use dokime_domain::model::execution::notes::NoteOwner;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::identity::NoteId;

/* === Constants === */

pub(crate) const NOTE_TEXT_BYTES: usize = 16 * 1024;
pub(crate) const NOTE_OWNER_BYTES: usize = 4096;
pub(crate) const NOTE_ACTOR_BYTES: usize = 256;
pub(crate) const NOTE_RECEIPT_LIMIT: usize = 4096;
pub(crate) const NOTE_RECEIPT_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const NOTE_CHECKPOINT_LIMIT: usize = 64;
pub(crate) const NOTE_SCAN_LIMIT: usize = 10_000;
pub(crate) const NOTE_TIMESTAMP_BYTES: usize = 128;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeState {
    pub fn note_owner_scope(&self, owner: &NoteOwner) -> Result<RecordScope, String> {
        let cycle = self.cycles.get(owner.cycle_id()).ok_or("note owner cycle is absent")?;
        let mut scope = RecordScope::session(self.session_id.clone()).with_cycle(cycle.test_cycle_id.clone());
        match owner {
            NoteOwner::Cycle { .. } => {}
            NoteOwner::Case { case_id, .. } => {
                if case_id.as_str().len() > NOTE_OWNER_BYTES {
                    return Err("case identity exceeds annotation budget".into());
                }
                let case = cycle
                    .execution_ids
                    .iter()
                    .take(NOTE_SCAN_LIMIT)
                    .filter_map(|id| self.executions.get(id))
                    .map(|execution| execution.snapshot.case())
                    .find(|case| &case.test_case_id == case_id)
                    .ok_or("note case is absent or owner scan budget exceeded")?;
                scope.test_case_id = Some(case.test_case_id.clone());
                scope.template_id = Some(case.test_case_template_id.clone());
            }
            NoteOwner::Execution { execution_id, .. } => {
                let execution = cycle
                    .execution_ids
                    .iter()
                    .take(NOTE_SCAN_LIMIT)
                    .filter_map(|id| self.executions.get(id))
                    .find(|item| &item.test_execution_id == execution_id)
                    .ok_or("note execution is absent or owner scan budget exceeded")?;
                if execution.test_case_id.as_str().len() > NOTE_OWNER_BYTES
                    || execution.scope.test_case_id.as_ref().is_some_and(|id| id.as_str().len() > NOTE_OWNER_BYTES)
                {
                    return Err("case identity exceeds annotation budget".into());
                }
                scope = execution.scope.clone();
                scope.session_id = Some(self.session_id.clone());
                scope.test_cycle_id = Some(cycle.test_cycle_id.clone());
                scope.test_case_id = Some(execution.test_case_id.clone());
                scope.test_execution_id = Some(execution.test_execution_id.clone());
                scope.step_id = None;
                scope.criterion_id = None;
            }
        }
        Ok(scope)
    }

    pub fn canonical_note_scope(&self, given: &RecordScope) -> Result<RecordScope, String> {
        let owner = match (&given.test_cycle_id, &given.test_case_id, &given.test_execution_id) {
            (Some(cycle_id), _, Some(execution_id)) => Some(NoteOwner::Execution {
                cycle_id: cycle_id.clone(),
                execution_id: execution_id.clone(),
            }),
            (Some(cycle_id), Some(case_id), None) => Some(NoteOwner::Case {
                cycle_id: cycle_id.clone(),
                case_id: case_id.clone(),
            }),
            (Some(cycle_id), None, None) => Some(NoteOwner::Cycle { cycle_id: cycle_id.clone() }),
            (None, None, None) => None,
            _ => return Err("note owner requires its cycle identity".into()),
        };
        let mut actual = match &owner {
            Some(owner) => self.note_owner_scope(owner)?,
            None => {
                if given.session_id.is_some() {
                    RecordScope::session(self.session_id.clone())
                } else {
                    RecordScope::default()
                }
            }
        };
        if given.step_id.is_some() || given.criterion_id.is_some() {
            let Some(NoteOwner::Execution { cycle_id, execution_id }) = &owner else {
                return Err("step/criterion notes require an execution".into());
            };
            let execution = self
                .executions
                .get(execution_id)
                .filter(|execution| &execution.test_cycle_id == cycle_id)
                .ok_or("execution disappeared")?;
            let mut remaining = NOTE_SCAN_LIMIT;
            let mut found = false;
            for step in &execution.steps {
                remaining = remaining.checked_sub(1).ok_or("note step scan budget exceeded")?;
                if given.step_id.as_ref().is_some_and(|id| id != &step.step_id) {
                    continue;
                }
                if let Some(id) = &given.criterion_id {
                    for criterion in &step.criteria {
                        remaining = remaining.checked_sub(1).ok_or("note criterion scan budget exceeded")?;
                        if &criterion.criterion_id == id {
                            found = true;
                            break;
                        }
                    }
                } else {
                    found = true;
                }
                if found {
                    break;
                }
            }
            if !found {
                return Err("note step/criterion does not belong to execution".into());
            }
            actual.step_id = given.step_id.clone();
            actual.criterion_id = given.criterion_id.clone();
        }
        macro_rules! check {
            ($field:ident) => {
                if given.$field.as_ref().is_some_and(|id| actual.$field.as_ref() != Some(id)) {
                    return Err(concat!("note scope mismatch: ", stringify!($field)).into());
                }
            };
        }
        check!(session_id);
        check!(test_cycle_id);
        check!(test_case_id);
        check!(test_execution_id);
        check!(template_id);
        check!(suite_id);
        Ok(actual)
    }

    pub fn validate_operator_note(&self, note: &OperatorNote) -> Result<(), String> {
        if self.shutting_down {
            return Err("runtime is shutting down".into());
        }
        if note.scope.test_case_id.as_ref().is_some_and(|id| id.as_str().len() > NOTE_OWNER_BYTES)
            || note.created_at.as_str().len() > NOTE_TIMESTAMP_BYTES
            || note.message.trim().is_empty()
            || note.message.len() > NOTE_TEXT_BYTES
            || note.author.as_ref().is_some_and(|author| author.len() > NOTE_ACTOR_BYTES)
        {
            return Err("note text/author exceeds native payload bounds or is empty".into());
        }
        if self.canonical_note_scope(&note.scope)? != note.scope {
            return Err("note scope is not canonical".into());
        }
        if self.note_scope_sealed(&note.scope) {
            return Err(NOTE_OWNER_SEALED.into());
        }
        Ok(())
    }

    // A Sealed Cycle Or Execution Keeps The Notes Its Report Froze
    pub fn note_scope_sealed(&self, scope: &RecordScope) -> bool {
        let cycle_sealed = scope.test_cycle_id.as_ref().and_then(|id| self.cycles.get(id)).is_some_and(|cycle| cycle.saved.is_some());
        let execution_sealed =
            scope.test_execution_id.as_ref().and_then(|id| self.executions.get(id)).is_some_and(|execution| execution.saved.is_some());
        cycle_sealed || execution_sealed
    }

    // An Execution That Sealed Froze Every Ancestor Note It Captured, So Editing
    // A Cycle Or Case Note Stops Once Any Execution Beneath It Has Sealed.
    pub fn note_edit_sealed(&self, scope: &RecordScope) -> bool {
        if self.note_scope_sealed(scope) {
            return true;
        }
        if scope.test_execution_id.is_some() {
            return false;
        }
        let Some(cycle) = scope.test_cycle_id.as_ref().and_then(|id| self.cycles.get(id)) else {
            return false;
        };
        cycle
            .execution_ids
            .iter()
            .filter_map(|id| self.executions.get(id))
            .filter(|execution| scope.test_case_id.as_ref().is_none_or(|id| id == &execution.test_case_id))
            .any(|execution| execution.saved.is_some())
    }

    pub fn find_note(&self, id: &NoteId) -> Option<&OperatorNote> {
        self.iter_notes().find(|note| &note.note_id == id)
    }

    pub fn iter_notes(&self) -> impl Iterator<Item = &OperatorNote> {
        self.session_records().notes().iter().chain(self.cycles.values().flat_map(|cycle| {
            cycle.records.notes().iter().chain(
                cycle.execution_ids.iter().filter_map(|id| self.executions.get(id)).flat_map(|execution| execution.records.notes().iter()),
            )
        }))
    }
    pub fn iter_health_items(&self) -> impl Iterator<Item = &HealthItem> {
        self.session_records().health().iter().chain(self.cycles.values().flat_map(|cycle| {
            cycle.records.health().iter().chain(
                cycle.execution_ids.iter().filter_map(|id| self.executions.get(id)).flat_map(|execution| execution.records.health().iter()),
            )
        }))
    }
}

impl RuntimeState {
    pub fn annotation_sets(
        &self, owner: Option<&NoteOwner>, include_context: bool,
    ) -> Result<Vec<&dokime_domain::state::records::set::TestRecordSet>, String> {
        if let Some(owner) = owner {
            self.note_owner_scope(owner)?;
        }
        let mut remaining = NOTE_SCAN_LIMIT;
        let mut sets = Vec::new();
        fn add<'a>(
            sets: &mut Vec<&'a dokime_domain::state::records::set::TestRecordSet>, remaining: &mut usize,
            records: &'a dokime_domain::state::records::set::TestRecordSet,
        ) -> Result<(), String> {
            let cost = 1usize.saturating_add(records.notes().len()).saturating_add(records.health().len());
            *remaining = remaining.checked_sub(cost).ok_or("annotation scan budget exceeded")?;
            sets.push(records);
            Ok(())
        }
        if owner.is_none() || include_context {
            add(&mut sets, &mut remaining, self.session_records())?;
        }
        let cycles: Box<dyn Iterator<Item = &dokime_domain::model::execution::result::TestCycle> + '_> = match owner {
            Some(owner) => Box::new(self.cycles.get(owner.cycle_id()).into_iter()),
            None => Box::new(self.cycles.values()),
        };
        for cycle in cycles {
            add(&mut sets, &mut remaining, &cycle.records)?;
            for id in &cycle.execution_ids {
                let execution = self.executions.get(id).ok_or("cycle execution owner is absent")?;
                remaining = remaining.checked_sub(1).ok_or("annotation owner scan budget exceeded")?;
                let selected = match owner {
                    Some(NoteOwner::Execution { execution_id, .. }) => execution_id == &execution.test_execution_id,
                    Some(NoteOwner::Case { case_id, .. }) => case_id == &execution.test_case_id,
                    _ => true,
                };
                if selected {
                    add(&mut sets, &mut remaining, &execution.records)?;
                    if matches!(owner, Some(NoteOwner::Execution { .. })) {
                        break;
                    }
                }
            }
        }
        Ok(sets)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{NOTE_RECEIPT_LIMIT, NOTE_SCAN_LIMIT, NOTE_TEXT_BYTES};
    use harmos::state::Transaction;
    // Local Imports
    use crate::state::RuntimeState;
    use crate::transactions::{AddOperatorNote, CreateOperatorNote};
    use crate::{CaseId, NoteId, RecordScope, SessionId, TestCycleId, TestExecutionId};
    use dokime_domain::model;
    use dokime_domain::model::execution::result::CycleView;
    use dokime_domain::model::identity::CommandId;
    use model::execution::OperatorNote;
    use model::execution::notes::*;

    fn fixture() -> (RuntimeState, NoteAdmission) {
        let mut state = RuntimeState::new(SessionId::from_seed("notes-session"));
        crate::transactions::MarkRuntimeStarted {
            session_id: state.session_id.clone(),
            record_namespace: SessionId::from_seed("notes-epoch"),
        }
        .apply(&mut state);
        let cycle_id = TestCycleId::from_seed("notes-cycle");
        let case_id = CaseId::from_seed("notes-case");
        let cycle: CycleView = serde_json::from_value(serde_json::json!({
            "test_cycle_id":cycle_id,"metadata":{"identifier":"fixture.cycle","label":"Fixture"},"phase":"active",
            "created_at":crate::Timestamp::new("2026-10-19T10:00:00Z"),"updated_at":crate::Timestamp::new("2026-10-19T10:00:00Z"),
            "test_executions":[{
                "test_cycle_id": cycle_id,
                    "test_execution_id": TestExecutionId::from_seed("notes-execution"),
                    "test_case_id": case_id,
                    "status":"skipped",
                    "scope":{},
                "snapshot":dokime_domain::model::execution::snapshot::ExecutionSnapshot::materialize(
                    &serde_json::from_value(serde_json::json!({"suite_id":"fixture.registration",
                        "test_case_id":case_id,
                        "test_case_template_id":crate::TemplateId::from_seed("notes-template"),
                        "metadata":{"identifier":"fixture.case",
                        "label":"Case"}})).unwrap(),
                    None, Default::default(), None, Default::default(),
                ).unwrap()
            }]
        }))
        .unwrap();
        state.replace_cycle(cycle).expect("cycle projection has valid ownership");
        let owner = NoteOwner::Case { cycle_id, case_id };
        let mutation_id = CommandId::from_seed("notes-creation");
        let admission = NoteAdmission {
            mutation_id: mutation_id.clone(),
            owner: owner.clone(),
            note: OperatorNote {
                note_id: note_identity(state.process_epoch(), "operator", &mutation_id),
                scope: state.note_owner_scope(&owner).unwrap(),
                author: Some("operator".into()),
                message: "inspected connector".into(),
                created_at: crate::Timestamp::new("2026-10-19T10:00:00Z"),
            },
            checkpoint_targets: Vec::new(),
        };
        (state, admission)
    }

    #[test]
    fn note_mutations_restarts() {
        let (mut state, admission) = fixture();
        let mut create = CreateOperatorNote { admission };
        create.check(&state).unwrap();
        let scope = create.admission.note.scope.clone();

        crate::transactions::MarkRuntimeStarted {
            session_id: state.session_id.clone(),
            record_namespace: SessionId::from_seed("notes-restarted-epoch"),
        }
        .apply(&mut state);
        assert!(create.check(&state).unwrap_err().contains("epoch/actor"));

        create.admission.note.note_id = note_identity(state.process_epoch(), "operator", &create.admission.mutation_id);
        create.check(&state).unwrap();
        assert_eq!(create.admission.note.scope, scope);
    }

    #[test]
    fn retry_identity_retirement() {
        let (mut state, admission) = fixture();
        let create = CreateOperatorNote { admission: admission.clone() };
        create.check(&state).unwrap();
        create.apply(&mut state);
        assert_eq!(state.iter_notes().count(), 1);
        let mut retry = create.clone();
        retry.admission.note.created_at = crate::Timestamp::new("2026-10-19T10:00:01Z");
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
            cycle_id: TestCycleId::from_seed("notes-missing-owner"),
        };
        assert!(retry.check(&state).is_err());
        let old_id = create.admission.note.note_id;
        assert_ne!(old_id, note_identity(&SessionId::from_seed("notes-other-epoch"), "operator", &admission.mutation_id));
        assert_ne!(old_id, note_identity(state.process_epoch(), "another operator", &admission.mutation_id));
    }

    #[test]
    fn rejects_missing_owners() {
        let (mut state, admission) = fixture();
        let cycle_id = admission.owner.cycle_id().clone();
        let mut note = admission.note;
        note.scope.test_execution_id = Some(TestExecutionId::from_seed("notes-missing-execution"));
        let transaction = AddOperatorNote { note: note.clone() };
        assert!(transaction.check(&state).is_err());
        assert!(state.cycles[&cycle_id].records.notes().is_empty());
        note.scope.test_execution_id = None;
        note.scope.session_id = Some(SessionId::from_seed("notes-missing-session"));
        assert!(AddOperatorNote { note: note.clone() }.check(&state).is_err());
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
        for index in 0..NOTE_RECEIPT_LIMIT {
            let mut item = admission.clone();
            item.mutation_id = CommandId::from_seed(format!("notes-mutation-{index}"));
            item.note.note_id = note_identity(state.process_epoch(), "operator", &item.mutation_id);
            state.note_admissions.insert(item.note.note_id.clone(), item);
        }
        let existing = state.note_admissions.values().next().unwrap().clone();
        assert!(CreateOperatorNote { admission: admission.clone() }.check(&state).unwrap_err().contains("capacity"));
        CreateOperatorNote { admission: existing.clone() }.check(&state).unwrap();
        assert_eq!(state.note_admissions.len(), NOTE_RECEIPT_LIMIT);
        state.note_admissions.clear();
        for index in 0..600 {
            let mut item = admission.clone();
            item.note.note_id = NoteId::from_seed(format!("notes-size-{index}"));
            item.note.message = "x".repeat(NOTE_TEXT_BYTES);
            state.note_admissions.insert(item.note.note_id.clone(), item);
        }
        assert!(CreateOperatorNote { admission }.check(&state).unwrap_err().contains("capacity"));
    }

    #[test]
    fn scoped_read_bounds() {
        let (mut state, admission) = fixture();
        let cycle = state.cycle_views().next().unwrap();
        for index in 0..NOTE_SCAN_LIMIT {
            let mut cycle = cycle.clone();
            cycle.test_cycle_id = TestCycleId::from_seed(format!("notes-scan-cycle-{index}"));
            for execution in &mut cycle.test_executions {
                execution.test_cycle_id = cycle.test_cycle_id.clone();
                execution.test_execution_id = TestExecutionId::from_seed(format!("notes-scan-execution-{index}"));
            }
            state.replace_cycle(cycle).expect("cycle projection has valid ownership");
        }
        assert!(state.annotation_sets(None, false).is_err());
        assert!(state.annotation_sets(Some(&admission.owner), false).is_ok());
        state.session_records_mut().push_note(OperatorNote {
            note_id: NoteId::from_seed("notes-invalid-message"),
            scope: RecordScope::default(),
            author: None,
            message: "x".repeat(NOTE_TEXT_BYTES + 1),
            created_at: crate::Timestamp::new("2026-10-19T10:00:00Z"),
        });
        let invalid = &state.session_records().notes()[0];
        assert!(state.validate_operator_note(invalid).is_err());
    }
}
