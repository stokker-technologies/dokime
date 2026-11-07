/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Workflow Story: steps, callbacks, prompts, queue and session.

/* === Modules === */

pub(crate) mod definition;
pub(crate) mod embedded;
pub(crate) mod handle;
pub(crate) mod prompts;
pub(crate) mod queue;
pub(crate) mod session;
#[cfg(feature = "simulation")]
pub(crate) mod simulation;
pub(crate) mod steps;
pub(crate) mod commands;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_workflow() {
    commands::interactive_commands_downstream().await;
    commands::interactive_idle_stop().await;
    commands::embedded_execution_output().await;
    commands::refused_publication_fails().await;
    commands::runtime_bindings_metadata().await;
    commands::stated_section_step().await;
    commands::step_failure_isolation().await;
    commands::lifecycle_logs_failure().await;
    commands::context_scoped_stable().await;
    commands::declared_stream_decides().await;
    commands::streamed_artifact_chunks().await;
    commands::watched_directory_in().await;
    commands::cancelled_run_produced().await;
    commands::criterion_could_failed().await;
    commands::unanswered_question_id().await;
    commands::cancelling_waiting_execution().await;
    commands::cleanup_runs_pass().await;
    commands::initialize_failure_execution().await;
    commands::initialize_cancellation_cleanup().await;
    commands::cleanup_procedure_failure().await;
    commands::resources_release_verdict().await;
    commands::record_wait_publication().await;
    commands::capture_declared_scope().await;
    commands::settlement_late_receipt().await;
    commands::callbacks::interrupted_join_teardown().await;
}

// Embedded Provider Stages Own Process-Wide Fixture Registrations.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_workflow_embedded() {
    embedded::admitted_catalog_is_durable_before_exposure().await;
    embedded::replacement_settles_generation().await;
    embedded::shutdown_retains_owner().await;
    embedded::cancelled_shutdown_retry().await;
    embedded::failed_start_causes().await;
    embedded::cancelled_startup_owned().await;
    session::cancellation_retains_verdict().await;
    session::ignored_run_identity().await;
    session::ignored_signal_identity().await;
    session::refused_settlement_winds_down().await;
    session::abandoned_admission_closes().await;
    session::provider_failure_releases().await;
}

// Queue And Prompt Stages Drive Commands Through Their Own Runtime.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_workflow_queue() {
    queue::container_request_outcomes().await;
    queue::container_request_order().await;
    queue::container_seal_isolation().await;
    queue::unavailable_sidecar_context().await;
    queue::append_retains_active().await;
    queue::duplicate_case_executions().await;
    queue::run_queued_results().await;
    queue::continuation_ignores_failures().await;
    queue::later_cases_start().await;
    queue::running_queued_counts().await;
    queue::execution_retry_receipt().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_workflow_prompt_commands() {
    prompts::prompt_commands_records().await;
    prompts::condition_gated_runtime().await;
    prompts::stopping_cycle_waits().await;
    prompts::shutdown_preserves_prompts().await;
    prompts::cycle_stop_settles().await;
    prompts::execution_stop_settles().await;
    prompts::interruption_settles_prompts().await;
    prompts::answer_rules_commit().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn execution_workflow_pooled() {
    steps::harness_mirror_arms_before_rotation_and_retains_both_segments().await;
    steps::harness_copy_truncate_retains_the_previous_segment().await;
    steps::capture_seals_with_or_without_a_stop().await;
    steps::capture_spans_steps().await;
    commands::artifact_backpressure().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn execution_workflow_threaded() {
    session::live_replay_settles().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn plugin_settings_restart() {
    session::saved_change_waits_for_reload().await;
    session::reload_waits_for_cycle().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn plugin_hot_reload() {
    session::installs_apply_live().await;
    session::replacement_waits_for_cycle().await;
    session::reload_restarts_one_plugin().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn plugin_installation_replacement() {
    session::failed_installation_is_replaced().await;
    session::failed_installations_can_be_removed().await;
    session::refused_replacement_keeps_running().await;
}

// These Stages Serve An Embedded Template Whose Initialization Fails In This Environment Even Alone.
#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_workflow_registration() {
    embedded::registration_replaces_failed().await;
    embedded::cancelled_registration_until().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_workflow_asking() {
    prompts::answered_execution_question().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn execution_workflow_asking_threaded() {
    prompts::stopped_execution_question().await;
    prompts::stopped_cycle_question().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn execution_workflow_paused() {
    commands::unacknowledged_invoke_callback().await;
    commands::cleanup_joins_release().await;
    commands::pending_sampler_cleanup().await;
    commands::stalled_criterion_indeterminate().await;
    commands::record_nobody_error().await;
    commands::callbacks::shutdown_joins_owners().await;
}

#[dokime_tracing::trace_test]
#[test]
fn execution_workflow_sync() {
    commands::signal_refuses_deliveries();
    handle::scoped_parameter_values();
    definition::doc_example_contract().unwrap();
    definition::doc_example_matrix().unwrap();
    #[cfg(feature = "simulation")]
    simulation::declaration_boots_runtime();
}
