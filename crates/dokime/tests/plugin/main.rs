/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod analysis;
pub(crate) mod authoring;
pub(crate) mod calls;
pub(crate) mod demand;
pub(crate) mod documents;
pub(crate) mod harmos;
pub(crate) mod logging;
pub(crate) mod native;
pub(crate) mod observers;
pub(crate) mod parameters;
pub(crate) mod plugin_settings;
pub(crate) mod prelude;
pub(crate) mod prompts;
pub(crate) mod resources;
pub(crate) mod sdk_error;
pub(crate) mod stream_admission;

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
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn declarations_contract() {
    {
        eprintln!("Scenario: authoring::catalog_lifecycle::registration_inert_once");
        authoring::catalog_lifecycle::registration_inert_once();
    }
    {
        eprintln!("Scenario: authoring::attributed_functions_runs");
        run_scenario(authoring::attributed_functions_runs, false, None, false);
    }
    {
        eprintln!("Scenario: authoring::registration_phase_envelope");
        authoring::registration_phase_envelope();
    }
    {
        eprintln!("Scenario: authoring::template_declares_operations");
        authoring::template_declares_operations();
    }
    {
        eprintln!("Scenario: authoring::suite_declaration_contracts");
        authoring::suite_declaration_contracts();
    }
    {
        eprintln!("Scenario: authoring::template_reference_is_suite_scoped");
        authoring::template_reference_is_suite_scoped();
    }
    {
        eprintln!("Scenario: authoring::one_step_phases");
        authoring::one_step_phases();
    }
    {
        eprintln!("Scenario: authoring::step_reads_rustdoc");
        authoring::step_reads_rustdoc();
    }
    {
        eprintln!("Scenario: authoring::template_declaration_declares");
        authoring::template_declaration_declares();
    }
    {
        eprintln!("Scenario: authoring::obsolete_template_refused");
        authoring::obsolete_template_refused();
    }
    {
        eprintln!("Scenario: authoring::second_suite_refused");
        authoring::second_suite_refused();
    }
    {
        eprintln!("Scenario: authoring::criterion_method_decides");
        authoring::criterion_method_decides();
    }
    {
        eprintln!("Scenario: authoring::stream_method_publishes");
        authoring::stream_method_publishes();
    }
    {
        eprintln!("Scenario: harmos::stream_alias_publication");
        run_scenario(harmos::stream_alias_publication, false, None, false);
    }
    {
        eprintln!("Scenario: parameters::optional_values");
        parameters::optional_values();
    }
    {
        eprintln!("Scenario: parameters::overrides_and_sweeps");
        parameters::overrides_and_sweeps();
    }
    {
        eprintln!("Scenario: parameters::optional_choices");
        parameters::optional_choices();
    }
    {
        eprintln!("Scenario: parameters::list_param_vec");
        parameters::list_param_vec();
    }
    {
        eprintln!("Scenario: parameters::list_refuses_value");
        parameters::list_refuses_value();
    }
    {
        eprintln!("Scenario: parameters::list_choices_optionality");
        parameters::list_choices_optionality();
    }
    {
        eprintln!("Scenario: parameters::struct_declaration");
        parameters::struct_declaration();
    }
    {
        eprintln!("Scenario: parameters::same_struct_decodes");
        parameters::same_struct_decodes();
    }
    {
        eprintln!("Scenario: parameters::value_refuses_name");
        parameters::value_refuses_name();
    }
    {
        eprintln!("Scenario: parameters::list_choice_multiple");
        parameters::list_choice_multiple();
    }
    {
        eprintln!("Scenario: parameters::field_handle_references");
        parameters::field_handle_references();
    }
    {
        eprintln!("Scenario: parameters::choices_round_fields");
        parameters::choices_round_fields();
    }
    {
        eprintln!("Scenario: parameters::numeric_bounds_params");
        parameters::numeric_bounds_params();
    }
    {
        eprintln!("Scenario: parameters::macro_expansion_groups");
        parameters::macro_expansion_groups();
    }
    {
        eprintln!("Scenario: prompts::declared_prompt_declaration");
        run_scenario(prompts::declared_prompt_declaration, false, None, false);
    }
    {
        eprintln!("Scenario: prompts::answers_declared_constant");
        run_scenario(prompts::answers_declared_constant, false, None, false);
    }
    {
        eprintln!("Scenario: prompts::declaration_what_registers");
        prompts::declaration_what_registers();
    }
    {
        eprintln!("Scenario: prompts::prompt_level_round_trips");
        prompts::prompt_level_round_trips();
    }
}

