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

pub const ID: &str = "dokime.v1.Executions/Start";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsStartRequest, v1::ExecutionsStartResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsStartRequest) -> Result<v1::ExecutionsStartResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let controls=context.host.controls(&request.name)?;
let mode=match v1::ExecutionMode::try_from(request.mode) {Ok(v1::ExecutionMode::Interactive)=>v1::ExecutionMode::Interactive,Ok(v1::ExecutionMode::Automatic|v1::ExecutionMode::Unspecified)=>v1::ExecutionMode::Automatic,_=>return Err(Error::Invalid("Start Execution: Choose an Execution Mode".into()))};
if controls.can_cancel {if controls.mode!=mode as i32 {return Err(Error::Conflict("Start Execution: Resolve the Existing Run Mode".into()));}} else {context.host.start_existing(&request.name,mode).await?;}
Ok(v1::ExecutionsStartResponse {execution:Some(super::execution_state::detail(context,&request.name)?),receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
