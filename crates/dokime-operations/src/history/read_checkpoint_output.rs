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

#[harmos::operation(resources = [crate::hosting::Host<ReadCheckpointOutput>], response = CheckpointFileWindow, id = "dokime.history.read_checkpoint_output",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "ReadCheckpointOutput"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadCheckpointOutput {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub relative_path: String,
    pub offset: u64,
    pub max_bytes: u32,
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

    crate::hosted!(ReadCheckpointOutput);
}
