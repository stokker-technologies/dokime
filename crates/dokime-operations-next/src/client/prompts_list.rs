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
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Prompts/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::PromptsListRequest, v1::PromptsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PromptsListRequest) -> Result<v1::PromptsListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
if request.pending_only && request.page.is_some() {return Err(Error::Invalid("List Prompts: Pending Prompts Return Whole Without Page Controls".into()));}
let key=format!("prompts/{}/{:?}/{:?}/{}",request.parent,request.cycle_id,request.execution_id,request.pending_only);
let (saved,cut)=super::views::read(context,&key,request.at.as_ref(),async {
 let mut prompts=Vec::new();
 for id in context.host.store().identifiers()? {
  if request.execution_id.as_ref().is_some_and(|selected|selected!=&id) || !request.parent.is_empty() && request.parent!=id {continue;}
  let snapshot=super::execution_state::evidence(context,&id)?;
  if request.cycle_id.as_ref().is_some_and(|selected|selected!=&snapshot.execution.cycle) {continue;}
  for entry in &snapshot.evidence {
   if entry.kind!="prompt" {continue;}
   let question=dokime_protocol::context::PromptsCreateRequest::decode(entry.payload.as_slice())?;
   let name=super::projection::prompt_id(&id,&question.prompt,entry.position);
   let view=super::prompt_catalog::view(context,&name)?;
   if request.pending_only && view.response.is_some() {continue;}
   if request.pending_only && !context.host.pending()?.iter().any(|(execution,pending)|execution==&id && pending.prompt==question.prompt) {continue;}
   prompts.push(view);
  }
 }
 Ok(v1::PromptsListResponse {prompts,..Default::default()})
}).await?;
let (prompts,next_page_token)=if request.pending_only {(saved.prompts,String::new())} else {super::views::page(saved.prompts,request.page.as_ref(),context.settings.page_items)?};
super::bounded_reply(context,v1::PromptsListResponse {prompts,resolved_at:Some(cut),next_page_token})
    }).await
}

/* === Private Functions === */

/* === Tests === */
