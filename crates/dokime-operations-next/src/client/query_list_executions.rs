/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.query.list_executions";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::QueryListExecutionsRequest, v1::ExecutionPage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::QueryListExecutionsRequest) -> Result<v1::ExecutionPage> {
    super::complete(ID, context.settings.read_timeout, async {
        if request.page.is_some() || request.at.is_some() { return Err(Error::Invalid("ListExecutions Paging and Snapshot Selection Are Not Yet Served by the New Host".into())); }
        let mut rows = Vec::new();
        for identifier in context.host.store().identifiers()? {
            let snapshot = context.host.store().read(&identifier)?;
            let row = super::projection::summary(&snapshot);
            if let Some(selection) = &request.selection {
                if selection.started_at.is_some() || selection.archive.is_some() { return Err(Error::Invalid("ListExecutions Time and Archive Filters Are Not Yet Served by the New Host".into())); }
                if !selection.cycle_ids.is_empty() && !selection.cycle_ids.contains(&snapshot.execution.cycle) { continue; }
                if !selection.statuses.is_empty() && !selection.statuses.contains(&row.state) { continue; }
                if !selection.template_ids.is_empty() && !snapshot.execution.origin.as_ref().is_some_and(|origin| matches!(&origin.kind, Some(v1::origin::Kind::PluginRun(run)) if selection.template_ids.contains(&run.template))) { continue; }
            }
            rows.push(row);
        }
        super::projection::bounded(rows.len(), context.settings.page_items)?;
        Ok(v1::ExecutionPage { rows, complete: true, ..Default::default() })
    }).await
}

/* === Private Functions === */

/* === Tests === */
