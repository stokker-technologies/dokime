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

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn execution_seal_report() {
    // The Stages' Futures Are Large In Debug Builds: Drive Them On A Thread With A Roomy Stack
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(stages()))
        .unwrap()
        .join()
        .unwrap();
}

async fn stages() {
    Box::pin(planning::ordered_selections_provider()).await;
    Box::pin(planning::planning_refuses_inputs()).await;
    Box::pin(planning::template_conflicts_disambiguate()).await;
    Box::pin(planning::effective_native_materialization()).await;
    Box::pin(planning::payload_budget_allocation()).await;
    Box::pin(planning::materialization_rejects_nonfinite_parameters_before_registration()).await;
    Box::pin(planning::registered_cases_selection()).await;
    Box::pin(append::append_cases_preserve_ordinals()).await;
    Box::pin(append::explicit_append_retries()).await;
    Box::pin(append::explicit_append_small_write_budget()).await;
    Box::pin(append::explicit_append_small_write_budget_four_workers()).await;
    Box::pin(append::explicit_append_small_write_budget_single_worker()).await;
    Box::pin(prompts::execution_prompt_identity()).await;
    Box::pin(prompts::prompt_answers_identity()).await;
    Box::pin(prompts::prompt_condition_commit()).await;
    Box::pin(prompts::prompt_level_watched()).await;
    Box::pin(prompts::settled_prompt_refused()).await;
    Box::pin(records::step_awaits_record()).await;
    Box::pin(records::record_wait_expires()).await;
    Box::pin(notes::scoped_annotation_workflow()).await;
    Box::pin(notes::sealed_refuses_notes()).await;
    Box::pin(evidence::fixed_pages_gap()).await;
    Box::pin(evidence::tail_then_follow()).await;
    Box::pin(evidence::filtered_scans_resync()).await;
    Box::pin(evidence::capture_cursors_restarts()).await;
    Box::pin(evidence::oversized_delivery_evidence()).await;
    Box::pin(traceability::get_traceability_cursor()).await;
    Box::pin(traceability::traceability_bounds_clone()).await;
    Box::pin(history::deletion_validates_unsupported()).await;
    Box::pin(history::empty_cycles_retain_history()).await;
}
