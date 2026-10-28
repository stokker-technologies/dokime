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

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ReadExecutionFile);
}
