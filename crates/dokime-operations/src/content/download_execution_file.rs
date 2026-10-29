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

// Opens one admitted output on its original saved view or active producer inode.
// Saved reads verify the frozen descriptor length and digest. Live reads stop at
// the observed prefix; producer rewrites may make that read fail. Each original
// charged chunk is encoded before the next read. Completion describes bytes
// actually delivered. Consumers verify offsets, size, digest and final OK status.
#[harmos::operation(resources = [crate::hosting::Host<DownloadExecutionFile>], response = (), output = ExecutionFileDownloadFrame, id = "dokime.content.download_execution_file",
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "DownloadExecutionFile"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadExecutionFile {
    pub execution_id: String,
    pub relative_path: String,
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

    crate::hosted!(DownloadExecutionFile);
}
