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

#[harmos::operation(resources = [crate::hosting::Host<EditNote>], response = EditNoteResponse, id = "dokime.notes.edit_note",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Notes", method = "EditNote"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EditNote {
    // Explicit process epoch from discovery.
    #[row(description = "Explicit process epoch from discovery.")]
    pub epoch: String,

    pub note_id: String,

    // nonblank, at most 16 KiB UTF-8. Actor comes from authentication.
    #[row(description = "nonblank, at most 16 KiB UTF-8. Actor comes from authentication.")]
    pub message: String,
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

    crate::hosted!(EditNote);
}
