/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Documents Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use harmos_protocol::model::History;
use harmos_protocol::model::Position;
use harmos_protocol::model::ReadCut;
use harmos_protocol::model::ResourceRevision;
use prost::Message;

// Local Imports
use crate::host::documents::{append_parts, expansion, plot};
use crate::host::storage::administration;
use crate::host::{self, Context, paging, views};
use crate::{Error, Result};
use dokime_protocol::host::DocumentBinding;
use dokime_protocol::host::DocumentBindingResult;
use dokime_protocol::host::DocumentCard;
use dokime_protocol::host::DocumentCardBody;
use dokime_protocol::host::DocumentCardFailure;
use dokime_protocol::host::DocumentCardKind;
use dokime_protocol::host::DocumentCompletion;
use dokime_protocol::host::DocumentEvidenceCut;
use dokime_protocol::host::DocumentFrame;
use dokime_protocol::host::DocumentFrameFrame;
use dokime_protocol::host::DocumentFrontiers;
use dokime_protocol::host::DocumentHeader;
use dokime_protocol::host::DocumentLayoutNode;
use dokime_protocol::host::DocumentLayoutNodeNode;
use dokime_protocol::host::DocumentOwnerReference;
use dokime_protocol::host::DocumentPlacement;
use dokime_protocol::host::DocumentPlotDefinition;
use dokime_protocol::host::DocumentSection;
use dokime_protocol::host::DocumentSectionIdentity;
use dokime_protocol::host::DocumentSpan;
use dokime_protocol::host::DocumentSummary;
use dokime_protocol::host::DocumentSummaryGroup;
use dokime_protocol::host::DocumentSummaryGroupDefinition;
use dokime_protocol::host::DocumentSummaryTile;
use dokime_protocol::host::DocumentSummaryTileDefinition;
use dokime_protocol::host::DocumentTable;
use dokime_protocol::host::DocumentTableRow;
use dokime_protocol::host::DocumentTelemetryCut;
use dokime_protocol::host::DocumentsGetRequest;
use dokime_protocol::host::DocumentsGetResponse;
use dokime_protocol::host::document_binding_result;
use dokime_protocol::host::document_card_body;
use dokime_protocol::host::document_frame_frame;
use dokime_protocol::host::document_layout_node_node;
use dokime_protocol::host::plots::PlotSettingsValue;
use dokime_protocol::model::Card;
use dokime_protocol::model::Document;
use dokime_protocol::model::Downsample;
use dokime_protocol::model::EvidenceCard;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::PlotFilter;
use dokime_protocol::model::Section;
use dokime_protocol::model::card;
use dokime_storage::persisted::Scope;
use dokime_storage::{Execution, ExecutionCut, ExecutionState, Store};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Documents/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct DocumentRead {
    #[prost(message, optional, tag = "1")]
    document: Option<Document>,

    #[prost(message, optional, tag = "2")]
    execution: Option<Execution>,

    #[prost(uint32, tag = "3")]
    state: u32,

    #[prost(bytes = "vec", tag = "4")]
    seal: Vec<u8>,

    #[prost(uint64, tag = "5")]
    frontier: u64,

    #[prost(message, repeated, tag = "6")]
    frames: Vec<DocumentFrame>,

    #[prost(btree_map = "string, message", tag = "7")]
    preferences: BTreeMap<String, PlotSettingsValue>,

    #[prost(string, tag = "8")]
    backend_id: String,

    #[prost(string, tag = "9")]
    process_epoch: String,

    #[prost(bool, tag = "10")]
    cacheable: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[tracing::instrument(name = "document.operation", skip_all)]
pub async fn handle(context: &Context, request: DocumentsGetRequest) -> Result<DocumentsGetResponse> {
    host::complete(
        ID,
        context.settings.read_timeout,
        Box::pin(async {
            if !request.page_token.is_empty() && request.at.is_none() {
                return Err(Error::Invalid("Get Document: Continue with the Returned Cut".into()));
            }
            let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Get Document: Select Its Execution or Cycle".into()))?;
            let mut selection = request.clone();
            selection.at = None;
            selection.page_token.clear();
            selection.max_bytes = 0;
            selection.max_frames = 0;
            selection.structure_only = false;
            selection.card_ids.clear();
            let key = format!("document-read/{}", views::hex(&paging::bytes(&selection)));
            let owner = if scope.execution_id.is_empty() {
                Scope::Cycle(&scope.cycle_id).identifier()?
            } else {
                Scope::Execution(&scope.execution_id).identifier()?
            };
            let (saved, cut) = views::read(context, owner, &key, request.at.as_ref(), async {
                let (document, fixed) = context.host.current_document(scope, &request.document_id)?;
                let (mut reader, _cancellation) = context.host.store().read_session()?;
                reader.reuse_sample_reads()?;
                prepare_read(context, document, &fixed, &reader, scope.execution_id.is_empty()).await
            })
            .await?;
            let (mut reader, _cancellation) = context.host.store().read_session()?;
            reader.reuse_sample_reads()?;
            let saved = response_prepared(&request, saved, &reader)?;
            let mut page_cut = cut.clone();
            let mut selection = request.clone();
            selection.at = None;
            selection.page_token.clear();
            selection.max_bytes = 0;
            selection.max_frames = 0;
            page_cut.revisions.push(ResourceRevision {
                name: "document-selection".into(),
                revision: paging::bytes(&selection),
            });
            let start = views::offset(&request.page_token, &page_cut)?;
            if start > saved.frames.len() {
                return Err(Error::Invalid("Get Document: Continue Within the Returned Frame Sequence".into()));
            }
            let maximum = if request.max_bytes == 0 {
                context.settings.message_bytes
            } else {
                usize::try_from(request.max_bytes).unwrap_or(usize::MAX).min(context.settings.message_bytes)
            };
            let total = saved.frames.len();
            let mut reply = DocumentsGetResponse {
                resolved_at: Some(cut.clone()),
                ..Default::default()
            };
            for (offset, frame) in saved.frames.into_iter().enumerate().skip(start) {
                reply.frames.push(frame);
                reply.next_page_token = if offset + 1 < total { views::token(offset + 1, &page_cut) } else { String::new() };
                if reply.encoded_len() > maximum {
                    reply.frames.pop();
                    if reply.frames.is_empty() {
                        return Err(Error::Bound("Get Document: Split This Frame to Fit the Reply Byte Bound".into()));
                    }
                    reply.next_page_token = views::token(offset, &page_cut);
                    break;
                }
                let frames = if request.max_frames == 0 {
                    context.settings.page_items
                } else {
                    (request.max_frames as usize).min(context.settings.page_items)
                };
                if reply.frames.len() == frames {
                    break;
                }
            }
            Ok(reply)
        }),
    )
    .await
}

/// Build Document Frames from a Resolved Declaration and One Evidence Cut.
pub async fn response_at(
    context: &Context, request: &DocumentsGetRequest, document: Document, fixed: &ExecutionCut,
) -> Result<DocumentsGetResponse> {
    let (mut reader, _cancellation) = context.host.store().read_session()?;
    reader.reuse_sample_reads()?;
    response_at_with_reader(context, request, document, fixed, &reader).await
}

pub(crate) async fn response_at_with_reader(
    context: &Context, request: &DocumentsGetRequest, document: Document, fixed: &ExecutionCut, reader: &Store,
) -> Result<DocumentsGetResponse> {
    let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Get Document: Select Its Execution or Cycle".into()))?;
    let saved = prepare_read(context, document, fixed, reader, scope.execution_id.is_empty()).await?;
    response_prepared(request, saved, reader)
}

/* === Private Functions === */

async fn prepare_read(context: &Context, document: Document, fixed: &ExecutionCut, reader: &Store, cycle: bool) -> Result<DocumentRead> {
    let expanded = prepare_document(document, fixed, reader, context.settings.page_items)?;
    let document_id = expanded.document.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
    let mut preferences = BTreeMap::new();
    let mut sections = expanded.document.sections.iter().collect::<Vec<_>>();
    while let Some(section) = sections.pop() {
        for card in &section.cards {
            let Some(card::Body::Plot(plot)) = &card.body else { continue };
            let plot_id = plot.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
            if preferences.contains_key(plot_id) {
                continue;
            }
            let key = format!("plot-settings:{}/{}/{}", fixed.execution.identifier, document_id, plot_id);
            let initial = PlotSettingsValue {
                display_points: None,
                downsample: Downsample::Auto as i32,
            }
            .encode_to_vec();
            let saved = administration::read(context, &key, &initial, None).await?;
            preferences.insert(plot_id.to_owned(), PlotSettingsValue::decode(saved.payload.as_slice())?);
        }
        sections.extend(section.sections.iter());
    }
    let identity = context.host.identity();
    if !preferences.is_empty() {
        context.host.store().retain_read_positions(&[Position {
            history: Some(History {
                store: reader.identity().into(),
                epoch: fixed.execution.identifier.clone(),
            }),
            sequence: fixed.frontier,
        }])?;
    }
    Ok(DocumentRead {
        document: Some(expanded.document),
        execution: Some(fixed.execution.clone()),
        state: match fixed.state {
            ExecutionState::Open => 0,
            ExecutionState::Closed => 1,
            ExecutionState::Sealed => 2,
        },
        seal: fixed.seal.clone(),
        frontier: fixed.frontier,
        frames: expanded.frames.into_iter().map(frame).collect(),
        preferences,
        backend_id: identity.backend_id,
        process_epoch: identity.epoch,
        cacheable: if cycle { reader.cycle_is_sealed(&fixed.execution.cycle)? } else { fixed.state == ExecutionState::Sealed },
    })
}

fn prepare_document(mut document: Document, fixed: &ExecutionCut, reader: &Store, page_items: usize) -> Result<expansion::Expansion> {
    append_parts::replay(reader, fixed, &mut document, page_items)?;
    let mut expanded = expansion::expand(reader, fixed, &document, 0, page_items)?;
    while let Some(next) = expanded.next {
        let page = expansion::expand(reader, fixed, &document, next, page_items)?;
        merge_sections(&mut expanded.document.sections, page.document.sections)?;
        for kind in page.frames {
            if !expanded.frames.contains(&kind) {
                expanded.frames.push(kind);
            }
        }
        expanded.next = page.next;
    }
    Ok(expanded)
}

fn response_prepared(request: &DocumentsGetRequest, saved: DocumentRead, reader: &Store) -> Result<DocumentsGetResponse> {
    let fixed = ExecutionCut {
        execution: saved.execution.ok_or_else(|| Error::Precondition("Get Document: Resynchronize Its Execution".into()))?,
        state: match saved.state {
            0 => ExecutionState::Open,
            1 => ExecutionState::Closed,
            2 => ExecutionState::Sealed,
            _ => return Err(Error::Precondition("Get Document: Resynchronize Its Execution State".into())),
        },
        seal: saved.seal,
        frontier: saved.frontier,
    };
    let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Get Document: Select Its Execution or Cycle".into()))?;
    let cycle_matches = scope.cycle_id == fixed.execution.cycle;
    let execution_matches = scope.execution_id.is_empty() || scope.execution_id == fixed.execution.identifier;
    if !cycle_matches || !execution_matches {
        return Err(Error::Invalid("Get Document: Keep the Evidence Cut Within Its Selected Scope".into()));
    }
    let mut document = saved.document.ok_or_else(|| Error::Precondition("Get Document: Resynchronize Its Declaration".into()))?;
    let mut expansion_frames = saved.frames;
    if !request.card_ids.is_empty() {
        select_cards(&mut document, &request.card_ids)?;
        expansion_frames.clear();
    }
    let evidence_scope = EvidenceScope {
        cycle_id: Some(fixed.execution.cycle.clone()),
        execution_id: (!scope.execution_id.is_empty()).then(|| fixed.execution.identifier.clone()),
        ..Default::default()
    };
    let telemetry = DocumentTelemetryCut {
        store: saved.backend_id,
        epoch: if scope.execution_id.is_empty() { format!("cycle:{}", fixed.execution.cycle) } else { fixed.execution.identifier.clone() },
        position: fixed.frontier,
    };
    let document_id = document
        .metadata
        .as_ref()
        .map(|metadata| metadata.identifier.clone())
        .filter(|id| !id.is_empty())
        .ok_or_else(|| Error::Invalid("Get Document: Select a Document with an Identity".into()))?;
    let mut frames = vec![frame(document_frame_frame::Kind::Header(DocumentHeader {
        document_id: document_id.clone(),
        metadata: document.metadata.clone(),
        scope: Some(evidence_scope.clone()),
        frontiers: Some(DocumentFrontiers {
            process_epoch: saved.process_epoch,
            state_revision: fixed.frontier,
            telemetry: Some(telemetry.clone()),
            evidence: Some(telemetry.clone()),
            ..Default::default()
        }),
    }))];
    let mut layout = Vec::new();
    let mut cards = Vec::new();
    let mut sections = document.sections.iter().rev().map(|section| (None, section)).collect::<Vec<_>>();
    let mut ordinal = 0;
    let mut section_count = 0;
    let mut identities = BTreeSet::new();
    while let Some((parent, section)) = sections.pop() {
        let section_ordinal = ordinal;
        let node_id = section.metadata.as_ref().map_or(format!("section-{ordinal}"), |metadata| metadata.identifier.clone());
        let mut metadata = section.metadata.clone().unwrap_or_default();
        metadata.identifier = node_id.clone();
        frames.push(frame(document_frame_frame::Kind::SectionIdentity(DocumentSectionIdentity {
            node_id: node_id.clone(),
            metadata: Some(metadata.clone()),
            unavailable: None,
        })));
        layout.push(frame(document_frame_frame::Kind::LayoutNode(DocumentLayoutNode {
            ordinal,
            parent_ordinal: parent,
            node: Some(DocumentLayoutNodeNode {
                kind: Some(document_layout_node_node::Kind::Section(DocumentSection {
                    node_id,
                    metadata: Some(metadata),
                    repeat: None,
                })),
            }),
        })));
        ordinal += 1;
        section_count += 1;
        for (index, card) in section.cards.iter().enumerate() {
            let id = card
                .metadata
                .as_ref()
                .filter(|metadata| !metadata.identifier.is_empty())
                .map_or(format!("card-{section_ordinal}-{index}"), |metadata| metadata.identifier.clone());
            if !identities.insert(id.clone()) {
                return Err(Error::Invalid("Get Document: Give Every Card a Distinct Identity".into()));
            }
            let mut metadata = card.metadata.clone().unwrap_or_default();
            metadata.identifier = id.clone();
            let body = match card.body.as_ref() {
                Some(card::Body::Markdown(value)) => document_card_body::Kind::Markdown(value.clone()),
                Some(card::Body::Table(table)) => document_card_body::Kind::Table(DocumentTable { row_count: table.rows.len() as u64 }),
                Some(card::Body::Summary(_)) => document_card_body::Kind::Summary(DocumentSummary { group_count: 1 }),
                Some(card::Body::Plot(plot)) => document_card_body::Kind::Owner(DocumentOwnerReference {
                    kind: DocumentCardKind::Plot as i32,
                    plot_id: plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()),
                    ..Default::default()
                }),
                Some(card::Body::Evidence(kind)) => document_card_body::Kind::Owner(DocumentOwnerReference {
                    kind: evidence_kind(*kind)? as i32,
                    ..Default::default()
                }),
                None => return Err(Error::Invalid("Get Document: Supply Each Card's Body".into())),
            };
            frames.push(frame(document_frame_frame::Kind::Card(DocumentCard {
                metadata: Some(metadata.clone()),
                body: Some(DocumentCardBody { kind: Some(body) }),
                filters: filters(card),
            })));
            let placement = card.layout.unwrap_or_default();
            layout.push(frame(document_frame_frame::Kind::LayoutNode(DocumentLayoutNode {
                ordinal,
                parent_ordinal: Some(section_ordinal),
                node: Some(DocumentLayoutNodeNode {
                    kind: Some(document_layout_node_node::Kind::Placement(DocumentPlacement {
                        card_id: id.clone(),
                        // An Undeclared Layout Leaves Both Spans Absent So Clients Use Full Width.
                        column_span: (placement.columns > 0).then(|| DocumentSpan {
                            base: placement.columns,
                            ..Default::default()
                        }),
                        row_span: placement.rows,
                        order: placement.order,
                        ..Default::default()
                    })),
                }),
            })));
            ordinal += 1;
            cards.push((id, metadata, card));
        }
        sections.extend(section.sections.iter().rev().map(|child| (Some(section_ordinal), child)));
    }
    if request.card_ids.iter().any(|id| !identities.contains(id)) {
        return Err(Error::Invalid("Get Document: Select Card IDs from This Document".into()));
    }
    // Define All Summary Sources Before Binding or Layout.
    for (id, metadata, card) in &cards {
        if let Some(card::Body::Summary(summary)) = &card.body {
            frames.push(frame(document_frame_frame::Kind::GroupDefinition(DocumentSummaryGroupDefinition {
                card_id: id.clone(),
                ordinal: 0,
                metadata: Some(metadata.clone()),
                tile_count: summary.tiles.len() as u64,
            })));
            for (ordinal, tile) in summary.tiles.iter().enumerate() {
                frames.push(frame(document_frame_frame::Kind::TileDefinition(DocumentSummaryTileDefinition {
                    card_id: id.clone(),
                    group_ordinal: 0,
                    ordinal: ordinal as u64,
                    metadata: tile.metadata.clone(),
                    value: tile.value.clone(),
                    filters: filters(card),
                    unit: Some(tile.unit.clone()),
                    ..Default::default()
                })));
            }
        }
    }
    for (id, metadata, _) in &cards {
        let result = document_binding_result::Kind::Pending(true);
        frames.push(frame(document_frame_frame::Kind::Binding(DocumentBinding {
            source_id: id.clone(),
            metadata: Some(metadata.clone()),
            result: Some(DocumentBindingResult { kind: Some(result) }),
        })));
    }
    let repeat_count = expansion_frames
        .iter()
        .filter(|frame| {
            matches!(frame.frame.as_ref().and_then(|frame| frame.kind.as_ref()), Some(document_frame_frame::Kind::RepeatSelection(_)))
        })
        .count() as u64;
    frames.extend(expansion_frames);
    frames.extend(layout);
    let mut completion = DocumentCompletion {
        card_count: cards.len() as u64,
        binding_count: cards.len() as u64,
        layout_node_count: ordinal,
        section_count,
        repeat_count,
        partial: request.structure_only || !request.card_ids.is_empty(),
        ..Default::default()
    };
    completion.frame_count = frames.len() as u64 + 1;
    frames.push(frame(document_frame_frame::Kind::StructureComplete(completion)));
    frames.push(frame(document_frame_frame::Kind::EvidenceCut(DocumentEvidenceCut {
        cut: Some(telemetry),
        telemetry: true,
        cacheable: saved.cacheable,
    })));
    if !request.structure_only {
        for (id, metadata, card) in cards {
            if !request.card_ids.is_empty() && !request.card_ids.contains(&id) {
                continue;
            }
            let mut binding = document_binding_result::Kind::Literal(true);
            match &card.body {
                Some(card::Body::Table(table)) => {
                    for (ordinal, row) in table.rows.iter().enumerate() {
                        frames.push(frame(document_frame_frame::Kind::TableRow(DocumentTableRow {
                            card_id: id.clone(),
                            ordinal: ordinal as u64,
                            values: Some(row.clone()),
                        })));
                    }
                }
                Some(card::Body::Summary(summary)) => {
                    frames.push(frame(document_frame_frame::Kind::Binding(DocumentBinding {
                        source_id: id.clone(),
                        metadata: Some(metadata.clone()),
                        result: Some(DocumentBindingResult {
                            kind: Some(document_binding_result::Kind::Summary(DocumentSummary { group_count: 1 })),
                        }),
                    })));
                    frames.push(frame(document_frame_frame::Kind::SummaryGroup(DocumentSummaryGroup {
                        card_id: id.clone(),
                        ordinal: 0,
                        metadata: Some(metadata.clone()),
                        tile_count: summary.tiles.len() as u64,
                    })));
                    for (ordinal, tile) in summary.tiles.iter().enumerate() {
                        let expression = tile
                            .value
                            .as_ref()
                            .ok_or_else(|| Error::Invalid("Get Document Summary: Supply Each Tile's Expression".into()))?;
                        let result = reader.document_value_filtered(&fixed, expression, &filters(card));
                        let (value, unavailable) = match result {
                            Ok(value) => (Some(value), None),
                            Err(error) => (None, Some(error.to_string())),
                        };
                        frames.push(frame(document_frame_frame::Kind::SummaryTile(DocumentSummaryTile {
                            card_id: id.clone(),
                            group_ordinal: 0,
                            ordinal: ordinal as u64,
                            metadata: tile.metadata.clone(),
                            value,
                            unavailable,
                            unit: Some(tile.unit.clone()),
                            ..Default::default()
                        })));
                    }
                }
                Some(card::Body::Plot(plot)) => {
                    let plot_id = plot.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
                    let preferences = saved
                        .preferences
                        .get(plot_id)
                        .ok_or_else(|| Error::Precondition("Get Document: Resynchronize Its Plot Settings".into()))?;
                    let prepared = (|| -> Result<DocumentPlotDefinition> {
                        let mut plot = plot.clone();
                        plot.filters.extend(card.filters.clone());
                        let mut selected = plot::request(&plot, evidence_scope.clone())?;
                        if let Some(points) = preferences.display_points {
                            selected.max_points = Some(u64::from(points));
                        }
                        if preferences.downsample != Downsample::Auto as i32 {
                            selected.technique = preferences.downsample;
                        }
                        let position = Position {
                            history: Some(History {
                                store: reader.identity().into(),
                                epoch: fixed.execution.identifier.clone(),
                            }),
                            sequence: fixed.frontier,
                        };
                        selected.at = Some(ReadCut {
                            positions: vec![position],
                            revisions: Vec::new(),
                        });
                        Ok(DocumentPlotDefinition {
                            request: Some(selected),
                            presentation: Some(plot::presentation(reader, &fixed, &plot)?),
                            metadata: plot.metadata.clone(),
                            z_expression: plot.z.clone(),
                        })
                    })();
                    binding = match prepared {
                        Ok(plot) => document_binding_result::Kind::Plot(plot),
                        Err(error) => document_binding_result::Kind::Failure(DocumentCardFailure {
                            code: "plot_binding_failed".into(),
                            message: format!("Get Document Plot: {error}"),
                            budget: None,
                        }),
                    };
                }
                Some(card::Body::Evidence(_)) if !card.filters.is_empty() => {
                    binding = document_binding_result::Kind::Failure(DocumentCardFailure {
                        code: "evidence_filters_unserved".into(),
                        message: "Get Document Evidence Card: Clear Unsupported Filters".into(),
                        budget: None,
                    })
                }
                _ => (),
            }
            if !matches!(card.body, Some(card::Body::Summary(_))) {
                frames.push(frame(document_frame_frame::Kind::Binding(DocumentBinding {
                    source_id: id.clone(),
                    metadata: Some(metadata),
                    result: Some(DocumentBindingResult { kind: Some(binding) }),
                })));
            }
            frames.push(frame(document_frame_frame::Kind::CardComplete(id)));
        }
    }
    completion.frame_count = frames.len() as u64 + 1;
    frames.push(frame(document_frame_frame::Kind::Completion(completion)));
    Ok(DocumentsGetResponse { frames, ..Default::default() })
}

