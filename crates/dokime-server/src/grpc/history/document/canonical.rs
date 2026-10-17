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
use dokime_domain::model::identity::wire as common;
use dokime_domain::model::values::wire as value_wire;
use dokime_domain::operations::documents;
use dokime_domain::operations::history;
use dokime_domain::operations::plots;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(in crate::grpc) fn wire(
    value: &dokime_runtime::document::DocumentDefinition, scope: &dokime_domain::model::execution::RecordScope,
) -> Result<dokime_domain::model::history::wire::HistoricalDocument, Status> {
    use dokime_runtime::document::DocumentCardBody;
    let presentation = dokime_domain::model::documents::wire::DocumentHeader {
        document_id: value.document_id.as_str().to_owned(),
        metadata: Some(metadata(&value.metadata)?),
        scope: Some(canonical::scope(scope)?),
        frontiers: None,
    };
    let mut layout_nodes = projection::slots(layout_count(&value.layout.nodes)).map_err(projection_failure)?;
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
                table_rows.push(dokime_domain::model::documents::wire::DocumentTableRow {
                    card_id: projection::text(card.metadata.identifier.as_str()).map_err(projection_failure)?,
                    ordinal: row as u64,
                    values: Some(value_object(fields, &mut budget, 0)?),
                });
            }
        }
        cards.push(historical_card(card, scope)?);
    }
    Ok(dokime_domain::model::history::wire::HistoricalDocument { presentation: Some(presentation), layout_nodes, cards, table_rows })
}

/* === Private Functions === */
fn layout_count(nodes: &[dokime_runtime::document::DocumentNode]) -> usize {
    nodes.iter().fold(0usize, |count, node| {
        let children = match node {
            dokime_runtime::document::DocumentNode::Section(section) => layout_count(&section.children),
            dokime_runtime::document::DocumentNode::Card(_) => 0,
        };
        count.saturating_add(1).saturating_add(children)
    })
}
fn projection_failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted("historical document construction exceeds its bound"),
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
    source: &std::collections::BTreeMap<String, dokime_domain::model::values::Value>, budget: &mut value::Budget, depth: usize,
) -> Result<dokime_domain::model::values::wire::Value, Status> {
    let _ = depth;
    canonical::object(source, budget)
}

fn metadata(source: &dokime_domain::model::identity::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &dokime_domain::model::values::Value, budget: &mut value::Budget, depth: usize,
) -> Result<dokime_domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn historical_card(
    value: &dokime_runtime::document::DocumentCardDefinition, scope: &dokime_domain::model::execution::RecordScope,
) -> Result<dokime_domain::model::history::wire::HistoricalDocumentCard, Status> {
    use dokime_runtime::document::DocumentCardBody;
    let body = match &value.body {
        DocumentCardBody::Text { markdown } => {
            value::text(markdown)?;
            Some(dokime_domain::model::history::wire::HistoricalDocumentCardBody::Markdown(markdown.clone()))
        }
        DocumentCardBody::Table { rows } => Some(dokime_domain::model::history::wire::HistoricalDocumentCardBody::Table(
            dokime_domain::model::documents::wire::DocumentTable { row_count: rows.len() as u64 },
        )),
        DocumentCardBody::TelemetrySummary { summary } => {
            Some(dokime_domain::model::history::wire::HistoricalDocumentCardBody::Summary(historical_summary(summary)?))
        }
        DocumentCardBody::Plot { plot } => {
            Some(dokime_domain::model::history::wire::HistoricalDocumentCardBody::Plot(historical_plot(plot, scope)?))
        }
        DocumentCardBody::Steps => Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Steps, None, scope)?),
        DocumentCardBody::Criteria => Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Criteria, None, scope)?),
        DocumentCardBody::Traceability => {
            Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Traceability, None, scope)?)
        }
        DocumentCardBody::Notes => Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Notes, None, scope)?),
        DocumentCardBody::Records => Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Records, None, scope)?),
        DocumentCardBody::Artifacts => Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Artifacts, None, scope)?),
        DocumentCardBody::Media => Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Media, None, scope)?),
        DocumentCardBody::Health => Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::Health, None, scope)?),
        DocumentCardBody::TelemetryContext => {
            Some(owner_body(dokime_domain::model::documents::wire::DocumentCardKind::TelemetryContext, None, scope)?)
        }
    };
    Ok(dokime_domain::model::history::wire::HistoricalDocumentCard {
        metadata: Some(metadata(&value.metadata)?),
        filters: collect(value.filters.iter().map(historical_filter))?,
        body,
    })
}

