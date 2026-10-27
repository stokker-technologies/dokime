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

// Third Party Imports
use tonic::Status;

// Local Imports
use crate::grpc::enums::canonical as enums;
use crate::grpc::value::projection;
use crate::grpc::value::{self, canonical};
use dokime::domain::model::identity::wire as common;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::grpc) fn wire(
    value: &dokime::document::DocumentDefinition, scope: &dokime::domain::model::execution::RecordScope,
) -> Result<dokime::domain::model::history::wire::HistoricalDocument, Status> {
    use dokime::document::DocumentCardBody;
    let presentation = dokime::domain::model::documents::wire::DocumentHeader {
        document_id: value.document_id.as_str().to_owned(),
        metadata: Some(metadata(&value.metadata)?),
        scope: Some(canonical::scope(scope)?),
        frontiers: None,
    };
    let mut layout_nodes = projection::slots(layout_count(&value.layout.nodes)?).map_err(projection_failure)?;
    let mut ordinal = 0;
    for node in &value.layout.nodes {
        layout_nodes_for_node(node, None, &mut ordinal, &mut layout_nodes)?;
    }
    let mut cards = projection::slots(value.cards.len()).map_err(projection_failure)?;
    let rows = value.cards.iter().try_fold(0usize, |count, card| {
        let rows = match &card.body {
            DocumentCardBody::Table { rows } => rows.len(),
            _ => 0,
        };
        count.checked_add(rows).ok_or_else(construction_overflow)
    })?;
    let mut table_rows = projection::slots(rows).map_err(projection_failure)?;
    for card in &value.cards {
        if let DocumentCardBody::Table { rows } = &card.body {
            let mut budget = value::Budget::new()?;
            for (row, fields) in rows.iter().enumerate() {
                table_rows.push(dokime::domain::model::documents::wire::DocumentTableRow {
                    card_id: projection::text(card.metadata.identifier.as_str()).map_err(projection_failure)?,
                    ordinal: row as u64,
                    values: Some(value_object(fields, &mut budget, 0)?),
                });
            }
        }
        cards.push(historical_card(card, scope)?);
    }
    Ok(dokime::domain::model::history::wire::HistoricalDocument { presentation: Some(presentation), layout_nodes, cards, table_rows })
}

pub(in crate::grpc) fn historical_plot(
    value: &dokime::document::PlotDefinition, scope: &dokime::domain::model::execution::RecordScope,
) -> Result<dokime::domain::model::documents::wire::DocumentPlotDefinition, Status> {
    if let Some(window) = value.live_window_s {
        finite(window, "live_window_s")?;
    }
    let request = dokime::domain::model::plots::wire::PlotRequest {
        analysis: value.analysis.as_ref().map(analysis_selection).transpose()?,
        plot_id: Some(value.metadata.identifier.as_str().to_owned()),
        plot_type: enums::plot_type(&value.plot_type),
        x_scale: enums::axis_scale(&value.x_scale),
        y_scale: enums::axis_scale(&value.y_scale),
        stream: value.stream.clone().unwrap_or_default(),
        y_keys: value.y.clone(),
        marker_record_types: value.marker_record_types.clone(),
        filters: collect(value.filters.iter().map(plot_filter))?,
        x_key: value.x.clone(),
        series_by: value.series_by.clone(),
        x_expression: value.x_expression.as_ref().map(data_expression).transpose()?,
        y_expressions: collect(value.y_expressions.iter().map(data_expression))?,
        series_by_expression: value.series_by_expression.as_ref().map(data_expression).transpose()?,
        z_expression: value.z_expression.as_ref().map(data_expression).transpose()?,
        fft: Some(fft_options(&value.fft)?),
        interval: Some(interval_options(&value.interval)?),
        histogram: Some(histogram_options(&value.histogram)?),
        z_stream: value.z_stream.clone(),
        z_field: value.z_field.clone(),
        unit: value.unit.clone(),
        max_points: value.display_points.map(u64::from),
        window_s: value.live_window_s,
        scope: Some(canonical::scope(scope)?),
        technique: enums::downsample_technique(&value.downsample),
        summary_window: None,
    };
    let (x_axis, x_axis_unit) = enums::plot_axis(value.x_axis.as_ref());
    let presentation = dokime::domain::model::documents::wire::PlotPresentation {
        style: enums::plot_style(&value.style),
        fft: Some(fft_options(&value.fft)?),
        interval: Some(interval_options(&value.interval)?),
        histogram: Some(histogram_options(&value.histogram)?),
        reference_lines: collect(value.reference_lines.iter().map(reference_line))?,
        reference_bands: collect(value.reference_bands.iter().map(reference_band))?,
        live_window_s: value.live_window_s,
        x_format: value.x_format.as_ref().map(canonical::format),
        y_format: value.y_format.as_ref().map(canonical::format),
        x_axis,
        x_axis_unit,
    };
    Ok(dokime::domain::model::documents::wire::DocumentPlotDefinition {
        metadata: Some(metadata(&value.metadata)?),
        request: Some(request),
        presentation: Some(presentation),
        z_expression: value.z_expression.as_ref().map(data_expression).transpose()?,
    })
}

