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

#[harmos::operation(resources = [crate::hosting::Host<SetStorageSourceEnabled>], response = StorageSources, id = "dokime.settings.set_storage_source_enabled",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "SetStorageSourceEnabled"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SetStorageSourceEnabled {
    pub identifier: String,
    pub enabled: bool,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(SetStorageSourceEnabled);
}
