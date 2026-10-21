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
use tonic::Status;

// Local Imports
use super::super::value::{self, canonical};
use crate::grpc::transcode::Transcode;
use dokime::domain::model::documents::wire::DocumentFrameFrame as Frame;
use dokime::{CurrentDocumentCapture, document as native};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Frames {
    frames: VecDeque<dokime::domain::model::documents::wire::DocumentFrame>,
    bytes: usize,
    limit: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Frames {
    fn push(&mut self, frame: Frame) -> Result<(), Status> {
        let frame = dokime::domain::model::documents::wire::DocumentFrame { frame: Some(frame) };
        let bytes = frame.wire_len()?;
        if bytes > self.limit || self.bytes.saturating_add(bytes) > 32 * 1024 * 1024 || self.frames.len() >= 20_000 {
            return Err(Status::resource_exhausted("document response budget exceeded"));
        }
        self.bytes += bytes;
        self.frames.push_back(frame);
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn frames(
    captured: CurrentDocumentCapture, limit: usize,
) -> Result<VecDeque<dokime::domain::model::documents::wire::DocumentFrame>, Status> {
    use super::super::{enums::canonical as enums, history::document::canonical as authoring, plots};
    use native::capture::CardBindingResult;
    let mut output = Frames { frames: VecDeque::new(), bytes: 0, limit: limit.min(512 * 1024) };
    let capture = captured.capture;
    let definition = capture.definition;
    let scope = canonical::scope(&capture.selection.scope)?;
    let telemetry = captured.telemetry_cut.map(|cut| {
        use dokime::CurrentDocumentTelemetryCut as ReadCut;
        let (delivered, source, ordinal) = match cut {
            ReadCut::Window { delivered } => {
                (delivered, dokime::domain::model::documents::wire::DocumentTelemetryCutSource::RetainedWindow(true), None)
            }
            ReadCut::Series { epoch, delivered, ordinal } => {
                (delivered, dokime::domain::model::documents::wire::DocumentTelemetryCutSource::SeriesEpoch(epoch), Some(ordinal))
            }
        };
        dokime::domain::model::documents::wire::DocumentTelemetryCut {
            delivered_cursor: delivered,
            source: Some(source),
            series_ordinal: ordinal,
        }
    });
    output.push(Frame::Header(dokime::domain::model::documents::wire::DocumentHeader {
        document_id: definition.document_id.to_string(),
        metadata: Some(canonical::metadata(&definition.metadata)?),
        scope: Some(scope),
        frontiers: Some(dokime::domain::model::documents::wire::DocumentFrontiers {
            process_epoch: captured.process_epoch,
            state_revision: captured.state_revision,
            telemetry,
            artifacts_sha256: captured.artifacts_sha256,
        }),
    }))?;
    let mut ordinal = 0;
    layout_nodes(&mut output, definition.layout.nodes, None, &mut ordinal)?;
    let card_count = definition.cards.len() as u64;
    for card in definition.cards {
        use dokime::domain::model::documents::wire::DocumentCardBody as Wire;
        use native::DocumentCardBody as Body;
        let id = card.metadata.identifier.to_string();
        let body = match &card.body {
            Body::Text { markdown } => Wire::Markdown(markdown.clone()),
            Body::Table { rows } => Wire::Table(dokime::domain::model::documents::wire::DocumentTable { row_count: rows.len() as u64 }),
            Body::TelemetrySummary { summary } => {
                Wire::Summary(dokime::domain::model::documents::wire::DocumentSummary { group_count: summary.groups.len() as u64 })
            }
            body => {
                let kind = match body {
                    Body::Plot { .. } => dokime::domain::model::documents::wire::DocumentCardKind::Plot,
                    Body::Steps => dokime::domain::model::documents::wire::DocumentCardKind::Steps,
                    Body::Criteria => dokime::domain::model::documents::wire::DocumentCardKind::Criteria,
                    Body::Traceability => dokime::domain::model::documents::wire::DocumentCardKind::Traceability,
                    Body::Notes => dokime::domain::model::documents::wire::DocumentCardKind::Notes,
                    Body::Records => dokime::domain::model::documents::wire::DocumentCardKind::Records,
                    Body::Artifacts => dokime::domain::model::documents::wire::DocumentCardKind::Artifacts,
                    Body::Media => dokime::domain::model::documents::wire::DocumentCardKind::Media,
                    Body::Health => dokime::domain::model::documents::wire::DocumentCardKind::Health,
                    Body::TelemetryContext => dokime::domain::model::documents::wire::DocumentCardKind::TelemetryContext,
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
                Wire::Owner(Box::new(dokime::domain::model::documents::wire::DocumentOwnerReference {
                    kind,
                    plot_id: plot.as_ref().and_then(|plot| plot.request.as_ref()?.plot_id.clone()),
                    plot,
                }))
            }
        };
        output.push(Frame::Card(dokime::domain::model::documents::wire::DocumentCard {
            metadata: Some(canonical::metadata(&card.metadata)?),
            filters: card.filters.iter().map(plots::plot_filter_wire).collect::<Result<_, _>>()?,
            body: Some(body),
        }))?;
        if let Body::Table { rows } = card.body {
            for (ordinal, fields) in rows.into_iter().enumerate() {
                native::bounded::document_size(&fields, output.limit.saturating_sub(8192) / 2, 32)
                    .map_err(|_| Status::resource_exhausted("document row exceeds frame budget"))?;
                value::collection(fields.len())?;
                let mut budget = value::Budget::new();
                output.push(Frame::TableRow(dokime::domain::model::documents::wire::DocumentTableRow {
                    card_id: id.clone(),
                    ordinal: ordinal as u64,
                    values: Some(canonical::object(&fields, &mut budget)?),
                }))?;
            }
        } else if let Body::TelemetrySummary { summary } = card.body {
            for (group_ordinal, group) in summary.groups.into_iter().enumerate() {
                output.push(Frame::GroupDefinition(dokime::domain::model::documents::wire::DocumentSummaryGroupDefinition {
                    card_id: id.clone(),
                    ordinal: group_ordinal as u64,
                    metadata: Some(canonical::metadata(&group.metadata)?),
                    tile_count: group.tiles.len() as u64,
                }))?;
                for (ordinal, tile) in group.tiles.into_iter().enumerate() {
                    native::bounded::document_size(&tile, output.limit.saturating_sub(8192) / 2, 32)
                        .map_err(|_| Status::resource_exhausted("summary declaration exceeds frame budget"))?;
                    let conditions = tile
                        .conditions
                        .into_iter()
                        .map(|condition| {
                            Ok(dokime::domain::model::documents::wire::DocumentSummaryCondition {
                                operator: enums::filter_operator(&condition.operator),
                                value: Some(canonical::encode(&condition.value, &mut value::Budget::new(), 0)?),
                                state: summary_state(condition.state),
                            })
                        })
                        .collect::<Result<_, Status>>()?;
                    output.push(Frame::TileDefinition(dokime::domain::model::documents::wire::DocumentSummaryTileDefinition {
                        card_id: id.clone(),
                        group_ordinal: group_ordinal as u64,
                        ordinal: ordinal as u64,
                        metadata: Some(canonical::metadata(&tile.metadata)?),
                        value: Some(plots::data_expression_wire(&tile.value)?),
                        filters: tile.filters.iter().map(plots::plot_filter_wire).collect::<Result<_, _>>()?,
                        stat: summary_stat(tile.stat),
                        format: summary_format(tile.format),
                        presentation: summary_presentation(tile.presentation),
                        unit: tile.unit,
                        conditions,
                    }))?;
                }
            }
        }
    }
    let binding_count = capture.cards.len() as u64;
    for binding in capture.cards {
        let id = binding.metadata.identifier.to_string();
        let result = match &binding.result {
            CardBindingResult::Static => dokime::domain::model::documents::wire::DocumentBindingResult::Literal(true),
            CardBindingResult::Summary { groups } => dokime::domain::model::documents::wire::DocumentBindingResult::Summary(
                dokime::domain::model::documents::wire::DocumentSummary { group_count: groups.len() as u64 },
            ),
            CardBindingResult::Plot { descriptor } => {
                dokime::domain::model::documents::wire::DocumentBindingResult::Plot(Box::new(plots::document_plot_definition(descriptor)?))
            }
        };
        output.push(Frame::Binding(dokime::domain::model::documents::wire::DocumentBinding {
            source_id: binding.source_id.to_string(),
            metadata: Some(canonical::metadata(&binding.metadata)?),
            result: Some(result),
        }))?;
        if let CardBindingResult::Summary { groups } = binding.result {
            for (group_ordinal, group) in groups.into_iter().enumerate() {
                output.push(Frame::SummaryGroup(dokime::domain::model::documents::wire::DocumentSummaryGroup {
                    card_id: id.clone(),
                    ordinal: group_ordinal as u64,
                    metadata: Some(canonical::metadata(&group.metadata)?),
                    tile_count: group.tiles.len() as u64,
                }))?;
                for (ordinal, tile) in group.tiles.into_iter().enumerate() {
                    native::bounded::document_size(&tile, output.limit.saturating_sub(8192) / 2, 32)
                        .map_err(|_| Status::resource_exhausted("document summary exceeds frame budget"))?;
                    if !tile.valid_result() {
                        return Err(Status::internal("invalid summary unavailable result"));
                    }
                    let encoded = tile.value.as_ref().map(|item| canonical::encode(item, &mut value::Budget::new(), 0)).transpose()?;
                    output.push(Frame::SummaryTile(dokime::domain::model::documents::wire::DocumentSummaryTile {
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
                    }))?;
                }
            }
        }
    }
    let repeat_count = capture.repeats.len() as u64;
    for (id, instances) in capture.repeats {
        output.push(Frame::RepeatSelection(dokime::domain::model::documents::wire::DocumentRepeatSelection {
            node_id: id.to_string(),
            instance_count: instances.len() as u64,
        }))?;
        for (ordinal, instance) in instances.into_iter().enumerate() {
            output.push(Frame::RepeatInstance(dokime::domain::model::documents::wire::DocumentRepeatInstance {
                selection_id: id.to_string(),
                ordinal: ordinal as u64,
                suffix: instance.suffix,
                node_id: instance.node_id.to_string(),
                filter: Some(plots::plot_filter_wire(&instance.filter)?),
            }))?;
        }
    }
    let section_count = capture.sections.len() as u64;
    for (node_id, section) in capture.sections {
        if !section.valid_result() {
            return Err(Status::internal("invalid section unavailable reason"));
        }
        output.push(Frame::SectionIdentity(dokime::domain::model::documents::wire::DocumentSectionIdentity {
            node_id: node_id.to_string(),
            metadata: Some(canonical::metadata(&section.metadata)?),
            unavailable: section.unavailable,
        }))?;
    }
    for warning in capture.warnings {
        output.push(Frame::Warning(dokime::domain::model::documents::wire::DocumentWarning {
            code: warning.code.to_string(),
            message: warning.message,
        }))?;
    }
    output.push(Frame::Completion(dokime::domain::model::documents::wire::DocumentCompletion {
        frame_count: output.frames.len() as u64 + 1,
        card_count,
        layout_node_count: ordinal,
        binding_count,
        repeat_count,
        section_count,
    }))?;
    Ok(output.frames)
}

/* === Private Functions === */

fn single_stream(plot: &native::PlotDefinition) -> Option<&str> {
    use dokime::expressions::{DataSource, DataTransform};
    let mut streams = std::collections::BTreeSet::new();
    for expression in plot.y_expressions.iter().chain(&plot.x_expression).chain(&plot.series_by_expression).chain(&plot.z_expression) {
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

fn layout_nodes(output: &mut Frames, nodes: Vec<native::DocumentNode>, parent: Option<u64>, ordinal: &mut u64) -> Result<(), Status> {
    for node in nodes {
        let current = *ordinal;
        *ordinal += 1;
        let (node, children) = match node {
            native::DocumentNode::Section(section) => (
                dokime::domain::model::documents::wire::DocumentLayoutNodeNode::Section(Box::new(
                    dokime::domain::model::documents::wire::DocumentSection {
                        node_id: section.node_id.to_string(),
                        metadata: Some(canonical::metadata(&section.metadata)?),
                        repeat: section.repeat.as_ref().map(super::super::history::document::canonical::repeat).transpose()?,
                    },
                )),
                section.children,
            ),
            native::DocumentNode::Card(card) => (
                dokime::domain::model::documents::wire::DocumentLayoutNodeNode::Placement(
                    dokime::domain::model::documents::wire::DocumentPlacement {
                        card_id: card.card_id.to_string(),
                        column_span: Some(dokime::domain::model::documents::wire::DocumentSpan {
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
        output.push(Frame::LayoutNode(dokime::domain::model::documents::wire::DocumentLayoutNode {
            ordinal: current,
            parent_ordinal: parent,
            node: Some(node),
        }))?;
        layout_nodes(output, children, Some(current), ordinal)?;
    }
    Ok(())
}

fn density(value: native::DocumentCardDensity) -> dokime::domain::model::documents::wire::DocumentDensity {
    match value {
        native::DocumentCardDensity::Compact => dokime::domain::model::documents::wire::DocumentDensity::Compact,
        native::DocumentCardDensity::Normal => dokime::domain::model::documents::wire::DocumentDensity::Normal,
        native::DocumentCardDensity::Spacious => dokime::domain::model::documents::wire::DocumentDensity::Spacious,
    }
}

fn rail(value: native::DocumentCardRail) -> dokime::domain::model::documents::wire::DocumentRail {
    match value {
        native::DocumentCardRail::Main => dokime::domain::model::documents::wire::DocumentRail::Main,
        native::DocumentCardRail::Right => dokime::domain::model::documents::wire::DocumentRail::Right,
        native::DocumentCardRail::Left => dokime::domain::model::documents::wire::DocumentRail::Left,
    }
}

fn summary_stat(value: native::TelemetrySummaryStat) -> dokime::domain::model::documents::wire::DocumentSummaryStat {
    match value {
        native::TelemetrySummaryStat::Latest => dokime::domain::model::documents::wire::DocumentSummaryStat::Latest,
    }
}

fn summary_format(value: native::TelemetrySummaryFormat) -> dokime::domain::model::documents::wire::DocumentSummaryFormat {
    match value {
        native::TelemetrySummaryFormat::Auto => dokime::domain::model::documents::wire::DocumentSummaryFormat::Auto,
        native::TelemetrySummaryFormat::Number => dokime::domain::model::documents::wire::DocumentSummaryFormat::Number,
        native::TelemetrySummaryFormat::Percent => dokime::domain::model::documents::wire::DocumentSummaryFormat::Percent,
        native::TelemetrySummaryFormat::Duration => dokime::domain::model::documents::wire::DocumentSummaryFormat::Duration,
        native::TelemetrySummaryFormat::Distance => dokime::domain::model::documents::wire::DocumentSummaryFormat::Distance,
        native::TelemetrySummaryFormat::Status => dokime::domain::model::documents::wire::DocumentSummaryFormat::Status,
        native::TelemetrySummaryFormat::Text => dokime::domain::model::documents::wire::DocumentSummaryFormat::Text,
    }
}

fn summary_presentation(
    value: native::TelemetrySummaryPresentation,
) -> dokime::domain::model::documents::wire::DocumentSummaryPresentation {
    match value {
        native::TelemetrySummaryPresentation::Value => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Value,
        native::TelemetrySummaryPresentation::Progress => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Progress,
        native::TelemetrySummaryPresentation::Badge => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Badge,
    }
}

fn summary_state(value: native::TelemetrySummaryState) -> dokime::domain::model::documents::wire::DocumentSummaryState {
    match value {
        native::TelemetrySummaryState::Ok => dokime::domain::model::documents::wire::DocumentSummaryState::Ok,
        native::TelemetrySummaryState::Warning => dokime::domain::model::documents::wire::DocumentSummaryState::Warning,
        native::TelemetrySummaryState::Critical => dokime::domain::model::documents::wire::DocumentSummaryState::Critical,
        native::TelemetrySummaryState::Muted => dokime::domain::model::documents::wire::DocumentSummaryState::Muted,
    }
}

/* === Tests === */
