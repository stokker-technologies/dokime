/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod get_plugin_settings;
mod get_settings;
mod list_storage_sources;
mod read_application_settings;
mod reorder_storage_sources;
mod set_storage_source_enabled;
mod update_application_settings;
mod update_plugin_secret;
mod update_plugin_settings;

/* === Re-Exports === */

pub use get_plugin_settings::GetPluginSettings;
pub use get_settings::GetSettings;
pub use list_storage_sources::ListStorageSources;
pub use read_application_settings::ReadApplicationSettings;
pub use reorder_storage_sources::ReorderStorageSources;
pub use set_storage_source_enabled::SetStorageSourceEnabled;
pub use update_application_settings::UpdateApplicationSettings;
pub use update_plugin_secret::UpdatePluginSecret;
pub use update_plugin_settings::UpdatePluginSettings;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::{MUTATION, ORDINARY};
use crate::model::settings::wire::*;
use crate::model::values::wire::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
pub enum Operations {
    GetSettings(GetSettings),
    ListStorageSources(ListStorageSources),
    ReorderStorageSources(ReorderStorageSources),
    SetStorageSourceEnabled(SetStorageSourceEnabled),
    GetPluginSettings(GetPluginSettings),
    UpdatePluginSettings(UpdatePluginSettings),
    UpdatePluginSecret(UpdatePluginSecret),
    ReadApplicationSettings(ReadApplicationSettings),
    UpdateApplicationSettings(UpdateApplicationSettings),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for GetSettings {
    const PATH: &'static str = "/dokime.v1.Settings/GetSettings";
}

impl Endpoint for ListStorageSources {
    const PATH: &'static str = "/dokime.v1.Settings/ListStorageSources";
}

impl Endpoint for ReorderStorageSources {
    const PATH: &'static str = "/dokime.v1.Settings/ReorderStorageSources";
}

impl Endpoint for SetStorageSourceEnabled {
    const PATH: &'static str = "/dokime.v1.Settings/SetStorageSourceEnabled";
}

impl Endpoint for GetPluginSettings {
    const PATH: &'static str = "/dokime.v1.Settings/GetPluginSettings";
}

impl Endpoint for UpdatePluginSettings {
    const PATH: &'static str = "/dokime.v1.Settings/UpdatePluginSettings";
}

impl Endpoint for UpdatePluginSecret {
    const PATH: &'static str = "/dokime.v1.Settings/UpdatePluginSecret";
}

impl Endpoint for ReadApplicationSettings {
    const PATH: &'static str = "/dokime.v1.Settings/ReadApplicationSettings";
}

impl Endpoint for UpdateApplicationSettings {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateApplicationSettings";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
