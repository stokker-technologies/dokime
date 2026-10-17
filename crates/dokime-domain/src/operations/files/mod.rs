/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File sources watched during an execution: every matched file becomes one
// execution artifact under the watch's output folder. `FileWatch` streams its
// progress to a listener; `WatchFile` is the same watch over one file for a
// template, settling only when the watch stops, and may carry line matchers
// that turn captured lines into rows and records.

/* === Modules === */
mod file_watch;
mod watch_file;

/* === Re-Exports === */
pub use file_watch::FileWatch;
pub use watch_file::WatchFile;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Access;
use super::Endpoint;
use super::FOLLOW;

use crate::model::files::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Watches a source for the lifetime of the stream: until the caller cancels,
// the bound elapses or the execution seals. Each matched file is captured into
// `<output>/<path relative to the root>` and sealed with its length and digest.

// Watches one file for the calling step's execution until the instance is
// cancelled, the bound elapses or the execution seals. Its bytes become the
// artifact `artifact`, or `<output>/<file name>` when absent.

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    FileWatch(FileWatch),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for FileWatch {
    const PATH: &'static str = "/dokime.v1.Files/FileWatch";
    const ACCESS: Access = Access::Control;
    // A local root reads the host's filesystem, which a remote listener never exposes.
    const LOCAL_ONLY: bool = true;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
