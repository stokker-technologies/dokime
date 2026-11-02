/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence storage saves, reopens, recovers from a restart midway and honours cancellation through the real backends.

/* === Modules === */
pub(crate) mod engine_storage;
pub(crate) mod runtime_storage;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn storage_recovery() {
    runtime_storage::aggregate::unknown_column_exist().await;
    runtime_storage::aggregate::aggregate_width_is_enforced().await;
    runtime_storage::artifacts::artifact_close_boundary().await;
    runtime_storage::artifacts::artifact_native_batch_upload().await;
    runtime_storage::artifacts::concurrent_log_bytes().await;
    runtime_storage::operations::storage_operations_view().await;
    runtime_storage::records::managed_records_spool().await;
    runtime_storage::recovery::skipped_snapshot_recovery().await;
    runtime_storage::requests::prepared_start_refusal().await;
    runtime_storage::requests::receipt_initialize_storage().await;
    runtime_storage::requests::admitted_progress_stop().await;
    runtime_storage::requests::shutdown_settles_requests().await;
    runtime_storage::requests::queued_execution_request().await;
    engine_storage::portable_records_identity().await;
    engine_storage::cancellation::cancellation().await;
    engine_storage::nominal::nominal().await;
    engine_storage::objects::materialized_cases_reader().await;
    engine_storage::objects::suite_definitions_producer().await;
    engine_storage::objects::cycles_reopen_producers().await;
    engine_storage::objects::executions_reopen_results().await;
    engine_storage::reports::report_publication_reopens().await;
    engine_storage::sqlite::refuses_previous_data().await;
    engine_storage::sqlite::refuses_unversioned_schema().await;
    engine_storage::sqlite::preserves_unrelated_tables().await;
    engine_storage::sqlite::refuses_attachment_receipts().await;
    engine_storage::sqlite::refuses_second_drop().await;
    engine_storage::sqlite::sqlite_rolls_reopen().await;
    engine_storage::sqlite::cancelled_caller_progress().await;
    engine_storage::sqlite::close_drains_clones().await;
    engine_storage::sqlite::catalog::source_order_restart().await;
    engine_storage::sqlite::contract::paged_records_ascend().await;
    engine_storage::sqlite::contract::retries_are_exact().await;
    engine_storage::sqlite::contract::batch_rollback_retries().await;
    engine_storage::sqlite::contract::records_share_item_with_exact_retries().await;
    engine_storage::sqlite::contract::durable_reopen().await;
    engine_storage::sqlite::contract::bootstrap_identity_survives_before_admitted_open().await;
    engine_storage::sqlite::contract::bootstrap_defers_saved_sequence_validation_to_admitted_open().await;
    engine_storage::sqlite::contract::bootstrap_rejects_invalid_history_without_initializing_data().await;
    engine_storage::sqlite::contract::bootstrap_refuses_missing_head_with_saved_entries().await;
    engine_storage::sqlite::contract::corrupt_entry_refused().await;
    engine_storage::sqlite::contract::snapshots_pin_cuts().await;
    engine_storage::sqlite::contract::finished_files_publish().await;
    engine_storage::sqlite::contract::file_digests_match().await;
    engine_storage::sqlite::contract::cancelled_read_isolated().await;
    engine_storage::sqlite::contract::close_releases_owner().await;
    engine_storage::sqlite::production::file_retry_checks_entire_overlap_before_append().await;
    engine_storage::sqlite::production::checkpoint_recovery_fence().await;
    engine_storage::sqlite::production::shared_sha_identity().await;
    engine_storage::sqlite::production::mutation_receipts_outcomes().await;
    engine_storage::sqlite::production::mutation_receipts_admit_large_values_and_preserve_atomicity().await;
    engine_storage::sqlite::production::checkpoints_advance_boundaries().await;
    engine_storage::sqlite::production::late_checkpoints_settle().await;
}
