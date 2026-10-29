/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod application;

#[cfg(feature = "native")]
mod launch;

pub mod organization;

#[cfg(feature = "native")]
mod runtime;

#[cfg(feature = "native")]
mod settings;

mod storage;

#[cfg(feature = "native")]
mod view;

pub use crate::paths::{ApplicationPaths, DEFAULT_APPLICATION_ID};
pub use application::ApplicationOptions;
pub use harmos::storage::StorageSettings;
#[cfg(feature = "native")]
pub use launch::Config;

#[cfg(feature = "native")]
pub use runtime::{InstalledSidecar, RuntimeConfig, SidecarNoticeHandler};

#[cfg(feature = "native")]
pub use settings::{ApplicationMetadata, Settings};

pub use dokime_domain::model::settings::{Appearance, Density, ReportDefaults, ThemeMode};

pub use storage::{managed_storage_defaults, storage_defaults, storage_state_defaults};

#[cfg(feature = "storage")]
pub use storage::local_storage_defaults;
