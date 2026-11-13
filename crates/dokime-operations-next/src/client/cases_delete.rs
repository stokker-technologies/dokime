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

pub const ID: &str = "dokime.v1.Cases/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CasesDeleteRequest, v1::CasesDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::CasesDeleteRequest) -> Result<v1::CasesDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
super::execution_state::cycle(context,&request.parent)?;
context.host.store().delete_planned_case(&request.parent,&request.name)?;
context.host.signal_change();Ok(v1::CasesDeleteResponse {receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
