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

// Returns a bounded cycle page at one exact runtime revision.
#[harmos::operation(resources = [crate::hosting::Host<GetRuntimeShell>], response = RuntimeShellSnapshot, id = "dokime.observation.get_runtime_shell",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "GetRuntimeShell"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetRuntimeShell {
    #[row(description = "Zero selects the default page size (50); the maximum is 100.")]
    pub limit: u32,

    pub after_cycle_id: String,
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetRuntimeShell);
}
