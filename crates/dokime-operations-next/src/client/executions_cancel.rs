/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

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

pub const ID: &str = "dokime.v1.Executions/Cancel";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsCancelRequest, v1::ExecutionsCancelResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsCancelRequest) -> Result<v1::ExecutionsCancelResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
context.host.control_execution(&request.name,super::RunAction::Cancel)?;
Ok(v1::ExecutionsCancelResponse {execution:Some(super::execution_state::detail(context,&request.name)?),receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