fn historical_summary(
    value: &dokime_runtime::document::TelemetrySummaryDefinition,
) -> Result<dokime_domain::model::history::wire::HistoricalTelemetrySummary, Status> {
    value::collection(value.groups.len())?;
    Ok(dokime_domain::model::history::wire::HistoricalTelemetrySummary {
        groups: collect(value.groups.iter().map(|group| {
            value::collection(group.tiles.len())?;
            Ok(dokime_domain::model::history::wire::HistoricalSummaryGroup {
                metadata: Some(metadata(&group.metadata)?),
                tiles: collect(group.tiles.iter().map(historical_summary_tile))?,
            })
        }))?,
    })
}

fn historical_summary_tile(
    value: &dokime_runtime::document::TelemetrySummaryTileDefinition,
) -> Result<dokime_domain::model::history::wire::HistoricalSummaryTile, Status> {
    value::collection(value.filters.len())?;
    value::collection(value.conditions.len())?;
    Ok(dokime_domain::model::history::wire::HistoricalSummaryTile {
        metadata: Some(metadata(&value.metadata)?),
        value: Some(data_expression(&value.value)?),
        filters: collect(value.filters.iter().map(historical_filter))?,
        stat: summary_stat(value.stat.clone()),
        format: summary_format(value.format.clone()),
        presentation: summary_presentation(value.presentation.clone()),
        unit: value
            .unit
            .as_ref()
            .map(|unit| {
                value::text(unit)?;
                Ok::<_, Status>(unit.clone())
            })
            .transpose()?,
        conditions: collect(value.conditions.iter().map(|condition| {
            Ok(dokime_domain::model::history::wire::HistoricalSummaryCondition {
                operator: enums::filter_operator(&condition.operator),
                value: Some(project_value(&condition.value, &mut value::Budget::new(), 0)?),
                state: summary_state(condition.state.clone()),
            })
        }))?,
    })
}

fn historical_filter(
    value: &dokime_runtime::document::DocumentFilter,
) -> Result<dokime_domain::model::history::wire::HistoricalDocumentFilter, Status> {
    Ok(dokime_domain::model::history::wire::HistoricalDocumentFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator),
        value: value.value.as_ref().map(|item| project_value(item, &mut value::Budget::new(), 0)).transpose()?,
    })
}

fn data_expression(
    value: &dokime_runtime::expressions::DataExpression,
) -> Result<dokime_domain::model::plots::wire::DataExpression, Status> {
    value::collection(value.transforms.len())?;
    Ok(dokime_domain::model::plots::wire::DataExpression {
        label: value.label.as_deref().map(projection::text).transpose().map_err(projection_failure)?,
        source: Some(data_source(&value.source)?),
        transforms: collect(value.transforms.iter().map(data_transform))?,
    })
}

