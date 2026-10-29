/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator media captured on the backend host and stored as execution artifacts.

use serde::Deserialize;
use serde::Serialize;

use crate::model::content::wire::ArtifactMetadata;

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

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordingFramePayload {
    Progress(RecordingProgress),
    Completed(Box<ArtifactMetadata>),
}
