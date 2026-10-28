/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Remote Listener State

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<ReadListener>], response = RemoteListenerStatus, id = "dokime.remote.read_listener",
    policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "ReadListener"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadListener {}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ReadListener);
}
