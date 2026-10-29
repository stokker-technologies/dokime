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

#[harmos::operation(resources = [crate::hosting::Host<Screenshot>], response = ArtifactMetadata, id = "dokime.media.screenshot",
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "Screenshot"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Screenshot {
    pub scope: Option<ArtifactScope>,

    #[row(description = "Zero-based display index; absent selects the primary display.")]
    pub display: Option<u32>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(Screenshot);
}
