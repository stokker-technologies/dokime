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

#[harmos::operation(resources = [crate::hosting::Host<GetHealth>], response = BackendHealth, id = "dokime.backend.get_health",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "GetHealth"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetHealth {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetHealth);
}
