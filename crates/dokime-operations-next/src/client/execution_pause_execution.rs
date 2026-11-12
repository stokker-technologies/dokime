/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Pause Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.pause_execution";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionPauseExecutionRequest, v1::ControlReply>(ID, settings.mutation_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionPauseExecutionRequest) -> Result<v1::ControlReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        super::execution_state::read(context,&request.execution_id)?;
        context.host.control_execution(&request.execution_id,super::RunAction::Pause)?;
        Ok(v1::ControlReply { observed_cursor: Some(super::execution_state::observed(context)) })
    }).await
}

/* === Private Functions === */

/* === Tests === */
