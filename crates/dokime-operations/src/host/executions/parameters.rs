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
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ExecutionsParametersRequest, dokime_protocol::host::ExecutionsParametersResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ExecutionsParametersRequest,
) -> Result<dokime_protocol::host::ExecutionsParametersResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let position = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?.resolved_at;

        let snapshot = crate::host::execution_state::read(context, &request.name)?;
        let output = match dokime_protocol::host::ExecutionParameterSet::try_from(request.set) {
            Ok(dokime_protocol::host::ExecutionParameterSet::Inputs) => false,
            Ok(dokime_protocol::host::ExecutionParameterSet::Outputs) => true,
            _ => return Err(Error::Invalid("List Execution Parameters: Select Inputs or Outputs".into())),
        };
        let empty = std::collections::HashMap::new();
        let parameters =
            if output { &empty } else { &crate::host::execution_state::case(snapshot.execution.origin.as_ref().unwrap())?.parameters };
        let parameters = crate::host::execution_state::parameters(parameters);
        let parameters = parameters
            .into_iter()
            .map(|parameter| dokime_protocol::host::MaterializedParameter {
                name: parameter.name,
                value: parameter.value,
                source: dokime_protocol::host::ParameterSource::Definition as i32,
                ..Default::default()
            })
            .collect();
        Ok(dokime_protocol::host::ExecutionsParametersResponse { parameters, resolved_at: Some(position) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
