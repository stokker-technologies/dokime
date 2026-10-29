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

#[harmos::operation(resources = [crate::hosting::Host<ListPairedClients>], response = PairedClientPage, id = "dokime.remote.list_paired_clients",
    policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "ListPairedClients"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPairedClients {
    pub cursor: Option<String>,
    pub limit: u32,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListPairedClients);
}
