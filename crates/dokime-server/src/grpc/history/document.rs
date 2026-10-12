/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod construction;

/* === Re-Exports === */
pub(super) use construction::memory as construction_memory;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::Status;
// Local Imports
use super::super::enums;
use super::super::value;
use super::super::value::projection;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn wire(
    value: &dokime_app::document::DocumentDefinition,
    scope: &dokime_app::core::RecordScope,
) -> Result<v1::HistoricalDocument, Status> {
    use dokime_app::document::DocumentCardBody;
    let presentation = v1::DocumentHeader {
        document_id: value.document_id.as_str().to_owned(),
        metadata: Some(metadata(&value.metadata)?),
        scope: Some(projection::scope(scope).map_err(projection_failure)?),
        frontiers: None,
    };
    let mut layout_nodes = projection::slots(construction::layout_count(&value.layout.nodes))
        .map_err(projection_failure)?;
    let mut ordinal = 0;
    for node in &value.layout.nodes {
        layout_nodes_for_node(node, None, &mut ordinal, &mut layout_nodes)?;
    }
    let mut cards = projection::slots(value.cards.len()).map_err(projection_failure)?;
    let rows = value.cards.iter().fold(0usize, |count, card| {
        let rows = match &card.body {
            DocumentCardBody::Table { rows } => rows.len(),
            _ => 0,
        };
        count.saturating_add(rows)
    });
    let mut table_rows = projection::slots(rows).map_err(projection_failure)?;
    for card in &value.cards {
        if let DocumentCardBody::Table { rows } = &card.body {
            let mut budget = value::Budget::new();
            for (row, fields) in rows.iter().enumerate() {
                value::collection(fields.len())?;
                table_rows.push(v1::DocumentTableRow {
                    card_id: projection::text(card.metadata.identifier.as_str())
                        .map_err(projection_failure)?,
                    ordinal: row as u64,
                    values: Some(value_object(fields, &mut budget, 0)?),
                });
            }
        }
        cards.push(historical_card(card, scope)?);
    }
    Ok(v1::HistoricalDocument {
        presentation: Some(presentation),
        layout_nodes,
        cards,
        table_rows,
    })
}

/* === Private Functions === */
fn projection_failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => {
            Status::resource_exhausted("historical document construction exceeds its bound")
        }
        _ => Status::data_loss("historical document projection failed"),
    }
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = projection::slots(source.len()).map_err(projection_failure)?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn value_object(
    source: &std::collections::BTreeMap<String, dokime_app::core::Value>,
    budget: &mut value::Budget,
    depth: usize,
) -> Result<v1::ValueObject, Status> {
    projection::object_count(source, &mut value::Budget::new(), depth)
        .map_err(projection_failure)?;
    projection::object(source, budget, depth).map_err(projection_failure)
}

fn metadata(source: &dokime_app::core::Metadata) -> Result<v1::Metadata, Status> {
    projection::metadata_count(source).map_err(projection_failure)?;
    projection::metadata(source).map_err(projection_failure)
}

fn project_value(
    source: &dokime_app::core::Value,
    budget: &mut value::Budget,
    depth: usize,
) -> Result<v1::Value, Status> {
    projection::count(source, &mut value::Budget::new(), depth).map_err(projection_failure)?;
    projection::project(source, budget, depth).map_err(projection_failure)
}

