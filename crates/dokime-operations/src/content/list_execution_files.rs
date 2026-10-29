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

// Files currently owned by an execution in this runtime. These logical names
// select registered outputs; they never resolve arbitrary filesystem paths.
// Listings are ordered by relative_path. A continuation binds to execution,
// runtime epoch and metadata fingerprint; changed listings require a restart.
#[harmos::operation(resources = [crate::hosting::Host<ListExecutionFiles>], response = ListExecutionFilesReply, id = "dokime.content.list_execution_files",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "ListExecutionFiles"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionFiles {
    pub execution_id: String,
    pub limit: u32,
    pub after_path: String,
    pub cursor: Option<ExecutionFilesCursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListExecutionFiles);
}
