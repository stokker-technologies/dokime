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

pub const ID: &str = "dokime.v1.Cycles/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CyclesListRequest, v1::CyclesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CyclesListRequest) -> Result<v1::CyclesListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let mut selection=request.clone();selection.at=None;selection.page=None;
let key=format!("cycles/{}",super::views::hex(&super::paging::bytes(&selection)));
let (saved,cut)=super::views::read(context,&key,request.at.as_ref(),async { 
        let mut cycles = super::execution_state::cycles(context)?;
        let filter = request.filter.clone().unwrap_or_default();
        if filter.statuses.iter().any(|status| v1::CycleStatus::try_from(*status).is_err()) {
            return Err(Error::Invalid("List Cycles: Choose Valid Status Filters".into()));
        }
        let mut selected = Vec::new();
        for cycle in cycles.drain(..) {
            if !super::execution_state::archive_matches(filter.archive, cycle.archived)?
                || (!filter.statuses.is_empty() && !filter.statuses.contains(&cycle.status))
            {
                continue;
            }
            let metadata = cycle.metadata.clone().unwrap_or_default();
            if !filter.text.is_empty()
                && !format!("{} {} {}", cycle.cycle_id, metadata.label, metadata.description)
                    .to_lowercase()
                    .contains(&filter.text.to_lowercase())
            {
                continue;
            }
            let created = super::execution_state::time(cycle.created_at);
            if filter.created_after.is_some_and(|after| created < super::execution_state::time(Some(after)))
                || filter.created_before.is_some_and(|before| created > super::execution_state::time(Some(before)))
            {
                continue;
            }
            selected.push(cycle);
        }
        let order = request.order.unwrap_or_default();
        let key = v1::CycleOrderKey::try_from(order.key).map_err(|_| Error::Invalid("List Cycles: Choose a Valid Order".into()))?;
        selected.sort_by(|left, right| {
            let order = match key {
                v1::CycleOrderKey::Label => {
                    left.metadata.as_ref().map(|metadata| &metadata.label).cmp(&right.metadata.as_ref().map(|metadata| &metadata.label))
                }
                v1::CycleOrderKey::CreatedAt => {
                    super::execution_state::time(left.created_at).cmp(&super::execution_state::time(right.created_at))
                }
                _ => super::execution_state::time(left.updated_at).cmp(&super::execution_state::time(right.updated_at)),
            }
            .then_with(|| left.cycle_id.cmp(&right.cycle_id));
            if order == std::cmp::Ordering::Equal { left.cycle_id.cmp(&right.cycle_id) } else { order }
        });
        if order.descending {
            selected.reverse();
        }
if let Some(selection)=&request.selection {selected.retain(|cycle|(selection.cycle_ids.is_empty() || selection.cycle_ids.contains(&cycle.cycle_id)) && (selection.phases.is_empty() || selection.phases.contains(&cycle.phase)));}
Ok(v1::CyclesListResponse {cycles:selected,..Default::default()}) }).await?;
let (cycles,next_page_token)=super::views::page(saved.cycles,request.page.as_ref(),context.settings.page_items)?;
super::bounded_reply(context,v1::CyclesListResponse {cycles,resolved_at:Some(cut),next_page_token})
    }).await
}

/* === Private Functions === */

/* === Tests === */
