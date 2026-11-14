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

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ExecutionParameterSet;
use dokime_protocol::host::ExecutionsParametersRequest;
use dokime_protocol::host::ExecutionsParametersResponse;
use dokime_protocol::host::MaterializedParameter;
use dokime_protocol::host::ParameterSource;
use harmos_protocol::model::OperationContract;
use std::collections::HashMap;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionsParametersRequest, ExecutionsParametersResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ExecutionsParametersRequest) -> Result<ExecutionsParametersResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let position = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?.resolved_at;

        let snapshot = crate::host::execution_state::read(context, &request.name)?;
        let output = match ExecutionParameterSet::try_from(request.set) {
            Ok(ExecutionParameterSet::Inputs) => false,
            Ok(ExecutionParameterSet::Outputs) => true,
            _ => return Err(Error::Invalid("List Execution Parameters: Select Inputs or Outputs".into())),
        };
        let empty = HashMap::new();
        let parameters =
            if output { &empty } else { &crate::host::execution_state::case(snapshot.execution.origin.as_ref().unwrap())?.parameters };
        let parameters = crate::host::execution_state::parameters(parameters);
        let parameters = parameters
            .into_iter()
            .map(|parameter| MaterializedParameter {
                name: parameter.name,
                value: parameter.value,
                source: ParameterSource::Definition as i32,
                ..Default::default()
            })
            .collect();
        Ok(ExecutionsParametersResponse { parameters, resolved_at: Some(position) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
