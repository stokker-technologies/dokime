/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator media captured on the backend host and stored as execution artifacts.

mod audio_recording;
mod camera_recording;
mod photo;
mod screen_recording;
mod screenshot;

pub use audio_recording::AudioRecording;
pub use camera_recording::CameraRecording;
pub use photo::Photo;
pub use screen_recording::ScreenRecording;
pub use screenshot::Screenshot;

use super::Endpoint;
use super::FOLLOW;
use super::TRANSFER;
use dokime_domain::model::content::wire::ArtifactMetadata;
use dokime_domain::model::content::wire::ArtifactScope;
use dokime_domain::model::media::wire::*;

// Every capture needs a cycle and execution scope; storage keeps artifacts per execution.

#[harmos::catalog]
pub enum Operations {
    Screenshot(Screenshot),
    Photo(Photo),
    AudioRecording(AudioRecording),
    ScreenRecording(ScreenRecording),
    CameraRecording(CameraRecording),
}

impl Endpoint for Screenshot {
    const PATH: &'static str = "/dokime.v1.Media/Screenshot";
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for Photo {
    const PATH: &'static str = "/dokime.v1.Media/Photo";
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for AudioRecording {
    const PATH: &'static str = "/dokime.v1.Media/AudioRecording";
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for ScreenRecording {
    const PATH: &'static str = "/dokime.v1.Media/ScreenRecording";
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}

impl Endpoint for CameraRecording {
    const PATH: &'static str = "/dokime.v1.Media/CameraRecording";
    // The host's screen, camera and microphone are never exposed to a remote listener.
    const LOCAL_ONLY: bool = true;
}
