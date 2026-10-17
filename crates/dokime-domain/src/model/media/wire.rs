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

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::content::wire::ArtifactMetadata;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
