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

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.browse_executions";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryBrowseExecutionsRequest, v1::ExecutionBrowsePage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryBrowseExecutionsRequest) -> Result<v1::ExecutionBrowsePage> {
    super::complete(ID, context.settings.read_timeout, async {
        let filter = request.filter.clone().unwrap_or_default();
        let order = request.order.unwrap_or_default();
        if filter.statuses.iter().any(|status| v1::ExecutionState::try_from(*status).is_err()) {
            return Err(Error::Invalid("Browse Executions: Select Valid Execution States".into()));
        }
        let key =
            v1::ExecutionOrderKey::try_from(order.key).map_err(|_| Error::Invalid("Browse Executions: Choose a Valid Order".into()))?;
        let mut entries = Vec::new();
        let mut outcomes: std::collections::BTreeMap<String, Vec<v1::ExecutionOutcome>> = std::collections::BTreeMap::new();
        for identifier in context.host.store().identifiers()? {
            let cut = context.host.store().cut(&identifier)?;
            let metadata = super::history::metadata(context, &cut)?;
            let (_, _, archived) = context.host.store().execution_presentation(&identifier)?;
            if !super::execution_state::archive_matches(filter.archive, archived)?
                || (!filter.statuses.is_empty() && !filter.statuses.contains(&metadata.status))
                || (!filter.cycle_ids.is_empty() && !filter.cycle_ids.contains(&cut.execution.cycle))
                || (!filter.template_ids.is_empty()
                    && metadata.template_id.as_ref().is_none_or(|template| !filter.template_ids.contains(template)))
            {
                continue;
            }
            if !filter.text.is_empty()
                && !format!("{} {} {:?}", metadata.execution_id, metadata.title, metadata.template_id)
                    .to_lowercase()
                    .contains(&filter.text.to_lowercase())
            {
                continue;
            }
            if metadata.started_at.is_none() && (filter.started_after.is_some() || filter.started_before.is_some()) {
                continue;
            }
            let started = super::execution_state::time(metadata.started_at);
            if filter.started_after.is_some_and(|after| started < super::execution_state::time(Some(after)))
                || filter.started_before.is_some_and(|before| started > super::execution_state::time(Some(before)))
            {
                continue;
            }
            outcomes.entry(cut.execution.cycle.clone()).or_default().push(cut.execution.outcome.clone().unwrap_or_default());
            let checkpoint = if cut.state == dokime_storage::ExecutionState::Sealed {
                Some(super::history::checkpoint(context, &identifier)?)
            } else {
                None
            };
            entries.push(v1::ExecutionBrowseEntry {
                metadata: Some(metadata),
                checkpoint,
                availability: if cut.state == dokime_storage::ExecutionState::Sealed {
                    v1::ExecutionAvailability::Stored
                } else {
                    v1::ExecutionAvailability::Live
                } as i32,
                archived,
            });
        }
        entries.sort_by(|left, right| {
            let left = left.metadata.as_ref().unwrap();
            let right = right.metadata.as_ref().unwrap();
            match key {
                v1::ExecutionOrderKey::Title => left.title.cmp(&right.title),
                v1::ExecutionOrderKey::FinishedAt => {
                    super::execution_state::time(left.finished_at).cmp(&super::execution_state::time(right.finished_at))
                }
                _ => super::execution_state::time(left.started_at).cmp(&super::execution_state::time(right.started_at)),
            }
            .then_with(|| left.execution_id.cmp(&right.execution_id))
        });
        if order.descending {
            entries.reverse();
        }
        let mut cycles = Vec::new();
        for (cycle_id, outcomes) in outcomes {
            let counts = super::projection::counts(&outcomes);
            let mut group = v1::ExecutionBrowseCycle {
                cycle_id: cycle_id.clone(),
                result_status: counts.result_status,
                execution_count: outcomes.len() as u64,
                declared_count: counts.declared_count,
                not_evaluated_count: counts.not_evaluated_count,
                lifecycle: v1::ExecutionBrowseLifecycle::Completed as i32,
                ..Default::default()
            };
            let mut cases = std::collections::BTreeSet::new();
            for entry in &entries {
                let metadata = entry.metadata.as_ref().unwrap();
                if metadata.cycle_id.as_ref() != Some(&cycle_id) {
                    continue;
                }
                if !metadata.case_id.is_empty() {
                    cases.insert(metadata.case_id.clone());
                }
                if metadata.status != v1::ExecutionState::Sealed as i32 {
                    if context.host.controls(&metadata.execution_id)?.can_cancel {
                        group.running_count += 1;
                    } else {
                        group.queued_count += 1;
                    }
                }
                if metadata.started_at.is_some_and(|at| {
                    group
                        .started_at
                        .is_none_or(|previous| super::execution_state::time(Some(at)) < super::execution_state::time(Some(previous)))
                }) {
                    group.started_at = metadata.started_at;
                }
                if metadata.finished_at.is_some_and(|at| {
                    group
                        .finished_at
                        .is_none_or(|previous| super::execution_state::time(Some(at)) > super::execution_state::time(Some(previous)))
                }) {
                    group.finished_at = metadata.finished_at;
                }
            }
            group.case_count = cases.len() as u64;
            for outcome in &outcomes {
                if let Some(v1::execution_outcome::Attribution::Evaluated(outcome)) = &outcome.attribution {
                    match v1::ResultStatus::try_from(outcome.status).unwrap_or_default() {
                        v1::ResultStatus::Passed => group.passed_count += 1,
                        v1::ResultStatus::Failed => group.failed_count += 1,
                        v1::ResultStatus::Blocked => group.blocked_count += 1,
                        _ => {}
                    }
                }
            }
            if group.running_count > 0 {
                group.lifecycle = v1::ExecutionBrowseLifecycle::Running as i32;
                group.finished_at = None;
            } else if group.queued_count > 0 {
                group.lifecycle = v1::ExecutionBrowseLifecycle::Pending as i32;
                group.finished_at = None;
            }
            let cycle = super::execution_state::cycle(context, &cycle_id)?;
            group.label = cycle.metadata.map(|metadata| metadata.label).filter(|label| !label.is_empty());
            group.phase = Some(cycle.phase);
            cycles.push(group);
        }
        let cursor =
            super::paging::cursor(context, format!("history.executions/{filter:?}/{order:?}"), entries.iter().map(super::paging::bytes));
        let (entries, next_after_execution_id) = super::paging::page(
            entries,
            request.limit,
            &request.after_execution_id,
            request.cursor.as_ref(),
            &cursor,
            context.settings.page_items,
            |entry| &entry.metadata.as_ref().unwrap().execution_id,
        )?;
        super::history::bounded(
            context,
            v1::ExecutionBrowsePage {
                entries,
                next_after_execution_id,
                discovery_complete: true,
                cycles,
                cursor: Some(cursor),
                ..Default::default()
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
