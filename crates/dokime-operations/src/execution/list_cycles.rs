/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// `filter` and `order` must be repeated unchanged on every continuation, and
// `after_cycle_id` must still be a member of the filtered result. A changed
// selection, a changed order, or a continuation ID the filter excludes is
// rejected rather than silently restarted.
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

    crate::hosted!(ListCycles);
}
