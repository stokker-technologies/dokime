/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::CycleOrderKey;
use dokime_protocol::host::CycleStatus;
use dokime_protocol::host::CyclesListRequest;
use dokime_protocol::host::CyclesListResponse;
use harmos_protocol::model::OperationContract;
use std::cmp::Ordering;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CyclesListRequest, CyclesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: CyclesListRequest) -> Result<CyclesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let mut selection = request.clone();
        selection.at = None;
        selection.page = None;
        let key = format!("cycles/{}", crate::host::views::hex(&crate::host::paging::bytes(&selection)));
        let (saved, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let mut cycles = crate::host::execution_state::cycles(context)?;
            let filter = request.filter.clone().unwrap_or_default();
            if filter.statuses.iter().any(|status| CycleStatus::try_from(*status).is_err()) {
                return Err(Error::Invalid("List Cycles: Choose Valid Status Filters".into()));
            }
            let mut selected = Vec::new();
            for cycle in cycles.drain(..) {
                if !crate::host::execution_state::archive_matches(filter.archive, cycle.archived)?
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
                let created = crate::host::execution_state::time(cycle.created_at);
                if filter.created_after.is_some_and(|after| created < crate::host::execution_state::time(Some(after)))
                    || filter.created_before.is_some_and(|before| created > crate::host::execution_state::time(Some(before)))
                {
                    continue;
                }
                selected.push(cycle);
            }
            let order = request.order.unwrap_or_default();
            let key = CycleOrderKey::try_from(order.key).map_err(|_| Error::Invalid("List Cycles: Choose a Valid Order".into()))?;
            selected.sort_by(|left, right| {
                let order = match key {
                    CycleOrderKey::Label => {
                        left.metadata.as_ref().map(|metadata| &metadata.label).cmp(&right.metadata.as_ref().map(|metadata| &metadata.label))
                    }
                    CycleOrderKey::CreatedAt => {
                        crate::host::execution_state::time(left.created_at).cmp(&crate::host::execution_state::time(right.created_at))
                    }
                    _ => crate::host::execution_state::time(left.updated_at).cmp(&crate::host::execution_state::time(right.updated_at)),
                }
                .then_with(|| left.cycle_id.cmp(&right.cycle_id));
                if order == Ordering::Equal { left.cycle_id.cmp(&right.cycle_id) } else { order }
            });
            if order.descending {
                selected.reverse();
            }
            if let Some(selection) = &request.selection {
                selected.retain(|cycle| {
                    (selection.cycle_ids.is_empty() || selection.cycle_ids.contains(&cycle.cycle_id))
                        && (selection.phases.is_empty() || selection.phases.contains(&cycle.phase))
                });
            }
            Ok(CyclesListResponse { cycles: selected, ..Default::default() })
        })
        .await?;
        let (cycles, next_page_token) = crate::host::views::page(
            saved.cycles,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
        crate::host::bounded_reply(
            context,
            CyclesListResponse {
                cycles,
                resolved_at: Some(cut),
                next_page_token,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
