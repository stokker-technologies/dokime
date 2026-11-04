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
mod update_plugin_settings;

/* === Re-Exports === */

pub use get_plugin_settings::GetPluginSettings;
pub use get_settings::GetSettings;
pub use list_storage_sources::ListStorageSources;
pub use read_application_settings::ReadApplicationSettings;
pub use reorder_storage_sources::ReorderStorageSources;
pub use set_storage_source_enabled::SetStorageSourceEnabled;
pub use update_application_settings::UpdateApplicationSettings;
pub use update_plugin_settings::UpdatePluginSettings;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{MUTATION, ORDINARY};
use dokime_domain::model::settings::wire::*;
use dokime_domain::model::values::wire::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    GetSettings(GetSettings),
    ListStorageSources(ListStorageSources),
    ReorderStorageSources(ReorderStorageSources),
    SetStorageSourceEnabled(SetStorageSourceEnabled),
    GetPluginSettings(GetPluginSettings),
    UpdatePluginSettings(UpdatePluginSettings),
    ReadApplicationSettings(ReadApplicationSettings),
    UpdateApplicationSettings(UpdateApplicationSettings),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
