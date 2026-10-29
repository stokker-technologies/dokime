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

// Replays a bounded tail of the hub log, then follows it. Read access.
#[harmos::operation(resources = [crate::hosting::Host<WatchLogs>], response = (), output = LogFrame, id = "dokime.backend.watch_logs",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "WatchLogs"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchLogs {
    #[row(description = "Frames below this level are never delivered. Unspecified means TRACE.")]
    pub min_level: LogLevel,

    #[row(
        description = "Matching lines replayed from the end of the hub log before following it. Zero uses 500; larger values are clamped to 5000."
    )]
    pub replay_lines: u32,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(WatchLogs);
}
