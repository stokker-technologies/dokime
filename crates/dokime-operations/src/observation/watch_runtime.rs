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

// This is a latest-state invalidation stream, not a durable event log. The
// initial frame always resynchronizes from current state. Subsequent frames may
// coalesce commits. Query cycle/catalog details after a frame as needed.
// A different epoch or a future position is rejected; reconnect without a cursor
// to explicitly resynchronize. Disconnect never cancels admitted execution.
#[harmos::operation(resources = [crate::hosting::Host<WatchRuntime>], response = (), output = RuntimeStateFrame, id = "dokime.observation.watch_runtime",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "WatchRuntime"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchRuntime {
    pub cursor: Option<Cursor>,
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

    crate::hosted!(WatchRuntime);
}
