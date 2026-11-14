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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Commands/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<
        dokime_protocol::host::executions::CommandsCreateRequest,
        dokime_protocol::host::executions::CommandsCreateResponse,
    >(ID, settings.mutation_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::executions::CommandsCreateRequest,
) -> Result<dokime_protocol::host::executions::CommandsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let command = context.host.submit_command(request.clone())?;
        Ok(dokime_protocol::host::executions::CommandsCreateResponse {
            command: Some(command),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