#[test]
fn execution_contract() {
    {
        eprintln!("Scenario: demand::observers::native_metadata_revision");
        demand::observers::native_metadata_revision();
    }
    {
        eprintln!("Scenario: demand::instance_opens_resources");
        run_scenario(demand::instance_opens_resources, false, None, false);
    }
    {
        eprintln!("Scenario: demand::plugin_refuses_line");
        demand::plugin_refuses_line();
    }
    {
        eprintln!("Scenario: demand::listener_publishes_asked");
        run_scenario(demand::listener_publishes_asked, false, None, false);
    }
    {
        eprintln!("Scenario: demand::raw_field_keys");
        demand::raw_field_keys();
    }
    {
        eprintln!("Scenario: demand::record_demand_cancel");
        run_scenario(demand::record_demand_cancel, false, None, false);
    }
    {
        eprintln!("Scenario: observers::activation_starts_execution");
        run_scenario(observers::activation_starts_execution, false, None, false);
    }
    {
        eprintln!("Scenario: observers::template_events_criterion");
        run_scenario(observers::template_events_criterion, false, None, false);
    }
    {
        eprintln!("Scenario: observers::observer_failure_event");
        run_scenario(observers::observer_failure_event, false, None, false);
    }
    {
        eprintln!("Scenario: observers::cancellation_drops_cleanup");
        run_scenario(observers::cancellation_drops_cleanup, false, None, false);
    }
    {
        eprintln!("Scenario: resources::shared_within_between");
        run_scenario(resources::shared_within_between, false, None, false);
    }
    {
        eprintln!("Scenario: resources::execution_scope_calls");
        run_scenario(resources::execution_scope_calls, false, None, false);
    }
    {
        eprintln!("Scenario: resources::shutdown_joins_cleanup");
        run_scenario(resources::shutdown_joins_cleanup, false, None, false);
    }
    {
        eprintln!("Scenario: resources::unfinished_initialize_handles");
        run_scenario(resources::unfinished_initialize_handles, false, None, false);
    }
    {
        eprintln!("Scenario: resources::harness_finalizes");
        run_scenario(resources::harness_finalizes, false, None, false);
    }
    {
        eprintln!("Scenario: resources::harness_files_directory");
        run_scenario(resources::harness_files_directory, false, None, false);
    }
    {
        eprintln!("Scenario: resources::harness_timeout_up");
        run_scenario(resources::harness_timeout_up, false, None, false);
    }
    {
        eprintln!("Scenario: resources::harness_refuses_opening");
        run_scenario(resources::harness_refuses_opening, false, None, false);
    }
    {
        eprintln!("Scenario: resources::harness_preserves_failure");
        run_scenario(resources::harness_preserves_failure, false, None, false);
    }
    {
        eprintln!("Scenario: resources::harness_caller_settles");
        run_scenario(resources::harness_caller_settles, false, None, false);
    }
    {
        eprintln!("Scenario: resources::catalog_is_inert");
        resources::catalog_is_inert();
    }
}

#[test]
fn metadata_contract() {
    {
        eprintln!("Scenario: analysis::named_analysis_params");
        analysis::named_analysis_params();
    }
    {
        eprintln!("Scenario: analysis::analysis_original_lifecycle");
        run_scenario(analysis::analysis_original_lifecycle, false, None, false);
    }
    {
        eprintln!("Scenario: analysis::cancelled_analysis_shutdown");
        run_scenario(analysis::cancelled_analysis_shutdown, false, None, false);
    }
    {
        eprintln!("Scenario: analysis::analysis_refuses_them");
        analysis::analysis_refuses_them();
    }
    {
        eprintln!("Scenario: analysis::result_units_output");
        analysis::result_units_output();
    }
    {
        eprintln!("Scenario: calls::step_criterion_operation");
        run_scenario(calls::step_criterion_operation, false, None, false);
    }
    {
        eprintln!("Scenario: calls::unanswered_operation_name");
        run_scenario(calls::unanswered_operation_name, false, None, false);
    }
    {
        eprintln!("Scenario: calls::plugin_observer_operation");
        run_scenario(calls::plugin_observer_operation, false, None, false);
    }
    {
        eprintln!("Scenario: calls::catalog_operations_policy");
        run_scenario(calls::catalog_operations_policy, false, None, false);
    }
    {
        eprintln!("Scenario: documents::step_adds_declared");
        run_scenario(documents::step_adds_declared, false, None, false);
    }
    {
        eprintln!("Scenario: documents::observer_shares_write");
        run_scenario(documents::observer_shares_write, false, None, false);
    }
    {
        eprintln!("Scenario: documents::declared_section_so");
        run_scenario(documents::declared_section_so, false, None, false);
    }
    {
        eprintln!("Scenario: documents::typed_columns_compare");
        run_scenario(documents::typed_columns_compare, false, None, false);
    }
    {
        eprintln!("Scenario: logging::macros_format_context");
        run_scenario(logging::macros_format_context, false, None, false);
    }
    {
        eprintln!("Scenario: plugin_settings::section_folds_shows");
        plugin_settings::section_folds_shows();
    }
    {
        eprintln!("Scenario: plugin_settings::field_type_required");
        plugin_settings::field_type_required();
    }
    {
        eprintln!("Scenario: plugin_settings::section_links_default");
        plugin_settings::section_links_default();
    }
    {
        eprintln!("Scenario: plugin_settings::admitted_values_handed");
        plugin_settings::admitted_values_handed();
    }
    {
        eprintln!("Scenario: plugin_settings::value_outside_refused");
        plugin_settings::value_outside_refused();
    }
    {
        eprintln!("Scenario: plugin_settings::settings_links_ids");
        plugin_settings::settings_links_ids();
    }
    {
        eprintln!("Scenario: plugin_settings::saved_values_empty");
        plugin_settings::saved_values_empty();
    }
    {
        eprintln!("Scenario: sdk_error::sdk_error_boundary");
        run_scenario(sdk_error::sdk_error_boundary, false, None, false);
    }
    {
        eprintln!("Scenario: stream_admission::publication_refuses_waits");
        run_scenario(
            stream_admission::publication_refuses_waits,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: stream_admission::stream_handles_waits");
        run_scenario(stream_admission::stream_handles_waits, false, None, false);
    }
    {
        eprintln!("Scenario: stream_admission::declared_stream_publications");
        run_scenario(
            stream_admission::declared_stream_publications,
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
