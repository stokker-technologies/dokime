/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<BrowseCycles>], response = CycleBrowsePage, id = "dokime.history.browse_cycles",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "BrowseCycles"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowseCycles {
    pub limit: u32,
    pub at: Option<StorageCursor>,
    pub after_sequence: u64,
    pub filter: Option<CycleFilter>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(BrowseCycles);
}
