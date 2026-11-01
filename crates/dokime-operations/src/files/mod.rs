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
// progress to a listener; `WatchFile` is the same watch over files for a
// template, settling only when the watch stops, and may carry line matchers
// that turn captured lines into rows and records.

/* === Modules === */

mod await_file_watch_armed;
mod file_watch;
mod watch_file;

/* === Re-Exports === */

pub use await_file_watch_armed::AwaitFileWatchArmed;
pub use file_watch::FileWatch;
pub use watch_file::WatchFile;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::FOLLOW;
use dokime_domain::model::files::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    FileWatch(FileWatch),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for FileWatch {
    const PATH: &'static str = "/dokime.v1.Files/FileWatch";
    // A local root reads the host's filesystem, which a remote listener never exposes.
    const LOCAL_ONLY: bool = true;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
