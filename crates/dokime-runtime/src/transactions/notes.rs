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
use std::collections::BTreeMap;
// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::execution::notes::NOTE_CHECKPOINT_LIMIT;
use dokime_domain::model::execution::notes::NOTE_EDIT_NOT_AUTHOR;
use dokime_domain::model::execution::notes::NOTE_OWNER_SEALED;
use dokime_domain::model::execution::notes::NOTE_RECEIPT_BYTES;
use dokime_domain::model::execution::notes::NOTE_RECEIPT_LIMIT;
use dokime_domain::model::execution::notes::NoteAdmission;
use dokime_domain::model::execution::notes::note_identity;
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::state::RuntimeState;
use crate::transactions::records::draft::BuiltinDraft;
use dokime_domain::model::identity::NoteId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.evidence.note.add", version = 2, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddOperatorNote {
    pub note: OperatorNote,
}

#[harmos::transaction(id = "dokime.evidence.note.create", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateOperatorNote {
    pub admission: NoteAdmission,
}

#[harmos::transaction(id = "dokime.evidence.note.edit", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditOperatorNote {
    pub note_id: NoteId,
    pub actor: String,
    pub message: String,
    pub edited_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for AddOperatorNote {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        state.validate_operator_note(&self.note)
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.plan(state).1.collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        let (note, records) = self.plan(state);
        if let Some(target) = state.records_for_scope_mut(&note.scope) {
            target.push_note(note);
        }
        records.apply(state);
    }
}

impl AddOperatorNote {
    fn plan(
        &self,
        state: &RuntimeState,
    ) -> (OperatorNote, crate::transactions::records::draft::Drafts) {
        let note = self.note.clone();
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        records.draft(
            BuiltinDraft {
                record_type: BuiltinRecordType::OperatorNoteAdded,
                source: "operator".to_owned(),
                severity: RecordSeverity::Info,
                message: Some("Operator note added.".to_owned()),
                details: BTreeMap::from([(
                    "note_id".to_owned(),
                    Value::from(note.note_id.as_str()),
                )]),
                scope: Some(note.scope.clone()),
            },
            note.created_at.clone(),
        );
        (note, records)
    }
}

impl Transaction<RuntimeState> for CreateOperatorNote {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let next = &self.admission;
        let actor = next
            .note
            .author
            .as_deref()
            .filter(|actor| !actor.is_empty())
            .ok_or("note creation requires actor")?;
        if next.note.note_id != note_identity(state.process_epoch(), actor, &next.mutation_id) {
            return Err("note mutation identity does not match epoch/actor".into());
        }
        if let Some(existing) = state.note_admissions.get(&next.note.note_id) {
            return if same_creation(existing, next) {
                Ok(())
            } else {
                Err("note mutation identity reused with different input".into())
            };
        }
        state.validate_operator_note(&next.note)?;
        if state.note_owner_scope(&next.owner)? != next.note.scope {
            return Err("note owner differs from canonical scope".into());
        }
        if next.checkpoint_targets.len() > NOTE_CHECKPOINT_LIMIT
            || next
                .checkpoint_targets
                .windows(2)
                .any(|ids| ids[0] >= ids[1])
        {
            return Err("note checkpoint target budget/order is invalid".into());
        }
        let cycle = &state.cycles[next.owner.cycle_id()];
        for target in &next.checkpoint_targets {
            if !state
                .executions_for_cycle(cycle)
                .take(dokime_domain::model::execution::notes::NOTE_SCAN_LIMIT)
                .any(|execution| {
                    &execution.test_execution_id == target
                        && next
                            .note
                            .scope
                            .test_execution_id
                            .as_ref()
                            .is_none_or(|id| id == target)
                        && next
                            .note
                            .scope
                            .test_case_id
                            .as_ref()
                            .is_none_or(|id| id == &execution.test_case_id)
                })
            {
                return Err("note checkpoint target no longer belongs to owner".into());
            }
        }
        if !admission_fits(state, next) {
            return Err("note admission receipt capacity exhausted".into());
        }
        Ok(())
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        if !state
            .note_admissions
            .contains_key(&self.admission.note.note_id)
        {
            AddOperatorNote {
                note: self.admission.note.clone(),
            }
            .records(state, out);
        }
    }
    fn apply(&self, state: &mut RuntimeState) {
        if state
            .note_admissions
            .contains_key(&self.admission.note.note_id)
        {
            return;
        }
        AddOperatorNote {
            note: self.admission.note.clone(),
        }
        .apply(state);
        state
            .note_admissions
            .insert(self.admission.note.note_id.clone(), self.admission.clone());
    }
}

impl Transaction<RuntimeState> for EditOperatorNote {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        let note = state
            .find_note(&self.note_id)
            .ok_or("note is absent from current evidence")?;
        if note.author.as_deref() != Some(self.actor.as_str()) {
            return Err(NOTE_EDIT_NOT_AUTHOR.into());
        }
        if state.note_edit_sealed(&note.scope) {
            return Err(NOTE_OWNER_SEALED.into());
        }
        state.validate_operator_note(&OperatorNote {
            message: self.message.clone(),
            ..note.clone()
        })
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.plan(state).collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        let records = self.plan(state);
        let Some(scope) = state
            .find_note(&self.note_id)
            .map(|note| note.scope.clone())
        else {
            return;
        };
        if let Some(target) = state.records_for_scope_mut(&scope) {
            target.revise_note(&self.note_id, self.message.clone());
        }
        records.apply(state);
    }
}

impl EditOperatorNote {
    fn plan(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        if let Some(note) = state.find_note(&self.note_id) {
            records.draft(
                BuiltinDraft {
                    record_type: BuiltinRecordType::OperatorNoteEdited,
                    source: "operator".to_owned(),
                    severity: RecordSeverity::Info,
                    message: Some("Operator note edited.".to_owned()),
                    details: BTreeMap::from([(
                        "note_id".to_owned(),
                        Value::from(note.note_id.as_str()),
                    )]),
                    scope: Some(note.scope.clone()),
                },
                self.edited_at.clone(),
            );
        }
        records
    }
}

/* === Public Functions === */
pub fn admission_fits(state: &RuntimeState, next: &NoteAdmission) -> bool {
    state.note_admissions.len() < NOTE_RECEIPT_LIMIT
        && state
            .note_admissions
            .values()
            .fold(receipt_bytes(next), |bytes, item| {
                bytes.saturating_add(receipt_bytes(item))
            })
            <= NOTE_RECEIPT_BYTES
}

/* === Private Functions === */
fn same_creation(first: &NoteAdmission, next: &NoteAdmission) -> bool {
    first.mutation_id == next.mutation_id
        && first.owner == next.owner
        && first.note.scope == next.note.scope
        && first.note.author == next.note.author
        && first.note.message == next.note.message
}

fn receipt_bytes(receipt: &NoteAdmission) -> usize {
    // Conservative fixed UUID/timestamp/enum/container overhead plus payload.
    2048usize
        .saturating_add(receipt.note.message.len())
        .saturating_add(receipt.note.author.as_ref().map_or(0, String::len))
        .saturating_add(
            receipt
                .note
                .scope
                .test_case_id
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(match &receipt.owner {
            dokime_domain::model::execution::notes::NoteOwner::Case { case_id, .. } => case_id.as_str().len(),
            _ => 0,
        })
        .saturating_add(receipt.checkpoint_targets.len().saturating_mul(64))
}

/* === Tests === */
