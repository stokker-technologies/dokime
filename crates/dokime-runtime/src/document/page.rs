/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Page Continuations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{DocumentCardDefinition, DocumentDefinition, DocumentNode, bounded::DocumentLimits, capture::Capture, input::BorrowedInput};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct Cursor {
    stack: Vec<Frame>,
    complete: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Frame {
    child: usize,
    value: Option<Value>,
    repeat: super::repeat::Cursor,
}

pub(crate) struct Expansion {
    pub capture: Capture,
    pub queries: Vec<DocumentCardDefinition>,
    pub next: Option<Cursor>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Saved {
    pub definition: DocumentDefinition,
    pub selection: super::DocumentSelection,
    pub context: Vec<super::context::CapturedEntry>,
    pub cursor: Cursor,
    pub process_epoch: String,
    pub state_revision: u64,
}

pub(crate) struct Resume {
    pub saved: Saved,
    pub cut: crate::storage::sqlite::snapshots::Cut,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn resume(
    runtime: &crate::RuntimeHandle, token: &str, selection: &super::DocumentSelection,
    document: &dokime_domain::model::identity::DocumentId,
) -> crate::runtime::Result<Resume> {
    use crate::runtime::query::tokens::{Seek, Token, Tokens};
    let tokens = Tokens {
        key: runtime.local_storage.snapshot_key().await?,
        maximum: runtime.query_config().limits().token_bytes,
    };
    let Token::Continuation {
        cut,
        selection: binding,
        last: Seek::Document,
    } = tokens.decode(token, "dokime.documents.page")?
    else {
        return Err(invalid());
    };
    let bytes = runtime.local_storage.snapshot_contracts(cut.clone(), DocumentLimits::default().source_bytes).await?;
    let saved: Saved = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if &saved.selection != selection || &saved.definition.document_id != document || binding != identity(&saved)? {
        return Err(invalid());
    }
    Ok(Resume { saved, cut })
}

pub(crate) async fn token(
    runtime: &crate::RuntimeHandle, saved: &Saved, at: &harmos::storage::StorageCursor,
    cut: Option<&crate::storage::sqlite::snapshots::Cut>,
) -> crate::runtime::Result<String> {
    use crate::runtime::query::tokens::{Seek, Token, Tokens};
    let limit = DocumentLimits::default().source_bytes;
    let reached = super::bounded::document_size(saved, usize::MAX, DocumentLimits::default().depth)
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    if reached > limit {
        return Err(crate::Error::Document(super::DocumentFailure::Budget {
            budget: "continuation_bytes",
            limit,
            reached,
            card: saved.definition.metadata.identifier.to_string(),
        })
        .into());
    }
    let expires = match cut {
        Some(cut) => cut.expires,
        None => (time::OffsetDateTime::now_utc()
            + time::Duration::try_from(runtime.query_config().limits().snapshot_lifetime).map_err(|_| invalid())?)
        .into(),
    };
    let scope = match &saved.selection.scope.test_execution_id {
        Some(execution) => crate::storage::sqlite::snapshots::Scope::Execution(execution.to_string()),
        None => {
            crate::storage::sqlite::snapshots::Scope::Cycle(saved.selection.scope.test_cycle_id.as_ref().ok_or_else(invalid)?.to_string())
        }
    };
    let bytes = serde_json::to_vec(saved).map_err(|_| invalid())?;
    let mut captured = runtime
        .local_storage
        .snapshot_capture(scope, Some(at.clone()), Some(at.sequence), expires, bytes, None)
        .await?
        .ok_or_else(invalid)?;
    if let Some(original) = cut {
        if !runtime.local_storage.snapshot_available(original.clone()).await? {
            return Err(crate::storage::Error::QuerySnapshotUnavailable.into());
        }
        captured.retention = original.retention;
        captured.catalog = original.catalog;
        captured.metadata = original.metadata;
        captured.journal = original.journal;
    }
    let tokens = Tokens {
        key: runtime.local_storage.snapshot_key().await?,
        maximum: runtime.query_config().limits().token_bytes,
    };
    Ok(tokens.encode(
        &Token::Continuation {
            cut: captured,
            selection: identity(saved)?,
            last: Seek::Document,
        },
        "dokime.documents.page",
    )?)
}

pub(crate) async fn expand(
    runtime: &crate::RuntimeHandle, input: BorrowedInput<'_>, at: &harmos::storage::StorageCursor, mut cursor: Cursor,
) -> crate::runtime::Result<Expansion> {
    let definition = super::resolve::normalize_definition(input.definition.clone());
    if cursor.stack.is_empty() && !cursor.complete {
        cursor.stack.push(Frame::default());
    }
    let empty_repeats = super::resolve::RepeatScan::new([&definition])?.finish();
    let mut static_queries = Vec::new();
    let Capture { repeats, sections, .. } = super::resolve::capture_with_queries(
        BorrowedInput { definition: &definition, ..input },
        DocumentLimits::default(),
        Some(&empty_repeats),
        Some(&mut static_queries),
    )?;
    drop(static_queries);
    let mut capture = Capture {
        definition: definition.clone(),
        selection: input.selection.clone(),
        cards: Vec::new(),
        repeats,
        sections,
        warnings: Vec::new(),
    };
    let mut queries = Vec::new();
    let mut empty = Vec::new();
    let mut visited = 0;
    while !cursor.complete && visited < 64 {
        let (children, section) = location(&definition.layout.nodes, &cursor.stack)?;
        if !has_cards(children, &empty) || section.and_then(|section| section.repeat.as_ref()).is_some_and(|repeat| empty.contains(repeat))
        {
            advance(&mut cursor);
            continue;
        }
        let current = cursor.stack.last_mut().expect("page traversal has a root");
        if let Some(repeat) = section.and_then(|section| section.repeat.as_ref())
            && current.value.is_none()
        {
            current.value = super::repeat::next(runtime, input.selection, at, repeat, input.data, &mut current.repeat).await?;
            if current.value.is_none() {
                if !current.repeat.saw_values() {
                    empty.push(repeat.clone());
                }
                advance(&mut cursor);
                continue;
            }
            current.child = 0;
        }
        let Some(node) = children.get(current.child) else {
            if section.is_some_and(|section| section.repeat.is_some()) {
                current.value = None;
            } else {
                advance(&mut cursor);
            }
            continue;
        };
        if matches!(node, DocumentNode::Section(_)) {
            cursor.stack.push(Frame::default());
            continue;
        }
        let repeated = cursor.stack.iter().any(|frame| frame.value.is_some());
        if repeated {
            let before = cursor.clone();
            let mut selected = definition.clone();
            selected.layout.nodes = path(&definition.layout.nodes, &cursor.stack)?;
            let values = cursor.stack.iter().filter_map(|frame| frame.value.clone()).collect();
            let values = super::resolve::RepeatValues::path(values);
            let mut selected_queries = Vec::new();
            let selected = super::resolve::capture_with_queries(
                BorrowedInput { definition: &selected, ..input },
                DocumentLimits::default(),
                Some(&values),
                Some(&mut selected_queries),
            )?;
            let mut combined = capture.clone();
            merge(&mut combined, selected);
            let limits = DocumentLimits::default();
            let size = super::bounded::document_size(
                &(&combined.cards, &combined.sections, &combined.repeats),
                limits.materialized_bytes,
                limits.depth,
            );
            if let Err(error) = size {
                if !capture.cards.is_empty() {
                    return Ok(Expansion { capture, queries, next: Some(before) });
                }
                let card = combined
                    .cards
                    .first()
                    .map_or_else(|| definition.metadata.identifier.to_string(), |card| card.metadata.identifier.to_string());
                let (budget, limit, reached) = match error {
                    super::bounded::DocumentLimit::Bytes { limit, reached } => ("materialized_bytes", limit, reached),
                    super::bounded::DocumentLimit::Depth { limit, reached } => ("value_depth", limit, reached),
                    super::bounded::DocumentLimit::Encoding => return Err(invalid()),
                };
                return Err(crate::Error::Document(super::DocumentFailure::Budget { budget, limit, reached, card }).into());
            }
            capture = combined;
            for query in selected_queries {
                if !queries.iter().any(|previous: &DocumentCardDefinition| previous.metadata.identifier == query.metadata.identifier) {
                    queries.push(query);
                }
            }
            visited += 1;
        }
        cursor.stack.last_mut().expect("page traversal has a container").child += 1;
    }
    let next = (!cursor.complete).then_some(cursor);
    Ok(Expansion { capture, queries, next })
}

/* === Private Functions === */

fn identity(saved: &Saved) -> crate::runtime::Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(&("dokime.documents.page", &saved.selection, &saved.definition.document_id)).map_err(|_| invalid())?;
    Ok(Sha256::digest(bytes).into())
}

fn location<'a>(
    nodes: &'a [DocumentNode], stack: &[Frame],
) -> crate::runtime::Result<(&'a [DocumentNode], Option<&'a super::DocumentSectionNode>)> {
    let mut children = nodes;
    let mut section = None;
    for frame in stack.iter().take(stack.len().saturating_sub(1)) {
        let Some(DocumentNode::Section(selected)) = children.get(frame.child) else {
            return Err(invalid());
        };
        section = Some(selected.as_ref());
        children = &selected.children;
    }
    Ok((children, section))
}

fn path(nodes: &[DocumentNode], stack: &[Frame]) -> crate::runtime::Result<Vec<DocumentNode>> {
    let (frame, rest) = stack.split_first().ok_or_else(invalid)?;
    let mut node = nodes.get(frame.child).cloned().ok_or_else(invalid)?;
    if let DocumentNode::Section(section) = &mut node {
        section.children = path(&section.children, rest)?;
    }
    Ok(vec![node])
}

fn advance(cursor: &mut Cursor) {
    cursor.stack.pop();
    if let Some(parent) = cursor.stack.last_mut() {
        parent.child += 1;
    } else {
        cursor.complete = true;
    }
}

fn has_cards(nodes: &[DocumentNode], empty: &[super::DocumentSectionRepeat]) -> bool {
    nodes.iter().any(|node| match node {
        DocumentNode::Card(_) => true,
        DocumentNode::Section(section) => {
            !section.repeat.as_ref().is_some_and(|repeat| empty.contains(repeat)) && has_cards(&section.children, empty)
        }
    })
}

fn merge(target: &mut Capture, selected: Capture) {
    for card in selected.cards {
        if !target.cards.iter().any(|existing| existing.metadata.identifier == card.metadata.identifier) {
            target.cards.push(card);
        }
    }
    for (id, instances) in selected.repeats {
        let existing = target.repeats.entry(id).or_default();
        for instance in instances {
            if !existing.iter().any(|previous| previous.node_id == instance.node_id) {
                existing.push(instance);
            }
        }
    }
    target.sections.extend(selected.sections);
}

fn invalid() -> crate::runtime::Error {
    crate::runtime::Error::conflict("document_continuation", "document page does not match its captured layout")
}

/* === Tests === */
