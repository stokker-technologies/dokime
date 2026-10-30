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

use crate::{
    records::{BuiltinRecordType, RecordSeverity},
    state::RuntimeState,
    transactions::records::draft::{BuiltinDraft, Drafts},
};
use dokime_domain::model::{
    execution::effects::{EffectPreparation, PendingEffect},
    time::Timestamp,
    values::Value,
};
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.effect")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordEffect {
    pub ticket: String,
    pub change: EffectChange,
    pub at: Timestamp,
}

/* === Enums === */

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectChange {
    Prepared(Box<PendingEffect>),
    Applied,
    RolledBack,
    Restored,
}

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for RecordEffect {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        match &self.change {
            EffectChange::Prepared(effect) => {
                if effect.ticket.is_empty() || effect.resource.is_empty() || harmos::QualifiedId::parse(&effect.inverse_operation).is_err()
                {
                    return Err("effect requires a ticket, resource and owner-qualified encoded inverse".into());
                }
                self.at.to_datetime().map_err(|error| error.to_string())?;
                if effect.scope.test_execution_id.as_ref().is_none_or(|id| !state.executions.contains_key(id)) {
                    return Err("effect execution is missing".into());
                }

                if effect.ticket != self.ticket
                    || !effect.coalesced.is_empty()
                    || state.pending_effects.iter().any(|pending| {
                        pending.ticket == self.ticket || pending.coalesced.iter().any(|prepare| prepare.ticket == self.ticket)
                    })
                {
                    return Err("duplicate or mismatched effect ticket".into());
                }
                if state
                    .pending_effects
                    .iter()
                    .any(|pending| pending.resource == effect.resource && pending.scope.test_execution_id != effect.scope.test_execution_id)
                {
                    return Err("another execution owns this external resource".into());
                }
            }
            _ => {
                if !state.pending_effects.iter().any(|effect| {
                    effect.ticket == self.ticket
                        || (!matches!(self.change, EffectChange::Restored)
                            && effect.coalesced.iter().any(|prepare| prepare.ticket == self.ticket))
                }) {
                    return Err("effect ticket is not pending".into());
                }
            }
        }
        Ok(())
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.drafts(state).collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        let records = self.drafts(state);
        match &self.change {
            EffectChange::Prepared(effect) => {
                if let Some(pending) =
                    state.pending_effects.iter_mut().find(|pending| pending.scope == effect.scope && pending.resource == effect.resource)
                {
                    pending.coalesced.push(EffectPreparation {
                        ticket: effect.ticket.clone(),
                        source: effect.source.clone(),
                        description: effect.description.clone(),
                        prepared_at: effect.prepared_at.clone(),
                        applied_at: None,
                    });
                } else {
                    state.pending_effects.push((**effect).clone());
                }
            }
            EffectChange::Applied => {
                let effect = state
                    .pending_effects
                    .iter_mut()
                    .find(|effect| effect.ticket == self.ticket || effect.coalesced.iter().any(|prepare| prepare.ticket == self.ticket))
                    .expect("checked ticket");
                if effect.ticket == self.ticket {
                    effect.applied_at = Some(self.at.clone());
                } else {
                    effect.coalesced.iter_mut().find(|prepare| prepare.ticket == self.ticket).expect("checked preparation").applied_at =
                        Some(self.at.clone());
                }
            }
            EffectChange::RolledBack => state.pending_effects.retain(|effect| effect.ticket != self.ticket || !effect.coalesced.is_empty()),
            EffectChange::Restored => state.pending_effects.retain(|effect| effect.ticket != self.ticket),
        }
        records.apply(state);
    }
}

impl RecordEffect {
    fn drafts(&self, state: &RuntimeState) -> Drafts {
        let pending = state.pending_effects.iter().find(|effect| match &self.change {
            EffectChange::Prepared(prepared) => effect.scope == prepared.scope && effect.resource == prepared.resource,
            _ => effect.ticket == self.ticket || effect.coalesced.iter().any(|prepare| prepare.ticket == self.ticket),
        });
        let effect = match &self.change {
            EffectChange::Prepared(effect) => effect.as_ref(),
            _ => pending.expect("checked ticket"),
        };
        let preparation = effect.coalesced.iter().find(|prepare| prepare.ticket == self.ticket);
        let restoration_ticket = pending.map_or(effect.ticket.as_str(), |pending| pending.ticket.as_str());
        let coalesced = restoration_ticket != self.ticket;
        let resource_restored = matches!(self.change, EffectChange::Restored)
            || matches!(self.change, EffectChange::RolledBack) && effect.ticket == self.ticket && effect.coalesced.is_empty();
        let description = preparation.map_or(&effect.description, |prepare| &prepare.description);
        let source = preparation.map_or(&effect.source, |prepare| &prepare.source);
        let prepared_at = preparation.map_or(&effect.prepared_at, |prepare| &prepare.prepared_at);
        let applied_at = preparation.map_or(&effect.applied_at, |prepare| &prepare.applied_at);
        let kind = match self.change {
            EffectChange::Prepared(_) => BuiltinRecordType::ExecutionEffectPrepared,
            EffectChange::Applied => BuiltinRecordType::ExecutionEffectApplied,
            EffectChange::Restored | EffectChange::RolledBack => BuiltinRecordType::ExecutionEffectRestored,
        };
        let mut records = Drafts::new(state);
        records.capture_scope(effect.scope.clone());
        records.draft(
            BuiltinDraft {
                record_type: kind,
                source: source.clone(),
                severity: RecordSeverity::Info,
                message: Some(description.clone()),
                details: std::collections::BTreeMap::from([
                    ("ticket".into(), Value::from(self.ticket.clone())),
                    ("restoration_ticket".into(), Value::from(restoration_ticket.to_owned())),
                    ("coalesced".into(), Value::Bool(coalesced)),
                    ("resource_restored".into(), Value::Bool(resource_restored)),
                    ("resource".into(), Value::from(effect.resource.clone())),
                    ("prepared_at".into(), Value::from(prepared_at.to_string())),
                    (
                        "applied_at".into(),
                        if matches!(self.change, EffectChange::Applied) {
                            Value::from(self.at.to_string())
                        } else {
                            applied_at.as_ref().map(|at| Value::from(at.to_string())).unwrap_or(Value::Null)
                        },
                    ),
                ]),
                scope: Some(effect.scope.clone()),
            },
            self.at.clone(),
        );
        records
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
