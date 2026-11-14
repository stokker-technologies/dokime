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

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{
    Error, Result,
    client::{ClientSettings, Context},
};
use dokime_protocol::client::DocumentBinding;
use dokime_protocol::client::DocumentBindingResult;
use dokime_protocol::client::DocumentCard;
use dokime_protocol::client::DocumentCardBody;
use dokime_protocol::client::DocumentCardFailure;
use dokime_protocol::client::DocumentCardKind;
use dokime_protocol::client::DocumentCompletion;
use dokime_protocol::client::DocumentEvidenceCut;
use dokime_protocol::client::DocumentFrame;
use dokime_protocol::client::DocumentFrameFrame;
use dokime_protocol::client::DocumentFrontiers;
use dokime_protocol::client::DocumentHeader;
use dokime_protocol::client::DocumentLayoutNode;
use dokime_protocol::client::DocumentLayoutNodeNode;
use dokime_protocol::client::DocumentOwnerReference;
use dokime_protocol::client::DocumentPlacement;
use dokime_protocol::client::DocumentPlotDefinition;
use dokime_protocol::client::DocumentSection;
use dokime_protocol::client::DocumentSectionIdentity;
use dokime_protocol::client::DocumentSpan;
use dokime_protocol::client::DocumentSummary;
use dokime_protocol::client::DocumentSummaryGroup;
use dokime_protocol::client::DocumentSummaryGroupDefinition;
use dokime_protocol::client::DocumentSummaryTile;
use dokime_protocol::client::DocumentSummaryTileDefinition;
use dokime_protocol::client::DocumentTable;
use dokime_protocol::client::DocumentTableRow;
use dokime_protocol::client::DocumentTelemetryCut;
use dokime_protocol::client::DocumentsGetRequest;
use dokime_protocol::client::DocumentsGetResponse;
use dokime_protocol::client::document_binding_result::Kind as DocumentBindingResultKind;
use dokime_protocol::client::document_card_body::Kind as DocumentCardBodyKind;
use dokime_protocol::client::document_frame_frame::Kind as DocumentFrameFrameKind;
use dokime_protocol::client::document_layout_node_node::Kind as DocumentLayoutNodeNodeKind;
use dokime_protocol::v1;
use dokime_protocol::v1::{Card, PlotFilter, Section};
use dokime_storage::ExecutionState;
use std::collections::BTreeSet;

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Documents/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<DocumentsGetRequest, DocumentsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: DocumentsGetRequest) -> Result<DocumentsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        if !request.page_token.is_empty() && request.at.is_none() {
            return Err(Error::Invalid("Get Document: Continue with the Returned Cut".into()));
        }
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Get Document: Select Its Execution or Cycle".into()))?;
        let mut selection = request.clone();
        selection.at = None;
        selection.page_token.clear();
        selection.max_bytes = 0;
        let key = format!("document/{}", super::views::hex(&super::paging::bytes(&selection)));
        let (saved, cut) = super::views::read(context, &key, request.at.as_ref(), async {
            let (document, fixed) = context.host.current_document(scope, &request.document_id)?;
            let (reader, _cancellation) = context.host.store().read_session()?;
            let mut expanded = super::document_expansion::expand(&reader, &fixed, &document, 0, context.settings.page_items)?;
            while let Some(next) = expanded.next {
                let page = super::document_expansion::expand(&reader, &fixed, &document, next, context.settings.page_items)?;
                merge_sections(&mut expanded.document.sections, page.document.sections)?;
                for kind in page.frames {
                    if !expanded.frames.contains(&kind) {
                        expanded.frames.push(kind);
                    }
                }
                expanded.next = page.next;
            }
            let document = expanded.document;
            let evidence_scope = v1::EvidenceScope {
                cycle_id: Some(fixed.execution.cycle.clone()),
                execution_id: (!scope.execution_id.is_empty()).then(|| fixed.execution.identifier.clone()),
                ..Default::default()
            };
            let telemetry = DocumentTelemetryCut {
                store: context.host.identity().backend_id,
                epoch: if scope.execution_id.is_empty() {
                    format!("cycle:{}", fixed.execution.cycle)
                } else {
                    fixed.execution.identifier.clone()
                },
                position: fixed.frontier,
            };
            let mut frames = vec![frame(DocumentFrameFrameKind::Header(DocumentHeader {
                document_id: document
                    .metadata
                    .as_ref()
                    .map(|metadata| metadata.identifier.clone())
                    .filter(|id| !id.is_empty())
                    .ok_or_else(|| Error::Invalid("Get Document: Select a Document with an Identity".into()))?,
                metadata: document.metadata.clone(),
                scope: Some(evidence_scope.clone()),
                frontiers: Some(DocumentFrontiers {
                    process_epoch: context.host.identity().epoch,
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
                frames.push(frame(DocumentFrameFrameKind::SectionIdentity(DocumentSectionIdentity {
                    node_id: node_id.clone(),
                    metadata: Some(metadata.clone()),
                    unavailable: None,
                })));
                layout.push(frame(DocumentFrameFrameKind::LayoutNode(DocumentLayoutNode {
                    ordinal,
                    parent_ordinal: parent,
                    node: Some(DocumentLayoutNodeNode {
                        kind: Some(DocumentLayoutNodeNodeKind::Section(DocumentSection {
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
                        Some(v1::card::Body::Markdown(value)) => DocumentCardBodyKind::Markdown(value.clone()),
                        Some(v1::card::Body::Table(table)) => {
                            DocumentCardBodyKind::Table(DocumentTable { row_count: table.rows.len() as u64 })
                        }
                        Some(v1::card::Body::Summary(summary)) => DocumentCardBodyKind::Summary(DocumentSummary {
                            group_count: u64::from(!summary.tiles.is_empty()),
                        }),
                        Some(v1::card::Body::Plot(plot)) => DocumentCardBodyKind::Owner(DocumentOwnerReference {
                            kind: DocumentCardKind::Plot as i32,
                            plot_id: plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()),
                            ..Default::default()
                        }),
                        Some(v1::card::Body::Evidence(kind)) => DocumentCardBodyKind::Owner(DocumentOwnerReference {
                            kind: evidence_kind(*kind)? as i32,
                            ..Default::default()
                        }),
                        None => return Err(Error::Invalid("Get Document: Supply Each Card's Body".into())),
                    };
                    frames.push(frame(DocumentFrameFrameKind::Card(DocumentCard {
                        metadata: Some(metadata.clone()),
                        body: Some(DocumentCardBody { kind: Some(body) }),
                        filters: filters(card),
                    })));
                    let placement = card.layout.unwrap_or_default();
                    layout.push(frame(DocumentFrameFrameKind::LayoutNode(DocumentLayoutNode {
                        ordinal,
                        parent_ordinal: Some(section_ordinal),
                        node: Some(DocumentLayoutNodeNode {
                            kind: Some(DocumentLayoutNodeNodeKind::Placement(DocumentPlacement {
                                card_id: id.clone(),
                                column_span: Some(DocumentSpan {
                                    base: placement.columns.max(1),
                                    ..Default::default()
                                }),
                                row_span: placement.rows.max(1),
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
                if let Some(v1::card::Body::Summary(summary)) = &card.body {
                    frames.push(frame(DocumentFrameFrameKind::GroupDefinition(DocumentSummaryGroupDefinition {
                        card_id: id.clone(),
                        ordinal: 0,
                        metadata: Some(metadata.clone()),
                        tile_count: summary.tiles.len() as u64,
                    })));
                    for (ordinal, tile) in summary.tiles.iter().enumerate() {
                        frames.push(frame(DocumentFrameFrameKind::TileDefinition(DocumentSummaryTileDefinition {
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
            for (id, metadata, card) in &cards {
                let result = match &card.body {
                    Some(v1::card::Body::Summary(summary)) => DocumentBindingResultKind::Summary(DocumentSummary {
                        group_count: u64::from(!summary.tiles.is_empty()),
                    }),
                    Some(v1::card::Body::Markdown(_) | v1::card::Body::Table(_)) => DocumentBindingResultKind::Literal(true),
                    _ => DocumentBindingResultKind::Pending(true),
                };
                frames.push(frame(DocumentFrameFrameKind::Binding(DocumentBinding {
                    source_id: id.clone(),
                    metadata: Some(metadata.clone()),
                    result: Some(DocumentBindingResult { kind: Some(result) }),
                })));
            }
            let repeat_count =
                expanded.frames.iter().filter(|kind| matches!(kind, DocumentFrameFrameKind::RepeatSelection(_))).count() as u64;
            frames.extend(expanded.frames.into_iter().map(frame));
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
            frames.push(frame(DocumentFrameFrameKind::StructureComplete(completion)));
            frames.push(frame(DocumentFrameFrameKind::EvidenceCut(DocumentEvidenceCut {
                cut: Some(telemetry),
                telemetry: true,
                cacheable: if scope.execution_id.is_empty() {
                    context.host.store().cycle_is_sealed(&fixed.execution.cycle)?
                } else {
                    fixed.state == ExecutionState::Sealed
                },
            })));
            if !request.structure_only {
                for (id, metadata, card) in cards {
                    if !request.card_ids.is_empty() && !request.card_ids.contains(&id) {
                        continue;
                    }
                    let mut binding = DocumentBindingResultKind::Literal(true);
                    match &card.body {
                        Some(v1::card::Body::Table(table)) => {
                            for (ordinal, row) in table.rows.iter().enumerate() {
                                frames.push(frame(DocumentFrameFrameKind::TableRow(DocumentTableRow {
                                    card_id: id.clone(),
                                    ordinal: ordinal as u64,
                                    values: Some(row.clone()),
                                })));
                            }
                        }
                        Some(v1::card::Body::Summary(summary)) => {
                            frames.push(frame(DocumentFrameFrameKind::SummaryGroup(DocumentSummaryGroup {
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
                                frames.push(frame(DocumentFrameFrameKind::SummaryTile(DocumentSummaryTile {
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
                            binding = DocumentBindingResultKind::Summary(DocumentSummary {
                                group_count: u64::from(!summary.tiles.is_empty()),
                            });
                        }
                        Some(v1::card::Body::Plot(plot)) => {
                            let prepared = (|| -> Result<DocumentPlotDefinition> {
                                let mut plot = plot.clone();
                                plot.filters.extend(card.filters.clone());
                                let mut selected = super::document_plot::request(&plot, evidence_scope.clone())?;
                                if let Some((points, technique)) = context.host.store().plot_preferences(
                                    &fixed.execution.identifier,
                                    &request.document_id,
                                    plot.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str()),
                                )? {
                                    selected.max_points = points.map(u64::from);
                                    selected.technique = technique;
                                }
                                let position = h::Position {
                                    history: Some(h::History {
                                        store: context.host.store().identity().into(),
                                        epoch: fixed.execution.identifier.clone(),
                                    }),
                                    sequence: fixed.frontier,
                                };
                                context.host.store().retain_read_positions(std::slice::from_ref(&position))?;
                                selected.at = Some(h::ReadCut {
                                    positions: vec![position],
                                    revisions: Vec::new(),
                                });
                                let _envelope = super::plots_snapshots_list::snapshot_at(
                                    &reader,
                                    &fixed,
                                    &selected,
                                    context.settings.page_items,
                                    context.host.now_ns(),
                                    &context.host.identity().epoch,
                                )?;
                                Ok(DocumentPlotDefinition {
                                    request: Some(selected),
                                    presentation: Some(super::document_plot::presentation(&reader, &fixed, &plot)?),
                                    metadata: plot.metadata.clone(),
                                    z_expression: plot.z.clone(),
                                })
                            })();
                            binding = match prepared {
                                Ok(plot) => DocumentBindingResultKind::Plot(plot),
                                Err(error) => DocumentBindingResultKind::Failure(DocumentCardFailure {
                                    code: "plot_binding_failed".into(),
                                    message: format!("Get Document Plot: {error}"),
                                    budget: None,
                                }),
                            };
                        }
                        Some(v1::card::Body::Evidence(_)) if !card.filters.is_empty() => {
                            binding = DocumentBindingResultKind::Failure(DocumentCardFailure {
                                code: "evidence_filters_unserved".into(),
                                message: "Get Document Evidence Card: Clear Unsupported Filters".into(),
                                budget: None,
                            })
                        }
                        _ => (),
                    }
                    frames.push(frame(DocumentFrameFrameKind::Binding(DocumentBinding {
                        source_id: id.clone(),
                        metadata: Some(metadata),
                        result: Some(DocumentBindingResult { kind: Some(binding) }),
                    })));
                    frames.push(frame(DocumentFrameFrameKind::CardComplete(id)));
                }
            }
            completion.frame_count = frames.len() as u64 + 1;
            frames.push(frame(DocumentFrameFrameKind::Completion(completion)));
            Ok(DocumentsGetResponse { frames, ..Default::default() })
        })
        .await?;
        let start = super::views::offset(&request.page_token, &cut)?;
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
            reply.next_page_token = if offset + 1 < total { super::views::token(offset + 1, &cut) } else { String::new() };
            if reply.encoded_len() > maximum {
                reply.frames.pop();
                if reply.frames.is_empty() {
                    return Err(Error::Bound("Get Document: Split This Frame to Fit the Reply Byte Bound".into()));
                }
                reply.next_page_token = super::views::token(offset, &cut);
                break;
            }
            if reply.frames.len() == context.settings.page_items {
                break;
            }
        }
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

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
fn frame(kind: DocumentFrameFrameKind) -> DocumentFrame {
    DocumentFrame {
        frame: Some(DocumentFrameFrame { kind: Some(kind) }),
    }
}
fn filters(card: &Card) -> Vec<PlotFilter> {
    card.filters
        .iter()
        .map(|filter| v1::PlotFilter {
            expression: filter.expression.clone(),
            operator: filter.operator,
            value: filter.value.clone(),
        })
        .collect()
}
fn evidence_kind(kind: i32) -> Result<DocumentCardKind> {
    Ok(match v1::EvidenceCard::try_from(kind) {
        Ok(v1::EvidenceCard::Steps) => DocumentCardKind::Steps,
        Ok(v1::EvidenceCard::Criteria) => DocumentCardKind::Criteria,
        Ok(v1::EvidenceCard::Traceability) => DocumentCardKind::Traceability,
        Ok(v1::EvidenceCard::Notes) => DocumentCardKind::Notes,
        Ok(v1::EvidenceCard::Records) => DocumentCardKind::Records,
        Ok(v1::EvidenceCard::Artifacts) => DocumentCardKind::Artifacts,
        Ok(v1::EvidenceCard::Media) => DocumentCardKind::Media,
        Ok(v1::EvidenceCard::Health) => DocumentCardKind::Health,
        Ok(v1::EvidenceCard::StreamContext) => DocumentCardKind::TelemetryContext,
        _ => return Err(Error::Invalid("Capture Document: Select a Declared Evidence Card".into())),
    })
}

/* === Tests === */
