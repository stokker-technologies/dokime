/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod activation;
pub(crate) mod dispatch;
pub(crate) mod leases;
pub(crate) mod reload;
pub(crate) mod store;
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
fn activation_contract() {
    {
        eprintln!("Scenario: activation::broken_installed_skipped");
        run_scenario(activation::broken_installed_skipped, false, None, false);
    }
    {
        eprintln!("Scenario: activation::broken_configured_loud");
        run_scenario(activation::broken_configured_loud, false, None, false);
    }
    {
        eprintln!("Scenario: activation::multi_executable_partial");
        run_scenario(activation::multi_executable_partial, false, None, false);
    }
    {
        eprintln!("Scenario: activation::mismatched_unknown_initialize");
        run_scenario(
            activation::mismatched_unknown_initialize,
            false,
            None,
            false,
        );
    }
    {
        eprintln!("Scenario: activation::independently_built_import");
        run_scenario(activation::independently_built_import, false, None, false);
    }
    {
        eprintln!("Scenario: activation::queued_plugin_until");
        run_scenario(activation::queued_plugin_until, false, None, false);
    }
    {
        eprintln!("Scenario: activation::fixture_catalog_repeatable");
        activation::fixture_catalog_repeatable();
    }
    {
        eprintln!("Scenario: dispatch::external_bundle_live");
        run_scenario(dispatch::external_bundle_live, false, None, false);
    }
}

#[test]
fn execution_contract() {
    {
        eprintln!("Scenario: leases::failed_group_holds");
        run_scenario(leases::failed_group_holds, true, None, false);
    }
    {
        eprintln!("Scenario: leases::two_executions_rows");
        run_scenario(leases::two_executions_rows, true, None, false);
    }
    {
        eprintln!("Scenario: leases::two_holds_reading");
        run_scenario(leases::two_holds_reading, true, None, false);
    }
    {
        eprintln!("Scenario: leases::foreign_stream_refuses");
        run_scenario(leases::foreign_stream_refuses, true, None, false);
    }
    {
        eprintln!("Scenario: leases::template_record_on");
        run_scenario(leases::template_record_on, true, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: reload::incompatible_stream_stopping");
        run_scenario(reload::incompatible_stream_stopping, false, None, false);
    }
    {
        eprintln!("Scenario: reload::activation_failure_recovery");
        run_scenario(reload::activation_failure_recovery, false, None, false);
    }
    {
        eprintln!("Scenario: reload::queued_reload_run");
        run_scenario(reload::queued_reload_run, false, None, false);
    }
    {
        eprintln!("Scenario: reload::provider_reload_dependent");
        run_scenario(reload::provider_reload_dependent, false, None, false);
    }
}

#[test]
fn distribution_contract() {
    {
        eprintln!("Scenario: store::explicit_bundle_installation");
        run_scenario(store::explicit_bundle_installation, false, None, false);
    }
    {
        eprintln!("Scenario: store::inspect_stage_bundle");
        store::inspect_stage_bundle();
    }
    {
        eprintln!("Scenario: store::immutable_executable_survive");
        store::immutable_executable_survive();
    }
    {
        eprintln!("Scenario: store::consent_rechecks_host");
        store::consent_rechecks_host();
    }
    {
        eprintln!("Scenario: store::unavailable_bytes_metadata");
        store::unavailable_bytes_metadata();
    }
    {
        eprintln!("Scenario: store::unauthorized_files_kept");
        store::unauthorized_files_kept();
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: store::failed_settings_bytes");
        store::failed_settings_bytes();
    }
    {
        eprintln!("Scenario: store::failed_cleanup_retry");
        store::failed_cleanup_retry();
    }
    {
        eprintln!("Scenario: store::removal_pending_sweep");
        store::removal_pending_sweep();
    }
    {
        eprintln!("Scenario: store::discarding_unconsented_failed");
        store::discarding_unconsented_failed();
    }
    {
        eprintln!("Scenario: store::disabling_retains_next");
        store::disabling_retains_next();
    }
    {
        eprintln!("Scenario: store::bounded_pages_unknown");
        store::bounded_pages_unknown();
    }
    {
        eprintln!("Scenario: store::sweep_expires_cleanup");
        store::sweep_expires_cleanup();
    }
    {
        eprintln!("Scenario: store::startup_reclaims_staging");
        run_scenario(store::startup_reclaims_staging, false, None, false);
    }
    {
        eprintln!("Scenario: store::unchanged_installed_mismatch");
        run_scenario(store::unchanged_installed_mismatch, false, None, false);
    }
    {
        eprintln!("Scenario: store::installed_program_upgrades");
        run_scenario(store::installed_program_upgrades, false, None, false);
    }
    {
        eprintln!("Scenario: store::app_roots_independent");
        store::app_roots_independent();
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
