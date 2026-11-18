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

pub mod device;
pub mod operations;
pub mod prompts;
pub mod records;
pub mod settings;
pub mod streams;
pub mod suites;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::plugin::{HostOperation, Plugin};
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

/// Compose the Plugin-Level Catalogs and the Execution-Owned Templates.
pub fn plugin() -> Result<Plugin> {
    let mut plugin = Plugin::new(IDENTIFIER, env!("CARGO_PKG_VERSION"));
    for suite in suites::catalog() {
        plugin = plugin.suite(suite);
    }
    let plugin = operations::catalog(plugin)?;
    let plugin = streams::catalog(plugin);
    let mut plugin = plugin;
    for setting in settings::catalog() {
        plugin = plugin.setting(setting);
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
    Ok(plugin)
}

/// The Frozen Catalog the Host Admits, Without Opening a Socket.
pub fn declarations() -> Result<Declarations> {
    plugin()?.declarations()
}

/* === Private Functions === */

/* === Tests === */
