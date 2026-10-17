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
#[harmos_domain::message]
#[harmos_domain::operation(response = (), output = RecordingFrame, id = "dokime.media.screen_recording", version = 1,
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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
