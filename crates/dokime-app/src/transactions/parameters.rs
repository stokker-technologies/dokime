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
use dokime::core::{Identifier, MeasurementUnit, Metadata, RecordScope, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.parameters.scoped.upsert", version = 2, irreversible)]
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
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        apply_upsert_scoped_parameter(state, self.clone());
    }
}

/* === Public Functions === */
pub fn apply_upsert_scoped_parameter(
    state: &mut RuntimeState,
    update: UpsertScopedParameter,
) -> bool {
    let Some(parameters) = scoped_parameters_mut(state, &update.scope) else {
        return false;
    };
    let metadata = update
        .metadata
        .unwrap_or_else(|| synthetic_metadata(&update.key));
    upsert_parameter(
        parameters,
        &update.key,
        update.value,
        update.source,
        metadata,
        update.group_id,
        update.unit,
    )
}

/* === Private Functions === */
fn upsert_parameter(
    parameters: &mut BTreeMap<String, MaterializedParameter>,
    key: &str,
    value: Value,
    source: ParameterSource,
    metadata: Metadata,
    group_id: Option<Identifier>,
    unit: Option<MeasurementUnit>,
) -> bool {
    if let Some(existing) = parameters.get_mut(key) {
        let changed = existing.source != source
            || existing.value != value
            || existing.metadata != metadata
            || existing.group_id != group_id
            || existing.unit != unit;
        *existing = MaterializedParameter {
            metadata,
            group_id,
            source,
            value,
            unit,
        };
        return changed;
    }
    parameters.insert(
        key.to_owned(),
        MaterializedParameter {
            metadata,
            group_id,
            source,
            value,
            unit,
        },
    );
    true
}

fn scoped_parameters_mut<'a>(
    state: &'a mut RuntimeState,
    scope: &RecordScope,
) -> Option<&'a mut BTreeMap<String, MaterializedParameter>> {
    if let Some(test_execution_id) = scope.test_execution_id.as_ref() {
        return state
            .executions
            .get_mut(test_execution_id)
            .filter(|execution| {
                scope
                    .test_cycle_id
                    .as_ref()
                    .is_none_or(|id| id == &execution.test_cycle_id)
            })
            .map(|execution| &mut execution.output_parameters);
    }
    if scope.test_cycle_id.is_some() {
        return None;
    }
    Some(&mut state.run_parameters)
}

fn synthetic_metadata(key: &str) -> Metadata {
    Metadata::new(
        Identifier::new(key).expect("runtime synthetic parameter keys must be valid identifiers"),
        key,
        None,
    )
}

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down {
        Err("runtime is shutting down".to_owned())
    } else {
        Ok(())
    }
}

/* === Tests === */
