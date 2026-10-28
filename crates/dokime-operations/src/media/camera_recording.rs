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

#[harmos::operation(resources = [crate::hosting::Host<CameraRecording>], response = (), output = RecordingFrame, id = "dokime.media.camera_recording",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "CameraRecording"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraRecording {
    pub scope: Option<ArtifactScope>,
    pub camera: Option<u32>,

    // Absent records video only.
    #[row(description = "Absent records video only.")]
    pub audio: Option<RecordingAudio>,

    pub maximum_seconds: Option<u32>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(CameraRecording);
}
