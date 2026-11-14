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
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.executions.Commands/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::executions::CommandsGetRequest, dokime_protocol::host::executions::CommandsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::executions::CommandsGetRequest,
) -> Result<dokime_protocol::host::executions::CommandsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let command = context
            .host
            .store()
            .command(&request.parent, &request.name)?
            .ok_or_else(|| Error::NotFound("Get Execution Command: Use Its Accepted Execution and Command IDs".into()))?;
        let resolved_at = context.host.store().resource_value(&request.parent, "progress", request.at.as_ref())?.resolved_at;
        Ok(dokime_protocol::host::executions::CommandsGetResponse {
            command: Some(command),
            resolved_at: Some(resolved_at),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
