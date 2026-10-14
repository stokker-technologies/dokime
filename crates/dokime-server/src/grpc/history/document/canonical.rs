/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod memory;

/* === Re-Exports === */
pub(in crate::grpc) use memory::memory as construction_memory;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::Status;
// Local Imports
use crate::grpc::enums::canonical as enums;
use crate::grpc::value::projection;
use crate::grpc::value::{self, canonical};
use dokime_contract::operations::{common, documents, history, plots, value as value_wire};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(in crate::grpc) fn wire(
    value: &dokime_app::document::DocumentDefinition,
    scope: &dokime_app::core::RecordScope,
) -> Result<history::HistoricalDocument, Status> {
    use dokime_app::document::DocumentCardBody;
    let presentation = documents::DocumentHeader {
        document_id: value.document_id.as_str().to_owned(),
        metadata: Some(metadata(&value.metadata)?),
        scope: Some(canonical::scope(scope)?),
        frontiers: None,
    };
    let mut layout_nodes =
        projection::slots(memory::layout_count(&value.layout.nodes)).map_err(projection_failure)?;
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
                table_rows.push(documents::DocumentTableRow {
                    card_id: projection::text(card.metadata.identifier.as_str())
                        .map_err(projection_failure)?,
                    ordinal: row as u64,
                    values: Some(value_object(fields, &mut budget, 0)?),
                });
            }
        }
        cards.push(historical_card(card, scope)?);
    }
    Ok(history::HistoricalDocument {
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
) -> Result<value_wire::Value, Status> {
    let _ = depth;
    canonical::object(source, budget)
}

fn metadata(source: &dokime_app::core::Metadata) -> Result<common::Metadata, Status> {
    canonical::metadata(source)
}

fn project_value(
    source: &dokime_app::core::Value,
    budget: &mut value::Budget,
    depth: usize,
) -> Result<value_wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn historical_card(
    value: &dokime_app::document::DocumentCardDefinition,
    scope: &dokime_app::core::RecordScope,
) -> Result<history::HistoricalDocumentCard, Status> {
    use dokime_app::document::DocumentCardBody;
    let body = match &value.body {
        DocumentCardBody::Text { markdown } => {
            value::text(markdown)?;
            Some(history::HistoricalDocumentCardBody::Markdown(
                markdown.clone(),
            ))
        }
        DocumentCardBody::Table { rows } => Some(history::HistoricalDocumentCardBody::Table(
            documents::DocumentTable {
                row_count: rows.len() as u64,
            },
        )),
        DocumentCardBody::TelemetrySummary { summary } => Some(
            history::HistoricalDocumentCardBody::Summary(historical_summary(summary)?),
        ),
        DocumentCardBody::Plot { plot } => Some(history::HistoricalDocumentCardBody::Plot(
            historical_plot(plot, scope)?,
        )),
        DocumentCardBody::Steps => {
            Some(owner_body(documents::DocumentCardKind::Steps, None, scope)?)
        }
        DocumentCardBody::Criteria => Some(owner_body(
            documents::DocumentCardKind::Criteria,
            None,
            scope,
        )?),
        DocumentCardBody::Traceability => Some(owner_body(
            documents::DocumentCardKind::Traceability,
            None,
            scope,
        )?),
        DocumentCardBody::Notes => {
            Some(owner_body(documents::DocumentCardKind::Notes, None, scope)?)
        }
        DocumentCardBody::Records => Some(owner_body(
            documents::DocumentCardKind::Records,
            None,
            scope,
        )?),
        DocumentCardBody::Artifacts => Some(owner_body(
            documents::DocumentCardKind::Artifacts,
            None,
            scope,
        )?),
        DocumentCardBody::Media => {
            Some(owner_body(documents::DocumentCardKind::Media, None, scope)?)
        }
        DocumentCardBody::Health => Some(owner_body(
            documents::DocumentCardKind::Health,
            None,
            scope,
        )?),
        DocumentCardBody::TelemetryContext => Some(owner_body(
            documents::DocumentCardKind::TelemetryContext,
            None,
            scope,
        )?),
    };
    Ok(history::HistoricalDocumentCard {
        metadata: Some(metadata(&value.metadata)?),
        filters: collect(value.filters.iter().map(historical_filter))?,
        body,
    })
}

fn historical_summary(
    value: &dokime_app::document::TelemetrySummaryDefinition,
) -> Result<history::HistoricalTelemetrySummary, Status> {
    value::collection(value.groups.len())?;
    Ok(history::HistoricalTelemetrySummary {
        groups: collect(value.groups.iter().map(|group| {
            value::collection(group.tiles.len())?;
            Ok(history::HistoricalSummaryGroup {
                metadata: Some(metadata(&group.metadata)?),
                tiles: collect(group.tiles.iter().map(historical_summary_tile))?,
            })
        }))?,
    })
}

fn historical_summary_tile(
    value: &dokime_app::document::TelemetrySummaryTileDefinition,
) -> Result<history::HistoricalSummaryTile, Status> {
    value::collection(value.filters.len())?;
    value::collection(value.conditions.len())?;
    Ok(history::HistoricalSummaryTile {
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
            Ok(history::HistoricalSummaryCondition {
                operator: enums::filter_operator(&condition.operator),
                value: Some(project_value(
                    &condition.value,
                    &mut value::Budget::new(),
                    0,
                )?),
                state: summary_state(condition.state.clone()),
            })
        }))?,
    })
}

