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

#[harmos::operation(resources = [crate::hosting::Host<GetPluginSettings>], response = PluginRuntimeSettings, id = "dokime.settings.get_plugin_settings",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "GetPluginSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPluginSettings {
    pub plugin_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetPluginSettings);
}