fn select_cards(document: &mut Document, card_ids: &[String]) -> Result<()> {
    let mut found = BTreeSet::new();
    select_sections(&mut document.sections, card_ids, &mut found);
    if card_ids.iter().any(|id| !found.contains(id)) {
        return Err(Error::Invalid("Get Document: Select Card IDs from This Document".into()));
    }
    Ok(())
}

fn select_sections(sections: &mut Vec<Section>, card_ids: &[String], found: &mut BTreeSet<String>) {
    sections.retain_mut(|section| {
        section.cards.retain(|card| {
            let id = card.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
            if card_ids.iter().any(|selected| selected == id) {
                found.insert(id.to_owned());
                return true;
            }
            false
        });
        select_sections(&mut section.sections, card_ids, found);
        !section.cards.is_empty() || !section.sections.is_empty()
    });
}

fn merge_sections(target: &mut Vec<Section>, sections: Vec<Section>) -> Result<()> {
    for section in sections {
        let id = section
            .metadata
            .as_ref()
            .map(|metadata| metadata.identifier.as_str())
            .ok_or_else(|| Error::Invalid("Get Document: Expand Named Sections".into()))?;
        if let Some(existing) =
            target.iter_mut().find(|existing| existing.metadata.as_ref().is_some_and(|metadata| metadata.identifier == id))
        {
            if !section.cards.is_empty() {
                if !existing.cards.is_empty() {
                    return Err(Error::Invalid("Get Document: Expand Every Card Exactly Once".into()));
                }
                existing.cards = section.cards;
            }
            merge_sections(&mut existing.sections, section.sections)?;
        } else {
            target.push(section);
        }
    }
    Ok(())
}
fn frame(kind: document_frame_frame::Kind) -> DocumentFrame {
    DocumentFrame {
        frame: Some(DocumentFrameFrame { kind: Some(kind) }),
    }
}
fn filters(card: &Card) -> Vec<PlotFilter> {
    card.filters
        .iter()
        .map(|filter| PlotFilter {
            expression: filter.expression.clone(),
            operator: filter.operator,
            value: filter.value.clone(),
        })
        .collect()
}
fn evidence_kind(kind: i32) -> Result<DocumentCardKind> {
    Ok(match EvidenceCard::try_from(kind) {
        Ok(EvidenceCard::Steps) => DocumentCardKind::Steps,
        Ok(EvidenceCard::Criteria) => DocumentCardKind::Criteria,
        Ok(EvidenceCard::Traceability) => DocumentCardKind::Traceability,
        Ok(EvidenceCard::Notes) => DocumentCardKind::Notes,
        Ok(EvidenceCard::Records) => DocumentCardKind::Records,
        Ok(EvidenceCard::Artifacts) => DocumentCardKind::Artifacts,
        Ok(EvidenceCard::Media) => DocumentCardKind::Media,
        Ok(EvidenceCard::Health) => DocumentCardKind::Health,
        Ok(EvidenceCard::StreamContext) => DocumentCardKind::TelemetryContext,
        _ => return Err(Error::Invalid("Capture Document: Select a Declared Evidence Card".into())),
    })
}

