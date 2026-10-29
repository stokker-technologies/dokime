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

#[harmos::operation(resources = [crate::hosting::Host<ListNotes>], response = ListNotesResponse, id = "dokime.notes.list_notes",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Notes", method = "ListNotes"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListNotes {
    pub selection: Option<AnnotationSelection>,

    #[row(description = "default 50, maximum 100")]
    pub limit: u32,

    pub after_id: String,

    #[row(description = "Exact runtime revision and selection, required with after_id. Restart on change.")]
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListNotes);
}
