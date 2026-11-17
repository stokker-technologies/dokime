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
use crate::{Error, Result, host::Context};
use dokime_protocol::host::executions::CommandsGetRequest;
use dokime_protocol::host::executions::CommandsGetResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Commands/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: CommandsGetRequest) -> Result<CommandsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let selected = request.at.is_some();
        let (command, resolved_at) = super::read(context, &request.parent, &request.name, request.at).await.map_err(|source| {
            if selected {
                Error::Precondition(format!("Get Execution Command: Resynchronize Its Position Before Reading Again: {source}"))
            } else {
                source
            }
        })?;
        let command = command.ok_or_else(|| Error::NotFound("Get Execution Command: Use Its Accepted Execution and Command IDs".into()))?;
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
