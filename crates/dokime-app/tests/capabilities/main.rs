/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod documents;
pub(crate) mod evidence;
pub(crate) mod execution;
pub(crate) mod files;
pub(crate) mod installed_plugins;
#[path = "../support/mod.rs"]
pub mod support;
pub(crate) mod telemetry;

/* === Re-Exports === */

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

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let mut builder = if multithread {
        tokio::runtime::Builder::new_multi_thread()
    } else {
        tokio::runtime::Builder::new_current_thread()
    };
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn documents_contract() {
    {
        eprintln!("Scenario: documents::capture::native_capture_contexts");
        run_scenario(
            documents::capture::native_capture_contexts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::construction::default_document_order");
        run_scenario(
            documents::construction::default_document_order,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::construction::current_cycle_state");
        run_scenario(
            documents::construction::current_cycle_state,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::construction::cycle_report_executions");
        run_scenario(
            documents::construction::cycle_report_executions,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::construction::section_run_report");
        run_scenario(
            documents::construction::section_run_report,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::construction::current_fact_frontiers");
        run_scenario(
            documents::construction::current_fact_frontiers,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::construction::document_routes_samples");
        run_scenario(
            documents::construction::document_routes_samples,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::construction::saved_report_evidence");
        run_scenario(
            documents::construction::saved_report_evidence,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_html_data");
        run_scenario(
            documents::report_payloads::rendered_html_data,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_html_payload");
        run_scenario(
            documents::report_payloads::rendered_html_payload,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::materialized_report_axis");
        run_scenario(
            documents::report_payloads::materialized_report_axis,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_cycle_out");
        run_scenario(
            documents::report_payloads::rendered_cycle_out,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::execution_template_cases");
        run_scenario(
            documents::report_payloads::execution_template_cases,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_case_plots");
        run_scenario(
            documents::report_payloads::rendered_case_plots,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_case_one");
        run_scenario(
            documents::report_payloads::rendered_case_one,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_report_interval");
        run_scenario(
            documents::report_payloads::rendered_report_interval,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_html_groups");
        run_scenario(
            documents::report_payloads::rendered_html_groups,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::stated_section_wrote");
        run_scenario(
            documents::report_payloads::stated_section_wrote,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::rendered_report_happened");
        run_scenario(
            documents::report_payloads::rendered_report_happened,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::failed_step_frames");
        run_scenario(
            documents::report_payloads::failed_step_frames,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::report_payloads::telemetry_index_endpoints");
        run_scenario(
            documents::report_payloads::telemetry_index_endpoints,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: documents::reports::report_save_receipt");
        run_scenario(documents::reports::report_save_receipt, false, None, false);
    }
    {
        eprintln!("Scenario: documents::reports::cycle_report_reboot");
        run_scenario(documents::reports::cycle_report_reboot, false, None, false);
    }
}

#[test]
fn evidence_contract() {
    {
        eprintln!("Scenario: evidence::model::definitions_survive_completion");
        run_scenario(
            evidence::model::definitions_survive_completion,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::model::conflicting_contracts_rejected");
        run_scenario(
            evidence::model::conflicting_contracts_rejected,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::model::tracking_derives_kpis");
        run_scenario(evidence::model::tracking_derives_kpis, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::notes::empty_cycle_datasets");
        run_scenario(evidence::notes::empty_cycle_datasets, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::saturated_reads_plot");
        run_scenario(evidence::plots::saturated_reads_plot, true, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::snapshot_query_samples");
        run_scenario(evidence::plots::snapshot_query_samples, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::batch_request_cursors");
        run_scenario(evidence::plots::batch_request_cursors, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::execution_plots_samples");
        run_scenario(evidence::plots::execution_plots_samples, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::distribution_query_budget");
        run_scenario(
            evidence::plots::distribution_query_budget,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::plots::record_query_absent");
        run_scenario(evidence::plots::record_query_absent, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::selected_analysis_provenance");
        run_scenario(
            evidence::plots::selected_analysis_provenance,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::plots::selected_original_coordinates");
        run_scenario(
            evidence::plots::selected_original_coordinates,
            false,
            None,
            false,
        );
    }
    #[cfg(feature = "fft")]
    {
        eprintln!("Scenario: evidence::plots::selected_fft_manager");
        run_scenario(evidence::plots::selected_fft_manager, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::selected_analysis_read");
        run_scenario(evidence::plots::selected_analysis_read, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::selected_analysis_axes");
        run_scenario(evidence::plots::selected_analysis_axes, false, None, false);
    }
    {
        eprintln!("Scenario: evidence::plots::external_analysis_reload");
        run_scenario(
            evidence::plots::external_analysis_reload,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::publication::evidence_write_revision");
        run_scenario(
            evidence::publication::evidence_write_revision,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::publication::subscriptions_deliver_selection");
        run_scenario(
            evidence::publication::subscriptions_deliver_selection,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::publication::telemetry_batches_sample");
        run_scenario(
            evidence::publication::telemetry_batches_sample,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::publication::record_history_ranges");
        run_scenario(
            evidence::publication::record_history_ranges,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::publication::publish_record_warning");
        run_scenario(
            evidence::publication::publish_record_warning,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::publication::telemetry_refuses_partial");
        run_scenario(
            evidence::publication::telemetry_refuses_partial,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::records::native_publications_order");
        run_scenario(
            evidence::records::native_publications_order,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: evidence::traceability::resource_records_absent");
        run_scenario(
            evidence::traceability::resource_records_absent,
            false,
            None,
            false,
        );
    }
}

#[test]
fn execution_contract() {
    {
        eprintln!("Scenario: execution::artifacts::steps_streamed_media");
        run_scenario(
            execution::artifacts::steps_streamed_media,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::artifacts::abandoned_artifacts_execution");
        run_scenario(
            execution::artifacts::abandoned_artifacts_execution,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::authoring::resource_store_handle");
        run_scenario(
            execution::authoring::resource_store_handle,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::authoring::resource_store_removal");
        run_scenario(
            execution::authoring::resource_store_removal,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::authoring::wait_completes_duration");
        run_scenario(
            execution::authoring::wait_completes_duration,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::authoring::wait_until_shutdown");
        run_scenario(execution::authoring::wait_until_shutdown, true, None, false);
    }
    {
        eprintln!("Scenario: execution::authoring::file_helpers_scope");
        run_scenario(execution::authoring::file_helpers_scope, true, None, false);
    }
    {
        eprintln!("Scenario: execution::authoring::param_helpers_values");
        run_scenario(
            execution::authoring::param_helpers_values,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::authoring::traceability_param_group");
        run_scenario(
            execution::authoring::traceability_param_group,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::definition::doc_example_contract");
        run_scenario(
            execution::definition::doc_example_contract,
            true,
            None,
            false,
        )
        .expect("doc_example_contract succeeds");
    }
    {
        eprintln!("Scenario: execution::definition::doc_example_context");
        run_scenario(
            execution::definition::doc_example_context,
            true,
            None,
            false,
        )
        .expect("doc_example_context succeeds");
    }
    {
        eprintln!("Scenario: execution::definition::doc_example_matrix");
        execution::definition::doc_example_matrix().expect("doc_example_matrix succeeds");
    }
    {
        eprintln!("Scenario: execution::diagnostics::stored_provider_paths");
        run_scenario(
            execution::diagnostics::stored_provider_paths,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::embedded::registration_replaces_failed");
        run_scenario(
            execution::embedded::registration_replaces_failed,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::embedded::cancelled_registration_until");
        run_scenario(
            execution::embedded::cancelled_registration_until,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::embedded::replacement_settles_generation");
        run_scenario(
            execution::embedded::replacement_settles_generation,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::embedded::shutdown_retains_owner");
        run_scenario(
            execution::embedded::shutdown_retains_owner,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::embedded::cancelled_shutdown_retry");
        run_scenario(
            execution::embedded::cancelled_shutdown_retry,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::embedded::failed_start_causes");
        run_scenario(execution::embedded::failed_start_causes, false, None, false);
    }
    {
        eprintln!("Scenario: execution::embedded::cancelled_startup_owned");
        run_scenario(
            execution::embedded::cancelled_startup_owned,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::lifecycle::rerun_fresh_started");
        run_scenario(
            execution::lifecycle::rerun_fresh_started,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::lifecycle::stop_clear_state");
        run_scenario(execution::lifecycle::stop_clear_state, false, None, false);
    }
    {
        eprintln!("Scenario: execution::lifecycle::stopped_cycles_telemetry");
        run_scenario(
            execution::lifecycle::stopped_cycles_telemetry,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::lifecycle::pause_blocks_boundaries");
        run_scenario(
            execution::lifecycle::pause_blocks_boundaries,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::lifecycle::pause_refuses_states");
        run_scenario(
            execution::lifecycle::pause_refuses_states,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::planning::preview_start_plans");
        run_scenario(execution::planning::preview_start_plans, false, None, false);
    }
    {
        eprintln!("Scenario: execution::prompts::prompt_commands_records");
        run_scenario(
            execution::prompts::prompt_commands_records,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::duplicate_invalid_refused");
        run_scenario(
            execution::prompts::duplicate_invalid_refused,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::condition_gated_runtime");
        run_scenario(
            execution::prompts::condition_gated_runtime,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::stopping_cycle_waits");
        run_scenario(execution::prompts::stopping_cycle_waits, false, None, false);
    }
    {
        eprintln!("Scenario: execution::prompts::restart_closes_prompts");
        run_scenario(
            execution::prompts::restart_closes_prompts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::cycle_stop_settles_prompts");
        run_scenario(
            execution::prompts::cycle_stop_settles_prompts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::execution_stop_settles_prompts");
        run_scenario(
            execution::prompts::execution_stop_settles_prompts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::interruption_settles_prompts");
        run_scenario(
            execution::prompts::interruption_settles_prompts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::stopped_run_question");
        run_scenario(execution::prompts::stopped_run_question, true, None, false);
    }
    {
        eprintln!("Scenario: execution::prompts::requests_refuses_scopes");
        run_scenario(
            execution::prompts::requests_refuses_scopes,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::prompts::answer_rules_commit");
        run_scenario(execution::prompts::answer_rules_commit, false, None, false);
    }
    {
        eprintln!("Scenario: execution::queue::unavailable_sidecar_context");
        run_scenario(
            execution::queue::unavailable_sidecar_context,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::queue::append_reopens_cycle");
        run_scenario(execution::queue::append_reopens_cycle, false, None, false);
    }
    {
        eprintln!("Scenario: execution::queue::duplicate_case_executions");
        run_scenario(
            execution::queue::duplicate_case_executions,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::queue::run_queued_results");
        run_scenario(execution::queue::run_queued_results, false, None, false);
    }
    {
        eprintln!("Scenario: execution::queue::later_cases_start");
        run_scenario(execution::queue::later_cases_start, false, None, false);
    }
    {
        eprintln!("Scenario: execution::queue::request_retries_receipt");
        run_scenario(
            execution::queue::request_retries_receipt,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::reboot::native_history_index");
        run_scenario(execution::reboot::native_history_index, false, None, false);
    }
    {
        eprintln!("Scenario: execution::reboot::live_telemetry_state");
        run_scenario(execution::reboot::live_telemetry_state, false, None, false);
    }
    {
        eprintln!("Scenario: execution::run::cycle_exposes_rows");
        run_scenario(execution::run::cycle_exposes_rows, false, None, false);
    }
    {
        eprintln!("Scenario: execution::run::provider_lane_run");
        run_scenario(execution::run::provider_lane_run, false, None, false);
    }
    {
        eprintln!("Scenario: execution::run::failing_panicking_failed");
        run_scenario(execution::run::failing_panicking_failed, false, None, false);
    }
    {
        eprintln!("Scenario: execution::run::stop_holds_unwind");
        run_scenario(execution::run::stop_holds_unwind, false, None, false);
    }
    {
        eprintln!("Scenario: execution::run::execution_stop_spares_sibling");
        run_scenario(
            execution::run::execution_stop_spares_sibling,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::run::cycle_spanning_group");
        run_scenario(execution::run::cycle_spanning_group, false, None, false);
    }
    {
        eprintln!("Scenario: execution::run::fail_fast_failure");
        run_scenario(execution::run::fail_fast_failure, false, None, false);
    }
    {
        eprintln!("Scenario: execution::session::abandoned_transfer_log");
        run_scenario(
            execution::session::abandoned_transfer_log,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::session::live_stream_log_follow");
        run_scenario(
            execution::session::live_stream_log_follow,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::session::stopping_cycle_execution");
        run_scenario(
            execution::session::stopping_cycle_execution,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::session::stopping_ignored_run_settles");
        run_scenario(
            execution::session::stopping_ignored_run_settles,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::session::execution_scope_error");
        run_scenario(
            execution::session::execution_scope_error,
            false,
            None,
            false,
        );
    }
    #[cfg(feature = "sim")]
    {
        eprintln!("Scenario: execution::simulation::declaration_boots_runtime");
        execution::simulation::declaration_boots_runtime();
    }
    {
        eprintln!("Scenario: execution::sampling::paged_evidence_settles");
        run_scenario(
            execution::sampling::paged_evidence_settles,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::sampling::large_case_seals");
        run_scenario(execution::sampling::large_case_seals, false, None, false);
    }
    {
        eprintln!("Scenario: execution::sampling::whole_capture_plot_seals");
        run_scenario(
            execution::sampling::whole_capture_plot_seals,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::sampling::oversized_report_settles");
        run_scenario(
            execution::sampling::oversized_report_settles,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::sampling::logged_cycle_terminal");
        run_scenario(
            execution::sampling::logged_cycle_terminal,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::storage::permanent_finalization_settles");
        run_scenario(
            execution::storage::permanent_finalization_settles,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::storage::execution_save_sealing");
        run_scenario(
            execution::storage::execution_save_sealing,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::storage::terminal_capture_repaired");
        run_scenario(
            execution::storage::terminal_capture_repaired,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: execution::streams::live_rows_paths");
        run_scenario(execution::streams::live_rows_paths, false, None, false);
    }
    {
        eprintln!("Scenario: telemetry::sampling::captures_share_demand");
        run_scenario(
            telemetry::sampling::captures_share_demand,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: telemetry::sampling::new_capture_drains");
        run_scenario(telemetry::sampling::new_capture_drains, true, None, false);
    }
    {
        eprintln!("Scenario: telemetry::sampling::stopped_scope_sampler");
        run_scenario(
            telemetry::sampling::stopped_scope_sampler,
            true,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: telemetry::sampling::host_collector_stream");
        run_scenario(
            telemetry::sampling::host_collector_stream,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: telemetry::shared::shared_stream_publishes");
        run_scenario(
            telemetry::shared::shared_stream_publishes,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: telemetry::shared::shared_stream_schema_refused");
        run_scenario(
            telemetry::shared::shared_stream_schema_refused,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: telemetry::shared::shared_stream_owner_refused");
        run_scenario(
            telemetry::shared::shared_stream_owner_refused,
            false,
            None,
            false,
        );
    }
}

#[test]
fn files_contract() {
    {
        eprintln!("Scenario: files::watch::stream_local_tree");
        run_scenario(files::watch::stream_local_tree, true, None, false);
    }
    {
        eprintln!("Scenario: files::watch::mirror_local_tree");
        run_scenario(files::watch::mirror_local_tree, true, None, false);
    }
    {
        eprintln!("Scenario: files::watch::snapshot_local_tree");
        run_scenario(files::watch::snapshot_local_tree, true, None, false);
    }
    {
        eprintln!("Scenario: files::watch::watch_refusals");
        run_scenario(files::watch::watch_refusals, true, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: files::watch::mirror_keeps_last_copy");
        run_scenario(files::watch::mirror_keeps_last_copy, true, None, false);
    }
    {
        eprintln!("Scenario: files::watch::execution_seals_watch");
        run_scenario(files::watch::execution_seals_watch, true, None, false);
    }
    {
        eprintln!("Scenario: files::ftp::stream_ftp_source");
        run_scenario(files::ftp::stream_ftp_source, true, None, false);
    }
}

#[test]
fn installation_contract() {
    {
        eprintln!("Scenario: installed_plugins::brand::brand_toml_types");
        installed_plugins::brand::brand_toml_types();
    }
    {
        eprintln!("Scenario: installed_plugins::brand::app_brand_label");
        installed_plugins::brand::app_brand_label();
    }
    {
        eprintln!("Scenario: installed_plugins::brand::missing_logo_failing");
        installed_plugins::brand::missing_logo_failing();
    }
    {
        eprintln!("Scenario: installed_plugins::brand::runtime_appearance_brand");
        run_scenario(
            installed_plugins::brand::runtime_appearance_brand,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: installed_plugins::settings::boot_health_transaction");
        run_scenario(
            installed_plugins::settings::boot_health_transaction,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: installed_plugins::settings::install_definition_harmos");
        run_scenario(
            installed_plugins::settings::install_definition_harmos,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: installed_plugins::settings::artifact_registration_visible");
        run_scenario(
            installed_plugins::settings::artifact_registration_visible,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: installed_plugins::settings::composed_plugin_process");
        run_scenario(
            installed_plugins::settings::composed_plugin_process,
            false,
            None,
            false,
        );
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
