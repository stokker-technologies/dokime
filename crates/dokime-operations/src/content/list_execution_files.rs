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

// Files currently owned by an execution in this runtime. These logical names
// select registered outputs; they never resolve arbitrary filesystem paths.
// Listings are ordered by relative_path. A continuation binds to execution,
// runtime epoch and metadata fingerprint; changed listings require a restart.
#[harmos::operation(resources = [crate::hosting::Host<ListExecutionFiles>], response = ListExecutionFilesReply, id = "dokime.content.list_execution_files",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "ListExecutionFiles"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionFiles {
    pub execution_id: String,
    pub limit: u32,
    pub after_path: String,
    pub cursor: Option<ExecutionFilesCursor>,
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

    crate::hosted!(ListExecutionFiles);
}
