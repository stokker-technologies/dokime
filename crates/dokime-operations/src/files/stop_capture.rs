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

// Public, so a template reaches it through `ctx.operation(request)` without listing it.
// Takes the final pass and answers once sealed. The capture limits bound the
// seal, not a wall clock. Sealing is owned by the runtime, so abandoning this
// call abandons only its reply, and a repeat returns the same outcome.
#[harmos::operation(resources = [crate::hosting::Host<StopCapture>], response = CapturedFiles, id = "dokime.files.stop_capture",
        policy = FOLLOW, replay = harmos::operations::ReplaySafety::Idempotent, visibility = harmos::operations::Visibility::Public)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StopCapture {
    pub capture_id: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
