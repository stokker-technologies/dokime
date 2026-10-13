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
use super::{ExecutionProgressFrame, FOLLOW, WatchExecutionProgressRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Streams the latest process-local progress for one execution. This is operational state: it is coalesced, cleared at termination, and never persisted as telemetry or report evidence.
#[harmos::operation(request = WatchExecutionProgressRequest, response = (), output = ExecutionProgressFrame, id = "dokime.observation.watch_execution_progress", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "WatchExecutionProgress"))]
pub struct WatchExecutionProgress;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
