/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Seal And Report Story: cycle create, append, run, seal, report, open report.

/* === Modules === */

pub(crate) mod artifacts;
pub(crate) mod cancel;
pub(crate) mod contention;
pub(crate) mod cycles;
pub(crate) mod diagnostics;
pub(crate) mod lifecycle;
pub(crate) mod planning;
pub(crate) mod reboot;
pub(crate) mod records;
#[cfg(feature = "testing")]
pub(crate) mod report_background;
pub(crate) mod report_cancellation;
pub(crate) mod report_publication;
pub(crate) mod report_restart;
pub(crate) mod run;
pub(crate) mod storage;
pub(crate) mod streams;

/* === Re-Exports === */

pub(crate) use crate::service_support::prelude::*;

/* === Imports === */

// Standard Library Imports
use std::{future::Future, pin::Pin};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn published_cycle(runtime: &RuntimeHandle, id: &TestCycleId) -> dokime_domain::model::execution::result::CycleView {
    let cycle = cycle_snapshot(runtime, id).await;
    for execution in cycle.test_executions.iter().filter(|execution| execution.finished_at.is_some()) {
        runtime.wait_execution_report(&execution.test_execution_id).await.unwrap();
    }
    cycle_snapshot(runtime, id).await
}

/* === Private Functions === */

fn stage<F: Future<Output = ()>>(future: F) -> Pin<Box<F>> {
    eprintln!("Stage: {}", std::any::type_name::<F>());
    Box::pin(future)
}

/* === Tests === */

#[tokio::test]
async fn execution_seal_report() {
    // Each Story Stage Owns Its Async Frame Independently.
    stage(lifecycle::cycle_container_lifecycle()).await;
    stage(lifecycle::rerun_fresh_started()).await;
    stage(lifecycle::stop_archive_state()).await;
    stage(lifecycle::rename_admission_seal()).await;
    stage(lifecycle::stopped_telemetry_refused()).await;
    stage(lifecycle::execution_stop_leaves_cycle_idle()).await;
    stage(lifecycle::pause_blocks_boundaries()).await;
    #[cfg(feature = "testing")]
    stage(lifecycle::continuation_follows_drain()).await;
    stage(lifecycle::queued_execution_identity()).await;
    stage(lifecycle::continuation_after_failure()).await;
    stage(planning::empty_cycle_grows()).await;
    stage(planning::empty_cycle_seals()).await;
    stage(run::cycle_exposes_rows()).await;
    stage(run::provider_lane_run()).await;
    stage(run::failing_panicking_failed()).await;
    stage(run::stop_holds_unwind()).await;
    stage(run::stop_spares_sibling()).await;
    stage(run::cycle_spanning_group()).await;
    stage(run::group_continues_after_failure()).await;
    stage(storage::execution_save_sealing()).await;
    stage(storage::terminal_capture_repaired()).await;
    stage(cancel::cancel_skips_unstarted()).await;
    stage(reboot::native_history_index()).await;
    stage(reboot::stored_telemetry_reboot()).await;
    stage(diagnostics::stored_provider_paths()).await;
    stage(report_publication::dependency_publication_retry()).await;
    stage(report_publication::admitted_create_window()).await;
    stage(report_restart::persisted_cycle_restart()).await;
    stage(report_cancellation::stopped_report_boundary()).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn execution_seal_report_threaded() {
    stage(storage::permanent_finalization_settles()).await;
    stage(storage::unsealed_cycle_survives_restart()).await;
}

// The Smoke Provider Stages Share One Native SDK Fixture Binding.
#[tokio::test]
async fn execution_seal_report_smoke() {
    stage(contention::contended_reads_interactive()).await;
    stage(artifacts::saved_evidence_export()).await;
    stage(artifacts::required_evidence_files()).await;
    stage(artifacts::chunked_artifact_rewrite()).await;
}

// Each Stage Boots Its Own Runtime And Owns Its Native SDK Fixture State.
#[tokio::test]
async fn execution_seal_report_streams() {
    stage(streams::live_rows_paths()).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn execution_seal_report_transfers() {
    stage(artifacts::abandoned_transfer_slots()).await;
}

#[test]
fn execution_seal_report_sync() {
    cycles::catalog_reconciliation_preserves_execution_evidence();
    cycles::evidence_admission_requires_existing_owners();
    cycles::reruns_invalidate_steps();
    cycles::step_attempt_rewrites();
    cycles::append_checks_execution();
    cycles::append_admissions_identity();
    cycles::repeated_case_order();
    cycles::admission_freezes_them();
    cycles::settlement_replay_timestamp();
    cycles::recovery_completed_outcomes();
    cycles::execution_requires_snapshot();
    cycles::admission_validates_parameters();
    cycles::terminal_traceability_history();
    cycles::bounded_cycle_admission();
    cycles::copied_cycle_reusing();
    cycles::recovery_requires_request();
    cycles::recovery_keeps_intent();
    cycles::recovery_seals_receipts();
    cycles::copied_cycle_membership();
    cycles::cycle_admission_provenance();
    cycles::request_queues_evidence();
    cycles::request_admission_owner();
    cycles::execution_selection_attempts();
    cycles::selected_attempt_stop();
    cycles::canonical_execution_recovery();
    cycles::cycle_projection_atomic();
    cycles::start_plan_matches_admission();
    cycles::reset_plan_matches_applied_reset();
    records::plugin_may_declares();
    records::admission_order_publishers();
}
