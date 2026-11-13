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
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.executions.Commands/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::executions::CommandsCreateRequest, v1::executions::CommandsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::executions::CommandsCreateRequest) -> Result<v1::executions::CommandsCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let command = context.host.submit_command(request.clone())?;
        Ok(v1::executions::CommandsCreateResponse {
            command: Some(command),
            receipt: Some(super::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
