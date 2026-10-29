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

#[harmos::operation(resources = [crate::hosting::Host<BrowseExecutions>], response = ExecutionBrowsePage, id = "dokime.history.browse_executions",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "BrowseExecutions"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BrowseExecutions {
    pub limit: u32,
    pub after_execution_id: String,
    pub filter: Option<ExecutionFilter>,
    pub order: Option<ExecutionOrder>,
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

    crate::hosted!(BrowseExecutions);
}
