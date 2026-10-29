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

#[harmos::operation(resources = [crate::hosting::Host<GetAvailability>], operations = [harmos::builtins::host::Catalog], response = BackendAvailability, id = "dokime.backend.get_availability",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "GetAvailability"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetAvailability {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetAvailability);
}
