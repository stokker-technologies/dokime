/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Replays Saved Telemetry Before Following Durable Storage Changes.

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

#[harmos_domain::operation(response = (), output = TelemetrySamplesReply, id = "dokime.telemetry.watch_samples",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Telemetry", method = "WatchSamples"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchSamples {
    pub selection: Option<ListSamples>,
    pub follow: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
