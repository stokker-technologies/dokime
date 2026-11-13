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

pub const ID: &str = "dokime.v1.content.Artifacts/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::content::ArtifactsListRequest, v1::content::ArtifactsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::content::ArtifactsListRequest) -> Result<v1::content::ArtifactsListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let key=format!("artifacts/{}",super::views::hex(&request.scope.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default()));
let (saved,cut)=super::views::read(context,&key,request.at.as_ref(),async {
 let store=context.host.store();let mut after=String::new();let mut artifacts=Vec::new();
 loop {let page=store.artifacts_page(&after,context.settings.page_items)?;if page.is_empty(){break;}after=page.last().unwrap().artifact_id.clone();artifacts.extend(page.into_iter().filter(|artifact|request.scope.as_ref().is_none_or(|selected|artifact.scope.as_ref().is_some_and(|scope|selected.cycle_id.as_ref().is_none_or(|id|scope.cycle_id.as_ref()==Some(id)) && selected.execution_id.as_ref().is_none_or(|id|scope.execution_id.as_ref()==Some(id)) && selected.case_id.as_ref().is_none_or(|id|scope.case_id.as_ref()==Some(id))))));}
 Ok(v1::content::ArtifactsListResponse {artifacts,..Default::default()})
}).await?;let (artifacts,next_page_token)=super::views::page(saved.artifacts,request.page.as_ref(),context.settings.page_items)?;
super::bounded_reply(context,v1::content::ArtifactsListResponse {artifacts,resolved_at:Some(cut),next_page_token})
    }).await
}

/* === Private Functions === */

/* === Tests === */
