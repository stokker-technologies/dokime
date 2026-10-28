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

pub(crate) mod cycles;
pub(crate) mod records;
pub(crate) mod lifecycle;
pub(crate) mod planning;
pub(crate) mod run;
pub(crate) mod storage;
pub(crate) mod streams;
pub(crate) mod artifacts;
pub(crate) mod cancel;
pub(crate) mod reboot;
pub(crate) mod diagnostics;
pub(crate) mod contention;
pub(crate) mod report_publication;
pub(crate) mod report_restart;
pub(crate) mod report_cancellation;

/* === Re-Exports === */

pub(crate) use crate::service_support::prelude::*;

/* === Imports === */

// Standard Library Imports

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

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn execution_seal_report() {
    lifecycle::cycle_container_lifecycle().await;
    lifecycle::rerun_fresh_started().await;
    lifecycle::stop_archive_state().await;
    lifecycle::rename_admission_seal().await;
    lifecycle::stopped_telemetry_refused().await;
    lifecycle::execution_stop_leaves_cycle_idle().await;
    lifecycle::pause_blocks_boundaries().await;
    lifecycle::continuation_follows_seal().await;
    lifecycle::queued_execution_identity().await;
    lifecycle::continuation_after_failure().await;
    planning::empty_cycle_grows().await;
    planning::empty_cycle_seals().await;
    run::cycle_exposes_rows().await;
    run::provider_lane_run().await;
    run::failing_panicking_failed().await;
    run::stop_holds_unwind().await;
    run::stop_spares_sibling().await;
    run::cycle_spanning_group().await;
    run::group_continues_after_failure().await;
    storage::execution_save_sealing().await;
    storage::terminal_capture_repaired().await;
    cancel::cancel_skips_unstarted().await;
    reboot::native_history_index().await;
    reboot::stored_telemetry_reboot().await;
    diagnostics::stored_provider_paths().await;
    report_publication::dependency_publication_retry().await;
    report_publication::admitted_create_window().await;
    report_restart::persisted_cycle_restart().await;
    report_cancellation::stopped_report_boundary().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn execution_seal_report_threaded() {
    storage::permanent_finalization_settles().await;
}

// The Smoke Provider Stages Share One Embedded Fixture Binding.
#[tokio::test]
async fn execution_seal_report_smoke() {
    contention::contended_reads_interactive().await;
    artifacts::saved_evidence_export().await;
    artifacts::required_evidence_files().await;
    artifacts::chunked_artifact_rewrite().await;
}

// Each Of These Stages Boots Its Own Embedded Runtime And Owns Process-Wide Fixture State.
#[tokio::test]
async fn execution_seal_report_streams() {
    streams::live_rows_paths().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn execution_seal_report_transfers() {
    artifacts::abandoned_transfer_slots().await;
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
