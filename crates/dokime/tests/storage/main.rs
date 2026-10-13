/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg(feature = "storage")]

/* === Modules === */
pub(crate) mod objects;
pub(crate) mod reports;
#[path = "sqlite/main.rs"]
pub(crate) mod sqlite;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::storage::{
    ReadKind, ReadLimits, ReadSelection, ReadView, StorageBackend, StorageBatch, StorageCursor,
    StoragePart, StorageSchema, StoredRecord,
};

// Local Imports
use dokime::{
    core::{Identifier, RecordId, RecordScope, SessionId, Timestamp, Value},
    records::{RecordEntry, RecordPublishRequest, RecordSeverity},
    storage::{records, sqlite::Backend},
};

/* === Constants === */
const FRAME_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn entry(sequence: u64, volts: f64) -> RecordEntry {
    let mut record = RecordPublishRequest::new(
        Identifier::new("charger.measurement").unwrap(),
        RecordSeverity::Info,
    )
    .into_record(
        RecordId::from_seed(sequence.to_string()),
        1,
        Timestamp::new("2026-10-04T12:00:00Z"),
        "bench.charger",
    );
    record.scope = Some(RecordScope::session(SessionId::from_seed("bench")));
    record.details = BTreeMap::from([
        ("voltage".into(), Value::Number(volts)),
        ("current".into(), Value::Number(3.25)),
        ("ready".into(), Value::Bool(true)),
        (
            "limits".into(),
            Value::Array(vec![Value::Integer(-2), Value::Unsigned(u64::MAX)]),
        ),
    ]);
    RecordEntry::from_record(sequence, record)
}

/* === Tests === */