fn historical_card(
    value: &dokime_app::document::DocumentCardDefinition,
    scope: &dokime_app::core::RecordScope,
) -> Result<v1::HistoricalDocumentCard, Status> {
    use dokime_app::document::DocumentCardBody;
    let body = match &value.body {
        DocumentCardBody::Text { markdown } => {
            value::text(markdown)?;
            Some(v1::historical_document_card::Body::Markdown(
                markdown.clone(),
            ))
        }
        DocumentCardBody::Table { rows } => Some(v1::historical_document_card::Body::Table(
            v1::DocumentTable {
                row_count: rows.len() as u64,
            },
        )),
        DocumentCardBody::TelemetrySummary { summary } => Some(
            v1::historical_document_card::Body::Summary(historical_summary(summary)?),
        ),
        DocumentCardBody::Plot { plot } => Some(v1::historical_document_card::Body::Plot(
            historical_plot(plot, scope)?,
        )),
        DocumentCardBody::Steps => Some(owner_body(v1::DocumentCardKind::Steps, None, scope)?),
        DocumentCardBody::Criteria => {
            Some(owner_body(v1::DocumentCardKind::Criteria, None, scope)?)
        }
        DocumentCardBody::Traceability => {
            Some(owner_body(v1::DocumentCardKind::Traceability, None, scope)?)
        }
        DocumentCardBody::Notes => Some(owner_body(v1::DocumentCardKind::Notes, None, scope)?),
        DocumentCardBody::Records => Some(owner_body(v1::DocumentCardKind::Records, None, scope)?),
        DocumentCardBody::Artifacts => {
            Some(owner_body(v1::DocumentCardKind::Artifacts, None, scope)?)
        }
        DocumentCardBody::Media => Some(owner_body(v1::DocumentCardKind::Media, None, scope)?),
        DocumentCardBody::Health => Some(owner_body(v1::DocumentCardKind::Health, None, scope)?),
        DocumentCardBody::TelemetryContext => Some(owner_body(
            v1::DocumentCardKind::TelemetryContext,
            None,
            scope,
        )?),
    };
    Ok(v1::HistoricalDocumentCard {
        metadata: Some(metadata(&value.metadata)?),
        filters: collect(value.filters.iter().map(historical_filter))?,
        body,
    })
}

fn historical_summary(
    value: &dokime_app::document::TelemetrySummaryDefinition,
) -> Result<v1::HistoricalTelemetrySummary, Status> {
    value::collection(value.groups.len())?;
    Ok(v1::HistoricalTelemetrySummary {
        groups: collect(value.groups.iter().map(|group| {
            value::collection(group.tiles.len())?;
            Ok(v1::HistoricalSummaryGroup {
                metadata: Some(metadata(&group.metadata)?),
                tiles: collect(group.tiles.iter().map(historical_summary_tile))?,
            })
        }))?,
    })
}

fn historical_summary_tile(
    value: &dokime_app::document::TelemetrySummaryTileDefinition,
) -> Result<v1::HistoricalSummaryTile, Status> {
    value::collection(value.filters.len())?;
    value::collection(value.conditions.len())?;
    Ok(v1::HistoricalSummaryTile {
        metadata: Some(metadata(&value.metadata)?),
        value: Some(data_expression(&value.value)?),
        filters: collect(value.filters.iter().map(historical_filter))?,
        stat: summary_stat(value.stat.clone()) as i32,
        format: summary_format(value.format.clone()) as i32,
        presentation: summary_presentation(value.presentation.clone()) as i32,
        unit: value
            .unit
            .as_ref()
            .map(|unit| {
                value::text(unit)?;
                Ok::<_, Status>(unit.clone())
            })
            .transpose()?,
        conditions: collect(value.conditions.iter().map(|condition| {
            Ok(v1::HistoricalSummaryCondition {
                operator: enums::filter_operator(&condition.operator) as i32,
                value: Some(project_value(
                    &condition.value,
                    &mut value::Budget::new(),
                    0,
                )?),
                state: summary_state(condition.state.clone()) as i32,
            })
        }))?,
    })
}

fn historical_filter(
    value: &dokime_app::document::DocumentFilter,
) -> Result<v1::HistoricalDocumentFilter, Status> {
    Ok(v1::HistoricalDocumentFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator) as i32,
        value: value
            .value
            .as_ref()
            .map(|item| project_value(item, &mut value::Budget::new(), 0))
            .transpose()?,
    })
}

fn data_expression(
    value: &dokime_app::expressions::DataExpression,
) -> Result<v1::DataExpression, Status> {
    value::collection(value.transforms.len())?;
    Ok(v1::DataExpression {
        label: value
            .label
            .as_deref()
            .map(projection::text)
            .transpose()
            .map_err(projection_failure)?,
        source: Some(data_source(&value.source)?),
        transforms: collect(value.transforms.iter().map(data_transform))?,
    })
}

