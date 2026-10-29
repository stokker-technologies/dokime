/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// List Recent Remote Sessions

use serde::{Deserialize, Serialize};

use super::*;

/// Recent remote sessions on this host, newest first; `cursor` continues a previous page.
#[harmos::operation(resources = [crate::hosting::Host<ListSessions>], response = RemoteSessionPage, id = "dokime.remote.list_sessions",
    policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "ListSessions"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListSessions {
    pub cursor: Option<String>,
    pub limit: u32,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListSessions);
}
