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

#[harmos::operation(resources = [crate::hosting::Host<ReorderStorageSources>], response = StorageSources, id = "dokime.settings.reorder_storage_sources",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "ReorderStorageSources"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReorderStorageSources {
    pub identifiers: Vec<String>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ReorderStorageSources);
}
