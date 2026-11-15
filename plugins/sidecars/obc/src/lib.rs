/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Onboard Charger Validation on a Simulated Testbench

/* === Modules === */

pub mod analyses;
pub mod catalog;
pub mod device;
pub mod evidence;
pub mod measurement;
pub mod operations;
pub mod prompts;
pub mod records;
pub mod settings;
pub mod suites;
pub mod testbench;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime::plugin::{DeclarationTypes, HostOperation, Plugin};
use dokime::{Declarations, Result};

/* === Constants === */

pub const IDENTIFIER: &str = "obc";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Compose the Complete Plugin: Suites, Records, Prompts, Settings, Analysis and Manual Device.
pub fn plugin() -> Result<Plugin<impl DeclarationTypes>> {
    let manual = Arc::new(operations::Manual::new()?);
    let [read, set_power] = manual.operations();
    let mut plugin = Plugin::new(IDENTIFIER, env!("CARGO_PKG_VERSION"))
        .suite(suites::checks::suite())
        .suite(suites::startup::suite())
        .suite(suites::steady::suite())
        .suite(suites::protection::suite())
        .suite(suites::recovery::suite())
        .suite(suites::bench::suite())
        .analysis(analyses::ripple(), analyses::compute_ripple)
        .operation(read)
        .operation(set_power)
        .on_stop(move || manual.power_down());
    for setting in settings::declarations() {
        plugin = plugin.setting(setting);
    }
    for prompt in prompts::definitions() {
        plugin = plugin.prompt(prompt);
    }
    for operation in [
        HostOperation::PromptsCreate,
        HostOperation::EvidenceRecordsAppend,
        HostOperation::EvidenceRecordsList,
        HostOperation::EvidenceStatistics,
        HostOperation::EvidenceLogsAppend,
        HostOperation::EvidenceTraceabilityGet,
        HostOperation::EvidenceTraceabilitySet,
        HostOperation::ExecutionsProgressGet,
        HostOperation::ExecutionsProgressSet,
        HostOperation::FilesUploadsCreate,
        HostOperation::FilesUploadsAppend,
        HostOperation::FilesUploadsPublish,
        HostOperation::ContentArtifactsCreate,
        HostOperation::PluginsSettingsList,
        HostOperation::TimeGet,
        HostOperation::TimeSleep,
    ] {
        plugin = plugin.requires_host(operation);
    }
    let (pulse_started, pulse_completed) = suites::checks::contracts();
    let (startup_initialized, startup_started, startup_checkpoint, startup_completed) = suites::startup::contracts();
    let (steady_initialized, steady_started, steady_hold, steady_completed) = suites::steady::contracts();
    let (protection_initialized, protection_started, protection_faulted, protection_stopped) = suites::protection::contracts();
    let (recovery_initialized, recovery_started, recovery_faulted, recovery_cleared, recovery_stopped) = suites::recovery::contracts();
    Ok(plugin
        .record(pulse_started)
        .record(pulse_completed)
        .record(startup_initialized)
        .record(startup_started)
        .record(startup_checkpoint)
        .record(startup_completed)
        .record(steady_initialized)
        .record(steady_started)
        .record(steady_hold)
        .record(steady_completed)
        .record(protection_initialized)
        .record(protection_started)
        .record(protection_faulted)
        .record(protection_stopped)
        .record(recovery_initialized)
        .record(recovery_started)
        .record(recovery_faulted)
        .record(recovery_cleared)
        .record(recovery_stopped)
        .record(suites::bench::contracts()))
}

/// The Frozen Catalog the Host Admits, Without Opening a Socket.
pub fn declarations() -> Result<Declarations> {
    plugin()?.declarations()
}

/* === Private Functions === */

/* === Tests === */
