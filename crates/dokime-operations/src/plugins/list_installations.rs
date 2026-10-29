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

#[harmos::operation(resources = [crate::hosting::Host<ListInstallations>], response = InstallationPage, id = "dokime.plugins.list_installations",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "ListInstallations"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListInstallations {
    pub page_size: u32,
    pub after: Option<InstallationCursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListInstallations);
}
