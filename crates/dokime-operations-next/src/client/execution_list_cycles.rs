/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution List Cycles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.list_cycles";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionListCyclesRequest, v1::ListCyclesReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionListCyclesRequest) -> Result<v1::ListCyclesReply> {
    super::complete(ID, context.settings.read_timeout, async {
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
        let cursor = super::paging::cursor(context, format!("cycles/{filter:?}/{order:?}"), selected.iter().map(super::paging::bytes));
        let (cycles, next_after_cycle_id) = super::paging::page(
            selected,
            request.limit,
            &request.after_cycle_id,
            request.cursor.as_ref(),
            &cursor,
            context.settings.page_items,
            |row| &row.cycle_id,
        )?;
        Ok(v1::ListCyclesReply {
            cycles,
            next_after_cycle_id,
            cursor: Some(cursor),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
