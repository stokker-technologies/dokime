/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<FileWatch>], response = (), output = FileWatchFrame, id = "dokime.files.file_watch",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Files", method = "FileWatch"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWatch {
    // Absent takes the execution of the calling step's context; one of the two is required.
    #[row(description = "Absent takes the execution of the calling step's context; one of the two is required.")]
    pub execution_id: Option<String>,

    pub source: Option<FileWatchSource>,

    // Ordered rules: the last rule matching a path decides; a path no rule matches is excluded.
    #[row(description = "Ordered rules: the last rule matching a path decides; a path no rule matches is excluded.")]
    pub rules: Vec<FileRule>,

    // The artifact folder, a relative path, that mirrors the matched structure.
    #[row(description = "The artifact folder, a relative path, that mirrors the matched structure.")]
    pub output: String,

    pub policy: FileWatchPolicy,

    #[serde(default)]
    pub selection: FileWatchSelection,

    // Opt in to waiting for a missing root or matching file, bounded independently of capture.
    pub wait_for_source_seconds: Option<u32>,

    // Discovery and capture interval: default 1000, 50–60000 milliseconds.
    #[row(description = "Discovery and capture interval: default 1000, 50–60000 milliseconds.")]
    pub poll_interval_ms: Option<u32>,

    // The watch seals and completes after this many seconds; absent runs until cancelled or sealed.
    #[row(description = "The watch seals and completes after this many seconds; absent runs until cancelled or sealed.")]
    pub maximum_seconds: Option<u32>,

    // Files the watch may track: default 64, maximum 256.
    #[row(description = "Files the watch may track: default 64, maximum 256.")]
    pub maximum_files: Option<u32>,

    // Bytes the watch may capture across its files: default 256 MiB, maximum 4 GiB.
    #[row(description = "Bytes the watch may capture across its files: default 256 MiB, maximum 4 GiB.")]
    pub maximum_bytes: Option<u64>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(FileWatch);
}
