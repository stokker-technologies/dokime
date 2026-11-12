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

mod administration;
mod capabilities;
mod clock;
mod dispatch;
mod documents;
mod error;
mod execution;
mod host;
mod plugin;
pub mod process;
mod run_control;
pub mod services;
mod streams;

/* === Re-Exports === */

pub use capabilities::ExecutionCapabilities;
pub use clock::{ControlledClock, SchedulerClock};
pub use documents::{FixedSnapshot, RenderSettings, RenderedDocument};
pub use error::Error;
pub use execution::{Cycle, CycleResult, RunState};
pub use host::{Host, HostSettings};
pub use plugin::{EntryState, PlacementSettings, PluginProcess, describe_plugin};

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
