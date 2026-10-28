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

#[harmos::operation(resources = [crate::hosting::Host<WatchRuntime>], response = (), output = RuntimeStateFrame, id = "dokime.observation.watch_runtime",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "WatchRuntime"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchRuntime {
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(WatchRuntime);
}
