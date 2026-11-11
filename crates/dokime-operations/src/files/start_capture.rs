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
// Returns once the baseline is taken. The calling step's execution owns the
// capture, which runs across later steps until stopped or the execution ends.
#[harmos::operation(resources = [crate::hosting::Host<StartCapture>], response = CaptureStarted, id = "dokime.files.start_capture",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Public)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StartCapture {
    pub source: FileSource,

    #[row(description = "The artifact name for a file source; otherwise a relative prefix that preserves relative paths.")]
    pub output: String,

    pub mode: CaptureMode,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl StartCapture {
    pub fn new(source: FileSource, output: impl Into<String>, mode: impl Into<CaptureMode>) -> Self {
        Self {
            source,
            output: output.into(),
            mode: mode.into(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
