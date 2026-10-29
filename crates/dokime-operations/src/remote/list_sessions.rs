/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// List Recent Remote Sessions

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Recent remote sessions on this host, newest first; `cursor` continues a previous page.
#[harmos::operation(resources = [crate::hosting::Host<ListSessions>], response = RemoteSessionPage, id = "dokime.remote.list_sessions",
    policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "ListSessions"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListSessions {
    pub cursor: Option<String>,
    pub limit: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListSessions);
}
