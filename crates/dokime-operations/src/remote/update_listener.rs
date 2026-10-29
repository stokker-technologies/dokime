/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Apply Remote Listener Settings

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<UpdateListener>], response = RemoteListenerStatus, id = "dokime.remote.update_listener",
    policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "UpdateListener"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateListener {
    pub settings: Option<RemoteListenerSettings>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(UpdateListener);
}
