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

// Local Imports
use super::{ArchiveReply, ORDINARY, SetCycleArchivedRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Archives or unarchives one terminal cycle, live or retired to history.
#[harmos::operation(request = SetCycleArchivedRequest, response = ArchiveReply, id = "dokime.execution.set_cycle_archived", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "SetCycleArchived"))]
pub struct SetCycleArchived;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
