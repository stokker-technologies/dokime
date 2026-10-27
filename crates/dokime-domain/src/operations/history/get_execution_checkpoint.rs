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

// Bounded inspection of one existing saved seal; never flushes or publishes. NOT_FOUND means no saved seal is available, including an unsealed row.
#[harmos_domain::operation(response = ExecutionCheckpoint, id = "dokime.history.get_execution_checkpoint",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "GetExecutionCheckpoint"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetExecutionCheckpoint {
    pub execution_identifier: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
