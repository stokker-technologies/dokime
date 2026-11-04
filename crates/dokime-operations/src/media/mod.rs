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

// Local Imports
use super::FOLLOW;
use super::TRANSFER;
use dokime_domain::model::content::wire::ArtifactMetadata;
use dokime_domain::model::content::wire::ArtifactScope;
use dokime_domain::model::media::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

// Every capture needs a cycle and execution scope; storage keeps artifacts per execution.

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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
