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
use serde::{Deserialize, Serialize};
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[harmos::operation(response = (), output = RecordingFrame, id = "dokime.media.audio_recording", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "AudioRecording"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioRecording {
    pub scope: Option<ArtifactScope>,
    // Zero-based input device index; absent selects the default input.
    #[row(description = "Zero-based input device index; absent selects the default input.")]
    pub input: Option<u32>,
    // Recording stops and seals after this many seconds: default 600, maximum 3600.
    #[row(
        description = "Recording stops and seals after this many seconds: default 600, maximum 3600."
    )]
    pub maximum_seconds: Option<u32>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
