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

#[harmos::operation(resources = [crate::hosting::Host<CameraRecording>], response = (), output = RecordingFrame, id = "dokime.media.camera_recording",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "CameraRecording"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraRecording {
    pub scope: Option<ArtifactScope>,
    pub camera: Option<u32>,

    #[row(description = "Absent records video only.")]
    pub audio: Option<RecordingAudio>,

    pub maximum_seconds: Option<u32>,
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

    crate::hosted!(CameraRecording);
}
