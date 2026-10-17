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
// Archives or unarchives one terminal cycle, live or retired to history.
#[harmos::message]
#[harmos::operation(response = ArchiveReply, id = "dokime.execution.set_cycle_archived", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "SetCycleArchived"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SetCycleArchived {
    pub cycle_id: String,
    pub archived: bool,
    pub context: Option<MutationContext>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
