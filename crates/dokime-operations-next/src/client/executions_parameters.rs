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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Executions/Parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::ExecutionsParametersRequest, dokime_protocol::client::ExecutionsParametersResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::ExecutionsParametersRequest,
) -> Result<dokime_protocol::client::ExecutionsParametersResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let position = context.host.store().resource_value(&request.name, "progress", request.at.as_ref())?.resolved_at;

        let snapshot = super::execution_state::read(context, &request.name)?;
        let output = match dokime_protocol::client::ExecutionParameterSet::try_from(request.set) {
            Ok(dokime_protocol::client::ExecutionParameterSet::Inputs) => false,
            Ok(dokime_protocol::client::ExecutionParameterSet::Outputs) => true,
            _ => return Err(Error::Invalid("List Execution Parameters: Select Inputs or Outputs".into())),
        };
        let empty = std::collections::HashMap::new();
        let parameters =
            if output { &empty } else { &super::execution_state::case(snapshot.execution.origin.as_ref().unwrap())?.parameters };
        let parameters = super::execution_state::parameters(parameters);
        let parameters = parameters
            .into_iter()
            .map(|parameter| dokime_protocol::client::MaterializedParameter {
                name: parameter.name,
                value: parameter.value,
                source: dokime_protocol::client::ParameterSource::Definition as i32,
                ..Default::default()
            })
            .collect();
        Ok(dokime_protocol::client::ExecutionsParametersResponse { parameters, resolved_at: Some(position) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
