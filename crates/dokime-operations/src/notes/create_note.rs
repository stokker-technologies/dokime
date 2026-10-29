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

#[harmos::operation(resources = [crate::hosting::Host<CreateNote>], response = NoteReceipt, id = "dokime.notes.create_note",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Notes", method = "CreateNote"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateNote {
    #[row(description = "Explicit process epoch from discovery. Retry identities expire at restart.")]
    pub epoch: String,

    pub mutation_id: String,
    pub owner: Option<AnnotationOwner>,

    #[row(description = "nonblank, at most 16 KiB UTF-8. Actor comes from authentication.")]
    pub message: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(CreateNote);
}
