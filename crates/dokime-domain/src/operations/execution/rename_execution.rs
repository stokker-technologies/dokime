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
use serde::{Deserialize, Serialize};
// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Renames one live execution's case until an attempt of it seals.
#[harmos::message]
#[harmos::operation(response = RenameReply, id = "dokime.execution.rename_execution", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
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
