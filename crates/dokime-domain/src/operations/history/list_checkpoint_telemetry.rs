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
#[harmos::operation(response = CheckpointTelemetryPage, id = "dokime.history.list_checkpoint_telemetry", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "History", method = "ListCheckpointTelemetry"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListCheckpointTelemetry {
    pub checkpoint: Option<ExecutionCheckpoint>,
    // Original logical native stream identifier; never a Delta table name.
    #[row(description = "Original logical native stream identifier; never a Delta table name.")]
    pub table: String,
    pub limit: u32,
    pub after: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