fn data_source(
    value: &dokime_app::expressions::DataSource,
) -> Result<v1::data_expression::Source, Status> {
    use dokime_app::expressions::DataSource;
    Ok(match value {
        DataSource::Telemetry { stream, field } => {
            value::text(stream)?;
            value::text(field)?;
            v1::data_expression::Source::Telemetry(v1::TelemetryDataSource {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataSource::Parameter { name } => {
            value::text(name)?;
            v1::data_expression::Source::Parameter(v1::ParameterDataSource { name: name.clone() })
        }
        DataSource::Dimension { name } => {
            value::text(name)?;
            v1::data_expression::Source::Dimension(v1::DimensionDataSource { name: name.clone() })
        }
        DataSource::Record { record_type } => {
            value::text(record_type)?;
            v1::data_expression::Source::Record(v1::RecordDataSource {
                record_type: record_type.clone(),
            })
        }
        DataSource::Artifact { artifact } => {
            value::text(artifact)?;
            v1::data_expression::Source::Artifact(v1::ArtifactDataSource {
                artifact: artifact.clone(),
            })
        }
    })
}

fn data_transform(
    value: &dokime_app::expressions::DataTransform,
) -> Result<v1::DataTransform, Status> {
    use dokime_app::expressions::DataTransform;
    use v1::data_transform::Kind;
    let kind = match value {
        DataTransform::Count => Kind::Count(v1::EmptyTransform {}),
        DataTransform::Mean => Kind::Mean(v1::EmptyTransform {}),
        DataTransform::Median => Kind::Median(v1::EmptyTransform {}),
        DataTransform::Min => Kind::Min(v1::EmptyTransform {}),
        DataTransform::Max => Kind::Max(v1::EmptyTransform {}),
        DataTransform::TimeBetween {
            previous_record_type,
        } => {
            value::text(previous_record_type)?;
            Kind::TimeBetween(v1::TimeBetweenTransform {
                previous_record_type: previous_record_type.clone(),
            })
        }
        DataTransform::TelemetryAtOrBefore { stream, field } => {
            value::text(stream)?;
            value::text(field)?;
            Kind::TelemetryAtOrBefore(v1::TelemetryAtOrBeforeTransform {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataTransform::Detail { name } => {
            value::text(name)?;
            Kind::Detail(v1::DetailTransform { name: name.clone() })
        }
        DataTransform::PerCase => Kind::PerCase(v1::EmptyTransform {}),
    };
    Ok(v1::DataTransform { kind: Some(kind) })
}

pub(in crate::grpc) fn historical_plot(
    value: &dokime_app::document::PlotDefinition,
    scope: &dokime_app::core::RecordScope,
) -> Result<v1::DocumentPlotDefinition, Status> {
    value::collection(value.y.len())?;
    value::collection(value.y_expressions.len())?;
    value::collection(value.filters.len())?;
    value::collection(value.marker_record_types.len())?;
    value::collection(value.reference_lines.len())?;
    value::collection(value.reference_bands.len())?;
    let strings = [
        value.stream.as_deref(),
        value.x.as_deref(),
        value.z_stream.as_deref(),
        value.z_field.as_deref(),
        value.series_by.as_deref(),
        value.unit.as_deref(),
    ];
    for item in strings.into_iter().flatten() {
        value::text(item)?;
    }
    for item in &value.y {
        value::text(item)?;
    }
    for item in &value.marker_record_types {
        value::text(item)?;
    }
    value::text(&value.group)?;
    if let Some(window) = value.live_window_s {
        finite(window, "live_window_s")?;
    }
    let request = v1::PlotRequest {
        analysis: value
            .analysis
            .as_ref()
            .map(super::super::plots::analysis::selection_wire)
            .transpose()?,
        plot_id: Some(value.metadata.identifier.as_str().to_owned()),
        plot_type: enums::plot_type(&value.plot_type) as i32,
        x_scale: enums::axis_scale(&value.x_scale) as i32,
        y_scale: enums::axis_scale(&value.y_scale) as i32,
        stream: value.stream.clone().unwrap_or_default(),
        y_keys: value.y.clone(),
        marker_record_types: value.marker_record_types.clone(),
        filters: collect(value.filters.iter().map(plot_filter))?,
        x_key: value.x.clone(),
        series_by: value.series_by.clone(),
        x_expression: value
            .x_expression
            .as_ref()
            .map(data_expression)
            .transpose()?,
        y_expressions: collect(value.y_expressions.iter().map(data_expression))?,
        series_by_expression: value
            .series_by_expression
            .as_ref()
            .map(data_expression)
            .transpose()?,
        z_expression: value
            .z_expression
            .as_ref()
            .map(data_expression)
            .transpose()?,
        fft: Some(fft_options(&value.fft)?),
        interval: Some(interval_options(&value.interval)?),
        histogram: Some(histogram_options(&value.histogram)?),
        z_stream: value.z_stream.clone(),
        z_field: value.z_field.clone(),
        unit: value.unit.clone(),
        max_points: None,
        window_s: value.live_window_s,
        scope: Some(projection::scope(scope).map_err(projection_failure)?),
        cursor: None,
        technique: v1::DownsampleTechnique::Unspecified as i32,
    };
    let (x_axis, x_axis_unit) = enums::plot_axis(value.x_axis.as_ref());
    let presentation = v1::PlotPresentation {
        group: value.group.clone(),
        style: enums::plot_style(&value.style) as i32,
        fft: Some(fft_options(&value.fft)?),
        interval: Some(interval_options(&value.interval)?),
        histogram: Some(histogram_options(&value.histogram)?),
        reference_lines: collect(value.reference_lines.iter().map(reference_line))?,
        reference_bands: collect(value.reference_bands.iter().map(reference_band))?,
        live_window_s: value.live_window_s,
        x_format: value::optional_value_format(value.x_format.as_ref()),
        y_format: value::optional_value_format(value.y_format.as_ref()),
        x_axis,
        x_axis_unit,
    };
    Ok(v1::DocumentPlotDefinition {
        metadata: Some(metadata(&value.metadata)?),
        request: Some(request),
        presentation: Some(presentation),
        z_expression: value
            .z_expression
            .as_ref()
            .map(data_expression)
            .transpose()?,
    })
}

fn plot_filter(value: &dokime_app::document::DocumentFilter) -> Result<v1::PlotFilter, Status> {
    Ok(v1::PlotFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator) as i32,
        value: value
            .value
            .as_ref()
            .map(|item| project_value(item, &mut value::Budget::new(), 0))
            .transpose()?,
    })
}

fn finite(value: f64, field: &str) -> Result<(), Status> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(Status::data_loss(format!(
            "stored plot {field} is nonfinite"
        )))
    }
}

fn fft_options(value: &dokime_app::document::FftOptions) -> Result<v1::PlotFftOptions, Status> {
    for item in [
        value.sample_rate_hz,
        value.min_frequency_hz,
        value.max_frequency_hz,
    ]
    .into_iter()
    .flatten()
    {
        finite(item, "fft option")?;
    }
    Ok(v1::PlotFftOptions {
        sample_rate_hz: value.sample_rate_hz,
        window: enums::fft_window(&value.window) as i32,
        detrend: enums::fft_detrend(&value.detrend) as i32,
        level_unit: enums::frequency_level_unit(&value.level_unit) as i32,
        include_dc: value.include_dc,
        min_frequency_hz: value.min_frequency_hz,
        max_frequency_hz: value.max_frequency_hz,
    })
}

fn interval_options(
    value: &dokime_app::document::SigmaIntervalOptions,
) -> Result<v1::PlotIntervalOptions, Status> {
    finite(value.bin_width, "interval bin_width")?;
    finite(value.sigma, "interval sigma")?;
    Ok(v1::PlotIntervalOptions {
        bin_width: value.bin_width,
        sigma: value.sigma,
        clip_to_observed: value.clip_to_observed,
    })
}

fn histogram_options(
    value: &dokime_app::document::HistogramOptions,
) -> Result<v1::PlotHistogramOptions, Status> {
    if value.bins > u64::MAX as usize {
        return Err(Status::data_loss(
            "stored histogram bin count overflows wire",
        ));
    }
    if let Some(unit) = &value.unit {
        value::text(unit)?;
    }
    Ok(v1::PlotHistogramOptions {
        bins: value.bins as u64,
        unit: value.unit.clone(),
    })
}

fn reference_line(
    value: &dokime_app::document::ReferenceLine,
) -> Result<v1::PlotReferenceLine, Status> {
    finite(value.source_scale, "reference line source_scale")?;
    if let Some(item) = value.value {
        finite(item, "reference line value")?;
    }
    if let Some(label) = &value.label {
        value::text(label)?;
    }
    value::text(&value.color)?;
    Ok(v1::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis) as i32,
        value: value.value,
        source: value.source.as_ref().map(data_expression).transpose()?,
        source_scale: value.source_scale,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn reference_band(
    value: &dokime_app::document::ReferenceBand,
) -> Result<v1::PlotReferenceBand, Status> {
    if let Some(item) = value.min {
        finite(item, "reference band min")?;
    }
    if let Some(item) = value.max {
        finite(item, "reference band max")?;
    }
    if let Some(label) = &value.label {
        value::text(label)?;
    }
    value::text(&value.color)?;
    Ok(v1::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis) as i32,
        min: value.min,
        max: value.max,
        center: value.center.as_ref().map(data_expression).transpose()?,
        tolerance: value.tolerance.as_ref().map(data_expression).transpose()?,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn summary_stat(value: dokime_app::document::TelemetrySummaryStat) -> v1::DocumentSummaryStat {
    match value {
        dokime_app::document::TelemetrySummaryStat::Latest => v1::DocumentSummaryStat::Latest,
    }
}

fn summary_format(
    value: dokime_app::document::TelemetrySummaryFormat,
) -> v1::DocumentSummaryFormat {
    match value {
        dokime_app::document::TelemetrySummaryFormat::Auto => v1::DocumentSummaryFormat::Auto,
        dokime_app::document::TelemetrySummaryFormat::Number => v1::DocumentSummaryFormat::Number,
        dokime_app::document::TelemetrySummaryFormat::Percent => v1::DocumentSummaryFormat::Percent,
        dokime_app::document::TelemetrySummaryFormat::Duration => {
            v1::DocumentSummaryFormat::Duration
        }
        dokime_app::document::TelemetrySummaryFormat::Distance => {
            v1::DocumentSummaryFormat::Distance
        }
        dokime_app::document::TelemetrySummaryFormat::Status => v1::DocumentSummaryFormat::Status,
        dokime_app::document::TelemetrySummaryFormat::Text => v1::DocumentSummaryFormat::Text,
    }
}

fn summary_presentation(
    value: dokime_app::document::TelemetrySummaryPresentation,
) -> v1::DocumentSummaryPresentation {
    match value {
        dokime_app::document::TelemetrySummaryPresentation::Value => {
            v1::DocumentSummaryPresentation::Value
        }
        dokime_app::document::TelemetrySummaryPresentation::Progress => {
            v1::DocumentSummaryPresentation::Progress
        }
        dokime_app::document::TelemetrySummaryPresentation::Badge => {
            v1::DocumentSummaryPresentation::Badge
        }
    }
}

fn summary_state(value: dokime_app::document::TelemetrySummaryState) -> v1::DocumentSummaryState {
    match value {
        dokime_app::document::TelemetrySummaryState::Ok => v1::DocumentSummaryState::Ok,
        dokime_app::document::TelemetrySummaryState::Warning => v1::DocumentSummaryState::Warning,
        dokime_app::document::TelemetrySummaryState::Critical => v1::DocumentSummaryState::Critical,
        dokime_app::document::TelemetrySummaryState::Muted => v1::DocumentSummaryState::Muted,
    }
}

fn owner_body(
    kind: v1::DocumentCardKind,
    plot_id: Option<String>,
    _scope: &dokime_app::core::RecordScope,
) -> Result<v1::historical_document_card::Body, Status> {
    Ok(v1::historical_document_card::Body::Owner(
        v1::DocumentOwnerReference {
            kind: kind as i32,
            plot_id,
            plot: None,
        },
    ))
}

fn layout_nodes_for_node(
    node: &dokime_app::document::DocumentNode,
    parent: Option<u64>,
    ordinal: &mut u64,
    output: &mut Vec<v1::DocumentLayoutNode>,
) -> Result<(), Status> {
    use dokime_app::document::DocumentNode;
    match node {
        DocumentNode::Section(section) => {
            let current = *ordinal;
            output.push(v1::DocumentLayoutNode {
                ordinal: current,
                parent_ordinal: parent,
                node: Some(v1::document_layout_node::Node::Section(Box::new(
                    v1::DocumentSection {
                        node_id: section.node_id.as_str().to_owned(),
                        metadata: Some(metadata(&section.metadata)?),
                        repeat: section.repeat.as_ref().map(repeat).transpose()?,
                    },
                ))),
            });
            *ordinal += 1;
            for child in &section.children {
                layout_nodes_for_node(child, Some(current), ordinal, output)?;
            }
        }
        DocumentNode::Card(card) => {
            output.push(v1::DocumentLayoutNode {
                ordinal: *ordinal,
                parent_ordinal: parent,
                node: Some(v1::document_layout_node::Node::Placement(
                    v1::DocumentPlacement {
                        card_id: card.card_id.as_str().to_owned(),
                        column_span: Some(span(&card.layout.col_span)),
                        row_span: u32::from(card.layout.row_span),
                        order: card.layout.order,
                        density: density(&card.layout.density) as i32,
                        rail: rail(&card.layout.rail) as i32,
                    },
                )),
            });
            *ordinal += 1;
        }
    }
    Ok(())
}

pub(in crate::grpc) fn repeat(
    source: &dokime_app::document::DocumentSectionRepeat,
) -> Result<v1::DocumentRepeat, Status> {
    value::text(&source.label_template)?;
    if let Some(id) = &source.id_template {
        value::text(id)?;
    }
    Ok(v1::DocumentRepeat {
        value: Some(data_expression(&source.value)?),
        label_template: source.label_template.clone(),
        id_template: source.id_template.clone(),
        order_by: source.order_by.as_ref().map(data_expression).transpose()?,
        filters: collect(source.filters.iter().map(plot_filter))?,
    })
}

fn span(value: &dokime_app::document::ResponsiveSpan) -> v1::DocumentSpan {
    v1::DocumentSpan {
        base: u32::from(value.base),
        sm: value.sm.map(u32::from),
        md: value.md.map(u32::from),
        lg: value.lg.map(u32::from),
        xl: value.xl.map(u32::from),
    }
}

fn density(value: &dokime_app::document::DocumentCardDensity) -> v1::DocumentDensity {
    match value {
        dokime_app::document::DocumentCardDensity::Compact => v1::DocumentDensity::Compact,
        dokime_app::document::DocumentCardDensity::Normal => v1::DocumentDensity::Normal,
        dokime_app::document::DocumentCardDensity::Spacious => v1::DocumentDensity::Spacious,
    }
}

fn rail(value: &dokime_app::document::DocumentCardRail) -> v1::DocumentRail {
    match value {
        dokime_app::document::DocumentCardRail::Main => v1::DocumentRail::Main,
        dokime_app::document::DocumentCardRail::Right => v1::DocumentRail::Right,
        dokime_app::document::DocumentCardRail::Left => v1::DocumentRail::Left,
    }
}

/* === Tests === */
