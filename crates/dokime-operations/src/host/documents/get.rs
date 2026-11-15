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
use prost::Message;

// Local Imports
use crate::host::storage::administration;
use crate::{
    Error, Result,
    host::{ClientSettings, Context},
};
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
use dokime_protocol::host::document_binding_result::Kind as DocumentBindingResultKind;
use dokime_protocol::host::document_card_body::Kind as DocumentCardBodyKind;
use dokime_protocol::host::document_frame_frame::Kind as DocumentFrameFrameKind;
use dokime_protocol::host::document_layout_node_node::Kind as DocumentLayoutNodeNodeKind;
use dokime_protocol::host::plots::PlotSettingsValue;
use dokime_protocol::model::Card;
use dokime_protocol::model::Downsample;
use dokime_protocol::model::PlotFilter;
use dokime_protocol::model::Section;
use dokime_storage::ExecutionState;
use std::collections::BTreeSet;

use dokime_protocol::model::EvidenceCard;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::card::Body as CardBody;

use harmos_protocol::model::History;
use harmos_protocol::model::OperationContract;
use harmos_protocol::model::Position;
use harmos_protocol::model::ReadCut;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Documents/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<DocumentsGetRequest, DocumentsGetResponse>(ID, settings.read_timeout)
}
#[tracing::instrument(name = "document.operation", skip_all)]
pub async fn handle(context: &Context, request: DocumentsGetRequest) -> Result<DocumentsGetResponse> {
    crate::host::complete(
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
            let key = format!("document/{}", crate::host::views::hex(&crate::host::paging::bytes(&selection)));
            let (saved, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
                let (mut document, fixed) = context.host.current_document(scope, &request.document_id)?;
                let (reader, _cancellation) = context.host.store().read_session()?;
                crate::host::documents::append_parts::replay(&reader, &fixed, &mut document, context.settings.page_items)?;
                let mut expanded = crate::host::documents::expansion::expand(&reader, &fixed, &document, 0, context.settings.page_items)?;
                while let Some(next) = expanded.next {
                    let page = crate::host::documents::expansion::expand(&reader, &fixed, &document, next, context.settings.page_items)?;
                    merge_sections(&mut expanded.document.sections, page.document.sections)?;
                    for kind in page.frames {
                        if !expanded.frames.contains(&kind) {
                            expanded.frames.push(kind);
                        }
                    }
                    expanded.next = page.next;
                }
                let document = expanded.document;
                let evidence_scope = EvidenceScope {
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
                            Some(CardBody::Markdown(value)) => DocumentCardBodyKind::Markdown(value.clone()),
                            Some(CardBody::Table(table)) => {
                                DocumentCardBodyKind::Table(DocumentTable { row_count: table.rows.len() as u64 })
                            }
                            Some(CardBody::Summary(_)) => DocumentCardBodyKind::Summary(DocumentSummary { group_count: 1 }),
                            Some(CardBody::Plot(plot)) => DocumentCardBodyKind::Owner(DocumentOwnerReference {
                                kind: DocumentCardKind::Plot as i32,
                                plot_id: plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()),
                                ..Default::default()
                            }),
                            Some(CardBody::Evidence(kind)) => DocumentCardBodyKind::Owner(DocumentOwnerReference {
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
                    if let Some(CardBody::Summary(summary)) = &card.body {
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
                for (id, metadata, _) in &cards {
                    let result = DocumentBindingResultKind::Pending(true);
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
                            Some(CardBody::Table(table)) => {
                                for (ordinal, row) in table.rows.iter().enumerate() {
                                    frames.push(frame(DocumentFrameFrameKind::TableRow(DocumentTableRow {
                                        card_id: id.clone(),
                                        ordinal: ordinal as u64,
                                        values: Some(row.clone()),
                                    })));
                                }
                            }
                            Some(CardBody::Summary(summary)) => {
                                frames.push(frame(DocumentFrameFrameKind::Binding(DocumentBinding {
                                    source_id: id.clone(),
                                    metadata: Some(metadata.clone()),
                                    result: Some(DocumentBindingResult {
                                        kind: Some(DocumentBindingResultKind::Summary(DocumentSummary { group_count: 1 })),
                                    }),
                                })));
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
                            }
                            Some(CardBody::Plot(plot)) => {
                                let plot_id = plot.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
                                let key = format!("plot-settings:{}/{}/{}", fixed.execution.identifier, request.document_id, plot_id);
                                let initial = PlotSettingsValue {
                                    display_points: None,
                                    downsample: Downsample::Auto as i32,
                                }
                                .encode_to_vec();
                                let preferences = administration::read(context, &key, &initial, None).await?;
                                let preferences = PlotSettingsValue::decode(preferences.payload.as_slice())?;
                                let prepared = (|| -> Result<DocumentPlotDefinition> {
                                    let mut plot = plot.clone();
                                    plot.filters.extend(card.filters.clone());
                                    let mut selected = crate::host::documents::plot::request(&plot, evidence_scope.clone())?;
                                    if let Some(points) = preferences.display_points {
                                        selected.max_points = Some(u64::from(points));
                                    }
                                    if preferences.downsample != Downsample::Auto as i32 {
                                        selected.technique = preferences.downsample;
                                    }
                                    let position = Position {
                                        history: Some(History {
                                            store: context.host.store().identity().into(),
                                            epoch: fixed.execution.identifier.clone(),
                                        }),
                                        sequence: fixed.frontier,
                                    };
                                    context.host.store().retain_read_positions(std::slice::from_ref(&position))?;
                                    selected.at = Some(ReadCut {
                                        positions: vec![position],
                                        revisions: Vec::new(),
                                    });
                                    Ok(DocumentPlotDefinition {
                                        request: Some(selected),
                                        presentation: Some(crate::host::documents::plot::presentation(&reader, &fixed, &plot)?),
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
                            Some(CardBody::Evidence(_)) if !card.filters.is_empty() => {
                                binding = DocumentBindingResultKind::Failure(DocumentCardFailure {
                                    code: "evidence_filters_unserved".into(),
                                    message: "Get Document Evidence Card: Clear Unsupported Filters".into(),
                                    budget: None,
                                })
                            }
                            _ => (),
                        }
                        if !matches!(card.body, Some(CardBody::Summary(_))) {
                            frames.push(frame(DocumentFrameFrameKind::Binding(DocumentBinding {
                                source_id: id.clone(),
                                metadata: Some(metadata),
                                result: Some(DocumentBindingResult { kind: Some(binding) }),
                            })));
                        }
                        frames.push(frame(DocumentFrameFrameKind::CardComplete(id)));
                    }
                }
                completion.frame_count = frames.len() as u64 + 1;
                frames.push(frame(DocumentFrameFrameKind::Completion(completion)));
                Ok(DocumentsGetResponse { frames, ..Default::default() })
            })
            .await?;
            let start = crate::host::views::offset(&request.page_token, &cut)?;
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
                reply.next_page_token = if offset + 1 < total { crate::host::views::token(offset + 1, &cut) } else { String::new() };
                if reply.encoded_len() > maximum {
                    reply.frames.pop();
                    if reply.frames.is_empty() {
                        return Err(Error::Bound("Get Document: Split This Frame to Fit the Reply Byte Bound".into()));
                    }
                    reply.next_page_token = crate::host::views::token(offset, &cut);
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
