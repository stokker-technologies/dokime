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

#[harmos::operation(resources = [crate::hosting::Host<AudioRecording>], response = (), output = RecordingFrame, id = "dokime.media.audio_recording",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "AudioRecording"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioRecording {
    pub scope: Option<ArtifactScope>,

    #[row(description = "Zero-based input device index; absent selects the default input.")]
    pub input: Option<u32>,

    #[row(description = "Recording stops and seals after this many seconds: default 600, maximum 3600.")]
    pub maximum_seconds: Option<u32>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(AudioRecording);
}
