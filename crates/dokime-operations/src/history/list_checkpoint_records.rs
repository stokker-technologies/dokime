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

#[harmos::operation(resources = [crate::hosting::Host<ListCheckpointRecords>], response = CheckpointRecordPage, id = "dokime.history.list_checkpoint_records",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "ListCheckpointRecords"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCheckpointRecords {
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub limit: u32,
    pub after: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
