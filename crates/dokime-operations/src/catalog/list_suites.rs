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

// Pages are ordered by registered suite ID. A changed cursor requires restarting
// pagination; pages are not pinned across runtime changes.
#[harmos::operation(resources = [crate::hosting::Host<ListSuites>], response = ListSuitesReply, id = "dokime.catalog.list_suites",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "ListSuites"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListSuites {
    pub limit: u32,
    pub after_suite_id: String,
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListSuites);
}
