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
// Shared, so a template reaches it through `ctx.host()` without listing it.
// A step built-in only: listeners watch through `FileWatch`, and a gRPC
// binding would make the contract a plugin requires differ from the host's.
#[harmos_domain::message]
#[harmos_domain::operation(response = WatchFileOutcome, id = "dokime.files.watch_file", version = 1,
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Shared)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchFile {
    // Absent takes the execution of the calling step's context; one of the two is required.
    #[row(description = "Absent takes the execution of the calling step's context; one of the two is required.")]
    pub execution_id: Option<String>,
    // A local root or FTP URL naming the watched file itself.
    #[row(description = "A local root or FTP URL naming the watched file itself.")]
    pub source: Option<FileWatchSource>,
    // The artifact folder, a relative path.
    #[row(description = "The artifact folder, a relative path.")]
    pub output: String,
    // The artifact name, a relative path; absent names it `<output>/<file name>`.
    #[row(description = "The artifact name, a relative path; absent names it `<output>/<file name>`.")]
    pub artifact: Option<String>,
    pub policy: FileWatchPolicy,
    // Discovery and capture interval: default 1000, 50–60000 milliseconds.
    #[row(description = "Discovery and capture interval: default 1000, 50–60000 milliseconds.")]
    pub poll_interval_ms: Option<u32>,
    // The watch seals and settles after this many seconds; absent runs until cancelled or sealed.
    #[row(description = "The watch seals and settles after this many seconds; absent runs until cancelled or sealed.")]
    pub maximum_seconds: Option<u32>,
    // Bytes the watch may capture: default 256 MiB, maximum 4 GiB.
    #[row(description = "Bytes the watch may capture: default 256 MiB, maximum 4 GiB.")]
    pub maximum_bytes: Option<u64>,
    // Stream policy only: each captured line is offered to every matcher and marker.
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
