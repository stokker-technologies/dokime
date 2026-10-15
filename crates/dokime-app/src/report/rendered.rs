/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports rendered on request into memory. Nothing here writes a file or
// touches a seal: a sealed report stays the bytes finalization published,
// and these renderings are only ever delivered to the reader that asked.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::storage::QueryValue;
// Local Imports
use super::html::HtmlReport;
use crate::{
    Error, Result, RuntimeHandle,
    runtime::storage::{cycle::CycleSnapshot, execution::ExecutionView},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Renders a cycle's report from `snapshot`, sealed members or current ones.
pub(crate) async fn cycle(
    runtime: &RuntimeHandle,
    snapshot: CycleSnapshot,
    maximum: NonZeroUsize,
) -> Result<QueryValue<Vec<u8>>> {
    let html = HtmlReport::from_cycle(runtime, snapshot).await?;
    render(html, maximum).await
}

/// Renders one execution's report from `view`. `final_report` holds a settled
/// execution to the checks its seal would have applied.
pub(crate) async fn execution(
    runtime: &RuntimeHandle,
    view: &ExecutionView,
    final_report: bool,
    maximum: NonZeroUsize,
) -> Result<QueryValue<Vec<u8>>> {
    let html = HtmlReport::from_snapshot(runtime, view, final_report, maximum).await?;
    render(html, maximum).await
}

/* === Private Functions === */
async fn render(html: HtmlReport, maximum: NonZeroUsize) -> Result<QueryValue<Vec<u8>>> {
    tokio::task::spawn_blocking(move || {
        let length = html.length(maximum)?;
        let bytes = length
            .checked_add(std::mem::size_of::<Vec<u8>>())
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| Error::InvalidReportInputs("rendered report is empty".into()))?;
        Ok(html
            .manager
            .clone()
            .produce(bytes, |_| html.bytes(length).map_err(failure))?
            .try_into_value()?)
    })
    .await?
}

fn failure(source: Error) -> harmos::Error {
    harmos::Error::backend(
        "render report on request",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */
