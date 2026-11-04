/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Live All Frames From The Same Bounded Checkpoints Used By Reports.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::context::RuntimePlotEnvelopeContext;
use crate::plot::{PlotPayloadEnvelope, PlotPayloadRequest, plot_payload_diagnostics};
use crate::runtime::{Result, handle::RuntimeHandle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn query(
    context: &RuntimePlotEnvelopeContext, runtime: &RuntimeHandle, request: &PlotPayloadRequest, at: &harmos::storage::StorageCursor,
    current: bool,
) -> Result<Option<harmos::storage::ReadBatch<PlotPayloadEnvelope>>> {
    if request.window_s.is_some() || request.max_points == Some(0) {
        return Ok(None);
    }
    if !current && !runtime.storage().cursor_saved().covers(at)? {
        return Ok(None);
    }
    let Some(scope) = &request.scope else { return Ok(None) };
    let executions = if let Some(execution) = &scope.test_execution_id {
        vec![execution.to_string()]
    } else {
        context
            .snapshot
            .state()
            .executions
            .values()
            .filter(|execution| execution.scope.is_visible_in(scope))
            .map(|execution| execution.test_execution_id.to_string())
            .collect()
    };
    let mut bounded = request.clone();
    if bounded.max_points.is_none() {
        bounded.max_points = Some(runtime.read_application_settings().await?.settings.report.plot_points as usize);
    }
    let Some((mut payload, through)) = runtime.local_storage.live_plot_previews(executions, bounded, at.clone(), current).await? else {
        return Ok(None);
    };
    let (run, cycle) = context.run_and_cycle_start(request.scope.as_ref());
    payload.run_started_at = run.map(ToString::to_string);
    payload.cycle_started_at = cycle.map(ToString::to_string);
    payload.generated_at = context.generated_at.to_string();
    let mut payload = harmos::storage::ReadBatch::new(payload);
    if request.analysis.is_some()
        && payload.analysis.is_none()
        && let Some(analysis) = super::analysis::Prepared::resolve(runtime, context.snapshot.state(), request)?
    {
        payload = analysis.execute(runtime, request, &through, payload, super::telemetry_contract(context, &request.stream)).await?;
    }
    let diagnostics = plot_payload_diagnostics(&payload);
    let warnings = if request.summary_window.is_some() && payload.summaries.is_empty() {
        vec!["Exact zoom statistics are unavailable from the bounded All preview.".into()]
    } else {
        Vec::new()
    };
    Ok(Some(payload.try_map(|payload| {
        Ok::<_, crate::runtime::Error>(PlotPayloadEnvelope {
            plot_id: request.plot_id.as_deref().unwrap_or(&request.stream).to_owned(),
            revision: through.sequence,
            payload,
            diagnostics,
            warnings,
        })
    })?))
}

/* === Private Functions === */

/* === Tests === */