fn data_source(value: &dokime_runtime::expressions::DataSource) -> Result<dokime_domain::model::plots::wire::DataExpressionSource, Status> {
    use dokime_runtime::expressions::DataSource;
    Ok(match value {
        DataSource::Telemetry { stream, field } => {
            value::text(stream)?;
            value::text(field)?;
            dokime_domain::model::plots::wire::DataExpressionSource::Telemetry(dokime_domain::model::plots::wire::TelemetryDataSource {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataSource::Parameter { name } => {
            value::text(name)?;
            dokime_domain::model::plots::wire::DataExpressionSource::Parameter(dokime_domain::model::plots::wire::ParameterDataSource {
                name: name.clone(),
            })
        }
        DataSource::Dimension { name } => {
            value::text(name)?;
            dokime_domain::model::plots::wire::DataExpressionSource::Dimension(dokime_domain::model::plots::wire::DimensionDataSource {
                name: name.clone(),
            })
        }
        DataSource::Record { record_type } => {
            value::text(record_type)?;
            dokime_domain::model::plots::wire::DataExpressionSource::Record(dokime_domain::model::plots::wire::RecordDataSource {
                record_type: record_type.clone(),
            })
        }
        DataSource::Artifact { artifact } => {
            value::text(artifact)?;
            dokime_domain::model::plots::wire::DataExpressionSource::Artifact(dokime_domain::model::plots::wire::ArtifactDataSource {
                artifact: artifact.clone(),
            })
        }
    })
}

fn data_transform(value: &dokime_runtime::expressions::DataTransform) -> Result<dokime_domain::model::plots::wire::DataTransform, Status> {
    use dokime_domain::model::plots::wire::DataTransformKind as Kind;
    use dokime_runtime::expressions::DataTransform;
    let kind = match value {
        DataTransform::Aggregate(aggregate) => {
            Kind::Aggregate(dokime_domain::model::plots::wire::AggregateTransform { aggregate: *aggregate })
        }
        DataTransform::TimeBetween { previous_record_type } => {
            value::text(previous_record_type)?;
            Kind::TimeBetween(dokime_domain::model::plots::wire::TimeBetweenTransform {
                previous_record_type: previous_record_type.clone(),
            })
        }
        DataTransform::TelemetryAtOrBefore { stream, field } => {
            value::text(stream)?;
            value::text(field)?;
            Kind::TelemetryAtOrBefore(dokime_domain::model::plots::wire::TelemetryAtOrBeforeTransform {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataTransform::Detail { name } => {
            value::text(name)?;
            Kind::Detail(dokime_domain::model::plots::wire::DetailTransform { name: name.clone() })
        }
        DataTransform::PerCase => Kind::PerCase(dokime_domain::model::plots::wire::EmptyTransform {}),
    };
    Ok(dokime_domain::model::plots::wire::DataTransform { kind: Some(kind) })
}

pub(in crate::grpc) fn historical_plot(
    value: &dokime_runtime::document::PlotDefinition, scope: &dokime_domain::model::execution::RecordScope,
) -> Result<dokime_domain::model::documents::wire::DocumentPlotDefinition, Status> {
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
    if let Some(window) = value.live_window_s {
        finite(window, "live_window_s")?;
    }
    let request = dokime_domain::model::plots::wire::PlotRequest {
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
        cursor: None,
        technique: enums::downsample_technique(&value.downsample),
        summary_window: None,
    };
    let (x_axis, x_axis_unit) = enums::plot_axis(value.x_axis.as_ref());
    let presentation = dokime_domain::model::documents::wire::PlotPresentation {
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
    Ok(dokime_domain::model::documents::wire::DocumentPlotDefinition {
        metadata: Some(metadata(&value.metadata)?),
        request: Some(request),
        presentation: Some(presentation),
        z_expression: value.z_expression.as_ref().map(data_expression).transpose()?,
    })
}

fn plot_filter(value: &dokime_runtime::document::DocumentFilter) -> Result<dokime_domain::model::plots::wire::PlotFilter, Status> {
    Ok(dokime_domain::model::plots::wire::PlotFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator),
        value: value.value.as_ref().map(|item| project_value(item, &mut value::Budget::new(), 0)).transpose()?,
    })
}

fn finite(value: f64, field: &str) -> Result<(), Status> {
    if value.is_finite() { Ok(()) } else { Err(Status::data_loss(format!("stored plot {field} is nonfinite"))) }
}

fn fft_options(value: &dokime_runtime::document::FftOptions) -> Result<dokime_domain::model::plots::wire::PlotFftOptions, Status> {
    for item in [value.sample_rate_hz, value.min_frequency_hz, value.max_frequency_hz].into_iter().flatten() {
        finite(item, "fft option")?;
    }
    Ok(dokime_domain::model::plots::wire::PlotFftOptions {
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
    value: &dokime_runtime::document::SigmaIntervalOptions,
) -> Result<dokime_domain::model::plots::wire::PlotIntervalOptions, Status> {
    finite(value.bin_width, "interval bin_width")?;
    finite(value.sigma, "interval sigma")?;
    Ok(dokime_domain::model::plots::wire::PlotIntervalOptions {
        bin_width: value.bin_width,
        sigma: value.sigma,
        clip_to_observed: value.clip_to_observed,
    })
}

fn histogram_options(
    value: &dokime_runtime::document::HistogramOptions,
) -> Result<dokime_domain::model::plots::wire::PlotHistogramOptions, Status> {
    if value.bins > u64::MAX as usize {
        return Err(Status::data_loss("stored histogram bin count overflows wire"));
    }
    if let Some(unit) = &value.unit {
        value::text(unit)?;
    }
    Ok(dokime_domain::model::plots::wire::PlotHistogramOptions { bins: value.bins as u64, unit: value.unit.clone() })
}

fn reference_line(
    value: &dokime_runtime::document::ReferenceLine,
) -> Result<dokime_domain::model::documents::wire::PlotReferenceLine, Status> {
    finite(value.source_scale, "reference line source_scale")?;
    if let Some(item) = value.value {
        finite(item, "reference line value")?;
    }
    if let Some(label) = &value.label {
        value::text(label)?;
    }
    value::text(&value.color)?;
    Ok(dokime_domain::model::documents::wire::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis),
        value: value.value,
        source: value.source.as_ref().map(data_expression).transpose()?,
        source_scale: value.source_scale,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn reference_band(
    value: &dokime_runtime::document::ReferenceBand,
) -> Result<dokime_domain::model::documents::wire::PlotReferenceBand, Status> {
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
    Ok(dokime_domain::model::documents::wire::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis),
        min: value.min,
        max: value.max,
        center: value.center.as_ref().map(data_expression).transpose()?,
        tolerance: value.tolerance.as_ref().map(data_expression).transpose()?,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn summary_stat(value: dokime_runtime::document::TelemetrySummaryStat) -> dokime_domain::model::documents::wire::DocumentSummaryStat {
    match value {
        dokime_runtime::document::TelemetrySummaryStat::Latest => dokime_domain::model::documents::wire::DocumentSummaryStat::Latest,
    }
}

fn summary_format(value: dokime_runtime::document::TelemetrySummaryFormat) -> dokime_domain::model::documents::wire::DocumentSummaryFormat {
    match value {
        dokime_runtime::document::TelemetrySummaryFormat::Auto => dokime_domain::model::documents::wire::DocumentSummaryFormat::Auto,
        dokime_runtime::document::TelemetrySummaryFormat::Number => dokime_domain::model::documents::wire::DocumentSummaryFormat::Number,
        dokime_runtime::document::TelemetrySummaryFormat::Percent => dokime_domain::model::documents::wire::DocumentSummaryFormat::Percent,
        dokime_runtime::document::TelemetrySummaryFormat::Duration => {
            dokime_domain::model::documents::wire::DocumentSummaryFormat::Duration
        }
        dokime_runtime::document::TelemetrySummaryFormat::Distance => {
            dokime_domain::model::documents::wire::DocumentSummaryFormat::Distance
        }
        dokime_runtime::document::TelemetrySummaryFormat::Status => dokime_domain::model::documents::wire::DocumentSummaryFormat::Status,
        dokime_runtime::document::TelemetrySummaryFormat::Text => dokime_domain::model::documents::wire::DocumentSummaryFormat::Text,
    }
}

fn summary_presentation(
    value: dokime_runtime::document::TelemetrySummaryPresentation,
) -> dokime_domain::model::documents::wire::DocumentSummaryPresentation {
    match value {
        dokime_runtime::document::TelemetrySummaryPresentation::Value => {
            dokime_domain::model::documents::wire::DocumentSummaryPresentation::Value
        }
        dokime_runtime::document::TelemetrySummaryPresentation::Progress => {
            dokime_domain::model::documents::wire::DocumentSummaryPresentation::Progress
        }
        dokime_runtime::document::TelemetrySummaryPresentation::Badge => {
            dokime_domain::model::documents::wire::DocumentSummaryPresentation::Badge
        }
    }
}

fn summary_state(value: dokime_runtime::document::TelemetrySummaryState) -> dokime_domain::model::documents::wire::DocumentSummaryState {
    match value {
        dokime_runtime::document::TelemetrySummaryState::Ok => dokime_domain::model::documents::wire::DocumentSummaryState::Ok,
        dokime_runtime::document::TelemetrySummaryState::Warning => dokime_domain::model::documents::wire::DocumentSummaryState::Warning,
        dokime_runtime::document::TelemetrySummaryState::Critical => dokime_domain::model::documents::wire::DocumentSummaryState::Critical,
        dokime_runtime::document::TelemetrySummaryState::Muted => dokime_domain::model::documents::wire::DocumentSummaryState::Muted,
    }
}

fn owner_body(
    kind: dokime_domain::model::documents::wire::DocumentCardKind, plot_id: Option<String>,
    _scope: &dokime_domain::model::execution::RecordScope,
) -> Result<dokime_domain::model::history::wire::HistoricalDocumentCardBody, Status> {
    Ok(dokime_domain::model::history::wire::HistoricalDocumentCardBody::Owner(
        dokime_domain::model::documents::wire::DocumentOwnerReference { kind, plot_id, plot: None },
    ))
}

fn layout_nodes_for_node(
    node: &dokime_runtime::document::DocumentNode, parent: Option<u64>, ordinal: &mut u64,
    output: &mut Vec<dokime_domain::model::documents::wire::DocumentLayoutNode>,
) -> Result<(), Status> {
    use dokime_runtime::document::DocumentNode;
    match node {
        DocumentNode::Section(section) => {
            let current = *ordinal;
            output.push(dokime_domain::model::documents::wire::DocumentLayoutNode {
                ordinal: current,
                parent_ordinal: parent,
                node: Some(dokime_domain::model::documents::wire::DocumentLayoutNodeNode::Section(Box::new(
                    dokime_domain::model::documents::wire::DocumentSection {
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
            output.push(dokime_domain::model::documents::wire::DocumentLayoutNode {
                ordinal: *ordinal,
                parent_ordinal: parent,
                node: Some(dokime_domain::model::documents::wire::DocumentLayoutNodeNode::Placement(
                    dokime_domain::model::documents::wire::DocumentPlacement {
                        card_id: card.card_id.as_str().to_owned(),
                        column_span: Some(span(&card.layout.col_span)),
                        row_span: u32::from(card.layout.row_span),
                        order: card.layout.order,
                        density: density(&card.layout.density),
                        rail: rail(&card.layout.rail),
                    },
                )),
            });
            *ordinal += 1;
        }
    }
    Ok(())
}

pub(in crate::grpc) fn repeat(
    source: &dokime_runtime::document::DocumentSectionRepeat,
) -> Result<dokime_domain::model::documents::wire::DocumentRepeat, Status> {
    value::text(&source.label_template)?;
    if let Some(id) = &source.id_template {
        value::text(id)?;
    }
    Ok(dokime_domain::model::documents::wire::DocumentRepeat {
        value: Some(data_expression(&source.value)?),
        label_template: source.label_template.clone(),
        id_template: source.id_template.clone(),
        order_by: source.order_by.as_ref().map(data_expression).transpose()?,
        filters: collect(source.filters.iter().map(plot_filter))?,
    })
}

fn span(value: &dokime_runtime::document::ResponsiveSpan) -> dokime_domain::model::documents::wire::DocumentSpan {
    dokime_domain::model::documents::wire::DocumentSpan {
        base: u32::from(value.base),
        sm: value.sm.map(u32::from),
        md: value.md.map(u32::from),
        lg: value.lg.map(u32::from),
        xl: value.xl.map(u32::from),
    }
}

fn density(value: &dokime_runtime::document::DocumentCardDensity) -> dokime_domain::model::documents::wire::DocumentDensity {
    match value {
        dokime_runtime::document::DocumentCardDensity::Compact => dokime_domain::model::documents::wire::DocumentDensity::Compact,
        dokime_runtime::document::DocumentCardDensity::Normal => dokime_domain::model::documents::wire::DocumentDensity::Normal,
        dokime_runtime::document::DocumentCardDensity::Spacious => dokime_domain::model::documents::wire::DocumentDensity::Spacious,
    }
}

fn rail(value: &dokime_runtime::document::DocumentCardRail) -> dokime_domain::model::documents::wire::DocumentRail {
    match value {
        dokime_runtime::document::DocumentCardRail::Main => dokime_domain::model::documents::wire::DocumentRail::Main,
        dokime_runtime::document::DocumentCardRail::Right => dokime_domain::model::documents::wire::DocumentRail::Right,
        dokime_runtime::document::DocumentCardRail::Left => dokime_domain::model::documents::wire::DocumentRail::Left,
    }
}

fn analysis_selection(
    selection: &dokime_runtime::analysis::Selection,
) -> Result<dokime_domain::model::plots::wire::AnalysisSelection, Status> {
    value::text(&selection.id)?;
    Ok(dokime_domain::model::plots::wire::AnalysisSelection {
        id: selection.id.clone(),
        version: selection.version,
        parameters: Some(canonical::object(&selection.parameters, &mut value::Budget::new())?),
    })
}

/* === Tests === */
