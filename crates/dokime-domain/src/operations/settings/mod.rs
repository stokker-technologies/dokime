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
mod read_storage_policy;
mod reorder_storage_sources;
mod set_storage_source_enabled;
mod update_appearance;
mod update_data_root;
mod update_plugin_credential;
mod update_plugin_settings;
mod update_report_defaults;
mod update_storage_policy;

/* === Re-Exports === */

pub use get_plugin_settings::GetPluginSettings;
pub use get_settings::GetSettings;
pub use list_storage_sources::ListStorageSources;
pub use read_application_settings::ReadApplicationSettings;
pub use read_storage_policy::ReadStoragePolicy;
pub use reorder_storage_sources::ReorderStorageSources;
pub use set_storage_source_enabled::SetStorageSourceEnabled;
pub use update_appearance::UpdateAppearance;
pub use update_data_root::UpdateDataRoot;
pub use update_plugin_credential::UpdatePluginCredential;
pub use update_plugin_settings::UpdatePluginSettings;
pub use update_report_defaults::UpdateReportDefaults;
pub use update_storage_policy::UpdateStoragePolicy;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::ORDINARY;
use crate::model::settings::wire::*;
use crate::model::values::wire::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

// An absent data root restores the default beneath the application root.

#[harmos_domain::catalog]
pub enum Operations {
    ReadStoragePolicy(ReadStoragePolicy),
    UpdateStoragePolicy(UpdateStoragePolicy),
    GetSettings(GetSettings),
    ListStorageSources(ListStorageSources),
    ReorderStorageSources(ReorderStorageSources),
    SetStorageSourceEnabled(SetStorageSourceEnabled),
    GetPluginSettings(GetPluginSettings),
    UpdatePluginSettings(UpdatePluginSettings),
    UpdatePluginCredential(UpdatePluginCredential),
    ReadApplicationSettings(ReadApplicationSettings),
    UpdateAppearance(UpdateAppearance),
    UpdateDataRoot(UpdateDataRoot),
    UpdateReportDefaults(UpdateReportDefaults),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for ReadStoragePolicy {
    const PATH: &'static str = "/dokime.v1.Settings/ReadStoragePolicy";
}

impl Endpoint for UpdateStoragePolicy {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateStoragePolicy";
}

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

impl Endpoint for UpdatePluginCredential {
    const PATH: &'static str = "/dokime.v1.Settings/UpdatePluginCredential";
}

impl Endpoint for ReadApplicationSettings {
    const PATH: &'static str = "/dokime.v1.Settings/ReadApplicationSettings";
}

impl Endpoint for UpdateAppearance {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateAppearance";
}

impl Endpoint for UpdateReportDefaults {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateReportDefaults";
}

impl Endpoint for UpdateDataRoot {
    const PATH: &'static str = "/dokime.v1.Settings/UpdateDataRoot";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
