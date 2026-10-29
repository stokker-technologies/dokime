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

#[harmos::operation(resources = [crate::hosting::Host<ListCaseParameters>], response = ListCaseParametersReply, id = "dokime.execution.list_case_parameters",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListCaseParameters"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCaseParameters {
    pub cycle_id: String,
    pub case_id: String,
    pub limit: u32,
    pub after_name: String,
    pub cursor: Option<ParameterCursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListCaseParameters);
}
