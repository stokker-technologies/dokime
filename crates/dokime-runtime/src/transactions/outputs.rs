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
use crate::definition::MaterializedParameter;
use crate::state::RuntimeState;
use crate::streams::AggregateOutput;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.outputs.execution.finalize")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinalizeExecutionOutputs {
    pub test_execution_id: TestExecutionId,
    pub outputs: Vec<AggregateOutput>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for FinalizeExecutionOutputs {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> Result<(), Self::Error> {
        for output in &self.outputs {
            output.metadata()?;
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        finalize_execution_outputs(state, &self.test_execution_id, self.outputs.clone());
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn finalize_execution_outputs(state: &mut RuntimeState, test_execution_id: &TestExecutionId, outputs: Vec<AggregateOutput>) {
    if let Some(execution) = state.executions.get_mut(test_execution_id) {
        replace_aggregate_parameters(&mut execution.output_parameters, outputs);
    }
}

fn replace_aggregate_parameters(parameters: &mut BTreeMap<String, MaterializedParameter>, outputs: Vec<AggregateOutput>) {
    let group_id = aggregate_group_id();
    parameters
        .retain(|key, parameter| !(key.starts_with("telemetry.") && (parameter.source.is_derived() || parameter.group_id == group_id)));
    for output in outputs {
        let parameter = output.parameter().expect("checked aggregate metadata");
        parameters.insert(output.key(), parameter);
    }
}

fn aggregate_group_id() -> Option<Identifier> {
    Some(Identifier::new("telemetry.aggregate").expect("aggregate group id is valid"))
}

/* === Tests === */
