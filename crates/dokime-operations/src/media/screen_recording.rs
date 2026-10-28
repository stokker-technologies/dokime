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

#[harmos::operation(resources = [crate::hosting::Host<ScreenRecording>], response = (), output = RecordingFrame, id = "dokime.media.screen_recording",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "ScreenRecording"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenRecording {
    pub scope: Option<ArtifactScope>,
    pub display: Option<u32>,

    // Absent records video only.
    #[row(description = "Absent records video only.")]
    pub audio: Option<RecordingAudio>,

    pub maximum_seconds: Option<u32>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ScreenRecording);
}