fn historical_filter(
    value: &dokime_app::document::DocumentFilter,
) -> Result<history::HistoricalDocumentFilter, Status> {
    Ok(history::HistoricalDocumentFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator),
        value: value
            .value
            .as_ref()
            .map(|item| project_value(item, &mut value::Budget::new(), 0))
            .transpose()?,
    })
}

fn data_expression(
    value: &dokime_app::expressions::DataExpression,
) -> Result<plots::DataExpression, Status> {
    value::collection(value.transforms.len())?;
    Ok(plots::DataExpression {
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
) -> Result<plots::DataExpressionSource, Status> {
    use dokime_app::expressions::DataSource;
    Ok(match value {
        DataSource::Telemetry { stream, field } => {
            value::text(stream)?;
            value::text(field)?;
            plots::DataExpressionSource::Telemetry(plots::TelemetryDataSource {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataSource::Parameter { name } => {
            value::text(name)?;
            plots::DataExpressionSource::Parameter(plots::ParameterDataSource {
                name: name.clone(),
            })
        }
        DataSource::Dimension { name } => {
            value::text(name)?;
            plots::DataExpressionSource::Dimension(plots::DimensionDataSource {
                name: name.clone(),
            })
        }
        DataSource::Record { record_type } => {
            value::text(record_type)?;
            plots::DataExpressionSource::Record(plots::RecordDataSource {
                record_type: record_type.clone(),
            })
        }
        DataSource::Artifact { artifact } => {
            value::text(artifact)?;
            plots::DataExpressionSource::Artifact(plots::ArtifactDataSource {
                artifact: artifact.clone(),
            })
        }
    })
}

fn data_transform(
    value: &dokime_app::expressions::DataTransform,
) -> Result<plots::DataTransform, Status> {
    use dokime_app::expressions::DataTransform;
    use plots::DataTransformKind as Kind;
    let kind = match value {
        DataTransform::Aggregate(aggregate) => Kind::Aggregate(plots::AggregateTransform {
            aggregate: *aggregate,
        }),
        DataTransform::TimeBetween {
            previous_record_type,
        } => {
            value::text(previous_record_type)?;
            Kind::TimeBetween(plots::TimeBetweenTransform {
                previous_record_type: previous_record_type.clone(),
            })
        }
        DataTransform::TelemetryAtOrBefore { stream, field } => {
            value::text(stream)?;
            value::text(field)?;
            Kind::TelemetryAtOrBefore(plots::TelemetryAtOrBeforeTransform {
                stream: stream.clone(),
                field: field.clone(),
            })
        }
        DataTransform::Detail { name } => {
            value::text(name)?;
            Kind::Detail(plots::DetailTransform { name: name.clone() })
        }
        DataTransform::PerCase => Kind::PerCase(plots::EmptyTransform {}),
    };
    Ok(plots::DataTransform { kind: Some(kind) })
}

pub(in crate::grpc) fn historical_plot(
    value: &dokime_app::document::PlotDefinition,
    scope: &dokime_app::core::RecordScope,
) -> Result<documents::DocumentPlotDefinition, Status> {
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
    let request = plots::PlotRequest {
        analysis: value
            .analysis
            .as_ref()
            .map(analysis_selection)
            .transpose()?,
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
        max_points: value.display_points.map(u64::from),
        window_s: value.live_window_s,
        scope: Some(canonical::scope(scope)?),
        cursor: None,
        technique: enums::downsample_technique(&value.downsample),
    };
    let (x_axis, x_axis_unit) = enums::plot_axis(value.x_axis.as_ref());
    let presentation = documents::PlotPresentation {
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
    Ok(documents::DocumentPlotDefinition {
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

fn plot_filter(value: &dokime_app::document::DocumentFilter) -> Result<plots::PlotFilter, Status> {
    Ok(plots::PlotFilter {
        expression: Some(data_expression(&value.expression)?),
        operator: enums::filter_operator(&value.operator),
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

fn fft_options(value: &dokime_app::document::FftOptions) -> Result<plots::PlotFftOptions, Status> {
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
    Ok(plots::PlotFftOptions {
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
    value: &dokime_app::document::SigmaIntervalOptions,
) -> Result<plots::PlotIntervalOptions, Status> {
    finite(value.bin_width, "interval bin_width")?;
    finite(value.sigma, "interval sigma")?;
    Ok(plots::PlotIntervalOptions {
        bin_width: value.bin_width,
        sigma: value.sigma,
        clip_to_observed: value.clip_to_observed,
    })
}

fn histogram_options(
    value: &dokime_app::document::HistogramOptions,
) -> Result<plots::PlotHistogramOptions, Status> {
    if value.bins > u64::MAX as usize {
        return Err(Status::data_loss(
            "stored histogram bin count overflows wire",
        ));
    }
    if let Some(unit) = &value.unit {
        value::text(unit)?;
    }
    Ok(plots::PlotHistogramOptions {
        bins: value.bins as u64,
        unit: value.unit.clone(),
    })
}

fn reference_line(
    value: &dokime_app::document::ReferenceLine,
) -> Result<documents::PlotReferenceLine, Status> {
    finite(value.source_scale, "reference line source_scale")?;
    if let Some(item) = value.value {
        finite(item, "reference line value")?;
    }
    if let Some(label) = &value.label {
        value::text(label)?;
    }
    value::text(&value.color)?;
    Ok(documents::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis),
        value: value.value,
        source: value.source.as_ref().map(data_expression).transpose()?,
        source_scale: value.source_scale,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn reference_band(
    value: &dokime_app::document::ReferenceBand,
) -> Result<documents::PlotReferenceBand, Status> {
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
    Ok(documents::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis),
        min: value.min,
        max: value.max,
        center: value.center.as_ref().map(data_expression).transpose()?,
        tolerance: value.tolerance.as_ref().map(data_expression).transpose()?,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn summary_stat(
    value: dokime_app::document::TelemetrySummaryStat,
) -> documents::DocumentSummaryStat {
    match value {
        dokime_app::document::TelemetrySummaryStat::Latest => {
            documents::DocumentSummaryStat::Latest
        }
    }
}

fn summary_format(
    value: dokime_app::document::TelemetrySummaryFormat,
) -> documents::DocumentSummaryFormat {
    match value {
        dokime_app::document::TelemetrySummaryFormat::Auto => {
            documents::DocumentSummaryFormat::Auto
        }
        dokime_app::document::TelemetrySummaryFormat::Number => {
            documents::DocumentSummaryFormat::Number
        }
        dokime_app::document::TelemetrySummaryFormat::Percent => {
            documents::DocumentSummaryFormat::Percent
        }
        dokime_app::document::TelemetrySummaryFormat::Duration => {
            documents::DocumentSummaryFormat::Duration
        }
        dokime_app::document::TelemetrySummaryFormat::Distance => {
            documents::DocumentSummaryFormat::Distance
        }
        dokime_app::document::TelemetrySummaryFormat::Status => {
            documents::DocumentSummaryFormat::Status
        }
        dokime_app::document::TelemetrySummaryFormat::Text => {
            documents::DocumentSummaryFormat::Text
        }
    }
}

fn summary_presentation(
    value: dokime_app::document::TelemetrySummaryPresentation,
) -> documents::DocumentSummaryPresentation {
    match value {
        dokime_app::document::TelemetrySummaryPresentation::Value => {
            documents::DocumentSummaryPresentation::Value
        }
        dokime_app::document::TelemetrySummaryPresentation::Progress => {
            documents::DocumentSummaryPresentation::Progress
        }
        dokime_app::document::TelemetrySummaryPresentation::Badge => {
            documents::DocumentSummaryPresentation::Badge
        }
    }
}

fn summary_state(
    value: dokime_app::document::TelemetrySummaryState,
) -> documents::DocumentSummaryState {
    match value {
        dokime_app::document::TelemetrySummaryState::Ok => documents::DocumentSummaryState::Ok,
        dokime_app::document::TelemetrySummaryState::Warning => {
            documents::DocumentSummaryState::Warning
        }
        dokime_app::document::TelemetrySummaryState::Critical => {
            documents::DocumentSummaryState::Critical
        }
        dokime_app::document::TelemetrySummaryState::Muted => {
            documents::DocumentSummaryState::Muted
        }
    }
}

fn owner_body(
    kind: documents::DocumentCardKind,
    plot_id: Option<String>,
    _scope: &dokime_app::core::RecordScope,
) -> Result<history::HistoricalDocumentCardBody, Status> {
    Ok(history::HistoricalDocumentCardBody::Owner(
        documents::DocumentOwnerReference {
            kind,
            plot_id,
            plot: None,
        },
    ))
}

fn layout_nodes_for_node(
    node: &dokime_app::document::DocumentNode,
    parent: Option<u64>,
    ordinal: &mut u64,
    output: &mut Vec<documents::DocumentLayoutNode>,
) -> Result<(), Status> {
    use dokime_app::document::DocumentNode;
    match node {
        DocumentNode::Section(section) => {
            let current = *ordinal;
            output.push(documents::DocumentLayoutNode {
                ordinal: current,
                parent_ordinal: parent,
                node: Some(documents::DocumentLayoutNodeNode::Section(Box::new(
                    documents::DocumentSection {
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
            output.push(documents::DocumentLayoutNode {
                ordinal: *ordinal,
                parent_ordinal: parent,
                node: Some(documents::DocumentLayoutNodeNode::Placement(
                    documents::DocumentPlacement {
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
    source: &dokime_app::document::DocumentSectionRepeat,
) -> Result<documents::DocumentRepeat, Status> {
    value::text(&source.label_template)?;
    if let Some(id) = &source.id_template {
        value::text(id)?;
    }
    Ok(documents::DocumentRepeat {
        value: Some(data_expression(&source.value)?),
        label_template: source.label_template.clone(),
        id_template: source.id_template.clone(),
        order_by: source.order_by.as_ref().map(data_expression).transpose()?,
        filters: collect(source.filters.iter().map(plot_filter))?,
    })
}

fn span(value: &dokime_app::document::ResponsiveSpan) -> documents::DocumentSpan {
    documents::DocumentSpan {
        base: u32::from(value.base),
        sm: value.sm.map(u32::from),
        md: value.md.map(u32::from),
        lg: value.lg.map(u32::from),
        xl: value.xl.map(u32::from),
    }
}

fn density(value: &dokime_app::document::DocumentCardDensity) -> documents::DocumentDensity {
    match value {
        dokime_app::document::DocumentCardDensity::Compact => documents::DocumentDensity::Compact,
        dokime_app::document::DocumentCardDensity::Normal => documents::DocumentDensity::Normal,
        dokime_app::document::DocumentCardDensity::Spacious => documents::DocumentDensity::Spacious,
    }
}

fn rail(value: &dokime_app::document::DocumentCardRail) -> documents::DocumentRail {
    match value {
        dokime_app::document::DocumentCardRail::Main => documents::DocumentRail::Main,
        dokime_app::document::DocumentCardRail::Right => documents::DocumentRail::Right,
        dokime_app::document::DocumentCardRail::Left => documents::DocumentRail::Left,
    }
}

fn analysis_selection(
    selection: &dokime::analysis::Selection,
) -> Result<plots::AnalysisSelection, Status> {
    value::text(&selection.id)?;
    Ok(plots::AnalysisSelection {
        id: selection.id.clone(),
        version: selection.version,
        parameters: Some(canonical::object(
            &selection.parameters,
            &mut value::Budget::new(),
        )?),
    })
}

/* === Tests === */