/* === Tests === */

crate::native::unary!(DocumentsGetRequest, DocumentsGetResponse, context, "dokime.host.v1", "Documents", "Get");

#[cfg(test)]
mod tests {
    use super::{DocumentRead, frame, prepare_document, response_prepared};
    use crate::Error;
    use dokime_protocol::host::plots::PlotSettingsValue;
    use dokime_protocol::host::{
        DocumentFrame, DocumentScope, DocumentsGetRequest, DocumentsGetResponse, document_binding_result, document_frame_frame,
    };
    use dokime_protocol::model::{
        Aggregate, AggregateExpression, Card, DataExpression, Document, Downsample, FieldReference, Import, Metadata, Origin, Plot,
        PlotKind, Section, Summary, SummaryTile, Table, TableRow, Value, card, data_expression, origin, value,
    };
    use dokime_storage::{Evidence, Execution, Sample, Store};
    use harmos_protocol::model::{Declarations, StreamDeclaration};
    use prost::Message;
    use prost_types::{
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
        field_descriptor_proto::{Label, Type},
    };
    use std::collections::{BTreeMap, HashMap};

    #[derive(Clone, PartialEq, Message)]
    struct Row {
        #[prost(uint64, tag = "1")]
        value: u64,
    }

    fn fixture() -> (tempfile::TempDir, Store, DocumentRead, DocumentsGetRequest) {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        store
            .create(&Execution {
                identifier: "execution".into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
                }),
                catalog: Declarations {
                    types: Some(FileDescriptorSet {
                        file: vec![FileDescriptorProto {
                            name: Some("document.proto".into()),
                            package: Some("document".into()),
                            syntax: Some("proto3".into()),
                            message_type: vec![DescriptorProto {
                                name: Some("Row".into()),
                                field: vec![FieldDescriptorProto {
                                    name: Some("value".into()),
                                    number: Some(1),
                                    label: Some(Label::Optional as i32),
                                    r#type: Some(Type::Uint64 as i32),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            }],
                            ..Default::default()
                        }],
                    }),
                    streams: vec![StreamDeclaration {
                        id: "readings".into(),
                        row: "document.Row".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }
                .encode_to_vec(),
                ..Default::default()
            })
            .unwrap();
        store.open_stream("execution", "readings").unwrap();
        store
            .append(
                "execution",
                Evidence {
                    kind: "step:start".into(),
                    step: "measure".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        store
            .sample(
                "execution",
                Sample {
                    stream: "readings".into(),
                    source_ns: 10,
                    row: Row { value: u64::MAX }.encode_to_vec(),
                    ..Default::default()
                },
            )
            .unwrap();
        store
            .append(
                "execution",
                Evidence {
                    kind: "step:end".into(),
                    step: "measure".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let expression = DataExpression {
            kind: Some(data_expression::Kind::Stream(FieldReference {
                identifier: "readings".into(),
                field: "value".into(),
            })),
            ..Default::default()
        };
        let document = Document {
            metadata: Some(Metadata {
                identifier: "report".into(),
                ..Default::default()
            }),
            sections: vec![Section {
                metadata: Some(Metadata {
                    identifier: "results".into(),
                    ..Default::default()
                }),
                cards: vec![
                    Card {
                        metadata: Some(Metadata {
                            identifier: "table".into(),
                            ..Default::default()
                        }),
                        body: Some(card::Body::Table(Table {
                            rows: vec![TableRow {
                                cells: HashMap::from([(
                                    "value".into(),
                                    Value {
                                        kind: Some(value::Kind::Integer(i64::MIN)),
                                    },
                                )]),
                            }],
                        })),
                        ..Default::default()
                    },
                    Card {
                        metadata: Some(Metadata {
                            identifier: "summary".into(),
                            ..Default::default()
                        }),
                        body: Some(card::Body::Summary(Summary {
                            tiles: vec![SummaryTile {
                                metadata: Some(Metadata {
                                    identifier: "maximum".into(),
                                    ..Default::default()
                                }),
                                value: Some(DataExpression {
                                    kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                                        input: Some(Box::new(expression.clone())),
                                        kind: Aggregate::Last as i32,
                                        ..Default::default()
                                    }))),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    },
                    Card {
                        metadata: Some(Metadata {
                            identifier: "plot".into(),
                            ..Default::default()
                        }),
                        body: Some(card::Body::Plot(Plot {
                            metadata: Some(Metadata {
                                identifier: "signal".into(),
                                ..Default::default()
                            }),
                            kind: PlotKind::TimeSeries as i32,
                            y: vec![expression],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
        };
        let fixed = store.cut("execution").unwrap();
        let expanded = prepare_document(document, &fixed, &store, 1).unwrap();
        let saved = DocumentRead {
            document: Some(expanded.document),
            execution: Some(fixed.execution),
            frontier: fixed.frontier,
            frames: expanded.frames.into_iter().map(frame).collect(),
            preferences: BTreeMap::from([(
                "signal".into(),
                PlotSettingsValue {
                    display_points: Some(128),
                    downsample: Downsample::Lttb as i32,
                },
            )]),
            backend_id: store.identity().into(),
            process_epoch: "epoch".into(),
            ..Default::default()
        };
        let request = DocumentsGetRequest {
            scope: Some(DocumentScope {
                cycle_id: "cycle".into(),
                execution_id: "execution".into(),
            }),
            document_id: "report".into(),
            ..Default::default()
        };
        (root, store, saved, request)
    }

    fn results(response: &DocumentsGetResponse, id: &str) -> Vec<DocumentFrame> {
        response
            .frames
            .iter()
            .filter(|frame| match frame.frame.as_ref().and_then(|frame| frame.kind.as_ref()) {
                Some(document_frame_frame::Kind::TableRow(row)) => row.card_id == id,
                Some(document_frame_frame::Kind::SummaryTile(tile)) => tile.card_id == id,
                Some(document_frame_frame::Kind::SummaryGroup(group)) => group.card_id == id,
                Some(document_frame_frame::Kind::Binding(binding)) => binding.source_id == id,
                Some(document_frame_frame::Kind::CardComplete(card)) => card == id,
                _ => false,
            })
            .cloned()
            .collect()
    }

    #[test]
    fn retained_card_results() {
        let (_root, store, saved, request) = fixture();
        let (reader, _cancellation) = store.read_session().unwrap();
        let full = response_prepared(&request, saved.clone(), &reader).unwrap();
        store
            .append(
                "execution",
                Evidence {
                    kind: "step:start".into(),
                    step: "later".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        store
            .sample(
                "execution",
                Sample {
                    stream: "readings".into(),
                    source_ns: 20,
                    row: Row { value: 1 }.encode_to_vec(),
                    ..Default::default()
                },
            )
            .unwrap();
        store
            .append(
                "execution",
                Evidence {
                    kind: "step:end".into(),
                    step: "later".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        for id in ["table", "summary", "plot"] {
            let retained = DocumentRead::decode(saved.encode_to_vec().as_slice()).unwrap();
            let selected = response_prepared(
                &DocumentsGetRequest {
                    card_ids: vec![id.into()],
                    ..request.clone()
                },
                retained,
                &reader,
            )
            .unwrap();
            assert_eq!(selected.frames.first(), full.frames.first());
            assert_eq!(results(&selected, id), results(&full, id));
            for other in ["table", "summary", "plot"].into_iter().filter(|other| *other != id) {
                assert!(results(&selected, other).is_empty());
            }
        }
        let Some(document_frame_frame::Kind::SummaryTile(tile)) =
            full.frames.iter().find_map(|frame| match frame.frame.as_ref().and_then(|frame| frame.kind.as_ref()) {
                Some(kind @ document_frame_frame::Kind::SummaryTile(_)) => Some(kind),
                _ => None,
            })
        else {
            panic!("summary result missing")
        };
        assert_eq!(tile.value.as_ref().unwrap().kind, Some(value::Kind::UnsignedInteger(u64::MAX)));
        assert!(full.frames.iter().any(|frame| matches!(frame.frame.as_ref().and_then(|frame| frame.kind.as_ref()), Some(document_frame_frame::Kind::Binding(binding)) if matches!(binding.result.as_ref().and_then(|result| result.kind.as_ref()), Some(document_binding_result::Kind::Plot(_))))));
    }

    #[test]
    fn foreign_card_selectors() {
        let (_root, store, saved, mut request) = fixture();
        request.card_ids = vec!["missing".into()];
        assert!(matches!(response_prepared(&request, saved.clone(), &store), Err(Error::Invalid(_))));
        request.card_ids.clear();
        request.scope.as_mut().unwrap().cycle_id = "other".into();
        assert!(matches!(response_prepared(&request, saved, &store), Err(Error::Invalid(_))));
    }
}