pub(in crate::grpc) fn repeat(
    source: &dokime::document::DocumentSectionRepeat,
) -> Result<dokime::domain::model::documents::wire::DocumentRepeat, Status> {
    Ok(dokime::domain::model::documents::wire::DocumentRepeat {
        value: Some(data_expression(&source.value)?),
        label_template: source.label_template.clone(),
        id_template: source.id_template.clone(),
        order_by: source.order_by.as_ref().map(data_expression).transpose()?,
        filters: collect(source.filters.iter().map(plot_filter))?,
    })
}

/* === Private Functions === */

fn layout_count(nodes: &[dokime::document::DocumentNode]) -> Result<usize, Status> {
    nodes.iter().try_fold(0usize, |count, node| {
        let children = match node {
            dokime::document::DocumentNode::Section(section) => layout_count(&section.children)?,
            dokime::document::DocumentNode::Card(_) => 0,
        };
        count.checked_add(1).and_then(|count| count.checked_add(children)).ok_or_else(construction_overflow)
    })
}

fn construction_overflow() -> Status {
    Status::resource_exhausted("historical document construction overflows")
}

fn projection_failure(error: dokime::harmos::Error) -> Status {
    match error {
        dokime::harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted("historical document construction exceeds its bound"),
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
    source: &std::collections::BTreeMap<String, dokime::domain::model::values::Value>, budget: &mut value::Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    let _ = depth;
    canonical::object(source, budget)
}

fn metadata(source: &dokime::domain::model::identity::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &dokime::domain::model::values::Value, budget: &mut value::Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn historical_card(
    value: &dokime::document::DocumentCardDefinition, scope: &dokime::domain::model::execution::RecordScope,
) -> Result<dokime::domain::model::history::wire::HistoricalDocumentCard, Status> {
    use dokime::document::DocumentCardBody;
    let body = match &value.body {
        DocumentCardBody::Text { markdown } => {
            Some(dokime::domain::model::history::wire::HistoricalDocumentCardBody::Markdown(markdown.clone()))
        }
        DocumentCardBody::Table { rows } => Some(dokime::domain::model::history::wire::HistoricalDocumentCardBody::Table(
            dokime::domain::model::documents::wire::DocumentTable { row_count: rows.len() as u64 },
        )),
        DocumentCardBody::TelemetrySummary { summary } => {
            Some(dokime::domain::model::history::wire::HistoricalDocumentCardBody::Summary(historical_summary(summary)?))
        }
        DocumentCardBody::Plot { plot } => {
            Some(dokime::domain::model::history::wire::HistoricalDocumentCardBody::Plot(historical_plot(plot, scope)?))
        }
        DocumentCardBody::Steps => Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Steps, None, scope)?),
        DocumentCardBody::Criteria => Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Criteria, None, scope)?),
        DocumentCardBody::Traceability => {
            Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Traceability, None, scope)?)
        }
        DocumentCardBody::Notes => Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Notes, None, scope)?),
        DocumentCardBody::Records => Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Records, None, scope)?),
        DocumentCardBody::Artifacts => Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Artifacts, None, scope)?),
        DocumentCardBody::Media => Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Media, None, scope)?),
        DocumentCardBody::Health => Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::Health, None, scope)?),
        DocumentCardBody::TelemetryContext => {
            Some(owner_body(dokime::domain::model::documents::wire::DocumentCardKind::TelemetryContext, None, scope)?)
        }
    };
    Ok(dokime::domain::model::history::wire::HistoricalDocumentCard {
        metadata: Some(metadata(&value.metadata)?),
        filters: collect(value.filters.iter().map(historical_filter))?,
        body,
    })
}

