/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod steps;
pub(crate) mod workflow;

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
    builder.start_paused(paused);
    builder.build().expect("scenario runtime")
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: workflow::interactive_commands_downstream");
        run_scenario(
            workflow::interactive_commands_downstream,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: workflow::interactive_idle_stop");
        run_scenario(workflow::interactive_idle_stop, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::unacknowledged_invoke_callback");
        run_scenario(workflow::unacknowledged_invoke_callback, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::embedded_execution_output");
        run_scenario(workflow::embedded_execution_output, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::refused_publication_fails_execution");
        run_scenario(
            workflow::refused_publication_fails_execution,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: workflow::runtime_bindings_metadata");
        run_scenario(workflow::runtime_bindings_metadata, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::stated_section_step");
        run_scenario(workflow::stated_section_step, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::traceability_failure_up");
        run_scenario(workflow::traceability_failure_up, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::failing_step_backtrace");
        run_scenario(workflow::failing_step_backtrace, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::lifecycle_logs_failure");
        run_scenario(workflow::lifecycle_logs_failure, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::panicking_step_diagnostic");
        run_scenario(workflow::panicking_step_diagnostic, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::context_scoped_stable");
        run_scenario(workflow::context_scoped_stable, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::declared_stream_decides");
        run_scenario(workflow::declared_stream_decides, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::streamed_artifact_chunks");
        run_scenario(workflow::streamed_artifact_chunks, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::watched_directory_in");
        run_scenario(workflow::watched_directory_in, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::one_directory_execution");
        run_scenario(workflow::one_directory_execution, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::token_reused_refused");
        run_scenario(workflow::token_reused_refused, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::unknown_workflow_starts");
        run_scenario(workflow::unknown_workflow_starts, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::cancelled_run_produced");
        run_scenario(workflow::cancelled_run_produced, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::criterion_could_failed");
        run_scenario(workflow::criterion_could_failed, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::unanswered_question_id");
        run_scenario(workflow::unanswered_question_id, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::declined_answer_asked");
        run_scenario(workflow::declined_answer_asked, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::cancelling_waiting_execution");
        run_scenario(workflow::cancelling_waiting_execution, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::phase_drain_reached");
        run_scenario(workflow::phase_drain_reached, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::cleanup_runs_pass");
        run_scenario(workflow::cleanup_runs_pass, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::initialize_failure_execution");
        run_scenario(workflow::initialize_failure_execution, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::initialize_cancellation_cleanup");
        run_scenario(
            workflow::initialize_cancellation_cleanup,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: workflow::cleanup_procedure_failure");
        run_scenario(workflow::cleanup_procedure_failure, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::cleanup_waits_timeout");
        run_scenario(workflow::cleanup_waits_timeout, false, None, true);
    }
    {
        eprintln!("Scenario: workflow::pending_sampler_cleanup");
        run_scenario(workflow::pending_sampler_cleanup, false, None, true);
    }
    {
        eprintln!("Scenario: workflow::stalled_criterion_indeterminate");
        run_scenario(workflow::stalled_criterion_indeterminate, false, None, true);
    }
    {
        eprintln!("Scenario: workflow::resources_release_verdict");
        run_scenario(workflow::resources_release_verdict, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::diagnostics_preserve_success");
        run_scenario(workflow::diagnostics_preserve_success, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::record_wait_publication");
        run_scenario(workflow::record_wait_publication, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::record_nobody_error");
        run_scenario(workflow::record_nobody_error, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::capture_declared_scope");
        run_scenario(workflow::capture_declared_scope, false, None, false);
    }
    {
        eprintln!("Scenario: workflow::signal_refuses_deliveries");
        workflow::signal_refuses_deliveries();
    }
    {
        eprintln!("Scenario: workflow::artifact_backpressure");
        run_scenario(workflow::artifact_backpressure, true, Some(2), false);
    }
    {
        eprintln!("Scenario: workflow::callbacks::shutdown_joins_owners");
        run_scenario(
            workflow::callbacks::shutdown_joins_owners,
            false,
            None,
            true,
        );
    }
    {
        eprintln!("Scenario: workflow::callbacks::interrupted_join_teardown");
        run_scenario(
            workflow::callbacks::interrupted_join_teardown,
            false,
            None,
            false,
        );
    }
}

#[test]
fn step_builtins_contract() {
    #[cfg(unix)]
    {
        eprintln!("Scenario: steps::process_output_artifact");
        run_scenario(steps::process_output_artifact, true, Some(2), false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: steps::process_cancel_exit");
        run_scenario(steps::process_cancel_exit, true, Some(2), false);
    }
    {
        eprintln!("Scenario: steps::swept_watch_warning");
        run_scenario(steps::swept_watch_warning, true, Some(2), false);
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
