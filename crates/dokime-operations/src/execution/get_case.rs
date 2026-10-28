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

#[harmos::operation(resources = [crate::hosting::Host<GetCase>], response = CaseDetail, id = "dokime.execution.get_case",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "GetCase"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCase {
    pub cycle_id: String,
    pub case_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetCase);
}
