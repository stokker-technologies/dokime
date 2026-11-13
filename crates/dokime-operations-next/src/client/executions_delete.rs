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

pub const ID: &str = "dokime.v1.Executions/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsDeleteRequest, v1::ExecutionsDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsDeleteRequest) -> Result<v1::ExecutionsDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let execution=super::execution_state::detail(context,&request.name)?;
let cut=context.host.store().cut(&request.name)?;
context.host.store().delete_sealed_execution(&request.request_id,&request.name,&cut.seal)?;
context.host.signal_change();Ok(v1::ExecutionsDeleteResponse {execution:Some(execution),receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
