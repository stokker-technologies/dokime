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

// Streams the latest process-local progress for one execution. This is operational state: it is coalesced, cleared at termination, and never persisted as telemetry or report evidence.
#[harmos::operation(resources = [crate::hosting::Host<WatchExecutionProgress>], response = (), output = ExecutionProgressFrame, id = "dokime.observation.watch_execution_progress",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "WatchExecutionProgress"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchExecutionProgress {
    pub test_execution_id: String,
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

    crate::hosted!(WatchExecutionProgress);
}
