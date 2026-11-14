/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Commands Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::CommandsCreateResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Commands/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CommandsCreateRequest, CommandsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: CommandsCreateRequest) -> Result<CommandsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let command = context.host.submit_command(request.clone())?;
        Ok(CommandsCreateResponse {
            command: Some(command),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
