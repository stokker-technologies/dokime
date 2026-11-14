/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Commands Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::executions::CommandsGetRequest;
use dokime_protocol::host::executions::CommandsGetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Commands/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CommandsGetRequest, CommandsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: CommandsGetRequest) -> Result<CommandsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let command = context
            .host
            .store()
            .command(&request.parent, &request.name)?
            .ok_or_else(|| Error::NotFound("Get Execution Command: Use Its Accepted Execution and Command IDs".into()))?;
        let resolved_at = context.host.store().resource_value(&request.parent, "progress", request.at.as_ref())?.resolved_at;
        Ok(CommandsGetResponse {
            command: Some(command),
            resolved_at: Some(resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CommandsGetRequest, CommandsGetResponse, context, "dokime.host.v1.executions", "Commands", "Get");
