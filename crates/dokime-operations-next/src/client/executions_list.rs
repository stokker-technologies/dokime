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

pub const ID: &str = "dokime.v1.Executions/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ExecutionsListRequest, v1::ExecutionsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionsListRequest) -> Result<v1::ExecutionsListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let mut selection=request.clone();selection.at=None;selection.page=None;
let key=format!("executions/{}",super::views::hex(&super::paging::bytes(&selection)));
let (saved,cut)=super::views::read(context,&key,request.at.as_ref(),async {
 let mut executions=Vec::new();
 for id in context.host.store().identifiers()? {
  let frozen=context.host.store().metadata(&id)?;
  if !request.parent.is_empty() && frozen.execution.cycle!=request.parent {continue;}
  let detail=super::execution_state::detail(context,&id)?;
  let summary=detail.summary.as_ref().unwrap();
  if let Some(selection)=&request.selection {
   if !selection.cycle_ids.is_empty() && !selection.cycle_ids.contains(&frozen.execution.cycle) || !selection.statuses.is_empty() && !selection.statuses.contains(&summary.state) {continue;}
   if !selection.template_ids.is_empty() && !frozen.execution.origin.as_ref().is_some_and(|origin|matches!(&origin.kind,Some(v1::origin::Kind::PluginRun(run)) if selection.template_ids.contains(&run.template))) {continue;}
   if !super::execution_state::archive_matches(selection.archive,summary.archived)? {continue;}
  }
  if let Some(filter)=&request.filter {if !filter.text.is_empty() && !format!("{id} {}",context.host.store().execution_presentation(&id)?.1).to_lowercase().contains(&filter.text.to_lowercase()){continue;}}
  executions.push(detail);
 }
 let outcomes=executions.iter().filter_map(|execution|execution.outcome.as_ref());let outcome_counts=super::projection::counts(outcomes);
 Ok(v1::ExecutionsListResponse {executions,outcome_counts:Some(outcome_counts),..Default::default()})
}).await?;let (executions,next_page_token)=super::views::page(saved.executions,request.page.as_ref(),context.settings.page_items)?;
super::bounded_reply(context,v1::ExecutionsListResponse {executions,outcome_counts:saved.outcome_counts,resolved_at:Some(cut),next_page_token})
    }).await
}

/* === Private Functions === */

/* === Tests === */
