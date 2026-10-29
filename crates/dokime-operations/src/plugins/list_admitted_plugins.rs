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

#[harmos::operation(resources = [crate::hosting::Host<ListAdmittedPlugins>], response = AdmittedPluginPage, id = "dokime.plugins.list_admitted_plugins",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "ListAdmittedPlugins"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListAdmittedPlugins {
    pub page_size: u32,
    pub after: Option<AdmittedPluginCursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListAdmittedPlugins);
}
