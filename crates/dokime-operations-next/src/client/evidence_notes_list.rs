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

pub const ID: &str = "dokime.v1.evidence.Notes/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::evidence::NotesListRequest, v1::evidence::NotesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::evidence::NotesListRequest) -> Result<v1::evidence::NotesListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let selection=request.selection.clone().unwrap_or_default();
let key=format!("notes/{}",super::views::hex(&super::paging::bytes(&selection)));
let (saved,cut)=super::views::read(context,&key,request.at.as_ref(),async {
 let store=context.host.store();let scope=selection.owner.as_ref().map(|owner|store.note_owner(owner)).transpose()?;
 let mut notes=Vec::new();let mut after=String::new();
 loop {let page=store.notes_page(scope.as_ref().and_then(|scope|scope.cycle_id.as_deref()),scope.as_ref().and_then(|scope|scope.execution_id.as_deref()),selection.include_context,&after,context.settings.page_items)?;
 if page.is_empty(){break;} after=page.last().unwrap().note_id.clone();notes.extend(page);}
 Ok(v1::evidence::NotesListResponse {notes,..Default::default()})
}).await?;
let (notes,next_page_token)=super::views::page(saved.notes,request.page.as_ref(),context.settings.page_items)?;
super::bounded_reply(context,v1::evidence::NotesListResponse {notes,resolved_at:Some(cut),next_page_token})
    }).await
}

/* === Private Functions === */

/* === Tests === */
