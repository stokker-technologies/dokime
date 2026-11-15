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

// Local Imports
use crate::host::storage::administration;
use crate::{Result, host::Context};
use dokime_protocol::host::evidence::OperatorNote;
use prost::Message;

use dokime_protocol::host::evidence::NotesListRequest;
use dokime_protocol::host::evidence::NotesListResponse;

use crate::host::ClientSettings;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Notes/List";

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
        let key = format!("notes/{}", crate::host::views::hex(&crate::host::paging::bytes(&selection)));
        let (saved, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let store = context.host.store();
            let scope = selection.owner.as_ref().map(|owner| store.note_owner(owner)).transpose()?;
            let (entries, _) = administration::scan(context, "note:", None).await?;
            let mut notes = Vec::new();
            for entry in entries {
                let note = OperatorNote::decode(entry.payload.as_slice())?;
                if let Some(selected) = &scope {
                    let matches = note.scope.as_ref().is_some_and(|actual| {
                        selected.cycle_id.as_ref().is_none_or(|cycle| actual.cycle_id.as_ref() == Some(cycle))
                            && selected.execution_id.as_ref().is_none_or(|execution| {
                                actual.execution_id.as_ref() == Some(execution)
                                    || selection.include_context && actual.execution_id.is_none()
                            })
                    });
                    if !matches {
                        continue;
                    }
                }
                notes.push(note);
            }
            Ok(NotesListResponse { notes, ..Default::default() })
        })
        .await?;
        let (notes, next_page_token) = crate::host::views::page(
            saved.notes,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
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

/* === Tests === */

crate::native::unary!(NotesListRequest, NotesListResponse, context, "dokime.host.v1.evidence", "Notes", "List");
