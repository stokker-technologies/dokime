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
use prost::Message;
use tonic::Status;
use v1::document_frame::Frame;
// Local Imports
use super::super::value;
use dokime_app::{CurrentDocumentCapture, document as native};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Frames {
    frames: VecDeque<v1::DocumentFrame>,
    bytes: usize,
    limit: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Frames {
    fn push(&mut self, frame: Frame) -> Result<(), Status> {
        let frame = v1::DocumentFrame { frame: Some(frame) };
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
) -> Result<VecDeque<v1::DocumentFrame>, Status> {
    use super::super::{enums, history::document as authoring, plots};
    use native::capture::CardBindingResult;
    let mut output = Frames {
        frames: VecDeque::new(),
        bytes: 0,
        limit: limit.min(512 * 1024),
    };
    let capture = captured.capture;
    let definition = capture.definition;
    let scope = value::scope(&capture.selection.scope);
    let telemetry = captured.telemetry_cut.map(|cut| {
        use dokime_app::CurrentDocumentTelemetryCut as ReadCut;
        let (delivered, source, ordinal) = match cut {
            ReadCut::Window { delivered } => (
                delivered,
                v1::document_telemetry_cut::Source::RetainedWindow(true),
                None,
            ),
            ReadCut::Series {
                epoch,
                delivered,
                ordinal,
            } => (
                delivered,
                v1::document_telemetry_cut::Source::SeriesEpoch(epoch),
                Some(ordinal),
            ),
        };
        v1::DocumentTelemetryCut {
            delivered_cursor: delivered,
            source: Some(source),
            series_ordinal: ordinal,
        }
    });
    output.push(Frame::Header(Box::new(v1::DocumentHeader {
        document_id: definition.document_id.to_string(),
        metadata: Some(value::checked_metadata(&definition.metadata)?),
        scope: Some(scope),
        frontiers: Some(v1::DocumentFrontiers {
            process_epoch: captured.process_epoch,
            state_revision: captured.state_revision,
            telemetry,
            artifacts_sha256: captured.artifacts_sha256,
        }),
    })))?;
    let mut ordinal = 0;
    layout_nodes(&mut output, definition.layout.nodes, None, &mut ordinal)?;
    let card_count = definition.cards.len() as u64;
    for card in definition.cards {
        use native::DocumentCardBody as Body;
        use v1::document_card::Body as Wire;
        let id = card.metadata.identifier.to_string();
        let body = match &card.body {
            Body::Text { markdown } => Wire::Markdown(markdown.clone()),
            Body::Table { rows } => Wire::Table(v1::DocumentTable {
                row_count: rows.len() as u64,
            }),
            Body::TelemetrySummary { summary } => Wire::Summary(v1::DocumentSummary {
                group_count: summary.groups.len() as u64,
            }),
            body => {
                let kind = match body {
                    Body::Plot { .. } => v1::DocumentCardKind::Plot,
                    Body::Steps => v1::DocumentCardKind::Steps,
                    Body::Criteria => v1::DocumentCardKind::Criteria,
                    Body::Traceability => v1::DocumentCardKind::Traceability,
                    Body::Notes => v1::DocumentCardKind::Notes,
                    Body::Records => v1::DocumentCardKind::Records,
                    Body::Artifacts => v1::DocumentCardKind::Artifacts,
                    Body::Media => v1::DocumentCardKind::Media,
                    Body::Health => v1::DocumentCardKind::Health,
                    Body::TelemetryContext => v1::DocumentCardKind::TelemetryContext,
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
                Wire::Owner(Box::new(v1::DocumentOwnerReference {
                    kind: kind as i32,
                    plot_id: plot
                        .as_ref()
                        .and_then(|plot| plot.request.as_ref()?.plot_id.clone()),
                    plot,
                }))
            }
        };
        output.push(Frame::Card(v1::DocumentCard {
            metadata: Some(value::checked_metadata(&card.metadata)?),
            filters: card
                .filters
                .iter()
                .map(plots::plot_filter_wire)
                .collect::<Result<_, _>>()?,
            body: Some(body),
        }))?;
        if let Body::Table { rows } = card.body {
            for (ordinal, fields) in rows.into_iter().enumerate() {
                native::bounded::document_size(&fields, output.limit.saturating_sub(8192) / 2, 32)
                    .map_err(|_| Status::resource_exhausted("document row exceeds frame budget"))?;
                value::collection(fields.len())?;
                let mut budget = value::Budget::new();
                let fields = fields
                    .iter()
                    .map(|(key, item)| {
                        value::text(key)?;
                        Ok((key.clone(), value::encode(item, &mut budget, 0)?))
                    })
                    .collect::<Result<_, Status>>()?;
                output.push(Frame::TableRow(v1::DocumentTableRow {
                    card_id: id.clone(),
                    ordinal: ordinal as u64,
                    values: Some(v1::ValueObject { fields }),
                }))?;
            }
        } else if let Body::TelemetrySummary { summary } = card.body {
            for (group_ordinal, group) in summary.groups.into_iter().enumerate() {
                output.push(Frame::GroupDefinition(v1::DocumentSummaryGroupDefinition {
                    card_id: id.clone(),
                    ordinal: group_ordinal as u64,
                    metadata: Some(value::checked_metadata(&group.metadata)?),
                    tile_count: group.tiles.len() as u64,
                }))?;
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
                            Ok(v1::DocumentSummaryCondition {
                                operator: enums::filter_operator(&condition.operator) as i32,
                                value: Some(value::encode(
                                    &condition.value,
                                    &mut value::Budget::new(),
                                    0,
                                )?),
                                state: summary_state(condition.state),
                            })
                        })
                        .collect::<Result<_, Status>>()?;
                    output.push(Frame::TileDefinition(v1::DocumentSummaryTileDefinition {
                        card_id: id.clone(),
                        group_ordinal: group_ordinal as u64,
                        ordinal: ordinal as u64,
                        metadata: Some(value::checked_metadata(&tile.metadata)?),
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
                    }))?;
                }
            }
        }
    }
    let binding_count = capture.cards.len() as u64;
    for binding in capture.cards {
        let id = binding.metadata.identifier.to_string();
        let result = match &binding.result {
            CardBindingResult::Static => v1::document_binding::Result::Literal(true),
            CardBindingResult::Summary { groups } => {
                v1::document_binding::Result::Summary(v1::DocumentSummary {
                    group_count: groups.len() as u64,
                })
            }
            CardBindingResult::Plot { descriptor } => v1::document_binding::Result::Plot(Box::new(
                plots::document_plot_definition(descriptor)?,
            )),
        };
        output.push(Frame::Binding(v1::DocumentBinding {
            source_id: binding.source_id.to_string(),
            metadata: Some(value::checked_metadata(&binding.metadata)?),
            result: Some(result),
        }))?;
        if let CardBindingResult::Summary { groups } = binding.result {
            for (group_ordinal, group) in groups.into_iter().enumerate() {
                output.push(Frame::SummaryGroup(v1::DocumentSummaryGroup {
                    card_id: id.clone(),
                    ordinal: group_ordinal as u64,
                    metadata: Some(value::checked_metadata(&group.metadata)?),
                    tile_count: group.tiles.len() as u64,
                }))?;
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
                        .map(|item| value::encode(item, &mut value::Budget::new(), 0))
                        .transpose()?;
                    output.push(Frame::SummaryTile(v1::DocumentSummaryTile {
                        card_id: id.clone(),
                        group_ordinal: group_ordinal as u64,
                        ordinal: ordinal as u64,
                        metadata: Some(value::checked_metadata(&tile.metadata)?),
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
        output.push(Frame::RepeatSelection(v1::DocumentRepeatSelection {
            node_id: id.to_string(),
            instance_count: instances.len() as u64,
        }))?;
        for (ordinal, instance) in instances.into_iter().enumerate() {
            output.push(Frame::RepeatInstance(v1::DocumentRepeatInstance {
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
        output.push(Frame::SectionIdentity(v1::DocumentSectionIdentity {
            node_id: node_id.to_string(),
            metadata: Some(value::checked_metadata(&section.metadata)?),
            unavailable: section.unavailable,
        }))?;
    }
    for warning in capture.warnings {
        output.push(Frame::Warning(v1::DocumentWarning {
            code: warning.code.to_string(),
            message: warning.message,
        }))?;
    }
    output.push(Frame::Completion(v1::DocumentCompletion {
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
                v1::document_layout_node::Node::Section(Box::new(v1::DocumentSection {
                    node_id: section.node_id.to_string(),
                    metadata: Some(value::checked_metadata(&section.metadata)?),
                    repeat: section
                        .repeat
                        .as_ref()
                        .map(super::super::history::document::repeat)
                        .transpose()?,
                })),
                section.children,
            ),
            native::DocumentNode::Card(card) => (
                v1::document_layout_node::Node::Placement(v1::DocumentPlacement {
                    card_id: card.card_id.to_string(),
                    column_span: Some(v1::DocumentSpan {
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
                }),
                Vec::new(),
            ),
        };
        output.push(Frame::LayoutNode(v1::DocumentLayoutNode {
            ordinal: current,
            parent_ordinal: parent,
            node: Some(node),
        }))?;
        layout_nodes(output, children, Some(current), ordinal)?;
    }
    Ok(())
}

fn density(value: native::DocumentCardDensity) -> i32 {
    match value {
        native::DocumentCardDensity::Compact => 1,
        native::DocumentCardDensity::Normal => 2,
        native::DocumentCardDensity::Spacious => 3,
    }
}

fn rail(value: native::DocumentCardRail) -> i32 {
    match value {
        native::DocumentCardRail::Main => 1,
        native::DocumentCardRail::Right => 2,
        native::DocumentCardRail::Left => 3,
    }
}

fn summary_stat(value: native::TelemetrySummaryStat) -> i32 {
    match value {
        native::TelemetrySummaryStat::Latest => 1,
    }
}

fn summary_format(value: native::TelemetrySummaryFormat) -> i32 {
    match value {
        native::TelemetrySummaryFormat::Auto => 1,
        native::TelemetrySummaryFormat::Number => 2,
        native::TelemetrySummaryFormat::Percent => 3,
        native::TelemetrySummaryFormat::Duration => 4,
        native::TelemetrySummaryFormat::Distance => 5,
        native::TelemetrySummaryFormat::Status => 6,
        native::TelemetrySummaryFormat::Text => 7,
    }
}

fn summary_presentation(value: native::TelemetrySummaryPresentation) -> i32 {
    match value {
        native::TelemetrySummaryPresentation::Value => 1,
        native::TelemetrySummaryPresentation::Progress => 2,
        native::TelemetrySummaryPresentation::Badge => 3,
    }
}

fn summary_state(value: native::TelemetrySummaryState) -> i32 {
    match value {
        native::TelemetrySummaryState::Ok => 1,
        native::TelemetrySummaryState::Warning => 2,
        native::TelemetrySummaryState::Critical => 3,
        native::TelemetrySummaryState::Muted => 4,
    }
}

/* === Tests === */
