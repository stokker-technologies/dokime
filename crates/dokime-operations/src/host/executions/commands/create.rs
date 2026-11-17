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
use crate::{Result, host::Context};
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::CommandsCreateResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Commands/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: CommandsCreateRequest) -> Result<CommandsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let command = context.host.submit_command(request.clone()).await?;
        let receipt = super::receipt(context, &request.parent, &request.command_id).await?;
        Ok(CommandsCreateResponse {
            command: Some(command),
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CommandsCreateRequest, CommandsCreateResponse, context, "dokime.host.v1.executions", "Commands", "Create");
