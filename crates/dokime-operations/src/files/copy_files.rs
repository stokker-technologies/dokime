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

const POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
    // A Cancelled Copy Still Seals What It Took And Answers Within Its Attempt
    // Deadline. Cancellation Must Not Impose The Shorter Run-Stop Reply Budget.
    cancellation_timeout: FOLLOW.attempt_timeout,
    ..FOLLOW
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Public, so a template reaches it through `ctx.operation(request)` without listing it.
// Copies its source into the calling step's execution once, as it is now. The
// capture limits bound the copy, not a wall clock.
#[harmos::operation(resources = [crate::hosting::Host<CopyFiles>], response = CapturedFiles, id = "dokime.files.copy_files",
        policy = POLICY, visibility = harmos::operations::Visibility::Public)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CopyFiles {
    pub source: FileSource,

    #[row(description = "The artifact name for a file source; otherwise a relative prefix that preserves relative paths.")]
    pub output: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl CopyFiles {
    pub fn new(source: FileSource, output: impl Into<String>) -> Self {
        Self { source, output: output.into() }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
