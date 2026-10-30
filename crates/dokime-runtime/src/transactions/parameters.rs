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
use crate::definition::{MaterializedParameter, ParameterSource};
use crate::state::RuntimeState;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::MeasurementUnit;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.parameters.scoped.upsert")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpsertScopedParameter {
    pub scope: RecordScope,
    pub key: String,
    pub value: Value,
    pub source: ParameterSource,
    pub metadata: Option<Metadata>,
    pub group_id: Option<Identifier>,
    pub unit: Option<MeasurementUnit>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for UpsertScopedParameter {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        if self.scope.test_cycle_id.is_some() && self.scope.test_execution_id.is_none() {
            return Err("parameters belong to an execution, not a cycle".into());
        }
        if let Some(id) = &self.scope.test_execution_id {
            let execution = state.executions.get(id).ok_or("parameter execution not found")?;
            if execution.saved.is_some() {
                return Err("sealed execution parameters are immutable".into());
            }
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        let Some(parameters) = scoped_parameters_mut(state, &self.scope) else {
            return;
        };
        parameters.insert(
            self.key.clone(),
            MaterializedParameter {
                metadata: self.metadata.clone().unwrap_or_else(|| synthetic_metadata(&self.key)),
                group_id: self.group_id.clone(),
                source: self.source.clone(),
                value: self.value.clone(),
                unit: self.unit.clone(),
            },
        );
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn scoped_parameters_mut<'a>(state: &'a mut RuntimeState, scope: &RecordScope) -> Option<&'a mut BTreeMap<String, MaterializedParameter>> {
    if let Some(test_execution_id) = scope.test_execution_id.as_ref() {
        return state
            .executions
            .get_mut(test_execution_id)
            .filter(|execution| scope.test_cycle_id.as_ref().is_none_or(|id| id == &execution.test_cycle_id))
            .map(|execution| &mut execution.output_parameters);
    }
    if scope.test_cycle_id.is_some() {
        return None;
    }
    Some(&mut state.run_parameters)
}

fn synthetic_metadata(key: &str) -> Metadata {
    Metadata::new(Identifier::new(key).expect("runtime synthetic parameter keys must be valid identifiers"), key, None)
}

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".to_owned()) } else { Ok(()) }
}

/* === Tests === */
