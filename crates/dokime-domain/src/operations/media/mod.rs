/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator media captured on the backend host and stored as execution artifacts.

/* === Modules === */
mod audio_recording;
mod camera_recording;
mod photo;
mod screen_recording;
mod screenshot;

/* === Re-Exports === */
pub use audio_recording::AudioRecording;
pub use camera_recording::CameraRecording;
pub use photo::Photo;
pub use screen_recording::ScreenRecording;
pub use screenshot::Screenshot;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::content::{ArtifactMetadata, ArtifactScope};
use super::{Access, Endpoint, FOLLOW, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Every capture needs a cycle and execution scope; storage keeps artifacts per execution.






// Which audio sources a video recording mixes into its soundtrack.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingAudio {
    pub microphone: bool,
    pub system: bool,
}





#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingProgress {
    pub elapsed_ms: u64,
    pub size_bytes: u64,
}

// Progress frames while recording, then one Completed frame carrying the sealed
// artifact. A cancelled recording is still sealed, without a final frame.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingFrame {
    pub payload: Option<RecordingFramePayload>,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordingFramePayload {
    Progress(RecordingProgress),
    Completed(Box<ArtifactMetadata>),
}

#[harmos::catalog]
pub enum Operations {
    Screenshot(Screenshot),
    Photo(Photo),
    AudioRecording(AudioRecording),
    ScreenRecording(ScreenRecording),
    CameraRecording(CameraRecording),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for Screenshot {
    const PATH: &'static str = "/dokime.v1.Media/Screenshot";
    const ACCESS: Access = Access::Control;
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for Photo {
    const PATH: &'static str = "/dokime.v1.Media/Photo";
    const ACCESS: Access = Access::Control;
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for AudioRecording {
    const PATH: &'static str = "/dokime.v1.Media/AudioRecording";
    const ACCESS: Access = Access::Control;
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for ScreenRecording {
    const PATH: &'static str = "/dokime.v1.Media/ScreenRecording";
    const ACCESS: Access = Access::Control;
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for CameraRecording {
    const PATH: &'static str = "/dokime.v1.Media/CameraRecording";
    const ACCESS: Access = Access::Control;
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
