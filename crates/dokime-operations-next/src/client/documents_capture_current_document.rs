/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Capture a Fixed Document View

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{
    Error, Result,
    client::{Context, Output},
};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.documents.capture_current_document";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::DocumentsCaptureCurrentDocumentRequest, v1::DocumentsCaptureCurrentDocumentOutput>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: v1::DocumentsCaptureCurrentDocumentRequest,
) -> Result<Output<v1::DocumentsCaptureCurrentDocumentOutput>> {
    let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Capture Document: Select Its Execution and Cycle".into()))?;
    if request.page.is_some() {
        return Err(Error::Invalid("Capture Document: Refresh the Initial Page Before Selecting a Repeated Section".into()));
    }
    let (document, cut) = context.host.current_document(scope, &request.document_id)?;
    let context = context.clone();
    let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
    tokio::spawn(async move {
        let work = async {
            use v1::document_frame_frame::Kind;
            let evidence_scope = v1::EvidenceScope {
                cycle_id: Some(cut.execution.cycle.clone()),
                execution_id: Some(cut.execution.identifier.clone()),
                ..Default::default()
            };
            let telemetry = v1::DocumentTelemetryCut {
                store: context.host.identity().backend_id.clone(),
                epoch: cut.execution.identifier.clone(),
                position: cut.frontier,
            };
            let header = v1::DocumentHeader {
                document_id: document.metadata.as_ref().map_or(request.document_id.clone(), |metadata| metadata.identifier.clone()),
                metadata: document.metadata.clone(),
                scope: Some(evidence_scope.clone()),
                frontiers: Some(v1::DocumentFrontiers {
                    process_epoch: context.host.identity().epoch,
                    state_revision: cut.frontier,
                    telemetry: Some(telemetry.clone()),
                    evidence: Some(telemetry.clone()),
                    ..Default::default()
                }),
            };
            send(&sender, &context, Kind::Header(header)).await?;
            send(
                &sender,
                &context,
                Kind::EvidenceCut(v1::DocumentEvidenceCut {
                    cut: Some(telemetry),
                    telemetry: true,
                    cacheable: cut.state == dokime_storage::ExecutionState::Sealed,
                }),
            )
            .await?;
            let mut sections = document.sections.iter().rev().map(|section| (None, section)).collect::<Vec<_>>();
            let mut cards = Vec::new();
            let mut ordinal = 0;
            let mut section_count = 0;
            let mut ids = std::collections::BTreeSet::new();
            while let Some((parent, section)) = sections.pop() {
                if section.repeat.is_some() {
                    return Err(Error::Invalid("Capture Document: Expand the Repeated Section Against Its Fixed Evidence Cut".into()));
                }
                let section_ordinal = ordinal;
                let node_id = section.metadata.as_ref().map_or(format!("section-{ordinal}"), |metadata| metadata.identifier.clone());
                send(
                    &sender,
                    &context,
                    Kind::LayoutNode(v1::DocumentLayoutNode {
                        ordinal,
                        parent_ordinal: parent,
                        node: Some(v1::DocumentLayoutNodeNode {
                            kind: Some(v1::document_layout_node_node::Kind::Section(v1::DocumentSection {
                                node_id,
                                metadata: section.metadata.clone(),
                                repeat: None,
                            })),
                        }),
                    }),
                )
                .await?;
                ordinal += 1;
                section_count += 1;
                for (index, card) in section.cards.iter().enumerate() {
                    let id = card
                        .metadata
                        .as_ref()
                        .filter(|metadata| !metadata.identifier.is_empty())
                        .map_or(format!("card-{section_ordinal}-{index}"), |metadata| metadata.identifier.clone());
                    if !ids.insert(id.clone()) {
                        return Err(Error::Invalid("Capture Document: Give Every Card a Distinct ID".into()));
                    }
                    let layout = card.layout.unwrap_or_default();
                    send(
                        &sender,
                        &context,
                        Kind::LayoutNode(v1::DocumentLayoutNode {
                            ordinal,
                            parent_ordinal: Some(section_ordinal),
                            node: Some(v1::DocumentLayoutNodeNode {
                                kind: Some(v1::document_layout_node_node::Kind::Placement(v1::DocumentPlacement {
                                    card_id: id.clone(),
                                    column_span: Some(v1::DocumentSpan {
                                        base: layout.columns.max(1),
                                        ..Default::default()
                                    }),
                                    row_span: layout.rows.max(1),
                                    order: layout.order,
                                    ..Default::default()
                                })),
                            }),
                        }),
                    )
                    .await?;
                    ordinal += 1;
                    cards.push((id, card));
                }
                sections.extend(section.sections.iter().rev().map(|child| (Some(section_ordinal), child)));
            }
            if request.card_ids.iter().any(|id| !ids.contains(id)) {
                return Err(Error::Invalid("Capture Document: Select Card IDs from This Document".into()));
            }
            let mut completion = v1::DocumentCompletion {
                layout_node_count: ordinal,
                section_count,
                card_count: cards.len() as u64,
                ..Default::default()
            };
            send(&sender, &context, Kind::StructureComplete(completion)).await?;
            for (id, card) in cards {
                if !request.card_ids.is_empty() && !request.card_ids.contains(&id) {
                    continue;
                }
                let mut metadata = card.metadata.clone().unwrap_or_default();
                metadata.identifier = id.clone();
                let body = match card.body.as_ref() {
                    Some(v1::card::Body::Markdown(text)) => v1::document_card_body::Kind::Markdown(text.clone()),
                    Some(v1::card::Body::Table(table)) => {
                        v1::document_card_body::Kind::Table(v1::DocumentTable { row_count: table.rows.len() as u64 })
                    }
                    Some(v1::card::Body::Summary(summary)) => v1::document_card_body::Kind::Summary(v1::DocumentSummary {
                        group_count: u64::from(!summary.tiles.is_empty()),
                    }),
                    Some(v1::card::Body::Plot(plot)) => v1::document_card_body::Kind::Owner(v1::DocumentOwnerReference {
                        kind: v1::DocumentCardKind::Plot as i32,
                        plot_id: plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()),
                        plot: None,
                    }),
                    Some(v1::card::Body::Evidence(kind)) => v1::document_card_body::Kind::Owner(v1::DocumentOwnerReference {
                        kind: evidence_kind(*kind)? as i32,
                        ..Default::default()
                    }),
                    None => return Err(Error::Invalid("Capture Document: Supply Each Card's Body".into())),
                };
                send(
                    &sender,
                    &context,
                    Kind::Card(v1::DocumentCard {
                        metadata: Some(metadata),
                        body: Some(v1::DocumentCardBody { kind: Some(body) }),
                        filters: card
                            .filters
                            .iter()
                            .map(|filter| v1::PlotFilter {
                                expression: filter.expression.clone(),
                                operator: filter.operator,
                                value: filter.value.clone(),
                            })
                            .collect(),
                    }),
                )
                .await?;
                if request.structure_only {
                    continue;
                }
                match card.body.as_ref() {
                    Some(v1::card::Body::Table(table)) => {
                        for (ordinal, row) in table.rows.iter().enumerate() {
                            send(
                                &sender,
                                &context,
                                Kind::TableRow(v1::DocumentTableRow {
                                    card_id: id.clone(),
                                    ordinal: ordinal as u64,
                                    values: Some(row.clone()),
                                }),
                            )
                            .await?;
                        }
                    }
                    Some(v1::card::Body::Summary(summary)) => {
                        send(
                            &sender,
                            &context,
                            Kind::SummaryGroup(v1::DocumentSummaryGroup {
                                card_id: id.clone(),
                                ordinal: 0,
                                metadata: card.metadata.clone(),
                                tile_count: summary.tiles.len() as u64,
                            }),
                        )
                        .await?;
                        for (ordinal, tile) in summary.tiles.iter().enumerate() {
                            let expression = tile
                                .value
                                .as_ref()
                                .ok_or_else(|| Error::Invalid("Capture Summary: Supply Each Tile's Value Expression".into()))?;
                            let result = context.host.store().document_value(&cut, expression);
                            let (value, unavailable) = match result {
                                Ok(value) => (Some(value), None),
                                Err(error) => (None, Some(error.to_string())),
                            };
                            send(
                                &sender,
                                &context,
                                Kind::SummaryTile(v1::DocumentSummaryTile {
                                    card_id: id.clone(),
                                    group_ordinal: 0,
                                    ordinal: ordinal as u64,
                                    metadata: tile.metadata.clone(),
                                    value,
                                    unit: Some(tile.unit.clone()),
                                    unavailable,
                                    ..Default::default()
                                }),
                            )
                            .await?;
                        }
                    }
                    Some(v1::card::Body::Plot(plot)) => {
                        let mut plot_request = super::document_plot::request(plot, evidence_scope.clone())?;
                        plot_request.at = Some(v1::StorageCursor {
                            history: Some(v1::StoreHistory {
                                store: context.host.identity().backend_id,
                                epoch: cut.execution.identifier.clone(),
                            }),
                            sequence: cut.frontier,
                        });
                        if let Some((points, technique)) = context.host.store().plot_preferences(
                            &cut.execution.identifier,
                            &request.document_id,
                            plot.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str()),
                        )? {
                            plot_request.max_points = points.map(u64::from);
                            plot_request.technique = technique;
                        }
                        let definition = v1::DocumentPlotDefinition {
                            metadata: plot.metadata.clone(),
                            request: Some(plot_request),
                            presentation: Some(v1::PlotPresentation { style: plot.style, ..Default::default() }),
                            z_expression: plot.z.clone(),
                        };
                        send(
                            &sender,
                            &context,
                            Kind::Binding(v1::DocumentBinding {
                                source_id: id.clone(),
                                metadata: None,
                                result: Some(v1::DocumentBindingResult {
                                    kind: Some(v1::document_binding_result::Kind::Plot(definition)),
                                }),
                            }),
                        )
                        .await?;
                        completion.binding_count += 1;
                    }
                    _ => (),
                }
                send(&sender, &context, Kind::CardComplete(id)).await?;
            }
            send(&sender, &context, Kind::Completion(completion)).await?;
            sender
                .send(Ok(v1::DocumentsCaptureCurrentDocumentOutput {
                    value: Some(v1::documents_capture_current_document_output::Value::Response(v1::ClientEmpty {})),
                }))
                .await
                .map_err(|_| Error::Cancelled)?;
            Ok(())
        };
        if let Err(error) = super::complete(ID, context.settings.read_timeout, work).await {
            let _ = sender.send(Err(super::status(error))).await;
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(output))
}

