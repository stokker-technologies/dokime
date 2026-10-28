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

#[harmos::operation(resources = [crate::hosting::Host<GetInstallation>], response = PluginInstallation, id = "dokime.plugins.get_installation",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "GetInstallation"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetInstallation {
    pub install_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetInstallation);
}
