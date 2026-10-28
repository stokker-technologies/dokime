/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod catalog;
pub(crate) mod config;
pub(crate) mod dispatch;
mod handle;
mod model;
pub(crate) mod store;

pub use catalog::{PluginIdentity, PluginMetadata, inspect, inspect_with_archive_limit};
pub use config::{PluginBundle, PluginProvenance};
pub use model::{Installation, PluginEntry, PluginLastBoot, PluginSettings, PluginState, ProgramValues};
pub use store::{PluginError, PluginPage, PluginStore};

pub const NATIVE_TARGET: &str = env!("DOKIME_NATIVE_TARGET");
