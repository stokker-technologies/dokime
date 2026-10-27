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

#[cfg(feature = "host")]
mod launch;

pub mod organization;

#[cfg(feature = "host")]
mod runtime;

#[cfg(feature = "host")]
mod settings;

mod storage;

#[cfg(feature = "host")]
mod view;

/* === Re-Exports === */

pub use crate::paths::{ApplicationPaths, DEFAULT_APPLICATION_ID};
pub use application::ApplicationOptions;
pub use harmos::storage::StorageSettings;
#[cfg(feature = "host")]
pub use launch::Config;

#[cfg(feature = "host")]
pub use runtime::{InstalledSidecar, RuntimeConfig, SidecarNoticeHandler};

#[cfg(feature = "host")]
pub use settings::{ApplicationMetadata, Settings};

pub use dokime_domain::model::settings::{Appearance, Density, ReportDefaults, ThemeMode};

pub use storage::{managed_storage_defaults, storage_defaults, storage_state_defaults};

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
