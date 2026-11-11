/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Planning, append, prompts, records, notes, evidence, traceability and history of an execution over the served surface.

/* === Modules === */

mod append;
mod evidence;
mod history;
mod notes;
mod planning;
mod prompts;
mod records;
mod traceability;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn run<F: std::future::Future<Output = ()>>(make_future: impl FnOnce() -> F + Send + 'static) {
    // Construct And Drive Each Debug-Build Future On The Story's Original Large Stack.
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(make_future()))
        .unwrap()
        .join()
        .unwrap();
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn planning() {
    run(|| async {
        stage(Box::pin(planning::ordered_selections_provider())).await;
        stage(Box::pin(planning::planning_refuses_inputs())).await;
        stage(Box::pin(planning::template_conflicts_disambiguate())).await;
        stage(Box::pin(planning::effective_native_materialization())).await;
        stage(Box::pin(planning::payload_budget_allocation())).await;
        stage(Box::pin(planning::materialization_rejects_nonfinite_parameters_before_registration())).await;
        stage(Box::pin(planning::registered_cases_selection())).await;
    });
}

#[dokime_tracing::trace_test]
#[test]
fn append() {
    run(|| async {
        stage(Box::pin(append::append_cases_preserve_ordinals())).await;
        // Independent Roots Keep Every Budget's Complete Append And Sealing Checks.
        tokio::join!(
            stage(Box::pin(append::explicit_append_retries())),
            stage(Box::pin(append::explicit_append_small_write_budget())),
            stage(Box::pin(append::explicit_append_small_write_budget_four_workers())),
            stage(Box::pin(append::explicit_append_small_write_budget_single_worker())),
        );
    });
}

#[dokime_tracing::trace_test]
#[test]
fn prompts() {
    run(|| async {
        stage(Box::pin(prompts::execution_prompt_identity())).await;
        stage(Box::pin(prompts::prompt_answers_identity())).await;
        stage(Box::pin(prompts::prompt_condition_commit())).await;
        stage(Box::pin(prompts::prompt_level_watched())).await;
        stage(Box::pin(prompts::settled_prompt_refused())).await;
    });
}

#[dokime_tracing::trace_test]
#[test]
fn records() {
    run(|| async {
        stage(Box::pin(records::step_awaits_record())).await;
        stage(Box::pin(records::record_wait_expires())).await;
    });
}

#[dokime_tracing::trace_test]
#[test]
fn notes() {
    run(|| async {
        stage(Box::pin(notes::scoped_annotation_workflow())).await;
        stage(Box::pin(notes::sealed_refuses_notes())).await;
    });
}

#[dokime_tracing::trace_test]
#[test]
fn evidence() {
    run(|| async {
        stage(Box::pin(evidence::fixed_pages_gap())).await;
        stage(Box::pin(evidence::tail_then_follow())).await;
        stage(Box::pin(evidence::filtered_scans_resync())).await;
        stage(Box::pin(evidence::capture_cursors_restarts())).await;
        stage(Box::pin(evidence::oversized_delivery_evidence())).await;
    });
}

#[dokime_tracing::trace_test]
#[test]
fn traceability() {
    run(|| async {
        stage(Box::pin(traceability::get_traceability_cursor())).await;
        stage(Box::pin(traceability::traceability_bounds_clone())).await;
    });
}

#[dokime_tracing::trace_test]
#[test]
fn history() {
    run(|| async {
        stage(Box::pin(history::deletion_validates_unsupported())).await;
        stage(Box::pin(history::empty_cycles_retain_history())).await;
    });
}
