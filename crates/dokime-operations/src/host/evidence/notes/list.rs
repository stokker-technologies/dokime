/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Notes List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::{OperationContract, ReadCut, ResourceRevision};
use prost::Message;

// Local Imports
use crate::host::storage::administration;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::evidence::{NotesListRequest, NotesListResponse, OperatorNote};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/List";
const SELECTION: &str = "dokime.evidence.notes/selection";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<NotesListRequest, NotesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: NotesListRequest) -> Result<NotesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let selection = request.selection.clone().unwrap_or_default();
        let revision = crate::host::paging::bytes(&selection);
        let at = request.at.clone().map(|cut| source_cut(cut, &revision)).transpose()?;
        let store = context.host.store();
        let scope = selection.owner.as_ref().map(|owner| store.note_owner(owner)).transpose()?;
        let (entries, at) = administration::scan(context, "note:", at).await?;
        let at = at.ok_or_else(|| Error::Precondition("List Notes: Return the Issued Storage Cut".into()))?;
        let cut = selection_cut(at, revision);
        let mut notes = Vec::new();
        for entry in entries {
            let note = OperatorNote::decode(entry.payload.as_slice())?;
            if let Some(selected) = &scope {
                let matches = note.scope.as_ref().is_some_and(|actual| {
                    selected.cycle_id.as_ref().is_none_or(|cycle| actual.cycle_id.as_ref() == Some(cycle))
                        && selected.execution_id.as_ref().is_none_or(|execution| {
                            actual.execution_id.as_ref() == Some(execution) || selection.include_context && actual.execution_id.is_none()
                        })
                });
                if !matches {
                    continue;
                }
            }
            notes.push(note);
        }
        let (notes, next_page_token) =
            crate::host::views::page(notes, &cut, request.page.as_ref(), context.settings.page_items, context.settings.message_bytes)?;
        crate::host::bounded_reply(
            context,
            NotesListResponse {
                notes,
                resolved_at: Some(cut),
                next_page_token,
            },
        )
    })
    .await
}

/* === Private Functions === */

pub(crate) fn source_cut(mut cut: ReadCut, revision: &[u8]) -> Result<ReadCut> {
    let selected = cut
        .revisions
        .iter()
        .position(|value| value.name == SELECTION)
        .ok_or_else(|| Error::Invalid("List Notes: Supply Its Returned Selection and Cut".into()))?;
    if cut.revisions.remove(selected).revision != revision || cut.revisions.iter().any(|value| value.name == SELECTION) {
        return Err(Error::Invalid("List Notes: Keep the Same Selection and Cut".into()));
    }
    Ok(cut)
}

pub(crate) fn selection_cut(mut cut: ReadCut, revision: Vec<u8>) -> ReadCut {
    cut.revisions.push(ResourceRevision { name: SELECTION.into(), revision });
    cut
}

/* === Tests === */

crate::native::unary!(NotesListRequest, NotesListResponse, context, "dokime.host.v1.evidence", "Notes", "List");
