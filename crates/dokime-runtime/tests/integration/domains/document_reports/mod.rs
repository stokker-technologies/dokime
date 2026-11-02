/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Reports Story: documents constructed from live and stored evidence,
// captured at their cut, rendered into cycle and execution reports, sealed,
// downloaded and read back after a reboot.

/* === Modules === */

mod capture;
mod construction;
mod report_payloads;
mod reports;

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

// One runtime carries the stages that build and read reports of executions the
// publishing executor settles on its own; each stage runs its own cycle.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_reports() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    construction::default_document_order(&runtime).await;
    construction::current_cycle_state(&runtime).await;
    construction::cycle_report_executions(&runtime).await;
    construction::saved_report_evidence(&runtime).await;
    report_payloads::latest_member_annotations(&runtime).await;
    report_payloads::cycle_report_planned(&runtime).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// The rendered payload shapes need no runtime: they render fixture input.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_report_payloads() {
    report_payloads::rendered_html_data().await;
    report_payloads::rendered_html_payload().await;
    report_payloads::materialized_report_axis().await;
    report_payloads::rendered_cycle_out().await;
    report_payloads::execution_template_cases().await;
    report_payloads::rendered_case_plots().await;
    report_payloads::rendered_case_one().await;
    report_payloads::rendered_report_interval().await;
    report_payloads::rendered_html_groups().await;
    report_payloads::stated_section_wrote().await;
    report_payloads::rendered_report_happened().await;
    report_payloads::failed_step_frames().await;
    report_payloads::normalised_report_rows().await;
    report_payloads::template_parameter_units().await;
    report_payloads::criterion_step_order().await;
    report_payloads::telemetry_index_endpoints().await;
}

// Each of these needs its own executor, configuration or cycle held open, so each boots its own runtime.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_report_live() {
    capture::notes_until_seal().await;
    #[cfg(feature = "fault-injection")]
    capture::undeclared_records_settle().await;
    construction::section_run_report().await;
    construction::current_fact_frontiers().await;
    construction::document_routes_samples().await;
    construction::repeated_sections_follow().await;
    reports::report_opens_anytime().await;
    reports::open_report_repeatedly().await;
    reports::cycle_report_reboot().await;
}

// The saved receipt builds its storage by hand and the download story holds a second execution.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn document_report_seal() {
    reports::report_save_receipt().await;
    reports::report_download_serves().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
#[cfg(feature = "testing")]
async fn concurrent_seal_is_durable() {
    reports::concurrent_seal_is_durable().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
#[cfg(feature = "testing")]
async fn checkpoint_preserves_inflight_seal() {
    reports::checkpoint_preserves_inflight_seal().await;
}