pub(crate) async fn portable_records_identity() {
    let root = tempfile::tempdir().unwrap();
    let writer = Backend::<()>::primary(root.path().to_path_buf(), Default::default())
        .await
        .unwrap();
    let history = writer.history().clone();
    let position = |sequence| StorageCursor {
        history: history.clone(),
        sequence,
    };
    let schema = records::schema();
    writer
        .write(
            StorageBatch::new(
                &[StoragePart::Schemas(&[StorageSchema::new(
                    position(1),
                    schema.clone(),
                    sqlite::storage::declared::<records::RecordRow>(),
                )])],
                (16 * 1024 * 1024).try_into().unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let expected = [entry(17, 229.5), entry(29, 230.5)];
    let stored = expected
        .iter()
        .enumerate()
        .map(|(index, entry)| StoredRecord {
            position: position(index as u64 + 2),
            schema: schema.clone(),
            scope: Some(Identifier::new("bench-epoch").unwrap()),
            item: Identifier::new(entry.record_id.to_string()).unwrap(),
            source: None,
            source_time: None,
            payload: records::encode(entry, FRAME_BYTES).unwrap(),
        })
        .collect::<Vec<_>>();
    writer
        .write(
            StorageBatch::new(
                &[StoragePart::Records(&stored)],
                (16 * 1024 * 1024).try_into().unwrap(),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    let saved = writer.cursor_saved().await.unwrap();
    writer.close().await.unwrap();

    let reader = Backend::<()>::primary(root.path().to_path_buf(), Default::default())
        .await
        .unwrap();
    assert_eq!(reader.history(), &history);
    let snapshot = reader.read_view(&saved).await.unwrap();
    let view = ReadView::new(
        Identifier::new("portable-reader").unwrap(),
        saved.clone(),
        saved,
        snapshot,
    )
    .unwrap();
    let limits = ReadLimits {
        rows: 1.try_into().unwrap(),
        bytes: FRAME_BYTES.try_into().unwrap(),
    };
    let mut selection = ReadSelection::new(ReadKind::Records);
    selection.schema = Some(schema);
    let mut cursor = None;
    let mut actual = Vec::new();
    for stored in &stored {
        let page = reader
            .read_records(&view, &selection, cursor.as_ref(), limits)
            .await
            .unwrap();
        assert_eq!(page.entries.as_slice(), std::slice::from_ref(stored));
        actual.push(records::decode(&page.entries[0].payload, FRAME_BYTES).unwrap());
        cursor = page.next.clone();
    }
    assert!(cursor.is_none());
    assert_eq!(actual, expected);
    reader.release_view(view.snapshot()).await.unwrap();
    reader.close().await.unwrap();
}

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
fn recovery_contract() {
    {
        eprintln!("Scenario: portable_records_identity");
        run_scenario(portable_records_identity, false, None, false);
    }
    {
        eprintln!("Scenario: objects::materialized_cases_reader");
        run_scenario(objects::materialized_cases_reader, false, None, false);
    }
    {
        eprintln!("Scenario: objects::suite_definitions_producer");
        run_scenario(objects::suite_definitions_producer, false, None, false);
    }
    {
        eprintln!("Scenario: objects::cycles_reopen_producers");
        run_scenario(objects::cycles_reopen_producers, false, None, false);
    }
    {
        eprintln!("Scenario: objects::executions_reopen_results");
        run_scenario(objects::executions_reopen_results, false, None, false);
    }
    {
        eprintln!("Scenario: objects::prompt_level_rows");
        objects::prompt_level_rows();
    }
    {
        eprintln!("Scenario: reports::report_publication_reopens");
        run_scenario(reports::report_publication_reopens, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_file_bytes");
        run_scenario(sqlite::bootstrap::private_file_bytes, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_file_restart");
        run_scenario(sqlite::bootstrap::private_file_restart, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_import_ownership");
        run_scenario(
            sqlite::bootstrap::private_import_ownership,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_import_abort");
        run_scenario(sqlite::bootstrap::private_import_abort, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::failed_private_fence");
        run_scenario(sqlite::bootstrap::failed_private_fence, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_rows_atomically");
        run_scenario(
            sqlite::bootstrap::private_rows_atomically,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_row_unchanged");
        run_scenario(sqlite::bootstrap::private_row_unchanged, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_manifest_schema");
        run_scenario(
            sqlite::bootstrap::private_manifest_schema,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::bootstrap::private_import_floor");
        run_scenario(sqlite::bootstrap::private_import_floor, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::catalog::source_order_restart");
        run_scenario(sqlite::catalog::source_order_restart, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::encoding::saved_descriptors_reopen");
        run_scenario(
            sqlite::encoding::saved_descriptors_reopen,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::files::finished_files_reopen");
        run_scenario(sqlite::files::finished_files_reopen, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::files::sealed_files_publication");
        run_scenario(sqlite::files::sealed_files_publication, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::native_frames_result");
        run_scenario(sqlite::frames::native_frames_result, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::missing_null_distinct");
        run_scenario(sqlite::frames::missing_null_distinct, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::frame_refusals_view");
        run_scenario(sqlite::frames::frame_refusals_view, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::cached_batch_detects");
        run_scenario(sqlite::frames::cached_batch_detects, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::native_correlations_order");
        run_scenario(
            sqlite::frames::native_correlations_order,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::frames::dropped_queued_view");
        run_scenario(sqlite::frames::dropped_queued_view, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::filtered_record_pages_cut");
        run_scenario(
            sqlite::frames::filtered_record_pages_cut,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::frames::postcard_frames_cut");
        run_scenario(sqlite::frames::postcard_frames_cut, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::keyed_scans_writes");
        run_scenario(sqlite::frames::keyed_scans_writes, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::frames::frame_in_budget");
        run_scenario(sqlite::frames::frame_in_budget, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::integrity::physical_pages_boundaries");
        run_scenario(
            sqlite::integrity::physical_pages_boundaries,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::integrity::native_pages_payloads");
        run_scenario(sqlite::integrity::native_pages_payloads, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::integrity::native_owner_positions");
        run_scenario(
            sqlite::integrity::native_owner_positions,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::integrity::physical_page_positions");
        run_scenario(
            sqlite::integrity::physical_page_positions,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::integrity::native_page_position");
        run_scenario(sqlite::integrity::native_page_position, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::integrity::native_records_boundaries");
        run_scenario(
            sqlite::integrity::native_records_boundaries,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::refuses_previous_data");
        run_scenario(sqlite::refuses_previous_data, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_unversioned_schema");
        run_scenario(sqlite::refuses_unversioned_schema, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_retired_intent");
        run_scenario(sqlite::refuses_retired_intent, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_attachment_receipts");
        run_scenario(sqlite::refuses_attachment_receipts, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::refuses_second_drop");
        run_scenario(sqlite::refuses_second_drop, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::sqlite_rolls_reopen");
        run_scenario(sqlite::sqlite_rolls_reopen, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::cancelled_caller_progress");
        run_scenario(sqlite::cancelled_caller_progress, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::close_drains_clones");
        run_scenario(sqlite::close_drains_clones, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::managed::compacted_snapshots_receipts");
        run_scenario(
            sqlite::managed::compacted_snapshots_receipts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::managed::primary_history_close");
        run_scenario(sqlite::managed::primary_history_close, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::managed::cancelled_backend_instance");
        sqlite::managed::cancelled_backend_instance();
    }
    {
        eprintln!("Scenario: sqlite::managed::manager_queries_evidence");
        run_scenario(
            sqlite::managed::manager_queries_evidence,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::managed::managed_journal_backends");
        run_scenario(
            sqlite::managed::managed_journal_backends,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::managed::captured_records_duplicates");
        run_scenario(
            sqlite::managed::captured_records_duplicates,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::managed::cancelled_boot_storage");
        run_scenario(sqlite::managed::cancelled_boot_storage, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::managed::oversized_admission_order");
        run_scenario(
            sqlite::managed::oversized_admission_order,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::managed::follower_bridges_admissions");
        run_scenario(
            sqlite::managed::follower_bridges_admissions,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::managed::replay_tail_saving");
        run_scenario(sqlite::managed::replay_tail_saving, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::managed::managed_producer_restart");
        run_scenario(
            sqlite::managed::managed_producer_restart,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::migration::refuses_retired_evidence");
        run_scenario(
            sqlite::migration::refuses_retired_evidence,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::migration::refuses_separate_database");
        run_scenario(
            sqlite::migration::refuses_separate_database,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::objects::scalar_failure_cursor");
        run_scenario(sqlite::objects::scalar_failure_cursor, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::objects::query_page_drop");
        run_scenario(sqlite::objects::query_page_drop, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::objects::stream_frames_charge");
        run_scenario(sqlite::objects::stream_frames_charge, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::objects::file_query_offset");
        run_scenario(sqlite::objects::file_query_offset, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::objects::pinned_page_waiting");
        run_scenario(sqlite::objects::pinned_page_waiting, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::objects::private_object_version");
        run_scenario(sqlite::objects::private_object_version, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::production::checkpoint_recovery_fence");
        run_scenario(
            sqlite::production::checkpoint_recovery_fence,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::production::shared_sha_identity");
        run_scenario(sqlite::production::shared_sha_identity, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::production::mutation_receipts_outcomes");
        run_scenario(
            sqlite::production::mutation_receipts_outcomes,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::production::checkpoints_advance_boundaries");
        run_scenario(
            sqlite::production::checkpoints_advance_boundaries,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::production::recovery_charges_limits");
        run_scenario(
            sqlite::production::recovery_charges_limits,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::recovery::primary_close_until");
        run_scenario(sqlite::recovery::primary_close_until, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::recovery::snapshot_tail_frontier");
        run_scenario(sqlite::recovery::snapshot_tail_frontier, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::revisions::retention::pruning_pins_coverage");
        run_scenario(
            sqlite::revisions::retention::pruning_pins_coverage,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::retention::bounded_retention_prefix");
        run_scenario(
            sqlite::revisions::retention::bounded_retention_prefix,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::retention::pruning_records_loss");
        run_scenario(
            sqlite::revisions::retention::pruning_records_loss,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::retention::cancelled_pruning_writes");
        run_scenario(
            sqlite::revisions::retention::cancelled_pruning_writes,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::retention::released_pin_page");
        run_scenario(
            sqlite::revisions::retention::released_pin_page,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::retention::new_versions_remain");
        run_scenario(
            sqlite::revisions::retention::new_versions_remain,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::object_revisions_latest");
        run_scenario(
            sqlite::revisions::object_revisions_latest,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::object_batches_identity");
        run_scenario(
            sqlite::revisions::object_batches_identity,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::revisions::manager_replicates_collections");
        run_scenario(
            sqlite::revisions::manager_replicates_collections,
            false,
            None,
            false,
        );
    }
}

#[test]
fn rows_contract() {
    {
        eprintln!("Scenario: sqlite::batch::native_batch_unchanged");
        run_scenario(sqlite::batch::native_batch_unchanged, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::batch_conflicts_prefix");
        run_scenario(sqlite::batch::batch_conflicts_prefix, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::cancelled_batch_owned");
        run_scenario(sqlite::batch::cancelled_batch_owned, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::grouped_pages_limits");
        run_scenario(sqlite::batch::grouped_pages_limits, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::saved_page_damage");
        run_scenario(sqlite::batch::saved_page_damage, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::failed_batch_retry");
        run_scenario(sqlite::batch::failed_batch_retry, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::live_owner_restored");
        run_scenario(sqlite::batch::live_owner_restored, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::live_owner_publication");
        run_scenario(sqlite::batch::live_owner_publication, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::large_native_limits");
        run_scenario(sqlite::batch::large_native_limits, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::batch::rolled_back_schema");
        run_scenario(sqlite::batch::rolled_back_schema, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::native_views_owner");
        run_scenario(sqlite::data::native_views_owner, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::view_release_reader");
        run_scenario(sqlite::data::view_release_reader, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::exact_retry_content");
        run_scenario(sqlite::data::exact_retry_content, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::conflicting_retry_prefix");
        run_scenario(sqlite::data::conflicting_retry_prefix, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::invalid_file_identity");
        run_scenario(sqlite::data::invalid_file_identity, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::file_publication_corruption");
        run_scenario(
            sqlite::data::file_publication_corruption,
            false,
            None,
            false,
        );
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: sqlite::data::file_writes_opening");
        run_scenario(sqlite::data::file_writes_opening, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::finished_files_frontier");
        run_scenario(sqlite::data::finished_files_frontier, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::cancelled_waiter_settlement");
        run_scenario(
            sqlite::data::cancelled_waiter_settlement,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::exhausted_read_retry");
        run_scenario(sqlite::data::exhausted_read_retry, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::typed_page_view");
        run_scenario(sqlite::data::typed_page_view, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::page_lookahead_blobs");
        run_scenario(sqlite::data::page_lookahead_blobs, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::durable_prefix_rollback");
        run_scenario(
            sqlite::data::recovery::durable_prefix_rollback,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::retries_conflicts_explicit");
        run_scenario(
            sqlite::data::recovery::retries_conflicts_explicit,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::snapshot_transaction_tail");
        run_scenario(
            sqlite::data::recovery::snapshot_transaction_tail,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::saved_corruption_rows");
        run_scenario(
            sqlite::data::recovery::saved_corruption_rows,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::primary_recovery_owner");
        run_scenario(
            sqlite::data::recovery::primary_recovery_owner,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::input_snapshot_bounded");
        run_scenario(
            sqlite::data::recovery::input_snapshot_bounded,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::failed_snapshot_retry");
        run_scenario(
            sqlite::data::recovery::failed_snapshot_retry,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::snapshot_requires_frontier");
        run_scenario(
            sqlite::data::recovery::snapshot_requires_frontier,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::data::recovery::cancelled_waiter_owned");
        run_scenario(
            sqlite::data::recovery::cancelled_waiter_owned,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::reads::saved_views_payloads");
        run_scenario(sqlite::reads::saved_views_payloads, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::stream_rows_views");
        run_scenario(sqlite::reads::stream_rows_views, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::cancelled_waiter_owned");
        run_scenario(sqlite::reads::cancelled_waiter_owned, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::missing_saved_evidence");
        run_scenario(sqlite::reads::missing_saved_evidence, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::saved_snapshot_checks");
        run_scenario(sqlite::reads::saved_snapshot_checks, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::sparse_pages_statistics");
        run_scenario(sqlite::reads::sparse_pages_statistics, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::small_rows_orders");
        run_scenario(sqlite::reads::small_rows_orders, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::native_query_capacity");
        run_scenario(sqlite::reads::native_query_capacity, false, None, false);
    }
    {
        eprintln!("Scenario: sqlite::reads::restart_committed_conflicts");
        run_scenario(
            sqlite::reads::restart_committed_conflicts,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: sqlite::reads::failed_native_retry");
        run_scenario(sqlite::reads::failed_native_retry, false, None, false);
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
