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

// Archives or unarchives one terminal execution, live or retired to history.
// Archiving is orthogonal to the lifecycle and admitted only on a terminal
// cycle or execution: it hides the row from lists that do not ask for archived
// rows and changes nothing else.
#[harmos::operation(resources = [crate::hosting::Host<SetExecutionArchived>], response = ArchiveReply, id = "dokime.execution.set_execution_archived",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "SetExecutionArchived"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SetExecutionArchived {
    pub execution_id: String,
    pub archived: bool,
    pub context: Option<MutationContext>,
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

    crate::hosted!(SetExecutionArchived);
}
