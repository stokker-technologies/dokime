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

// Local Imports
use super::{FOLLOW, RecordingFrame, ScreenRecordingRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::operation(request = ScreenRecordingRequest, response = (), output = RecordingFrame, id = "dokime.media.screen_recording", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Media", method = "ScreenRecording"))]
pub struct ScreenRecording;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
