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

mod await_file_watch_armed;
mod file_watch;
mod watch_file;

pub use await_file_watch_armed::AwaitFileWatchArmed;
pub use file_watch::FileWatch;
pub use watch_file::WatchFile;

use super::Endpoint;
use super::FOLLOW;
use dokime_domain::model::files::wire::*;

// Watches a source for the lifetime of the stream: until the caller cancels,

// the bound elapses or the execution seals. Each matched file is captured into

// `<output>/<path relative to the root>` and sealed with its length and digest.

// Watches one file for the calling step's execution until the instance is

// cancelled, the bound elapses or the execution seals. Its bytes become the

// artifact `artifact`, or `<output>/<file name>` when absent.

#[harmos::catalog]
pub enum Operations {
    FileWatch(FileWatch),
}

impl Endpoint for FileWatch {
    const PATH: &'static str = "/dokime.v1.Files/FileWatch";
    // A local root reads the host's filesystem, which a remote listener never exposes.
    const LOCAL_ONLY: bool = true;
}
