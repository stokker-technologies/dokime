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
use crate::RuntimeState;
use crate::{
    RuntimeConfig,
    evidence_runtime::EvidenceCounts,
    queries::{RuntimeCycleSummary, RuntimeOverviewFrame, RuntimeOverviewSnapshot, RuntimeRetentionDiagnostics},
    runtime::state::contracts::RuntimeContractLookup,
};
use dokime_domain::model::execution::result::CycleView;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn overview_entity_counts(state: &RuntimeState, artifact_records: usize) -> RuntimeOverviewFrame {
    RuntimeOverviewFrame {
        shell: None,
        suites: state.suites.suites().count(),
        cycles: state.cycles.len(),
        executions: state.executions.len(),
        installed_plugins: 0,
        records: state.records.record_visibility.len(),
        telemetry_streams: 0,
        telemetry_samples: 0,
        resources: state.resource_records().len(),
        health: state.health_items().len(),
        traceability: traceability_snapshot_count(state),
        notes: state.notes().len(),
        documents: state.executions.values().map(|execution| execution.documents.len()).sum::<usize>(),
        record_definitions: state.record_definitions.len(),
        telemetry_definitions: state.telemetry_definitions.len(),
        artifacts: artifact_records,
    }
}

pub(crate) fn evidence_counts(state: &RuntimeState, artifacts: usize) -> EvidenceCounts {
    EvidenceCounts {
        records: state.records.record_visibility.len(),
        // Telemetry is off the journaled state (durable streams); no cheap
        // synchronous count. Async count paths use `collect_telemetry`.
        telemetry: 0,
        prompts: state.pending_prompt_projection().len(),
        artifacts,
        health: state.health_items().len(),
        traceability: traceability_snapshot_count(state),
        notes: state.notes().len(),
    }
}

pub(crate) fn contract_lookup(state: &RuntimeState) -> RuntimeContractLookup {
    RuntimeContractLookup::from_registries(&state.telemetry_definitions)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn runtime_overview_snapshot(
    state: &RuntimeState, config: &RuntimeConfig, revision: u64, cycles: Vec<CycleView>, evidence_counts: EvidenceCounts,
    telemetry_cursor_entries: usize,
) -> RuntimeOverviewSnapshot {
    let cycle_summaries = cycles.iter().map(RuntimeCycleSummary::from_cycle).collect();
    RuntimeOverviewSnapshot {
        revision,
        shutting_down: state.shutting_down,
        session_id: state.session_id.clone(),
        identity: state.identity.clone(),
        hub_metadata: config.hub_metadata.clone(),
        test_suite_count: state.suites.suites().count(),
        cycle_summaries,
        cycles,
        evidence_counts,
        installed_plugin_errors: state.installed_plugin_errors.clone(),
        retention: retention_diagnostics(state, telemetry_cursor_entries),
    }
}

/* === Private Functions === */

fn retention_diagnostics(state: &RuntimeState, telemetry_cursor_entries: usize) -> RuntimeRetentionDiagnostics {
    RuntimeRetentionDiagnostics {
        state_cycles: state.cycles.len(),
        state_executions_total: state.executions.len(),
        state_record_count: state.record_count,
        telemetry_cursor_entries,
    }
}

fn traceability_snapshot_count(state: &RuntimeState) -> usize {
    crate::evidence_runtime::state::traceability_snapshots(state).len()
}

/* === Tests === */
