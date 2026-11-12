/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Get Execution Checkpoint

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.get_execution_checkpoint";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryGetExecutionCheckpointRequest, v1::ExecutionCheckpoint>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryGetExecutionCheckpointRequest) -> Result<v1::ExecutionCheckpoint> {
    super::complete(ID, context.settings.read_timeout, async { super::history::checkpoint(context, &request.execution_identifier) }).await
}

/* === Private Functions === */

/* === Tests === */
