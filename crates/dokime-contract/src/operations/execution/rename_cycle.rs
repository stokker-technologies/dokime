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
use super::{ORDINARY, RenameCycleRequest, RenameReply};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Renames one live cycle in any lifecycle state, sealed included.
#[harmos::operation(request = RenameCycleRequest, response = RenameReply, id = "dokime.execution.rename_cycle", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RenameCycle"))]
pub struct RenameCycle;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
