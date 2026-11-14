/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::ExecutionsListResponse;
use dokime_protocol::model::origin::Kind;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionsListRequest, ExecutionsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ExecutionsListRequest) -> Result<ExecutionsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let mut selection = request.clone();
        selection.at = None;
        selection.page = None;
        let key = format!("executions/{}", crate::host::views::hex(&crate::host::paging::bytes(&selection)));
        let (saved, cut) =
            crate::host::views::read(context, &key, request.at.as_ref(), async {
                let mut executions = Vec::new();
                for id in context.host.store().identifiers()? {
                    let frozen = context.host.store().metadata(&id)?;
                    if !request.parent.is_empty() && frozen.execution.cycle != request.parent {
                        continue;
                    }
                    let detail = crate::host::execution_state::detail(context, &id)?;
                    let summary = detail.summary.as_ref().unwrap();
                    if let Some(selection) = &request.selection {
                        if (!selection.cycle_ids.is_empty() && !selection.cycle_ids.contains(&frozen.execution.cycle))
                            || (!selection.statuses.is_empty() && !selection.statuses.contains(&summary.state))
                        {
                            continue;
                        }
                        if !selection.template_ids.is_empty() && !frozen.execution.origin.as_ref().is_some_and(|origin|
                        matches!(&origin.kind, Some(Kind::PluginRun(run)) if selection.template_ids.contains(&run.template))) { continue; }
                        if !crate::host::execution_state::archive_matches(selection.archive, summary.archived)? {
                            continue;
                        }
                    }
                    if let Some(filter) = &request.filter
                        && !filter.text.is_empty()
                        && !format!("{id} {}", context.host.store().execution_presentation(&id)?.1)
                            .to_lowercase()
                            .contains(&filter.text.to_lowercase())
                    {
                        continue;
                    }
                    executions.push(detail);
                }
                let outcomes = executions.iter().filter_map(|execution| execution.outcome.as_ref());
                let outcome_counts = crate::host::projection::counts(outcomes);
                Ok(ExecutionsListResponse {
                    executions,
                    outcome_counts: Some(outcome_counts),
                    ..Default::default()
                })
            })
            .await?;
        let (executions, next_page_token) = crate::host::views::page(
            saved.executions,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
        crate::host::bounded_reply(
            context,
            ExecutionsListResponse {
                executions,
                outcome_counts: saved.outcome_counts,
                resolved_at: Some(cut),
                next_page_token,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
