/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod application;
mod launch;
pub mod organization;
pub(crate) mod policy;
mod runtime;
mod settings;
mod view;

/* === Re-Exports === */

pub use crate::paths::{ApplicationPaths, DEFAULT_APPLICATION_ID};
pub use crate::runtime::storage::config::storage_defaults;
pub use application::ApplicationOptions;
pub use harmos::storage::StorageSettings;
pub use launch::Config;
pub use policy::{StoragePolicy, StoragePolicyView};
pub use runtime::{InstalledSidecar, RuntimeConfig, SidecarNoticeHandler};
pub use settings::{Appearance, Density, ReportDefaults, Settings, SettingsDocument, ThemeMode};
pub use view::ApplicationSettingsView;

#[cfg(test)]
pub(crate) use view::MAX_PLOT_POINTS;

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
