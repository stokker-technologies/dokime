/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Replays Saved Telemetry Before Following Durable Storage Changes.

use serde::{Deserialize, Serialize};

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<WatchSamples>], response = (), output = TelemetrySamplesReply, id = "dokime.telemetry.watch_samples",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Telemetry", method = "WatchSamples"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchSamples {
    pub selection: Option<ListSamples>,
    pub follow: bool,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(WatchSamples);
}
