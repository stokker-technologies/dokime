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

#[harmos::operation(resources = [crate::hosting::Host<FollowExecutionFile>], response = (), output = ExecutionFileFollowFrame, id = "dokime.content.follow_execution_file",
        policy = FOLLOW, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "FollowExecutionFile"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FollowExecutionFile {
    pub execution_id: String,
    pub relative_path: String,
    pub identity: Option<ExecutionFileIdentity>,
    pub offset: u64,

    // Defaults to 64 KiB; must not exceed 64 KiB.
    #[row(description = "Defaults to 64 KiB; must not exceed 64 KiB.")]
    pub max_bytes: u32,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(FollowExecutionFile);
}
