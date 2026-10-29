/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;
use serde::{Deserialize, Serialize};

#[harmos::operation(resources = [crate::hosting::Host<UpdateApplicationSettings>], response = SettingsSnapshot, id = "dokime.settings.update_application_settings",
    policy = MUTATION, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Settings", method = "UpdateApplicationSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateApplicationSettings {
    pub update: UpdateSettings,
}

#[cfg(feature = "runtime")]
mod runtime {

    use super::*;

    crate::hosted!(UpdateApplicationSettings);
}
