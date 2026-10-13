/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::VecDeque;
// Third Party Imports
use dokime_contract::operations::documents::DocumentFrameFrame as Frame;
use harmos::Protobuf;
use tonic::Status;
// Local Imports
use super::super::value::{self, canonical};
use dokime_app::{CurrentDocumentCapture, document as native};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Frames {
    frames: VecDeque<dokime_contract::operations::documents::DocumentFrame>,
    bytes: usize,
    limit: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Frames {
    fn push(&mut self, frame: Frame) -> Result<(), Status> {
        let frame = dokime_contract::operations::documents::DocumentFrame { frame: Some(frame) };
        let bytes = frame.encoded_len();
        if bytes > self.limit
            || self.bytes.saturating_add(bytes) > 32 * 1024 * 1024
            || self.frames.len() >= 20_000
        {
            return Err(Status::resource_exhausted(
                "document response budget exceeded",
            ));
        }
        self.bytes += bytes;
        self.frames.push_back(frame);
        Ok(())
    }
}

/* === Public Functions === */
pub(super) fn frames(
    captured: CurrentDocumentCapture,
    limit: usize,
) -> Result<VecDeque<dokime_contract::operations::documents::DocumentFrame>, Status> {
    use super::super::{
        enums::canonical as enums, history::document::canonical as authoring, plots,
    };
    use native::capture::CardBindingResult;
    let mut output = Frames {
        frames: VecDeque::new(),
        bytes: 0,
        limit: limit.min(512 * 1024),
    };
    let capture = captured.capture;
    let definition = capture.definition;
    let scope = canonical::scope(&capture.selection.scope)?;
    let telemetry = captured.telemetry_cut.map(|cut| {
        use dokime_app::CurrentDocumentTelemetryCut as ReadCut;
        let (delivered, source, ordinal) = match cut {
            ReadCut::Window { delivered } => (
                delivered,
                dokime_contract::operations::documents::DocumentTelemetryCutSource::RetainedWindow(
                    true,
                ),
                None,
            ),
            ReadCut::Series {
                epoch,
                delivered,
                ordinal,
            } => (
                delivered,
                dokime_contract::operations::documents::DocumentTelemetryCutSource::SeriesEpoch(
                    epoch,
                ),
                Some(ordinal),
            ),
        };
        dokime_contract::operations::documents::DocumentTelemetryCut {
            delivered_cursor: delivered,
            source: Some(source),
            series_ordinal: ordinal,
        }
    });
    output.push(Frame::Header(
        dokime_contract::operations::documents::DocumentHeader {
            document_id: definition.document_id.to_string(),
            metadata: Some(canonical::metadata(&definition.metadata)?),
            scope: Some(scope),
            frontiers: Some(dokime_contract::operations::documents::DocumentFrontiers {
                process_epoch: captured.process_epoch,
                state_revision: captured.state_revision,
                telemetry,
                artifacts_sha256: captured.artifacts_sha256,
            }),
        },
    ))?;
    let mut ordinal = 0;
    layout_nodes(&mut output, definition.layout.nodes, None, &mut ordinal)?;
    let card_count = definition.cards.len() as u64;
    for card in definition.cards {
        use dokime_contract::operations::documents::DocumentCardBody as Wire;
        use native::DocumentCardBody as Body;
        let id = card.metadata.identifier.to_string();
        let body = match &card.body {
            Body::Text { markdown } => Wire::Markdown(markdown.clone()),
            Body::Table { rows } => {
                Wire::Table(dokime_contract::operations::documents::DocumentTable {
                    row_count: rows.len() as u64,
                })
            }
            Body::TelemetrySummary { summary } => {
                Wire::Summary(dokime_contract::operations::documents::DocumentSummary {
                    group_count: summary.groups.len() as u64,
                })
            }
            body => {
                let kind = match body {
                    Body::Plot { .. } => {
                        dokime_contract::operations::documents::DocumentCardKind::Plot
                    }
                    Body::Steps => dokime_contract::operations::documents::DocumentCardKind::Steps,
                    Body::Criteria => {
                        dokime_contract::operations::documents::DocumentCardKind::Criteria
                    }
                    Body::Traceability => {
                        dokime_contract::operations::documents::DocumentCardKind::Traceability
                    }
                    Body::Notes => dokime_contract::operations::documents::DocumentCardKind::Notes,
                    Body::Records => {
                        dokime_contract::operations::documents::DocumentCardKind::Records
                    }
                    Body::Artifacts => {
                        dokime_contract::operations::documents::DocumentCardKind::Artifacts
                    }
                    Body::Media => dokime_contract::operations::documents::DocumentCardKind::Media,
                    Body::Health => {
                        dokime_contract::operations::documents::DocumentCardKind::Health
                    }
                    Body::TelemetryContext => {
                        dokime_contract::operations::documents::DocumentCardKind::TelemetryContext
                    }
                    _ => unreachable!("literal cards handled above"),
                };
                let plot = if let Body::Plot { plot } = body {
                    let mut wire = authoring::historical_plot(plot, &capture.selection.scope)?;
                    // A Plot Reading One Telemetry Stream Names It, As Live Requests Do.
                    if let Some(request) = wire.request.as_mut()
                        && request.stream.is_empty()
                        && let Some(stream) = single_stream(plot)
                    {
                        request.stream = stream.to_owned();
                    }
                    Some(wire)
                } else {
                    None
                };
                Wire::Owner(Box::new(
                    dokime_contract::operations::documents::DocumentOwnerReference {
                        kind,
                        plot_id: plot
                            .as_ref()
                            .and_then(|plot| plot.request.as_ref()?.plot_id.clone()),
                        plot,
                    },
                ))
            }
        };
        output.push(Frame::Card(
            dokime_contract::operations::documents::DocumentCard {
                metadata: Some(canonical::metadata(&card.metadata)?),
                filters: card
                    .filters
                    .iter()
                    .map(plots::plot_filter_wire)
                    .collect::<Result<_, _>>()?,
                body: Some(body),
            },
        ))?;
        if let Body::Table { rows } = card.body {
            for (ordinal, fields) in rows.into_iter().enumerate() {
                native::bounded::document_size(&fields, output.limit.saturating_sub(8192) / 2, 32)
                    .map_err(|_| Status::resource_exhausted("document row exceeds frame budget"))?;
                value::collection(fields.len())?;
                let mut budget = value::Budget::new();
                output.push(Frame::TableRow(
                    dokime_contract::operations::documents::DocumentTableRow {
                        card_id: id.clone(),
                        ordinal: ordinal as u64,
                        values: Some(canonical::object(&fields, &mut budget)?),
                    },
                ))?;
            }
        } else if let Body::TelemetrySummary { summary } = card.body {
            for (group_ordinal, group) in summary.groups.into_iter().enumerate() {
                output.push(Frame::GroupDefinition(
                    dokime_contract::operations::documents::DocumentSummaryGroupDefinition {
                        card_id: id.clone(),
                        ordinal: group_ordinal as u64,
                        metadata: Some(canonical::metadata(&group.metadata)?),
                        tile_count: group.tiles.len() as u64,
                    },
                ))?;
                for (ordinal, tile) in group.tiles.into_iter().enumerate() {
                    native::bounded::document_size(
                        &tile,
                        output.limit.saturating_sub(8192) / 2,
                        32,
                    )
                    .map_err(|_| {
                        Status::resource_exhausted("summary declaration exceeds frame budget")
                    })?;
                    let conditions = tile
                        .conditions
                        .into_iter()
                        .map(|condition| {
                            Ok(
                                dokime_contract::operations::documents::DocumentSummaryCondition {
                                    operator: enums::filter_operator(&condition.operator),
                                    value: Some(canonical::encode(
                                        &condition.value,
                                        &mut value::Budget::new(),
                                        0,
                                    )?),
                                    state: summary_state(condition.state),
                                },
                            )
                        })
                        .collect::<Result<_, Status>>()?;
                    output.push(Frame::TileDefinition(
                        dokime_contract::operations::documents::DocumentSummaryTileDefinition {
                            card_id: id.clone(),
                            group_ordinal: group_ordinal as u64,
                            ordinal: ordinal as u64,
                            metadata: Some(canonical::metadata(&tile.metadata)?),
                            value: Some(plots::data_expression_wire(&tile.value)?),
                            filters: tile
                                .filters
                                .iter()
                                .map(plots::plot_filter_wire)
                                .collect::<Result<_, _>>()?,
                            stat: summary_stat(tile.stat),
                            format: summary_format(tile.format),
                            presentation: summary_presentation(tile.presentation),
                            unit: tile.unit,
                            conditions,
                        },
                    ))?;
                }
            }
        }
    }
    let binding_count = capture.cards.len() as u64;
    for binding in capture.cards {
        let id = binding.metadata.identifier.to_string();
        let result = match &binding.result {
            CardBindingResult::Static => {
                dokime_contract::operations::documents::DocumentBindingResult::Literal(true)
            }
            CardBindingResult::Summary { groups } => {
                dokime_contract::operations::documents::DocumentBindingResult::Summary(
                    dokime_contract::operations::documents::DocumentSummary {
                        group_count: groups.len() as u64,
                    },
                )
            }
            CardBindingResult::Plot { descriptor } => {
                dokime_contract::operations::documents::DocumentBindingResult::Plot(Box::new(
                    plots::document_plot_definition(descriptor)?,
                ))
            }
        };
        output.push(Frame::Binding(
            dokime_contract::operations::documents::DocumentBinding {
                source_id: binding.source_id.to_string(),
                metadata: Some(canonical::metadata(&binding.metadata)?),
                result: Some(result),
            },
        ))?;
        if let CardBindingResult::Summary { groups } = binding.result {
            for (group_ordinal, group) in groups.into_iter().enumerate() {
                output.push(Frame::SummaryGroup(
                    dokime_contract::operations::documents::DocumentSummaryGroup {
                        card_id: id.clone(),
                        ordinal: group_ordinal as u64,
                        metadata: Some(canonical::metadata(&group.metadata)?),
                        tile_count: group.tiles.len() as u64,
                    },
                ))?;
                for (ordinal, tile) in group.tiles.into_iter().enumerate() {
                    native::bounded::document_size(
                        &tile,
                        output.limit.saturating_sub(8192) / 2,
                        32,
                    )
                    .map_err(|_| {
                        Status::resource_exhausted("document summary exceeds frame budget")
                    })?;
                    if !tile.valid_result() {
                        return Err(Status::internal("invalid summary unavailable result"));
                    }
                    let encoded = tile
                        .value
                        .as_ref()
                        .map(|item| canonical::encode(item, &mut value::Budget::new(), 0))
                        .transpose()?;
                    output.push(Frame::SummaryTile(
                        dokime_contract::operations::documents::DocumentSummaryTile {
                            card_id: id.clone(),
                            group_ordinal: group_ordinal as u64,
                            ordinal: ordinal as u64,
                            metadata: Some(canonical::metadata(&tile.metadata)?),
                            value: encoded,
                            stat: summary_stat(tile.stat),
                            format: summary_format(tile.format),
                            presentation: summary_presentation(tile.presentation),
                            unit: tile.unit,
                            state: tile.state.map(summary_state),
                            unavailable: tile.unavailable,
                        },
                    ))?;
                }
            }
        }
    }
    let repeat_count = capture.repeats.len() as u64;
    for (id, instances) in capture.repeats {
        output.push(Frame::RepeatSelection(
            dokime_contract::operations::documents::DocumentRepeatSelection {
                node_id: id.to_string(),
                instance_count: instances.len() as u64,
            },
        ))?;
        for (ordinal, instance) in instances.into_iter().enumerate() {
            output.push(Frame::RepeatInstance(
                dokime_contract::operations::documents::DocumentRepeatInstance {
                    selection_id: id.to_string(),
                    ordinal: ordinal as u64,
                    suffix: instance.suffix,
                    node_id: instance.node_id.to_string(),
                    filter: Some(plots::plot_filter_wire(&instance.filter)?),
                },
            ))?;
        }
    }
    let section_count = capture.sections.len() as u64;
    for (node_id, section) in capture.sections {
        if !section.valid_result() {
            return Err(Status::internal("invalid section unavailable reason"));
        }
        output.push(Frame::SectionIdentity(
            dokime_contract::operations::documents::DocumentSectionIdentity {
                node_id: node_id.to_string(),
                metadata: Some(canonical::metadata(&section.metadata)?),
                unavailable: section.unavailable,
            },
        ))?;
    }
    for warning in capture.warnings {
        output.push(Frame::Warning(
            dokime_contract::operations::documents::DocumentWarning {
                code: warning.code.to_string(),
                message: warning.message,
            },
        ))?;
    }
    output.push(Frame::Completion(
        dokime_contract::operations::documents::DocumentCompletion {
            frame_count: output.frames.len() as u64 + 1,
            card_count,
            layout_node_count: ordinal,
            binding_count,
            repeat_count,
            section_count,
        },
    ))?;
    Ok(output.frames)
}

/* === Private Functions === */
fn single_stream(plot: &native::PlotDefinition) -> Option<&str> {
    use dokime_app::expressions::{DataSource, DataTransform};
    let mut streams = std::collections::BTreeSet::new();
    for expression in plot
        .y_expressions
        .iter()
        .chain(&plot.x_expression)
        .chain(&plot.series_by_expression)
        .chain(&plot.z_expression)
    {
        if let DataSource::Telemetry { stream, .. } = &expression.source {
            streams.insert(stream.as_str());
        }
        for transform in &expression.transforms {
            if let DataTransform::TelemetryAtOrBefore { stream, .. } = transform {
                streams.insert(stream.as_str());
            }
        }
    }
    let mut streams = streams.into_iter();
    match (streams.next(), streams.next()) {
        (Some(stream), None) => Some(stream),
        _ => None,
    }
}
fn layout_nodes(
    output: &mut Frames,
    nodes: Vec<native::DocumentNode>,
    parent: Option<u64>,
    ordinal: &mut u64,
) -> Result<(), Status> {
    for node in nodes {
        let current = *ordinal;
        *ordinal += 1;
        let (node, children) = match node {
            native::DocumentNode::Section(section) => (
                dokime_contract::operations::documents::DocumentLayoutNodeNode::Section(Box::new(
                    dokime_contract::operations::documents::DocumentSection {
                        node_id: section.node_id.to_string(),
                        metadata: Some(canonical::metadata(&section.metadata)?),
                        repeat: section
                            .repeat
                            .as_ref()
                            .map(super::super::history::document::canonical::repeat)
                            .transpose()?,
                    },
                )),
                section.children,
            ),
            native::DocumentNode::Card(card) => (
                dokime_contract::operations::documents::DocumentLayoutNodeNode::Placement(
                    dokime_contract::operations::documents::DocumentPlacement {
                        card_id: card.card_id.to_string(),
                        column_span: Some(dokime_contract::operations::documents::DocumentSpan {
                            base: card.layout.col_span.base.into(),
                            sm: card.layout.col_span.sm.map(Into::into),
                            md: card.layout.col_span.md.map(Into::into),
                            lg: card.layout.col_span.lg.map(Into::into),
                            xl: card.layout.col_span.xl.map(Into::into),
                        }),
                        row_span: card.layout.row_span.into(),
                        order: card.layout.order,
                        density: density(card.layout.density),
                        rail: rail(card.layout.rail),
                    },
                ),
                Vec::new(),
            ),
        };
        output.push(Frame::LayoutNode(
            dokime_contract::operations::documents::DocumentLayoutNode {
                ordinal: current,
                parent_ordinal: parent,
                node: Some(node),
            },
        ))?;
        layout_nodes(output, children, Some(current), ordinal)?;
    }
    Ok(())
}

fn density(
    value: native::DocumentCardDensity,
) -> dokime_contract::operations::documents::DocumentDensity {
    match value {
        native::DocumentCardDensity::Compact => {
            dokime_contract::operations::documents::DocumentDensity::Compact
        }
        native::DocumentCardDensity::Normal => {
            dokime_contract::operations::documents::DocumentDensity::Normal
        }
        native::DocumentCardDensity::Spacious => {
            dokime_contract::operations::documents::DocumentDensity::Spacious
        }
    }
}

fn rail(value: native::DocumentCardRail) -> dokime_contract::operations::documents::DocumentRail {
    match value {
        native::DocumentCardRail::Main => {
            dokime_contract::operations::documents::DocumentRail::Main
        }
        native::DocumentCardRail::Right => {
            dokime_contract::operations::documents::DocumentRail::Right
        }
        native::DocumentCardRail::Left => {
            dokime_contract::operations::documents::DocumentRail::Left
        }
    }
}

fn summary_stat(
    value: native::TelemetrySummaryStat,
) -> dokime_contract::operations::documents::DocumentSummaryStat {
    match value {
        native::TelemetrySummaryStat::Latest => {
            dokime_contract::operations::documents::DocumentSummaryStat::Latest
        }
    }
}

fn summary_format(
    value: native::TelemetrySummaryFormat,
) -> dokime_contract::operations::documents::DocumentSummaryFormat {
    match value {
        native::TelemetrySummaryFormat::Auto => {
            dokime_contract::operations::documents::DocumentSummaryFormat::Auto
        }
        native::TelemetrySummaryFormat::Number => {
            dokime_contract::operations::documents::DocumentSummaryFormat::Number
        }
        native::TelemetrySummaryFormat::Percent => {
            dokime_contract::operations::documents::DocumentSummaryFormat::Percent
        }
        native::TelemetrySummaryFormat::Duration => {
            dokime_contract::operations::documents::DocumentSummaryFormat::Duration
        }
        native::TelemetrySummaryFormat::Distance => {
            dokime_contract::operations::documents::DocumentSummaryFormat::Distance
        }
        native::TelemetrySummaryFormat::Status => {
            dokime_contract::operations::documents::DocumentSummaryFormat::Status
        }
        native::TelemetrySummaryFormat::Text => {
            dokime_contract::operations::documents::DocumentSummaryFormat::Text
        }
    }
}

fn summary_presentation(
    value: native::TelemetrySummaryPresentation,
) -> dokime_contract::operations::documents::DocumentSummaryPresentation {
    match value {
        native::TelemetrySummaryPresentation::Value => {
            dokime_contract::operations::documents::DocumentSummaryPresentation::Value
        }
        native::TelemetrySummaryPresentation::Progress => {
            dokime_contract::operations::documents::DocumentSummaryPresentation::Progress
        }
        native::TelemetrySummaryPresentation::Badge => {
            dokime_contract::operations::documents::DocumentSummaryPresentation::Badge
        }
    }
}

fn summary_state(
    value: native::TelemetrySummaryState,
) -> dokime_contract::operations::documents::DocumentSummaryState {
    match value {
        native::TelemetrySummaryState::Ok => {
            dokime_contract::operations::documents::DocumentSummaryState::Ok
        }
        native::TelemetrySummaryState::Warning => {
            dokime_contract::operations::documents::DocumentSummaryState::Warning
        }
        native::TelemetrySummaryState::Critical => {
            dokime_contract::operations::documents::DocumentSummaryState::Critical
        }
        native::TelemetrySummaryState::Muted => {
            dokime_contract::operations::documents::DocumentSummaryState::Muted
        }
    }
}

/* === Tests === */
