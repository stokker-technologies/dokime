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

// Replays a bounded tail of the hub log, then follows it. Read access.
#[harmos::operation(resources = [crate::hosting::Host<WatchLogs>], response = (), output = LogFrame, id = "dokime.backend.watch_logs",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "WatchLogs"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchLogs {
    // Frames below this level are never delivered. Unspecified means TRACE.
    #[row(description = "Frames below this level are never delivered. Unspecified means TRACE.")]
    pub min_level: LogLevel,

    // Matching lines replayed from the end of the hub log before following it.
    // Zero uses 500; larger values are clamped to 5000.
    #[row(
        description = "Matching lines replayed from the end of the hub log before following it. Zero uses 500; larger values are clamped to 5000."
    )]
    pub replay_lines: u32,
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

    crate::hosted!(WatchLogs);
}
