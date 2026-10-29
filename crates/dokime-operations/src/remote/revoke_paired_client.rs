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

#[harmos::operation(resources = [crate::hosting::Host<RevokePairedClient>], response = (), id = "dokime.remote.revoke_paired_client",
    policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "RevokePairedClient"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokePairedClient {
    pub identifier: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(RevokePairedClient);
}