/* === Private Functions === */

async fn send(
    sender: &tokio::sync::mpsc::Sender<std::result::Result<v1::DocumentsCaptureCurrentDocumentOutput, tonic::Status>>, context: &Context,
    kind: v1::document_frame_frame::Kind,
) -> Result<()> {
    let frame = v1::DocumentsCaptureCurrentDocumentOutput {
        value: Some(v1::documents_capture_current_document_output::Value::Item(v1::DocumentFrame {
            frame: Some(v1::DocumentFrameFrame { kind: Some(kind) }),
        })),
    };
    if frame.encoded_len() > context.settings.message_bytes {
        return Err(Error::Invalid("Capture Document: Split This Card or Row to Fit the Message Budget".into()));
    }
    sender.send(Ok(frame)).await.map_err(|_| Error::Cancelled)
}
fn evidence_kind(kind: i32) -> Result<v1::DocumentCardKind> {
    Ok(match v1::EvidenceCard::try_from(kind) {
        Ok(v1::EvidenceCard::Steps) => v1::DocumentCardKind::Steps,
        Ok(v1::EvidenceCard::Criteria) => v1::DocumentCardKind::Criteria,
        Ok(v1::EvidenceCard::Traceability) => v1::DocumentCardKind::Traceability,
        Ok(v1::EvidenceCard::Notes) => v1::DocumentCardKind::Notes,
        Ok(v1::EvidenceCard::Records) => v1::DocumentCardKind::Records,
        Ok(v1::EvidenceCard::Artifacts) => v1::DocumentCardKind::Artifacts,
        Ok(v1::EvidenceCard::Media) => v1::DocumentCardKind::Media,
        Ok(v1::EvidenceCard::Health) => v1::DocumentCardKind::Health,
        Ok(v1::EvidenceCard::StreamContext) => v1::DocumentCardKind::TelemetryContext,
        _ => return Err(Error::Invalid("Capture Document: Select a Declared Evidence Card".into())),
    })
}

/* === Tests === */
