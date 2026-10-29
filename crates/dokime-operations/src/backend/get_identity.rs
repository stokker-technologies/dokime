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

#[harmos::operation(resources = [crate::hosting::Host<GetIdentity>], response = BackendIdentity, id = "dokime.backend.get_identity",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "GetIdentity"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetIdentity {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetIdentity);
}
