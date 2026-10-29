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

// Public, so a template reaches it through `ctx.host()` without listing it.
// A step built-in only: listeners watch through `FileWatch`, and a gRPC
// binding would make the contract a plugin requires differ from the host's.
// Watches one file for the calling step's execution until the instance is
// cancelled, the bound elapses or the execution seals. Its bytes become the
// artifact `artifact`, or `<output>/<file name>` when absent.
#[harmos::operation(resources = [crate::hosting::Host<WatchFile>], response = WatchFileOutcome, id = "dokime.files.watch_file",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Public)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchFile {
    #[row(description = "Absent takes the execution of the calling step's context; one of the two is required.")]
    pub execution_id: Option<String>,

    #[row(description = "A local root or FTP URL naming the watched file itself.")]
    pub source: Option<FileWatchSource>,

    #[row(description = "The artifact folder, a relative path.")]
    pub output: String,

    #[row(description = "The artifact name, a relative path; absent names it `<output>/<file name>`.")]
    pub artifact: Option<String>,

    pub policy: FileWatchPolicy,

    #[serde(default)]
    pub selection: FileWatchSelection,

    pub wait_for_source_seconds: Option<u32>,

    #[row(description = "Discovery and capture interval: default 1000, 50–60000 milliseconds.")]
    pub poll_interval_ms: Option<u32>,

    #[row(description = "The watch seals and settles after this many seconds; absent runs until cancelled or sealed.")]
    pub maximum_seconds: Option<u32>,

    #[row(description = "Bytes the watch may capture: default 256 MiB, maximum 4 GiB.")]
    pub maximum_bytes: Option<u64>,

    #[row(description = "Stream policy only: each captured line is offered to every matcher and marker.")]
    pub matchers: Vec<LineMatcher>,

    pub markers: Vec<LineMarker>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(WatchFile);
}
