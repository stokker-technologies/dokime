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
use harmos_protocol::model::{PageRequest, ReadCut, ResourceRevision};
use prost::Message;

// Local Imports
use crate::host::storage::administration;
use crate::host::{self, ClientSettings, Context, paging, views};
use crate::{Error, Result};
use dokime_protocol::host::evidence::{AnnotationSelection, NotesListRequest, NotesListResponse, OperatorNote};

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

pub async fn handle(context: &Context, request: NotesListRequest) -> Result<NotesListResponse> {
    host::complete(ID, context.settings.read_timeout, async {
        views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let (notes, cut) = read(context, &request).await?;
        let reply = response(&context.settings, notes, cut, request.page.as_ref())?;
        host::bounded_reply(context, reply)
    })
    .await
}

pub(crate) async fn capture(context: &Context, selection: Option<AnnotationSelection>) -> Result<Vec<NotesListResponse>> {
    host::complete(ID, context.settings.read_timeout, async {
        let request = NotesListRequest { selection, ..Default::default() };
        let (notes, cut) = read(context, &request).await?;
        pages(&context.settings, notes, cut)?.into_iter().map(|reply| host::bounded_reply(context, reply)).collect()
    })
    .await
}

/* === Private Functions === */

async fn read(context: &Context, request: &NotesListRequest) -> Result<(Vec<OperatorNote>, ReadCut)> {
    let selection = request.selection.as_ref().cloned().unwrap_or_default();
    let revision = paging::bytes(&selection);
    let at = request.at.clone().map(|cut| source_cut(cut, &revision)).transpose()?;
    let scope = selection.owner.as_ref().map(|owner| super::owner(context, owner)).transpose()?;
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
    Ok((notes, cut))
}

fn response(
    settings: &ClientSettings, notes: impl IntoIterator<Item = OperatorNote, IntoIter: ExactSizeIterator>, cut: ReadCut,
    page: Option<&PageRequest>,
) -> Result<NotesListResponse> {
    let (notes, next_page_token) = views::page(notes, &cut, page, settings.page_items, settings.message_bytes)?;
    Ok(NotesListResponse {
        notes,
        resolved_at: Some(cut),
        next_page_token,
    })
}

fn pages(settings: &ClientSettings, notes: Vec<OperatorNote>, cut: ReadCut) -> Result<Vec<NotesListResponse>> {
    let mut pages = Vec::new();
    let mut token = String::new();
    loop {
        let page = PageRequest { page_token: token, ..Default::default() };
        let reply = response(settings, notes.iter().cloned(), cut.clone(), Some(&page))?;
        token = reply.next_page_token.clone();
        pages.push(reply);
        if token.is_empty() {
            return Ok(pages);
        }
    }
}

pub(crate) fn source_cut(mut cut: ReadCut, revision: &[u8]) -> Result<ReadCut> {
    let selected = cut
        .revisions
        .iter()
        .position(|value| value.name == SELECTION)
        .ok_or_else(|| Error::Invalid("List Notes: List a Fresh Image and Use Its Returned Cut".into()))?;
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn captured_notes_preserve_rows_cuts_tokens_and_wire_pages() {
        let notes = (0..19)
            .map(|index| OperatorNote {
                note_id: index.to_string(),
                message: "0".repeat(index * 11),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let cut = selection_cut(ReadCut::default(), vec![1, 2, 3]);
        for (page_items, message_bytes) in [(2, 4096), (7, 850)] {
            let settings = ClientSettings {
                read_timeout: Duration::from_secs(1),
                mutation_timeout: Duration::from_secs(1),
                watch_timeout: Duration::from_secs(1),
                max_cases: 1,
                page_items,
                message_bytes,
                stream_buffer: 1,
            };
            let captured = pages(&settings, notes.clone(), cut.clone()).unwrap();
            assert!(captured.len() > 1);
            let mut token = String::new();
            let mut selected = Vec::new();
            for captured in captured {
                let request = PageRequest { page_token: token, ..Default::default() };
                let listed = response(&settings, notes.clone(), cut.clone(), Some(&request)).unwrap();
                assert_eq!(captured.encode_to_vec(), listed.encode_to_vec());
                assert_eq!(captured.resolved_at.as_ref(), Some(&cut));
                assert!(captured.encoded_len() <= message_bytes);
                assert!(captured.notes.len() <= page_items);
                token = captured.next_page_token;
                selected.extend(captured.notes);
            }
            assert!(token.is_empty());
            assert_eq!(selected, notes);
            let empty = pages(&settings, Vec::new(), cut.clone()).unwrap();
            assert_eq!(empty.len(), 1);
            assert!(empty[0].notes.is_empty());
            assert!(empty[0].next_page_token.is_empty());
            assert_eq!(empty[0].resolved_at.as_ref(), Some(&cut));
        }
    }
}
