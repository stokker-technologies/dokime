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

// Follow the original registered production from a locally committed offset.
// A nonzero offset requires the identity returned by the first metadata frame.
// Reader cancellation never cancels the producer. Producer session generations
// may change without changing this retained-content identity.
#[harmos::operation(resources = [crate::hosting::Host<FollowExecutionFile>], response = (), output = ExecutionFileFollowFrame, id = "dokime.content.follow_execution_file",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "FollowExecutionFile"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FollowExecutionFile {
    pub execution_id: String,
    pub relative_path: String,
    pub identity: Option<ExecutionFileIdentity>,
    pub offset: u64,

    #[row(description = "Defaults to 64 KiB; must not exceed 64 KiB.")]
    pub max_bytes: u32,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(FollowExecutionFile);
}
