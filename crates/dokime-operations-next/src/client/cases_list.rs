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

pub const ID: &str = "dokime.v1.Cases/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CasesListRequest, v1::CasesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CasesListRequest) -> Result<v1::CasesListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (mut reply,cut)=super::views::read(context,&format!("{ID}/{}/{}",request.parent,String::new()),request.at.as_ref(),async { 
        let cases = super::execution_state::cases(context, &request.parent)?;
        Ok(v1::CasesListResponse { cases,..Default::default() }) }).await?;
reply.resolved_at=Some(cut);Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
