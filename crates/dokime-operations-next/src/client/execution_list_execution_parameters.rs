/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution List Execution Parameters

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.list_execution_parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionListExecutionParametersRequest, v1::ListMaterializedParametersReply>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: v1::ExecutionListExecutionParametersRequest,
) -> Result<v1::ListMaterializedParametersReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let snapshot = super::execution_state::read(context, &request.execution_id)?;
        let output = match v1::ExecutionParameterSet::try_from(request.set) {
            Ok(v1::ExecutionParameterSet::Inputs) => false,
            Ok(v1::ExecutionParameterSet::Outputs) => true,
            _ => return Err(Error::Invalid("List Execution Parameters: Select Inputs or Outputs".into())),
        };
        let selection = v1::ParameterCursorSelection {
            kind: Some(if output {
                v1::parameter_cursor_selection::Kind::ExecutionOutputsId(request.execution_id)
            } else {
                v1::parameter_cursor_selection::Kind::ExecutionInputsId(request.execution_id)
            }),
        };
        let empty = std::collections::HashMap::new();
        let parameters =
            if output { &empty } else { &super::execution_state::case(snapshot.execution.origin.as_ref().unwrap())?.parameters };
        let (parameters, next_after_name, cursor) = super::execution_state::parameters(
            context,
            parameters,
            request.limit,
            &request.after_name,
            request.cursor.as_ref(),
            selection,
        )?;
        let parameters = parameters
            .into_iter()
            .map(|parameter| v1::MaterializedParameter {
                name: parameter.name,
                value: parameter.value,
                source: v1::ParameterSource::Definition as i32,
                ..Default::default()
            })
            .collect();
        Ok(v1::ListMaterializedParametersReply {
            parameters,
            next_after_name,
            cursor: Some(cursor),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
