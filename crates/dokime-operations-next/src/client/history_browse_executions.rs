/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// History Browse Executions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error,Result,client::Context};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

pub const ID: &str = "dokime.history.browse_executions";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::HistoryBrowseExecutionsRequest,v1::ExecutionBrowsePage>(ID,settings.read_timeout) }
pub async fn handle(context: &Context,request: v1::HistoryBrowseExecutionsRequest) -> Result<v1::ExecutionBrowsePage> { super::complete(ID,context.settings.read_timeout,async {
        let filter = request.filter.clone().unwrap_or_default(); let order = request.order.clone().unwrap_or_default();
        if filter.statuses.iter().any(|status| v1::ExecutionState::try_from(*status).is_err()) { return Err(Error::Invalid("Browse Executions: Select Valid Execution States".into())); }
        let key = v1::ExecutionOrderKey::try_from(order.key).map_err(|_| Error::Invalid("Browse Executions: Choose a Valid Order".into()))?;
        let mut entries = Vec::new();
        let mut outcomes: std::collections::BTreeMap<String,Vec<v1::ExecutionOutcome>> = std::collections::BTreeMap::new();
        for identifier in context.host.store().identifiers()? {
            let cut = context.host.store().cut(&identifier)?;
            let metadata = super::history::metadata(context,&cut)?;
            let (_,_,archived) = context.host.store().execution_presentation(&identifier)?;
            if !super::execution_state::archive_matches(filter.archive,archived)? || (!filter.statuses.is_empty() && !filter.statuses.contains(&metadata.status)) || (!filter.cycle_ids.is_empty() && !filter.cycle_ids.contains(&cut.execution.cycle)) || (!filter.template_ids.is_empty() && metadata.template_id.as_ref().is_none_or(|template| !filter.template_ids.contains(template))) { continue; }
            if !filter.text.is_empty() && !format!("{} {} {:?}",metadata.execution_id,metadata.title,metadata.template_id).to_lowercase().contains(&filter.text.to_lowercase()) { continue; }
            let started = super::execution_state::time(metadata.started_at);
            if filter.started_after.is_some_and(|after| started < super::execution_state::time(Some(after))) || filter.started_before.is_some_and(|before| started > super::execution_state::time(Some(before))) { continue; }
            outcomes.entry(cut.execution.cycle.clone()).or_default().extend(cut.execution.outcome.clone());
            let checkpoint = if cut.state == dokime_storage::ExecutionState::Sealed { Some(super::history::checkpoint(context,&identifier)?) } else { None };
            entries.push(v1::ExecutionBrowseEntry { metadata: Some(metadata),checkpoint,availability: if cut.state == dokime_storage::ExecutionState::Sealed { v1::ExecutionAvailability::Stored } else { v1::ExecutionAvailability::Live } as i32,archived });
        }
        entries.sort_by(|left,right| { let left = left.metadata.as_ref().unwrap(); let right = right.metadata.as_ref().unwrap(); match key { v1::ExecutionOrderKey::Title => left.title.cmp(&right.title),v1::ExecutionOrderKey::FinishedAt => super::execution_state::time(left.finished_at).cmp(&super::execution_state::time(right.finished_at)),_ => super::execution_state::time(left.started_at).cmp(&super::execution_state::time(right.started_at)) }.then_with(|| left.execution_id.cmp(&right.execution_id)) });
        if order.descending { entries.reverse(); }
        let cursor = super::paging::cursor(context,format!("history.executions/{filter:?}/{order:?}"),entries.iter().map(super::paging::bytes));
        let (entries,next_after_execution_id) = super::paging::page(entries,request.limit,&request.after_execution_id,request.cursor.as_ref(),&cursor,context.settings.page_items,|entry| &entry.metadata.as_ref().unwrap().execution_id)?;
        let cycles = outcomes.into_iter().map(|(cycle_id,outcomes)| { let counts = super::projection::counts(&outcomes); v1::ExecutionBrowseCycle { cycle_id,result_status: counts.result_status,execution_count: outcomes.len() as u64,declared_count: counts.declared_count,not_evaluated_count: counts.not_evaluated_count,..Default::default() } }).collect();
        super::history::bounded(context,v1::ExecutionBrowsePage { entries,next_after_execution_id,discovery_complete: true,cycles,cursor: Some(cursor),..Default::default() })
}).await }

/* === Private Functions === */

/* === Tests === */
