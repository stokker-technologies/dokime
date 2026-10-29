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

#[harmos::operation(resources = [crate::hosting::Host<GetCycle>], response = CycleSummary, id = "dokime.execution.get_cycle",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "GetCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCycle {
    pub cycle_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetCycle);
}
