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

#[harmos::operation(resources = [crate::hosting::Host<ListCycles>], response = ListCyclesReply, id = "dokime.execution.list_cycles",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "ListCycles"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCycles {
    pub limit: u32,
    pub after_cycle_id: String,
    pub cursor: Option<Cursor>,
    pub filter: Option<CycleFilter>,
    pub order: Option<CycleOrder>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListCycles);
}
