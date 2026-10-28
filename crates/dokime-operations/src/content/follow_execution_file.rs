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

#[harmos::operation(resources = [crate::hosting::Host<FollowExecutionFile>], response = (), output = ExecutionFileFollowFrame, id = "dokime.content.follow_execution_file",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Host,
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

    crate::hosted!(FollowExecutionFile);
}
