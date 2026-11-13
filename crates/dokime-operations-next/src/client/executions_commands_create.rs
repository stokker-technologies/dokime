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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.executions.Commands/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<
        dokime_protocol::client::executions::CommandsCreateRequest,
        dokime_protocol::client::executions::CommandsCreateResponse,
    >(ID, settings.mutation_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::executions::CommandsCreateRequest,
) -> Result<dokime_protocol::client::executions::CommandsCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let command = context.host.submit_command(request.clone())?;
        Ok(dokime_protocol::client::executions::CommandsCreateResponse {
            command: Some(command),
            receipt: Some(super::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
