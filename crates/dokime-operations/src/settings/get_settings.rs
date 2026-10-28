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

#[harmos::operation(resources = [crate::hosting::Host<GetSettings>], response = SettingsFrame, id = "dokime.settings.get_settings",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "GetSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSettings {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetSettings);
}