fn historical_summary(
    value: &dokime::document::TelemetrySummaryDefinition,
) -> Result<dokime::domain::model::history::wire::HistoricalTelemetrySummary, Status> {
    Ok(dokime::domain::model::history::wire::HistoricalTelemetrySummary {
        groups: collect(value.groups.iter().map(|group| {
            Ok(dokime::domain::model::history::wire::HistoricalSummaryGroup {
                metadata: Some(metadata(&group.metadata)?),
                tiles: collect(group.tiles.iter().map(historical_summary_tile))?,
            })
        }))?,
    })
}

fn historical_summary_tile(
    value: &dokime::document::TelemetrySummaryTileDefinition,
) -> Result<dokime::domain::model::history::wire::HistoricalSummaryTile, Status> {
    Ok(dokime::domain::model::history::wire::HistoricalSummaryTile {
        metadata: Some(metadata(&value.metadata)?),
        value: Some(data_expression(&value.value)?),
        filters: collect(value.filters.iter().map(historical_filter))?,
        stat: summary_stat(value.stat.clone()),
        format: summary_format(value.format.clone()),
        presentation: summary_presentation(value.presentation.clone()),
        unit: value.unit.as_ref().map(|unit| Ok::<_, Status>(unit.clone())).transpose()?,
        conditions: collect(value.conditions.iter().map(|condition| {
            Ok(dokime::domain::model::history::wire::HistoricalSummaryCondition {
                operator: enums::filter_operator(&condition.operator),
                value: Some(project_value(&condition.value, &mut value::Budget::new()?, 0)?),
                state: summary_state(condition.state.clone()),
            })
        }))?,
    })
}

fn historical_filter(
    value: &dokime::document::DocumentFilter,
) -> Result<dokime::domain::model::history::wire::HistoricalDocumentFilter, Status> {
    Ok(dokime::domain::model::history::wire::HistoricalDocumentFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator),
        value: value.value.as_ref().map(|item| project_value(item, &mut value::Budget::new()?, 0)).transpose()?,
    })
}

fn data_expression(value: &dokime::expressions::DataExpression) -> Result<dokime::domain::model::plots::wire::DataExpression, Status> {
    Ok(dokime::domain::model::plots::wire::DataExpression {
        label: value.label.as_deref().map(projection::text).transpose().map_err(projection_failure)?,
        source: Some(data_source(&value.source)?),
        transforms: collect(value.transforms.iter().map(data_transform))?,
    })
}

