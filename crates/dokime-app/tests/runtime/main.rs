/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod commands;
pub(crate) mod config_application;
pub(crate) mod config_settings;
pub(crate) mod identity;
pub(crate) mod lifecycle;
pub(crate) mod queries;
pub(crate) mod query_records;
pub(crate) mod settings;
pub(crate) mod storage;
#[path = "../support/mod.rs"]
pub mod support;

/* === Re-Exports === */
pub(crate) use crate::support::prelude::*;

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
fn provider_config(root: &Path) -> RuntimeConfig {
    config(root).with_plugins([fixture_bundle(root, "mini-sidecar")])
}

async fn provider_cycle(runtime: &RuntimeHandle, suffix: &str) -> TestCycleId {
    tokio::time::timeout(Duration::from_secs(30), async {
        while runtime
            .cycles()
            .await
            .iter()
            .any(|cycle| cycle.lifecycle == CycleStatus::Running)
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("previous provider finishes");
    let registration = runtime
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "mini-sidecar" && suite.suite_id == "mini.checks")
        .expect("mini suite registered");
    start_cases(
        runtime,
        metadata(
            &format!("fixture.cycle.{suffix}"),
            &format!("Fixture Cycle {suffix}"),
        ),
        materialize_cases(&registration.suite_id, &registration.suite),
        false,
    )
    .await
}

async fn record_health(runtime: &RuntimeHandle, message: &str) -> dokime_app::Result<()> {
    runtime
        .add_health_item(
            RecordScope::default(),
            dokime::execution::HealthSeverity::Info,
            message.to_owned(),
            None,
        )
        .await
}

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
fn domain_contract() {
    {
        eprintln!("Scenario: commands::receipts_respect_idempotency");
        run_scenario(commands::receipts_respect_idempotency, false, None, false);
    }
    {
        eprintln!("Scenario: commands::idempotency_survives_restart");
        run_scenario(commands::idempotency_survives_restart, false, None, false);
    }
    {
        eprintln!("Scenario: commands::accepted_provider_revision");
        run_scenario(commands::accepted_provider_revision, false, None, false);
    }
    {
        eprintln!("Scenario: commands::refusals_refuses_processor");
        run_scenario(commands::refusals_refuses_processor, false, None, false);
    }
    {
        eprintln!("Scenario: config_application::explicit_root_local");
        config_application::explicit_root_local();
    }
    {
        eprintln!("Scenario: config_application::saved_data_overridden");
        config_application::saved_data_overridden();
    }
    {
        eprintln!("Scenario: config_application::data_root_reopens_stable");
        run_scenario(
            config_application::data_root_reopens_stable,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: config_settings::backend_policy_restart");
        run_scenario(config_settings::backend_policy_restart, false, None, false);
    }
    {
        eprintln!("Scenario: config_settings::concurrent_policy_revision");
        run_scenario(
            config_settings::concurrent_policy_revision,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: identity::concurrent_creators_identity");
        run_scenario(identity::concurrent_creators_identity, true, Some(8), false);
    }
    {
        eprintln!("Scenario: identity::existing_identity_republishing");
        run_scenario(identity::existing_identity_republishing, false, None, false);
    }
    {
        eprintln!("Scenario: identity::corruption_refuses_identity");
        run_scenario(identity::corruption_refuses_identity, false, None, false);
    }
    {
        eprintln!("Scenario: lifecycle::node_identity_restart");
        run_scenario(lifecycle::node_identity_restart, false, None, false);
    }
    {
        eprintln!("Scenario: lifecycle::starts_shutdown_commands");
        run_scenario(lifecycle::starts_shutdown_commands, false, None, false);
    }
    {
        eprintln!("Scenario: lifecycle::old_incompatible_startup");
        run_scenario(lifecycle::old_incompatible_startup, false, None, false);
    }
    {
        eprintln!("Scenario: lifecycle::boot_reboots_intact");
        run_scenario(lifecycle::boot_reboots_intact, false, None, false);
    }
    {
        eprintln!("Scenario: queries::orchestrator_frames_exposed");
        run_scenario(queries::orchestrator_frames_exposed, false, None, false);
    }
    {
        eprintln!("Scenario: queries::live_revisions_coalesce");
        run_scenario(queries::live_revisions_coalesce, false, None, false);
    }
    {
        eprintln!("Scenario: queries::runtime_overview_payload");
        run_scenario(queries::runtime_overview_payload, false, None, false);
    }
    {
        eprintln!("Scenario: queries::telemetry_reads_stream");
        run_scenario(queries::telemetry_reads_stream, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::oversized_record_changes");
        run_scenario(query_records::oversized_record_changes, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::published_record_reads");
        run_scenario(query_records::published_record_reads, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::live_reader_cursor");
        run_scenario(query_records::live_reader_cursor, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::filtered_record_view_polling");
        run_scenario(
            query_records::filtered_record_view_polling,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: query_records::fixed_replay_append");
        run_scenario(query_records::fixed_replay_append, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::fixed_record_pressure");
        run_scenario(query_records::fixed_record_pressure, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::independent_readers_pages");
        run_scenario(query_records::independent_readers_pages, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::slow_dropped_create");
        run_scenario(query_records::slow_dropped_create, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::capture_failure_reader");
        run_scenario(query_records::capture_failure_reader, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::captured_records_failure");
        run_scenario(query_records::captured_records_failure, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::capture_failure_provider");
        run_scenario(query_records::capture_failure_provider, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::primary_capture_boots");
        run_scenario(query_records::primary_capture_boots, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::completed_record_capacity");
        run_scenario(query_records::completed_record_capacity, false, None, false);
    }
    {
        eprintln!("Scenario: query_records::native_entry_independent");
        run_scenario(query_records::native_entry_independent, false, None, false);
    }
    {
        eprintln!("Scenario: settings::live_status_answered");
        run_scenario(settings::live_status_answered, false, None, false);
    }
    {
        eprintln!("Scenario: settings::dispatcher_alone_probe");
        run_scenario(settings::dispatcher_alone_probe, false, None, false);
    }
    {
        eprintln!("Scenario: settings::discovered_settings_until");
        run_scenario(settings::discovered_settings_until, false, None, false);
    }
    {
        eprintln!("Scenario: settings::resolves_one_record");
        run_scenario(settings::resolves_one_record, false, None, false);
    }
    {
        eprintln!("Scenario: settings::secret_references_file");
        run_scenario(settings::secret_references_file, false, None, false);
    }
    {
        eprintln!("Scenario: settings::local_credentials_state");
        run_scenario(settings::local_credentials_state, false, None, false);
    }
    {
        eprintln!("Scenario: settings::source_settings_absent");
        run_scenario(settings::source_settings_absent, false, None, false);
    }
    {
        eprintln!("Scenario: storage::aggregate::unknown_column_exist");
        run_scenario(storage::aggregate::unknown_column_exist, false, None, false);
    }
    {
        eprintln!("Scenario: storage::artifacts::concurrent_log_bytes");
        run_scenario(storage::artifacts::concurrent_log_bytes, false, None, false);
    }
    {
        eprintln!("Scenario: storage::operations::storage_operations_view");
        run_scenario(
            storage::operations::storage_operations_view,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: storage::records::managed_records_spool");
        run_scenario(storage::records::managed_records_spool, false, None, false);
    }
    {
        eprintln!("Scenario: storage::recovery::cycle_survives_skipped_snapshot");
        run_scenario(
            storage::recovery::cycle_survives_skipped_snapshot,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: storage::requests::receipt_initialize_storage");
        run_scenario(
            storage::requests::receipt_initialize_storage,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: storage::requests::admitted_progress_stop");
        run_scenario(
            storage::requests::admitted_progress_stop,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: storage::requests::shutdown_settles_requests");
        run_scenario(
            storage::requests::shutdown_settles_requests,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: storage::requests::queued_execution_request");
        run_scenario(
            storage::requests::queued_execution_request,
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
