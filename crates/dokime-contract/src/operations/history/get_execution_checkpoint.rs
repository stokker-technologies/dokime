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
use super::{ExecutionCheckpoint, GetExecutionCheckpointRequest, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Bounded inspection of one existing saved seal; never flushes or publishes. NOT_FOUND means no saved seal is available, including an unsealed row.
#[harmos::operation(request = GetExecutionCheckpointRequest, response = ExecutionCheckpoint, id = "dokime.history.get_execution_checkpoint", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "GetExecutionCheckpoint"))]
pub struct GetExecutionCheckpoint;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
