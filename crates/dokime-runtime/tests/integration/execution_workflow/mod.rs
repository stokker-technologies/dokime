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

pub(crate) mod authoring;
pub(crate) mod definition;
pub(crate) mod prompts;
pub(crate) mod queue;
pub(crate) mod session;
#[cfg(feature = "simulation")]
pub(crate) mod simulation;
pub(crate) mod steps;
pub(crate) mod workflow;

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
async fn execution_workflow() {
    workflow::interactive_commands_downstream().await;
    workflow::interactive_idle_stop().await;
    workflow::runtime_bindings_metadata().await;
    workflow::stated_section_step().await;
    workflow::step_failure_isolation().await;
    workflow::lifecycle_logs_failure().await;
    workflow::context_scoped_stable().await;
    workflow::declared_stream_decides().await;
    workflow::streamed_artifact_chunks().await;
    workflow::watched_directory_in().await;
    workflow::cancelled_run_produced().await;
    workflow::criterion_could_failed().await;
    workflow::unanswered_question_id().await;
    workflow::cancelling_waiting_execution().await;
    workflow::cleanup_runs_pass().await;
    workflow::initialize_failure_execution().await;
    workflow::initialize_cancellation_cleanup().await;
    workflow::cleanup_procedure_failure().await;
    workflow::resources_release_verdict().await;
    workflow::record_wait_publication().await;
    workflow::capture_declared_scope().await;
    workflow::settlement_late_receipt().await;
    workflow::callbacks::interrupted_join_teardown().await;
    authoring::wait_completes_duration().await;
}

// Cancellation And Settlement Cross The Native SDK Connection.
#[tokio::test]
async fn execution_workflow_session() {
    session::cancellation_retains_verdict().await;
    session::ignored_run_identity().await;
    session::ignored_signal_identity().await;
    session::refused_settlement_winds_down().await;
    session::abandoned_admission_closes().await;
    session::provider_failure_releases().await;
}

// Queue And Prompt Stages Drive Commands Through Their Own Runtime.
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
    prompts::prompt_commands_records().await;
    prompts::condition_gated_runtime().await;
    prompts::stopping_cycle_waits().await;
    prompts::shutdown_preserves_prompts().await;
    prompts::cycle_stop_settles().await;
    prompts::execution_stop_settles().await;
    prompts::interruption_settles_prompts().await;
    prompts::answer_rules_commit().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn execution_workflow_pooled() {
    steps::harness_mirror_arms_before_rotation_and_retains_both_segments().await;
    steps::harness_copy_truncate_retains_the_previous_segment().await;
    steps::swept_watch_warning().await;
    workflow::artifact_backpressure().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn execution_workflow_threaded() {
    authoring::resource_store_handle().await;
    authoring::wait_until_shutdown().await;
    authoring::file_helpers_scope().await;
    authoring::param_helpers_values().await;
    definition::doc_example_contract().await.unwrap();
    definition::doc_example_context().await.unwrap();
    session::live_replay_settles().await;
}

#[tokio::test]
async fn execution_workflow_asking() {
    prompts::answered_execution_question().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn execution_workflow_asking_threaded() {
    prompts::stopped_execution_question().await;
    prompts::stopped_cycle_question().await;
}

#[tokio::test(start_paused = true)]
async fn execution_workflow_paused() {
    workflow::unacknowledged_invoke_callback().await;
    workflow::cleanup_joins_release().await;
    workflow::pending_sampler_cleanup().await;
    workflow::stalled_criterion_indeterminate().await;
    workflow::record_nobody_error().await;
    workflow::callbacks::shutdown_joins_owners().await;
}

#[test]
fn execution_workflow_sync() {
    workflow::signal_refuses_deliveries();
    definition::doc_example_matrix().unwrap();
    #[cfg(feature = "simulation")]
    simulation::declaration_boots_runtime();
}
