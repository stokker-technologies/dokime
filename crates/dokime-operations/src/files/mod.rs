/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File sources captured during an execution. `CopyFiles` copies once and
// settles. `StartCapture` returns once its baseline is taken, and the execution
// owns the capture until `StopCapture` seals it or the execution ends.

/* === Modules === */

mod copy_files;
mod start_capture;
mod stop_capture;

/* === Re-Exports === */

pub use copy_files::CopyFiles;
pub use start_capture::StartCapture;
pub use stop_capture::StopCapture;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::FOLLOW;
use dokime_domain::model::files::wire::*;

/* === Constants === */

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(CopyFiles, StartCapture, StopCapture);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
