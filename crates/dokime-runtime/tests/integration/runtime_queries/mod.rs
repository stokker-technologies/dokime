/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Runtime commands, lifecycle and queries run against one runtime and read back their records.

pub(crate) mod commands;
pub(crate) mod lifecycle;
pub(crate) mod queries;
pub(crate) mod query_records;

pub(crate) use crate::service_support::prelude::*;

use dokime_domain::model;

async fn record_health(runtime: &RuntimeHandle, message: &str) -> dokime_runtime::runtime::Result<()> {
    runtime.add_health_item(RecordScope::default(), model::execution::HealthSeverity::Info, message.to_owned(), None).await
}

#[tokio::test]
async fn runtime_queries() {
    commands::start_receipt_identity().await;
    commands::accepted_provider_revision().await;
    commands::refusals_refuses_processor().await;
    commands::idempotency_survives_restart().await;
    lifecycle::node_identity_restart().await;
    #[cfg(feature = "fault-injection")]
    lifecycle::fixture_backends_decorated().await;
    lifecycle::starts_shutdown_commands().await;
    lifecycle::old_incompatible_startup().await;
    lifecycle::boot_reboots_intact().await;
    queries::orchestrator_frames_exposed().await;
    queries::live_revisions_coalesce().await;
    queries::runtime_overview_payload().await;
    queries::telemetry_reads_stream().await;
    queries::telemetry_batch_receipts_and_order().await;
    query_records::oversized_record_changes().await;
    query_records::published_record_reads().await;
    query_records::live_reader_cursor().await;
    query_records::filtered_record_view().await;
    query_records::fixed_replay_append().await;
    query_records::independent_readers_pages().await;
    query_records::slow_dropped_create().await;
    query_records::capture_failure_reader().await;
    query_records::captured_records_failure().await;
    query_records::capture_failure_provider().await;
    query_records::primary_capture_boots().await;
    query_records::completed_record_capacity().await;
    query_records::native_entry_independent().await;
}
