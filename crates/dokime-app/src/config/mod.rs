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
pub mod branding;
mod launch;
pub(crate) mod policy;
mod runtime;
mod settings;
mod view;

/* === Re-Exports === */
pub use crate::runtime::storage::config::StorageSettings;
pub use application::ApplicationOptions;
pub use dokime::paths::{ApplicationPaths, DEFAULT_APPLICATION_ID};
pub use launch::Config;
pub use policy::{StoragePolicy, StoragePolicyView};
pub use runtime::{InstalledSidecar, RuntimeConfig, RuntimeCredentialConfig, SidecarNoticeHandler};
pub use settings::{Appearance, Density, ReportDefaults, Settings, SettingsDocument, ThemeMode};
pub use view::ApplicationSettingsView;

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
