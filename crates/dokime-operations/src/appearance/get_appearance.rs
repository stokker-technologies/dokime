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

#[harmos::operation(resources = [crate::hosting::Host<GetAppearance>], response = AppearanceSnapshot, id = "dokime.appearance.get_appearance",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Appearance", method = "GetAppearance"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetAppearance {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetAppearance);
}
