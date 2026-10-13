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
use super::{FOLLOW, LogFrame, WatchLogsRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Replays a bounded tail of the hub log, then follows it. Read access.
#[harmos::operation(request = WatchLogsRequest, response = (), output = LogFrame, id = "dokime.backend.watch_logs", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "WatchLogs"))]
pub struct WatchLogs;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
