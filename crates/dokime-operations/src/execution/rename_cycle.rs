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

// Renames one live cycle until it seals or is archived.
// A rename is admitted until the subject seals: a sealed, archived or History
// cycle and an execution whose case has a sealed attempt refuse it with
// FailedPrecondition, since the sealed report froze the label. An execution's
// label is its case's label, so every attempt of that case is renamed together.
#[harmos::operation(resources = [crate::hosting::Host<RenameCycle>], response = RenameReply, id = "dokime.execution.rename_cycle",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RenameCycle"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RenameCycle {
    pub cycle_id: String,

    #[row(description = "nonblank after trimming, at most 4096 bytes UTF-8")]
    pub label: String,

    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(RenameCycle);
}
