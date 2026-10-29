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

#[harmos::operation(resources = [crate::hosting::Host<ListCases>], response = ListCasesReply, id = "dokime.execution.list_cases",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListCases"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCases {
    pub cycle_id: String,
    pub limit: u32,
    pub after_case_id: String,
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListCases);
}
