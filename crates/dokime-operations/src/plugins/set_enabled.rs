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

#[harmos::operation(resources = [crate::hosting::Host<SetEnabled>], response = PluginInstallation, id = "dokime.plugins.set_enabled",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "SetEnabled"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SetEnabled {
    pub install_id: String,
    pub enabled: bool,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(SetEnabled);
}
