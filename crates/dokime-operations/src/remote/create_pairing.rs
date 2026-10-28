/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CreatePairing

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<CreatePairing>], response = CreatedPairing, id = "dokime.remote.create_pairing",
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "CreatePairing"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePairing {
    pub label: String,
    pub endpoint: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(CreatePairing);
}
