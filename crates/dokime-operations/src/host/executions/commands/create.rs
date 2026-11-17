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
use crate::{Error, Result, host::Context};
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_protocol::host::executions::CommandsCreateResponse;
use harmos_protocol::model::{OperationContract, Receipt};

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
        context.host.submit_command(request.clone()).await?;
        let (command, position) = super::read(context, &request.parent, &request.command_id, None).await?;
        Ok(CommandsCreateResponse {
            command: Some(
                command.ok_or_else(|| Error::NotFound("Create Execution Command: Inspect Its Execution Before Retrying".into()))?,
            ),
            receipt: Some(Receipt {
                request_id: request.request_id,
                accepted_at: Some(position.clone()),
                durable_at: Some(position),
                completed: true,
            }),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CommandsCreateRequest, CommandsCreateResponse, context, "dokime.host.v1.executions", "Commands", "Create");
