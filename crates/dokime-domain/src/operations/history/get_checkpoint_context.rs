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

#[harmos_domain::message]
#[harmos_domain::operation(response = CheckpointContext, id = "dokime.history.get_checkpoint_context", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "GetCheckpointContext"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCheckpointContext {
    pub checkpoint: Option<ExecutionCheckpoint>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
