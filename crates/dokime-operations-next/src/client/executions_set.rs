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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Executions/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsSetRequest, v1::ExecutionsSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsSetRequest) -> Result<v1::ExecutionsSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let _guard=context.administration.lock().await;
let previous=super::executions_get::handle(context,v1::ExecutionsGetRequest {name:request.name.clone(),..Default::default()}).await?;
if previous.revision!=request.expected_revision { return Err(Error::Conflict("Set Execution: Get Its Current Revision".into())); }
let value=request.value.ok_or_else(||Error::Invalid("Set Execution: Supply the Whole Writable Value".into()))?;
if value.label.trim().is_empty() { return Err(Error::Invalid("Set Execution: Supply a Nonempty Label".into())); }
context.host.store().set_execution_presentation(&request.name,Some(&value.label),Some(value.archived))?;
if previous.value.as_ref().is_some_and(|previous|previous.paused!=value.paused) { context.host.control_execution(&request.name,if value.paused {super::RunAction::Pause}else{super::RunAction::Resume})?; }
context.host.signal_change();Ok(v1::ExecutionsSetResponse {revision:super::paging::bytes(&value),value:Some(value),receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
