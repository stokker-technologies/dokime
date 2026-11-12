/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Submit Execution Command

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.submit_execution_command";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionSubmitExecutionCommandRequest, v1::ExecutionCommandReceipt>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionSubmitExecutionCommandRequest) -> Result<v1::ExecutionCommandReceipt> {
    super::complete(ID, context.settings.mutation_timeout, async { context.host.submit_command(request) }).await
}

/* === Private Functions === */

/* === Tests === */
