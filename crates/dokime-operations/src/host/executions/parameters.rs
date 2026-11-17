/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Parameters

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::{History, Position};

// Local Imports
use crate::host::{self, Context, execution_state};
use crate::{Error, Result};
use dokime_protocol::host::ExecutionParameterSet;
use dokime_protocol::host::ExecutionsParametersRequest;
use dokime_protocol::host::ExecutionsParametersResponse;
use dokime_protocol::host::MaterializedParameter;
use dokime_protocol::host::ParameterSource;
use dokime_storage::{ExecutionCut, Store};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionsParametersRequest) -> Result<ExecutionsParametersResponse> {
    host::complete(ID, context.settings.read_timeout, async {
        let store = context.host.store();
        let position = store.resource_value(&request.name, "progress", request.at.as_ref())?.resolved_at;
        let mut fixed = store.cut(&request.name)?;
        fixed.frontier = position.sequence;
        response_at(&store, &fixed, request.set)
    })
    .await
}

pub fn response_at(store: &Store, fixed: &ExecutionCut, set: i32) -> Result<ExecutionsParametersResponse> {
    let output = match ExecutionParameterSet::try_from(set) {
        Ok(ExecutionParameterSet::Inputs) => false,
        Ok(ExecutionParameterSet::Outputs) => true,
        _ => return Err(Error::Invalid("List Execution Parameters: Select Inputs or Outputs".into())),
    };
    let parameters = if output {
        Vec::new()
    } else {
        let origin =
            fixed.execution.origin.as_ref().ok_or_else(|| Error::Invalid("List Execution Parameters: Restore Its Frozen Origin".into()))?;
        let case = execution_state::case(origin)?;
        execution_state::parameters(&case.parameters)
    };
    let parameters = parameters
        .into_iter()
        .map(|parameter| MaterializedParameter {
            name: parameter.name,
            value: parameter.value,
            source: ParameterSource::Definition as i32,
            ..Default::default()
        })
        .collect();
    let position = Position {
        history: Some(History {
            store: store.identity().into(),
            epoch: fixed.execution.identifier.clone(),
        }),
        sequence: fixed.frontier,
    };
    Ok(ExecutionsParametersResponse { parameters, resolved_at: Some(position) })
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionsParametersRequest, ExecutionsParametersResponse, context, "dokime.host.v1", "Executions", "Parameters");
