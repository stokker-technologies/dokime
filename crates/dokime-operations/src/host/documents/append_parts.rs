/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Append Scoped Document Parts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Resources, Result};
use dokime_protocol::host::documents_append_parts_request::Part;
use dokime_protocol::host::{DocumentScope, DocumentsAppendPartsRequest, DocumentsAppendPartsResponse};
use dokime_protocol::model::Document;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Documents/AppendParts";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(resources: &Resources, request: DocumentsAppendPartsRequest) -> Result<DocumentsAppendPartsResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let _guard = resources.context.administration.lock().await;
    if let Some((_, receipt)) =
        resources.store.requested_evidence(&resources.execution, ID, &request.request_id, &request.encode_to_vec())?
    {
        return Ok(DocumentsAppendPartsResponse { receipt: Some(receipt) });
    }
    let cut = resources.store.metadata(&resources.execution)?;
    let scope = DocumentScope {
        execution_id: resources.execution.clone(),
        cycle_id: cut.execution.cycle.clone(),
    };
    let (mut document, cut) = resources.context.host.current_document(&scope, &request.document)?;
    replay(&resources.store, &cut, &mut document, resources.context.settings.page_items)?;
    apply(&mut document, &request)?;
    if document.encoded_len() > resources.context.settings.message_bytes {
        return Err(Error::Bound("Append Document Parts: Keep the Document Within the Configured Message Bound".into()));
    }
    let (_, receipt) =
        resources.evidence_requested(ID, &request.request_id, &request.encode_to_vec(), "document:part", request.encode_to_vec(), None)?;
    Ok(DocumentsAppendPartsResponse { receipt: Some(receipt) })
}

pub(crate) fn replay(
    store: &dokime_storage::Store, cut: &dokime_storage::ExecutionCut, document: &mut Document, page_items: usize,
) -> Result<()> {
    let mut after = 0;
    loop {
        let entries =
            store.evidence_page(&cut.execution.identifier, after, cut.frontier, std::num::NonZeroUsize::new(page_items).unwrap())?;
        if entries.is_empty() {
            return Ok(());
        }
        for entry in entries {
            after = entry.position;
            if entry.kind != "document:part" {
                continue;
            }
            let request = DocumentsAppendPartsRequest::decode(entry.payload.as_slice())?;
            if document.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.document) {
                apply(document, &request)?;
            }
        }
    }
}

/* === Private Functions === */

fn apply(document: &mut Document, request: &DocumentsAppendPartsRequest) -> Result<()> {
    if request.document.is_empty() || document.metadata.as_ref().is_none_or(|metadata| metadata.identifier != request.document) {
        return Err(Error::Invalid("Append Document Parts: Select the Frozen Document Identity".into()));
    }
    let part = request.part.as_ref().ok_or_else(|| Error::Invalid("Append Document Parts: Supply a Card or Subsection".into()))?;
    if request.section.is_empty() {
        return Err(Error::Invalid("Append Document Parts: Select a Nonempty Section Identity".into()));
    }
    let mut pending = document.sections.iter().collect::<Vec<_>>();
    let mut matches = 0;
    while let Some(section) = pending.pop() {
        matches += usize::from(section.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.section));
        pending.extend(section.sections.iter());
    }
    if matches != 1 {
        return Err(Error::Invalid("Append Document Parts: Select One Existing Section Identity".into()));
    }
    let mut sections = document.sections.iter_mut().collect::<Vec<_>>();
    let mut selected = None;
    while let Some(section) = sections.pop() {
        if section.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.section) {
            if selected.is_some() {
                return Err(Error::Invalid("Append Document Parts: Select One Distinct Section Identity".into()));
            }
            selected = Some(section);
            continue;
        }
        sections.extend(section.sections.iter_mut());
    }
    let section = selected.ok_or_else(|| Error::Invalid("Append Document Parts: Select an Existing Section Identity".into()))?;
    match part {
        Part::Subsection(value) => section.sections.push(value.clone()),
        Part::Card(value) => section.cards.push(value.clone()),
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Document, DocumentsAppendPartsRequest, Part, apply};
    use dokime_protocol::model::card::Body;
    use dokime_protocol::model::{Card, Metadata, Section};

    fn document() -> Document {
        Document {
            metadata: Some(Metadata {
                identifier: "report".into(),
                ..Default::default()
            }),
            sections: vec![Section {
                metadata: Some(Metadata {
                    identifier: "results".into(),
                    ..Default::default()
                }),
                ..Default::default()
            }],
        }
    }

    #[test]
    fn appends_cards_and_nested_sections_to_the_selected_document() {
        let mut document = document();
        let mut request = DocumentsAppendPartsRequest {
            document: "report".into(),
            section: "results".into(),
            part: Some(Part::Subsection(Section {
                metadata: Some(Metadata {
                    identifier: "nested".into(),
                    ..Default::default()
                }),
                ..Default::default()
            })),
            ..Default::default()
        };
        apply(&mut document, &request).unwrap();
        request.section = "nested".into();
        request.part = Some(Part::Card(Card {
            body: Some(Body::Markdown(String::new())),
            ..Default::default()
        }));
        apply(&mut document, &request).unwrap();
        assert_eq!(document.sections[0].sections[0].cards.len(), 1);
        assert!(document.sections[0].cards.is_empty());
    }

    #[test]
    fn invalid_selectors_and_missing_parts_leave_the_document_unchanged() {
        let mut document = document();
        let before = document.clone();
        for request in [
            DocumentsAppendPartsRequest {
                document: "other".into(),
                section: "results".into(),
                ..Default::default()
            },
            DocumentsAppendPartsRequest {
                document: "report".into(),
                section: "missing".into(),
                ..Default::default()
            },
            DocumentsAppendPartsRequest {
                document: "report".into(),
                section: "results".into(),
                ..Default::default()
            },
        ] {
            assert!(apply(&mut document, &request).is_err());
            assert_eq!(document, before);
        }
    }
}

crate::native::unary!(DocumentsAppendPartsRequest, DocumentsAppendPartsResponse, resources, "dokime.host.v1", "Documents", "AppendParts");
