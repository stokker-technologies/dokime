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

// Streams the latest process-local progress for one execution. This is operational state: it is coalesced, cleared at termination, and never persisted as telemetry or report evidence.
#[harmos::operation(resources = [crate::hosting::Host<WatchExecutionProgress>], response = (), output = ExecutionProgressFrame, id = "dokime.observation.watch_execution_progress",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "WatchExecutionProgress"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchExecutionProgress {
    pub test_execution_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(WatchExecutionProgress);
}
