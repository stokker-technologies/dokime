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

#[harmos::operation(resources = [crate::hosting::Host<EditNote>], response = EditNoteResponse, id = "dokime.notes.edit_note",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Notes", method = "EditNote"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EditNote {
    #[row(description = "Explicit process epoch from discovery.")]
    pub epoch: String,

    pub note_id: String,

    #[row(description = "nonblank, at most 16 KiB UTF-8. Actor comes from authentication.")]
    pub message: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(EditNote);
}
