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
use serde::{Deserialize, Serialize};
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[harmos::operation(response = ListCyclesReply, id = "dokime.execution.list_cycles", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
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
