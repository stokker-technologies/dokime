/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<UpdatePluginSettings>], response = PluginRuntimeSettings, id = "dokime.settings.update_plugin_settings",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "UpdatePluginSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePluginSettings {
    pub plugin_id: String,
    pub values: Option<Value>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(UpdatePluginSettings);
}