fn data_source(value: &dokime::expressions::DataSource) -> Result<dokime::domain::model::plots::wire::DataExpressionSource, Status> {
    use dokime::expressions::DataSource;
    Ok(match value {
        DataSource::Telemetry { stream, field } => {
            dokime::domain::model::plots::wire::DataExpressionSource::Telemetry(dokime::domain::model::plots::wire::TelemetryDataSource {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataSource::Parameter { name } => {
            dokime::domain::model::plots::wire::DataExpressionSource::Parameter(dokime::domain::model::plots::wire::ParameterDataSource {
                name: name.clone(),
            })
        }
        DataSource::Dimension { name } => {
            dokime::domain::model::plots::wire::DataExpressionSource::Dimension(dokime::domain::model::plots::wire::DimensionDataSource {
                name: name.clone(),
            })
        }
        DataSource::Record { record_type } => {
            dokime::domain::model::plots::wire::DataExpressionSource::Record(dokime::domain::model::plots::wire::RecordDataSource {
                record_type: record_type.clone(),
            })
        }
        DataSource::Artifact { artifact } => {
            dokime::domain::model::plots::wire::DataExpressionSource::Artifact(dokime::domain::model::plots::wire::ArtifactDataSource {
                artifact: artifact.clone(),
            })
        }
    })
}

fn data_transform(value: &dokime::expressions::DataTransform) -> Result<dokime::domain::model::plots::wire::DataTransform, Status> {
    use dokime::domain::model::plots::wire::DataTransformKind as Kind;
    use dokime::expressions::DataTransform;
    let kind = match value {
        DataTransform::Aggregate(aggregate) => {
            Kind::Aggregate(dokime::domain::model::plots::wire::AggregateTransform { aggregate: *aggregate })
        }
        DataTransform::TimeBetween { previous_record_type } => {
            Kind::TimeBetween(dokime::domain::model::plots::wire::TimeBetweenTransform {
                previous_record_type: previous_record_type.clone(),
            })
        }
        DataTransform::TelemetryAtOrBefore { stream, field } => {
            Kind::TelemetryAtOrBefore(dokime::domain::model::plots::wire::TelemetryAtOrBeforeTransform {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataTransform::Detail { name } => Kind::Detail(dokime::domain::model::plots::wire::DetailTransform { name: name.clone() }),
        DataTransform::PerCase => Kind::PerCase(dokime::domain::model::plots::wire::EmptyTransform {}),
    };
    Ok(dokime::domain::model::plots::wire::DataTransform { kind: Some(kind) })
}

fn plot_filter(value: &dokime::document::DocumentFilter) -> Result<dokime::domain::model::plots::wire::PlotFilter, Status> {
    Ok(dokime::domain::model::plots::wire::PlotFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator),
        value: value.value.as_ref().map(|item| project_value(item, &mut value::Budget::new()?, 0)).transpose()?,
    })
}

fn finite(value: f64, field: &str) -> Result<(), Status> {
    if value.is_finite() { Ok(()) } else { Err(Status::data_loss(format!("stored plot {field} is nonfinite"))) }
}

fn fft_options(value: &dokime::document::FftOptions) -> Result<dokime::domain::model::plots::wire::PlotFftOptions, Status> {
    for item in [value.sample_rate_hz, value.min_frequency_hz, value.max_frequency_hz].into_iter().flatten() {
        finite(item, "fft option")?;
    }
    Ok(dokime::domain::model::plots::wire::PlotFftOptions {
        sample_rate_hz: value.sample_rate_hz,
        window: enums::fft_window(&value.window),
        detrend: enums::fft_detrend(&value.detrend),
        level_unit: enums::frequency_level_unit(&value.level_unit),
        include_dc: value.include_dc,
        min_frequency_hz: value.min_frequency_hz,
        max_frequency_hz: value.max_frequency_hz,
    })
}

fn interval_options(
    value: &dokime::document::SigmaIntervalOptions,
) -> Result<dokime::domain::model::plots::wire::PlotIntervalOptions, Status> {
    finite(value.bin_width, "interval bin_width")?;
    finite(value.sigma, "interval sigma")?;
    Ok(dokime::domain::model::plots::wire::PlotIntervalOptions {
        bin_width: value.bin_width,
        sigma: value.sigma,
        clip_to_observed: value.clip_to_observed,
    })
}

fn histogram_options(
    value: &dokime::document::HistogramOptions,
) -> Result<dokime::domain::model::plots::wire::PlotHistogramOptions, Status> {
    if value.bins > u64::MAX as usize {
        return Err(Status::data_loss("stored histogram bin count overflows wire"));
    }
    Ok(dokime::domain::model::plots::wire::PlotHistogramOptions { bins: value.bins as u64, unit: value.unit.clone() })
}

fn reference_line(value: &dokime::document::ReferenceLine) -> Result<dokime::domain::model::documents::wire::PlotReferenceLine, Status> {
    finite(value.source_scale, "reference line source_scale")?;
    if let Some(item) = value.value {
        finite(item, "reference line value")?;
    }
    Ok(dokime::domain::model::documents::wire::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis),
        value: value.value,
        source: value.source.as_ref().map(data_expression).transpose()?,
        source_scale: value.source_scale,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn reference_band(value: &dokime::document::ReferenceBand) -> Result<dokime::domain::model::documents::wire::PlotReferenceBand, Status> {
    if let Some(item) = value.min {
        finite(item, "reference band min")?;
    }
    if let Some(item) = value.max {
        finite(item, "reference band max")?;
    }
    Ok(dokime::domain::model::documents::wire::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis),
        min: value.min,
        max: value.max,
        center: value.center.as_ref().map(data_expression).transpose()?,
        tolerance: value.tolerance.as_ref().map(data_expression).transpose()?,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn summary_stat(value: dokime::document::TelemetrySummaryStat) -> dokime::domain::model::documents::wire::DocumentSummaryStat {
    match value {
        dokime::document::TelemetrySummaryStat::Latest => dokime::domain::model::documents::wire::DocumentSummaryStat::Latest,
    }
}

fn summary_format(value: dokime::document::TelemetrySummaryFormat) -> dokime::domain::model::documents::wire::DocumentSummaryFormat {
    match value {
        dokime::document::TelemetrySummaryFormat::Auto => dokime::domain::model::documents::wire::DocumentSummaryFormat::Auto,
        dokime::document::TelemetrySummaryFormat::Number => dokime::domain::model::documents::wire::DocumentSummaryFormat::Number,
        dokime::document::TelemetrySummaryFormat::Percent => dokime::domain::model::documents::wire::DocumentSummaryFormat::Percent,
        dokime::document::TelemetrySummaryFormat::Duration => dokime::domain::model::documents::wire::DocumentSummaryFormat::Duration,
        dokime::document::TelemetrySummaryFormat::Distance => dokime::domain::model::documents::wire::DocumentSummaryFormat::Distance,
        dokime::document::TelemetrySummaryFormat::Status => dokime::domain::model::documents::wire::DocumentSummaryFormat::Status,
        dokime::document::TelemetrySummaryFormat::Text => dokime::domain::model::documents::wire::DocumentSummaryFormat::Text,
    }
}

fn summary_presentation(
    value: dokime::document::TelemetrySummaryPresentation,
) -> dokime::domain::model::documents::wire::DocumentSummaryPresentation {
    match value {
        dokime::document::TelemetrySummaryPresentation::Value => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Value,
        dokime::document::TelemetrySummaryPresentation::Progress => {
            dokime::domain::model::documents::wire::DocumentSummaryPresentation::Progress
        }
        dokime::document::TelemetrySummaryPresentation::Badge => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Badge,
    }
}

fn summary_state(value: dokime::document::TelemetrySummaryState) -> dokime::domain::model::documents::wire::DocumentSummaryState {
    match value {
        dokime::document::TelemetrySummaryState::Ok => dokime::domain::model::documents::wire::DocumentSummaryState::Ok,
        dokime::document::TelemetrySummaryState::Warning => dokime::domain::model::documents::wire::DocumentSummaryState::Warning,
        dokime::document::TelemetrySummaryState::Critical => dokime::domain::model::documents::wire::DocumentSummaryState::Critical,
        dokime::document::TelemetrySummaryState::Muted => dokime::domain::model::documents::wire::DocumentSummaryState::Muted,
    }
}

fn owner_body(
    kind: dokime::domain::model::documents::wire::DocumentCardKind, plot_id: Option<String>,
    _scope: &dokime::domain::model::execution::RecordScope,
) -> Result<dokime::domain::model::history::wire::HistoricalDocumentCardBody, Status> {
    Ok(dokime::domain::model::history::wire::HistoricalDocumentCardBody::Owner(
        dokime::domain::model::documents::wire::DocumentOwnerReference { kind, plot_id, plot: None },
    ))
}

fn layout_nodes_for_node(
    node: &dokime::document::DocumentNode, parent: Option<u64>, ordinal: &mut u64,
    output: &mut Vec<dokime::domain::model::documents::wire::DocumentLayoutNode>,
) -> Result<(), Status> {
    use dokime::document::DocumentNode;
    match node {
        DocumentNode::Section(section) => {
            let current = *ordinal;
            output.push(dokime::domain::model::documents::wire::DocumentLayoutNode {
                ordinal: current,
                parent_ordinal: parent,
                node: Some(dokime::domain::model::documents::wire::DocumentLayoutNodeNode::Section(Box::new(
                    dokime::domain::model::documents::wire::DocumentSection {
                        node_id: section.node_id.as_str().to_owned(),
                        metadata: Some(metadata(&section.metadata)?),
                        repeat: section.repeat.as_ref().map(repeat).transpose()?,
                    },
                ))),
            });
            *ordinal = ordinal.checked_add(1).ok_or_else(construction_overflow)?;
            for child in &section.children {
                layout_nodes_for_node(child, Some(current), ordinal, output)?;
            }
        }
        DocumentNode::Card(card) => {
            output.push(dokime::domain::model::documents::wire::DocumentLayoutNode {
                ordinal: *ordinal,
                parent_ordinal: parent,
                node: Some(dokime::domain::model::documents::wire::DocumentLayoutNodeNode::Placement(
                    dokime::domain::model::documents::wire::DocumentPlacement {
                        card_id: card.card_id.as_str().to_owned(),
                        column_span: Some(span(&card.layout.col_span)),
                        row_span: u32::from(card.layout.row_span),
                        order: card.layout.order,
                        density: density(&card.layout.density),
                        rail: rail(&card.layout.rail),
                    },
                )),
            });
            *ordinal = ordinal.checked_add(1).ok_or_else(construction_overflow)?;
        }
    }
    Ok(())
}

fn span(value: &dokime::document::ResponsiveSpan) -> dokime::domain::model::documents::wire::DocumentSpan {
    dokime::domain::model::documents::wire::DocumentSpan {
        base: u32::from(value.base),
        sm: value.sm.map(u32::from),
        md: value.md.map(u32::from),
        lg: value.lg.map(u32::from),
        xl: value.xl.map(u32::from),
    }
}

fn density(value: &dokime::document::DocumentCardDensity) -> dokime::domain::model::documents::wire::DocumentDensity {
    match value {
        dokime::document::DocumentCardDensity::Compact => dokime::domain::model::documents::wire::DocumentDensity::Compact,
        dokime::document::DocumentCardDensity::Normal => dokime::domain::model::documents::wire::DocumentDensity::Normal,
        dokime::document::DocumentCardDensity::Spacious => dokime::domain::model::documents::wire::DocumentDensity::Spacious,
    }
}

fn rail(value: &dokime::document::DocumentCardRail) -> dokime::domain::model::documents::wire::DocumentRail {
    match value {
        dokime::document::DocumentCardRail::Main => dokime::domain::model::documents::wire::DocumentRail::Main,
        dokime::document::DocumentCardRail::Right => dokime::domain::model::documents::wire::DocumentRail::Right,
        dokime::document::DocumentCardRail::Left => dokime::domain::model::documents::wire::DocumentRail::Left,
    }
}

fn analysis_selection(selection: &dokime::analysis::Selection) -> Result<dokime::domain::model::plots::wire::AnalysisSelection, Status> {
    Ok(dokime::domain::model::plots::wire::AnalysisSelection {
        id: selection.id.clone(),
        version: selection.version,
        parameters: Some(canonical::object(&selection.parameters, &mut value::Budget::new()?)?),
    })
}

/* === Tests === */
