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

// Archives or unarchives one terminal cycle, live or retired to history.
// Archiving is orthogonal to the lifecycle and admitted only on a terminal
// cycle or execution: it hides the row from lists that do not ask for archived
// rows and changes nothing else.
#[harmos::operation(resources = [crate::hosting::Host<SetCycleArchived>], response = ArchiveReply, id = "dokime.execution.set_cycle_archived",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "SetCycleArchived"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SetCycleArchived {
    pub cycle_id: String,
    pub archived: bool,
    pub context: Option<MutationContext>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(SetCycleArchived);
}
