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

#[harmos::operation(resources = [crate::hosting::Host<ListStorageSources>], response = StorageSources, id = "dokime.settings.list_storage_sources",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "ListStorageSources"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListStorageSources {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListStorageSources);
}
