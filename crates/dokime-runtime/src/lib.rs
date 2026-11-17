/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

mod capabilities;
mod clock;
mod dispatch;
mod documents;
mod error;
mod execution;
mod execution_manager;
mod host;
mod plugin;
pub mod process;
mod prompts;
mod reports;
mod run_control;
pub mod services;
mod streams;

/* === Re-Exports === */

pub use capabilities::ExecutionCapabilities;
pub use clock::{ControlledClock, SchedulerClock};
pub use dokime_operations::host::credentials;
pub use dokime_operations::host::credentials::CredentialNamespace;
pub use dokime_operations::host::files::captures::CaptureSettings;
pub use dokime_storage::{StorageSettings, storage_defaults};
pub use error::Error;
pub use execution::{Cycle, CycleResult, RunState};
pub use host::{Host, HostSettings};
pub use plugin::{
    INTERVAL_LAG, INTERVAL_WINDOW, PLUGINS_TIMEOUT, PlacementSettings, PluginProcess, STOP_GRACE, describe_plugin, plugin_settings,
};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
