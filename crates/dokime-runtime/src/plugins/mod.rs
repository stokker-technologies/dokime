/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod catalog;
pub(crate) mod config;
pub(crate) mod dispatch;
mod handle;
mod model;
pub(crate) mod store;

/* === Re-Exports === */

pub use catalog::{PluginEnvironmentReference, PluginIdentity, PluginMetadata, inspect, inspect_with_archive_limit};
pub use config::{PluginBundle, PluginProvenance};
pub use model::{Installation, PluginEntry, PluginLastBoot, PluginSettings, PluginState, ProgramValues};
pub use store::{PluginError, PluginPage, PluginStore};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const NATIVE_TARGET: &str = env!("DOKIME_NATIVE_TARGET");

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
