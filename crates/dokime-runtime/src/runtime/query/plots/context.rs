/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::request::request_needs_dimension_context;
use crate::execution::CycleView;
use crate::plot::{
    PlotPayloadRequest, enrich_plot_samples_with_cycle_context,
    enrich_plot_samples_with_traceability_context,
};
use crate::runtime::handle::RuntimeHandle;
use crate::traceability::TraceabilitySnapshot;
use crate::core::{RecordScope, Timestamp, ValueFormat};
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub(super) struct RuntimePlotEnvelopeContext {
    pub(super) generated_at: String,
    pub(super) snapshot: crate::queries::RuntimeStateSnapshot,
    cycles: Option<Vec<CycleView>>,
    traceability: Option<Vec<TraceabilitySnapshot>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimePlotEnvelopeContext {
    pub(super) async fn load(
        runtime: &RuntimeHandle,
        requests: &[PlotPayloadRequest],
    ) -> crate::runtime::Result<Self> {
        Ok(Self::at(runtime, requests, runtime.capture_state().await?).await)
    }

    pub(super) async fn at(
        runtime: &RuntimeHandle,
        requests: &[PlotPayloadRequest],
        snapshot: crate::queries::RuntimeStateSnapshot,
    ) -> Self {
        let needs_dimension_context = requests.iter().any(request_needs_dimension_context);
        // Cycles are also what an `elapsed_since_run`/`elapsed_since_cycle`
        // format counts from, so a request declaring one needs them loaded
        // too, even when nothing else here needs dimension context.
        let needs_cycles =
            needs_dimension_context || requests.iter().any(request_needs_elapsed_origin);
        let cycles = needs_cycles.then(|| snapshot.state().cycle_views().collect());
        let traceability = if needs_dimension_context {
            let mut values = crate::evidence_runtime::state::traceability_snapshots(snapshot.state());
            runtime.enrich_host_traceability(&mut values, None).await;
            Some(values)
        } else {
            None
        };
        Self {
            generated_at: crate::model::time::now_utc().to_string(),
            snapshot,
            cycles,
            traceability,
        }
    }

    pub(super) fn enrich_telemetry(
        &self,
        telemetry: Vec<TelemetrySample>,
        request: &PlotPayloadRequest,
    ) -> Vec<TelemetrySample> {
        let telemetry = match self.cycles.as_deref() {
            Some(cycles) => enrich_plot_telemetry(telemetry, cycles, request),
            None => telemetry,
        };
        match self.traceability.as_deref() {
            Some(traceability) => {
                enrich_plot_samples_with_traceability_context(telemetry, traceability)
            }
            None => telemetry,
        }
    }

    pub(super) fn run_and_cycle_start(
        &self,
        scope: Option<&RecordScope>,
    ) -> (Option<Timestamp>, Option<Timestamp>) {
        let (Some(cycles), Some(test_cycle_id)) = (
            self.cycles.as_deref(),
            scope.and_then(|scope| scope.test_cycle_id.as_ref()),
        ) else {
            return (None, None);
        };
        let Some(cycle) = cycles
            .iter()
            .find(|cycle| &cycle.test_cycle_id == test_cycle_id)
        else {
            return (None, None);
        };
        let run_started_at = scope
            .and_then(|scope| scope.test_execution_id.as_ref())
            .and_then(|execution_id| {
                cycle
                    .test_executions
                    .iter()
                    .find(|execution| &execution.test_execution_id == execution_id)
            })
            .and_then(|execution| execution.started_at.clone());

        (run_started_at, cycle.started_at.clone())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn request_needs_elapsed_origin(request: &PlotPayloadRequest) -> bool {
    fn is_elapsed(format: Option<&ValueFormat>) -> bool {
        matches!(
            format,
            Some(ValueFormat::ElapsedSinceRun { .. } | ValueFormat::ElapsedSinceCycle { .. })
        )
    }

    is_elapsed(request.x_format.as_ref()) || is_elapsed(request.y_format.as_ref())
}

fn enrich_plot_telemetry(
    telemetry: Vec<TelemetrySample>,
    cycles: &[CycleView],
    request: &PlotPayloadRequest,
) -> Vec<TelemetrySample> {
    telemetry
        .into_iter()
        .flat_map(|sample| {
            let cycle_id = sample
                .scope
                .as_ref()
                .and_then(|scope| scope.test_cycle_id.as_ref())
                .or_else(|| {
                    request
                        .scope
                        .as_ref()
                        .and_then(|scope| scope.test_cycle_id.as_ref())
                });
            let cycle = cycle_id
                .and_then(|cycle_id| cycles.iter().find(|cycle| &cycle.test_cycle_id == cycle_id));
            match cycle {
                Some(cycle) => enrich_plot_samples_with_cycle_context(vec![sample], cycle),
                None => vec![sample],
            }
        })
        .collect()
}

/* === Tests === */
