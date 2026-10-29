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

#[harmos::operation(resources = [crate::hosting::Host<ReadApplicationSettings>], response = SettingsSnapshot, id = "dokime.settings.read_application_settings",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "ReadApplicationSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadApplicationSettings {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ReadApplicationSettings);
}
