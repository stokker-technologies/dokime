/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<ListCheckpointTelemetry>], response = CheckpointTelemetryPage, id = "dokime.history.list_checkpoint_telemetry",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
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

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListCheckpointTelemetry);
}
