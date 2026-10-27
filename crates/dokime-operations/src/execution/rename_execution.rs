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

// Renames one live execution's case until an attempt of it seals.
#[harmos::operation(resources = [crate::hosting::Host<RenameExecution>], response = RenameReply, id = "dokime.execution.rename_execution",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Execution", method = "RenameExecution"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RenameExecution {
    pub execution_id: String,

    // nonblank after trimming, at most 4096 bytes UTF-8
    #[row(description = "nonblank after trimming, at most 4096 bytes UTF-8")]
    pub label: String,

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

    crate::hosted!(RenameExecution);
}
