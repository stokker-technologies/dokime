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

pub const ID: &str = "dokime.v1.Cases/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CasesGetRequest, v1::CasesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CasesGetRequest) -> Result<v1::CasesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (mut reply,cut)=super::views::read(context,&format!("{ID}/{}/{}",request.parent,request.name.clone()),request.at.as_ref(),async { 
        let case = super::execution_state::planned(context, &request.parent, &request.name)?;
        let summary = super::execution_state::cases(context, &request.parent)?.into_iter().find(|case| case.case_id == request.name);
        let parameter_count = super::execution_state::case(case.origin.as_ref().unwrap())?.parameters.len() as u64;
        Ok(v1::CasesGetResponse {value:Some(v1::CaseDetail {
            summary,
            parameter_count,
            generation_index: case.ordinal,
            observed_cursor: Some(super::execution_state::observed(context)),
            ..Default::default()
        }),..Default::default()}) }).await?;
reply.resolved_at=Some(cut);Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
