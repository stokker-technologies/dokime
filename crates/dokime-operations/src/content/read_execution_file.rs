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

// A bounded read for live previews/tailing, not an immutable full-file download.
// A native producer may rewrite/truncate a streaming file between or during
// reads. Each window reports its actual offset and observed length. An offset
// past the current end is clamped to that end. max_bytes defaults to 64 KiB and
// must not exceed 64 KiB. No EOF/finality is implied by an empty streaming window.
#[harmos::operation(resources = [crate::hosting::Host<ReadExecutionFile>], response = ExecutionFileWindow, id = "dokime.content.read_execution_file",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "ReadExecutionFile"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadExecutionFile {
    pub execution_id: String,
    pub relative_path: String,
    pub offset: u64,
    pub max_bytes: u32,
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

    crate::hosted!(ReadExecutionFile);
}
