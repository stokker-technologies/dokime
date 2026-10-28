/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<Install>], response = PluginInstallation, input = InstallPluginFrame, id = "dokime.plugins.install",
        policy = INSTALLATION, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "Install"))]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Install {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(Install);
}
