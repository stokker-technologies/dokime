/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Browse Cycles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.browse_cycles";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryBrowseCyclesRequest, v1::CycleBrowsePage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryBrowseCyclesRequest) -> Result<v1::CycleBrowsePage> {
    super::complete(ID, context.settings.read_timeout, async {
        use sha2::{Digest, Sha256};
        let filter = request.filter.unwrap_or_default();
        let selection = format!("cycles/{}", super::history::hex(&Sha256::digest(filter.encode_to_vec())));
        let current = context.host.store().history_frontier()?;
        let frontier = request.at.as_ref().map_or(current, |cursor| cursor.sequence);
        let at = super::history::position(context, frontier, &selection);
        if frontier > current
            || request.at.as_ref().is_some_and(|cursor| cursor != &at)
            || (request.after_sequence > 0 && request.at.is_none())
        {
            return Err(Error::Conflict("Browse Cycles: Restart with the Same Filter and a Fresh Storage Frontier".into()));
        }
        let mut rows = Vec::new();
        for (sequence, cycle) in context.host.store().saved_cycles(frontier)? {
            if !super::execution_state::archive_matches(filter.archive, cycle.archived)?
                || (!filter.statuses.is_empty() && !filter.statuses.contains(&cycle.status))
            {
                continue;
            }
            if !filter.text.is_empty()
                && !format!("{} {:?}", cycle.cycle_id, cycle.metadata).to_lowercase().contains(&filter.text.to_lowercase())
            {
                continue;
            }
            let time = super::execution_state::time(cycle.created_at);
            if filter.created_after.is_some_and(|after| time < super::execution_state::time(Some(after)))
                || filter.created_before.is_some_and(|before| time > super::execution_state::time(Some(before)))
            {
                continue;
            }
            rows.push((sequence, cycle));
        }
        if request.after_sequence > 0 && !rows.iter().any(|(sequence, _)| *sequence == request.after_sequence) {
            return Err(Error::Invalid("Browse Cycles: Use a Continuation Within the Captured Selection".into()));
        }
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        let mut rows = rows.into_iter().filter(|(sequence, _)| *sequence > request.after_sequence);
        let selected: Vec<_> = rows.by_ref().take(limit).collect();
        let next_after_sequence = if rows.next().is_some() { selected.last().map(|(sequence, _)| *sequence) } else { None };
        let entries = selected
            .into_iter()
            .map(|(_, cycle)| v1::SavedCycleSummary {
                cycle_id: cycle.cycle_id,
                metadata: cycle.metadata,
                phase: cycle.phase,
                created_at: cycle.created_at,
                updated_at: cycle.updated_at,
                execution_count: cycle.execution_count,
                archived: cycle.archived,
            })
            .collect();
        super::history::bounded(
            context,
            v1::CycleBrowsePage {
                entries,
                at: Some(at),
                next_after_sequence,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
