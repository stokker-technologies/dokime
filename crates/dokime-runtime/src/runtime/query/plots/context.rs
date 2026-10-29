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
use crate::plot::{PlotPayloadRequest, enrich_plot_samples_with_cycle_context, enrich_plot_samples_with_traceability_context};
use crate::runtime::handle::RuntimeHandle;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::traceability::TraceabilitySnapshot;
use dokime_domain::model::values::ValueFormat;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(super) struct RuntimePlotEnvelopeContext {
    pub(super) generated_at: harmos::storage::ReadBatch<String>,
    pub(super) snapshot: crate::queries::RuntimeStateSnapshot,
    cycle_context: bool,
    traceability: Option<harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimePlotEnvelopeContext {
    pub(super) async fn load_at(
        runtime: &RuntimeHandle, requests: &[PlotPayloadRequest], at: &harmos::storage::StorageCursor, session: &crate::storage::Manager,
    ) -> crate::runtime::Result<Self> {
        Self::at(runtime, requests, runtime.capture_state().await?, at, session).await
    }

    pub(super) async fn at(
        runtime: &RuntimeHandle, requests: &[PlotPayloadRequest], snapshot: crate::queries::RuntimeStateSnapshot,
        at: &harmos::storage::StorageCursor, session: &crate::storage::Manager,
    ) -> crate::runtime::Result<Self> {
        let needs_dimension_context = requests.iter().any(request_needs_dimension_context);
        // Cycles are also what an `elapsed_since_run`/`elapsed_since_cycle`
        // format counts from, so a request declaring one needs them loaded
        // too, even when nothing else here needs dimension context.
        let needs_cycles = needs_dimension_context || requests.iter().any(request_needs_elapsed_origin);

        let traceability = if needs_dimension_context {
            Some(runtime.retained_traceability_at_in(session, snapshot.state(), None, at, false).await?)
        } else {
            None
        };
        let captured_at = time::OffsetDateTime::now_utc();
        let clock = harmos::storage::ReadBatch::new(());
        let generated_at = clock.try_map(|_| crate::storage::encode_datetime(captured_at))?;
        Ok(Self {
            generated_at,
            snapshot,
            cycle_context: needs_cycles,
            traceability,
        })
    }

    pub(super) fn enrich_sample(&self, sample: &mut TelemetrySample, request: &PlotPayloadRequest) {
        if let Some(execution) = self.execution_for_scope(sample.scope.as_ref(), request) {
            enrich_plot_samples_with_cycle_context(std::slice::from_mut(sample), execution);
        }
        if let Some(traceability) = self.traceability.as_ref().map(|rows| rows.as_slice()) {
            enrich_plot_samples_with_traceability_context(std::slice::from_mut(sample), traceability);
        }
    }

    fn execution_for_scope(&self, scope: Option<&RecordScope>, request: &PlotPayloadRequest) -> Option<&TestExecution> {
        if !self.cycle_context {
            return None;
        }
        let scope = scope?;
        let id = scope.test_cycle_id.as_ref().or_else(|| request.scope.as_ref().and_then(|scope| scope.test_cycle_id.as_ref()))?;
        let state = self.snapshot.state();
        let cycle = state.cycles.get(id)?;
        if let Some(id) = scope.test_execution_id.as_ref() {
            return state.executions_for_cycle(cycle).find(|execution| &execution.test_execution_id == id);
        }
        let id = scope.test_case_id.as_ref()?;
        state.executions_for_cycle(cycle).filter(|execution| &execution.test_case_id == id).last()
    }
    pub(super) fn run_and_cycle_start(&self, scope: Option<&RecordScope>) -> (Option<&Timestamp>, Option<&Timestamp>) {
        if !self.cycle_context {
            return (None, None);
        }
        let state = self.snapshot.state();
        let Some(cycle) = scope.and_then(|scope| scope.test_cycle_id.as_ref()).and_then(|id| state.cycles.get(id)) else {
            return (None, None);
        };
        let run_started_at = scope
            .and_then(|scope| scope.test_execution_id.as_ref())
            .and_then(|id| state.executions_for_cycle(cycle).find(|execution| &execution.test_execution_id == id))
            .and_then(|execution| execution.started_at.as_ref());
        (run_started_at, cycle.started_at.as_ref())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn request_needs_elapsed_origin(request: &PlotPayloadRequest) -> bool {
    fn is_elapsed(format: Option<&ValueFormat>) -> bool {
        matches!(format, Some(ValueFormat::ElapsedSinceRun { .. } | ValueFormat::ElapsedSinceCycle { .. }))
    }

    is_elapsed(request.x_format.as_ref()) || is_elapsed(request.y_format.as_ref())
}

/* === Tests === */
