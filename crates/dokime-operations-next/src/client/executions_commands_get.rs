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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.executions.Commands/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::executions::CommandsGetRequest, dokime_protocol::client::executions::CommandsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::executions::CommandsGetRequest,
) -> Result<dokime_protocol::client::executions::CommandsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let command = context
            .host
            .store()
            .command(&request.parent, &request.name)?
            .ok_or_else(|| Error::NotFound("Get Execution Command: Use Its Accepted Execution and Command IDs".into()))?;
        let resolved_at = context.host.store().resource_value(&request.parent, "progress", request.at.as_ref())?.resolved_at;
        Ok(dokime_protocol::client::executions::CommandsGetResponse {
            command: Some(command),
            resolved_at: Some(resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
