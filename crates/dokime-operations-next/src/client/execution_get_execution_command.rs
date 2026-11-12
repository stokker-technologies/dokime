/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Get Execution Command

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.get_execution_command";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionGetExecutionCommandRequest, v1::ExecutionCommandReceipt>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionGetExecutionCommandRequest) -> Result<v1::ExecutionCommandReceipt> {
    super::complete(ID, context.settings.read_timeout, async {
        context
            .host
            .store()
            .command(&request.request_id, &request.command_id)?
            .ok_or_else(|| Error::NotFound("Get Execution Command: Use the Accepted Run and Command IDs".into()))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
