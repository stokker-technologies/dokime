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
use super::{ORDINARY, RenameExecutionRequest, RenameReply};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Renames one live execution's case in any lifecycle state, sealed included.
#[harmos::operation(request = RenameExecutionRequest, response = RenameReply, id = "dokime.execution.rename_execution", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RenameExecution"))]
pub struct RenameExecution;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
