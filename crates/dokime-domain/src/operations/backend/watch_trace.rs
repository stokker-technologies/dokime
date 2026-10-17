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
use super::{FOLLOW, TraceBatch, WatchTraceRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Pages the backend's live tracing output for one scope, then optionally follows
// it. Read access, like the hub log it complements.
#[harmos::operation(request = WatchTraceRequest, response = (), output = TraceBatch, id = "dokime.backend.watch_trace", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "WatchTrace"))]
pub struct WatchTrace;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
