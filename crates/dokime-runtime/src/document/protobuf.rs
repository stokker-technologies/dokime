/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Admission Of Canonical Document Messages Into The Host Model.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;

// Third Party Imports
use prost::Message;

// Local Imports
use super::*;
use dokime_domain::model::expressions::{DataExpression, DataSource, DataTransform};
use dokime_domain::model::identity::{Identifier, Metadata};
use dokime_domain::model::values::Value;
use dokime_protocol::v1 as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

pub trait IntoDocumentDefinition {
    fn into_document_definition(self) -> std::result::Result<DocumentDefinition, String>;
}

/* === Implementations === */

impl IntoDocumentDefinition for DocumentDefinition {
    fn into_document_definition(self) -> std::result::Result<DocumentDefinition, String> {
        self.validate_identity()?;
        Ok(self)
    }
}

impl IntoDocumentDefinition for wire::Document {
    fn into_document_definition(self) -> std::result::Result<DocumentDefinition, String> {
        let metadata = metadata(self.metadata)?;
        let mut document = DocumentDefinition::new(&metadata.label);
        document.id(metadata.identifier.as_str());
        document.metadata = metadata;
        let mut identifiers = BTreeSet::new();
        for section in self.sections {
            let node = section_node(section, &mut document.cards, &mut identifiers, 0)?;
            document.layout.nodes.push(node);
        }
        document.validate_identity()?;
        Ok(document)
    }
}

/* === Public Functions === */

pub fn decode_document(bytes: &[u8]) -> std::result::Result<DocumentDefinition, String> {
    if bytes.len() > super::limits::DocumentLimits::default().source_bytes {
        return Err("document exceeds its source byte budget".into());
    }
    wire::Document::decode(bytes).map_err(|error| error.to_string())?.into_document_definition()
}

pub fn decode_section(bytes: &[u8]) -> std::result::Result<DocumentSection, String> {
    let source = wire::Section::decode(bytes).map_err(|error| error.to_string())?;
    if !source.sections.is_empty() {
        return Err("nested sections must be declared in the template document".into());
    }
    Ok(DocumentSection {
        metadata: metadata(source.metadata)?,
        cards: source.cards.into_iter().map(card).collect::<std::result::Result<_, _>>()?,
        repeat: source.repeat.map(repeat).transpose()?,
    })
}

pub fn expression(source: wire::DataExpression) -> std::result::Result<DataExpression, String> {
    expression_at(source, 0)
}

/* === Private Functions === */

fn metadata(source: Option<wire::Metadata>) -> std::result::Result<Metadata, String> {
    let source = source.ok_or("document metadata is required")?;
    let id = Identifier::new(source.identifier).map_err(|error| error.to_string())?;
    if source.label.trim().is_empty() {
        return Err("document labels must be nonempty".into());
    }
    Ok(Metadata::new(id, source.label, (!source.description.is_empty()).then_some(source.description)))
}

fn section_node(
    source: wire::Section, cards: &mut Vec<DocumentCardDefinition>, identifiers: &mut BTreeSet<Identifier>, depth: usize,
) -> std::result::Result<DocumentNode, String> {
    if depth >= super::limits::DocumentLimits::default().layout_depth {
        return Err("document exceeds its section depth budget".into());
    }
    let metadata = metadata(source.metadata)?;
    unique(&metadata.identifier, identifiers)?;
    let mut children = Vec::new();
    for source in source.cards {
        let card = card(source)?;
        unique(&card.metadata.identifier, identifiers)?;
        children.push(DocumentNode::Card(DocumentCardPlacement {
            card_id: card.metadata.identifier.clone(),
            layout: card.layout.clone(),
        }));
        cards.push(card.into());
    }
    for section in source.sections {
        children.push(section_node(section, cards, identifiers, depth + 1)?);
    }
    Ok(DocumentNode::Section(Box::new(DocumentSectionNode {
        node_id: metadata.identifier.clone(),
        metadata,
        repeat: source.repeat.map(repeat).transpose()?,
        children,
    })))
}

fn unique(id: &Identifier, identifiers: &mut BTreeSet<Identifier>) -> std::result::Result<(), String> {
    if !identifiers.insert(id.clone()) {
        return Err(format!("duplicate document identifier {id}"));
    }
    if identifiers.len() > super::limits::DocumentLimits::default().expanded_nodes {
        return Err("document exceeds its node budget".into());
    }
    Ok(())
}

fn repeat(source: wire::SectionRepeat) -> std::result::Result<DocumentSectionRepeat, String> {
    Ok(DocumentSectionRepeat {
        value: expression(source.value.ok_or("section repeat requires a value")?)?,
        label_template: source.label_template,
        id_template: None,
        order_by: source.order_by.map(expression).transpose()?,
        filters: Vec::new(),
    })
}

fn card(source: wire::Card) -> std::result::Result<DocumentCard, String> {
    let metadata = metadata(source.metadata)?;
    let mut layout = DocumentCardLayout::default();
    if let Some(source) = source.layout {
        layout.col_span.base = u8::try_from(source.columns).map_err(|_| "card columns overflow")?;
        layout.row_span = u8::try_from(source.rows).map_err(|_| "card rows overflow")?;
        layout.order = source.order;
    }
    let body = match source.body.ok_or("card body is required")? {
        wire::card::Body::Markdown(markdown) => DocumentCardBody::Text { markdown },
        wire::card::Body::Plot(source) => DocumentCardBody::Plot { plot: Box::new(plot(source)?) },
        wire::card::Body::Summary(source) => {
            let mut group = TelemetrySummaryGroupDefinition::new(metadata.identifier.as_str(), &metadata.label);
            for tile in source.tiles {
                let meta = self::metadata(tile.metadata)?;
                let mut target =
                    TelemetrySummaryTileDefinition::new(&meta.label, expression(tile.value.ok_or("summary tile requires a value")?)?);
                target.metadata = meta;
                target.unit = (!tile.unit.is_empty()).then_some(tile.unit);
                group.tile(target);
            }
            DocumentCardBody::TelemetrySummary {
                summary: TelemetrySummaryDefinition { groups: vec![group] },
            }
        }
        wire::card::Body::Table(source) => DocumentCardBody::Table {
            rows: source
                .rows
                .into_iter()
                .map(|row| row.cells.into_iter().map(|(key, source)| Ok((key, value(source)?))).collect())
                .collect::<std::result::Result<_, String>>()?,
        },
        wire::card::Body::Evidence(source) => match wire::EvidenceCard::try_from(source).map_err(|_| "unknown evidence card")? {
            wire::EvidenceCard::Steps => DocumentCardBody::Steps,
            wire::EvidenceCard::Criteria => DocumentCardBody::Criteria,
            wire::EvidenceCard::Traceability => DocumentCardBody::Traceability,
            wire::EvidenceCard::Notes => DocumentCardBody::Notes,
            wire::EvidenceCard::Records => DocumentCardBody::Records,
            wire::EvidenceCard::Artifacts => DocumentCardBody::Artifacts,
            wire::EvidenceCard::Media => DocumentCardBody::Media,
            wire::EvidenceCard::Health => DocumentCardBody::Health,
            wire::EvidenceCard::StreamContext => DocumentCardBody::TelemetryContext,
            wire::EvidenceCard::Unspecified => return Err("evidence card kind is required".into()),
        },
    };
    Ok(DocumentCard {
        metadata,
        layout,
        filters: source.filters.into_iter().map(filter).collect::<std::result::Result<_, _>>()?,
        body,
    })
}

fn plot(source: wire::Plot) -> std::result::Result<PlotDefinition, String> {
    let meta = metadata(source.metadata)?;
    let kind = wire::PlotKind::from_declared_number(source.kind)?;
    let kind = match kind {
        wire::PlotKind::TimeSeries => PlotType::TimeSeries,
        wire::PlotKind::FrequencySeries => PlotType::FrequencySeries,
        wire::PlotKind::Xy => PlotType::Xy,
        wire::PlotKind::Trajectory => PlotType::Trajectory,
        wire::PlotKind::Contour => PlotType::Contour,
        wire::PlotKind::GridChart => PlotType::GridChart,
        wire::PlotKind::Bar => PlotType::Bar,
        wire::PlotKind::Pie => PlotType::Pie,
        wire::PlotKind::Histogram => PlotType::Histogram,
        wire::PlotKind::BoxPlot => PlotType::BoxPlot,
        wire::PlotKind::Unspecified => return Err("plot kind is required".into()),
    };
    let mut target = PlotDefinition::with_type(&meta.label, kind);
    target.metadata = meta;
    target.automatic_id = false;
    target.x_expression = source.x.map(expression).transpose()?;
    target.y_expressions = source.y.into_iter().map(expression).collect::<std::result::Result<_, _>>()?;
    target.z_expression = source.z.map(expression).transpose()?;
    target.series_by_expression = source.series_by.map(expression).transpose()?;
    target.filters = source.filters.into_iter().map(filter).collect::<std::result::Result<_, _>>()?;
    target.unit = (!source.unit.is_empty()).then_some(source.unit);
    target.marker_record_types = source.marker_records;
    target.histogram.bins = source.histogram_bins.map(|value| value as usize).unwrap_or(target.histogram.bins);
    target.fft.sample_rate_hz = source.sample_rate_hz;
    target.display_points = source.display_points;
    target.arrangement = source
        .arrangement
        .map(|value| match wire::BarArrangement::try_from(value) {
            Ok(wire::BarArrangement::Grouped) => Ok(BarArrangement::Grouped),
            Ok(wire::BarArrangement::Stacked) => Ok(BarArrangement::Stacked),
            _ => Err("unknown BAR arrangement"),
        })
        .transpose()?;
    target.pareto = source.pareto;
    target.hole = source.hole;
    target.xy_interval = source
        .interval
        .map(|value| match wire::XyInterval::try_from(value) {
            Ok(wire::XyInterval::Sigma) => Ok(XyInterval::Sigma),
            _ => Err("unknown XY interval"),
        })
        .transpose()?;
    target.reduction = source
        .reduction
        .map(|value| match wire::Reduction::try_from(value) {
            Ok(wire::Reduction::Mean) => Ok(Reduction::Mean),
            Ok(wire::Reduction::Min) => Ok(Reduction::Min),
            Ok(wire::Reduction::Max) => Ok(Reduction::Max),
            _ => Err("unknown plot reduction"),
        })
        .transpose()?;
    target.render = match wire::ContourRender::try_from(source.render) {
        Ok(wire::ContourRender::Unspecified | wire::ContourRender::Cells) => ContourRender::Cells,
        Ok(wire::ContourRender::Bands) => ContourRender::Bands,
        _ => return Err("unknown CONTOUR render mode".into()),
    };
    target.style = match wire::PlotStyle::try_from(source.style) {
        Ok(wire::PlotStyle::Unspecified | wire::PlotStyle::Line) => PlotStyle::Line,
        Ok(wire::PlotStyle::Points) => PlotStyle::Points,
        Ok(wire::PlotStyle::LinePoints) => PlotStyle::LinePoints,
        Ok(wire::PlotStyle::Step) => PlotStyle::Step,
        _ => return Err("unknown plot style".into()),
    };
    target.x_scale = axis_scale(source.x_scale)?;
    target.y_scale = axis_scale(source.y_scale)?;
    target.grid = source
        .grid
        .map(|source| {
            Ok::<_, String>(GridDefinition {
                row: source.row.map(expression).transpose()?,
                column: source.column.map(expression).transpose()?,
                frame: source.frame.map(expression).transpose()?,
                rows: source.rows.into_iter().map(entry).collect(),
                columns: source.columns.into_iter().map(entry).collect(),
                highlights: source
                    .highlights
                    .into_iter()
                    .map(|source| {
                        Ok(GridHighlight {
                            row: expression(source.row.ok_or("grid highlight requires row")?)?,
                            column: expression(source.column.ok_or("grid highlight requires column")?)?,
                            label: source.label,
                        })
                    })
                    .collect::<std::result::Result<_, String>>()?,
                summary_label: source.summary_label,
            })
        })
        .transpose()?;
    target.reference_lines = source
        .reference_lines
        .into_iter()
        .map(|source| {
            Ok(ReferenceLine {
                value: reference(source.value.ok_or("reference line requires value")?)?,
                label: Some(source.label),
                color: source.color,
            })
        })
        .collect::<std::result::Result<_, String>>()?;
    target.reference_bands = source
        .reference_bands
        .into_iter()
        .map(|source| {
            Ok(ReferenceBand {
                min: source.min.map(reference).transpose()?,
                max: source.max.map(reference).transpose()?,
                center: source.center.map(reference).transpose()?,
                tolerance: source.tolerance.map(reference).transpose()?,
                label: Some(source.label),
                color: source.color,
            })
        })
        .collect::<std::result::Result<_, String>>()?;
    validate_plot_definition(&target)?;
    Ok(target)
}

fn axis_scale(source: i32) -> std::result::Result<AxisScale, String> {
    match wire::AxisScale::try_from(source) {
        Ok(wire::AxisScale::Unspecified | wire::AxisScale::Linear) => Ok(AxisScale::Linear),
        Ok(wire::AxisScale::Log) => Ok(AxisScale::Log),
        _ => Err("unknown axis scale".into()),
    }
}

fn entry(source: wire::GridAxisEntry) -> GridAxisEntry {
    GridAxisEntry {
        key: source.key,
        label: source.label,
        order: source.order,
        group: source.group,
    }
}

fn expression_at(source: wire::DataExpression, depth: usize) -> std::result::Result<DataExpression, String> {
    if depth >= super::limits::DocumentLimits::default().depth {
        return Err("expression exceeds its depth budget".into());
    }
    use wire::data_expression::Kind;
    let mut target = match source.kind.ok_or("expression operand is required")? {
        Kind::Stream(field) => DataExpression {
            label: None,
            source: DataSource::Telemetry {
                stream: field.identifier,
                field: field.field,
            },
            transforms: Vec::new(),
        },
        Kind::Record(field) => DataExpression {
            label: None,
            source: DataSource::Record { record_type: field.identifier },
            transforms: if field.field.is_empty() { Vec::new() } else { vec![DataTransform::Detail { name: field.field }] },
        },
        Kind::Parameter(name) => DataExpression {
            label: None,
            source: DataSource::Parameter { name },
            transforms: Vec::new(),
        },
        Kind::Dimension(name) => DataExpression {
            label: None,
            source: DataSource::Dimension { name },
            transforms: Vec::new(),
        },
        Kind::Artifact(artifact) => DataExpression {
            label: None,
            source: DataSource::Artifact { artifact },
            transforms: Vec::new(),
        },
        Kind::Transform(source) => {
            let mut target = expression_at(*source.input.ok_or("transform input is required")?, depth + 1)?;
            let transform = match source.kind.ok_or("transform kind is required")? {
                wire::transform_expression::Kind::Scale(factor) if factor.is_finite() => DataTransform::Scale { factor },
                wire::transform_expression::Kind::Offset(value) if value.is_finite() => DataTransform::Offset { value },
                wire::transform_expression::Kind::Detail(name) => DataTransform::Detail { name },
                wire::transform_expression::Kind::TimeBetween(previous_record_type) => DataTransform::TimeBetween { previous_record_type },
                wire::transform_expression::Kind::PerCase(true) => DataTransform::PerCase,
                _ => return Err("invalid numeric or per-case transform".into()),
            };
            target.transforms.push(transform);
            target
        }
        Kind::Aggregate(source) => {
            let mut target = expression_at(*source.input.ok_or("aggregate input is required")?, depth + 1)?;
            use harmos::Aggregate as A;
            let aggregate = match wire::Aggregate::try_from(source.kind) {
                Ok(wire::Aggregate::Count) => A::Count,
                Ok(wire::Aggregate::Sum) => A::Sum,
                Ok(wire::Aggregate::Min) => A::Min,
                Ok(wire::Aggregate::Max) => A::Max,
                Ok(wire::Aggregate::Mean) => A::Mean,
                Ok(wire::Aggregate::Stddev) => A::Std,
                Ok(wire::Aggregate::Median) => A::Median,
                Ok(wire::Aggregate::Rms) => A::Rms,
                Ok(wire::Aggregate::Percentile)
                    if source.percentile.is_finite() && source.percentile.fract() == 0.0 && (0.0..=100.0).contains(&source.percentile) =>
                {
                    A::Percentile(source.percentile as u8)
                }
                _ => return Err("unsupported or invalid aggregate".into()),
            };
            target.transforms.push(DataTransform::Aggregate(aggregate));
            target
        }
        Kind::Literal(_) => return Err("a plot binding must select a field; literals belong in reference lines or table cells".into()),
    };
    if !source.label.is_empty() {
        target.label = Some(source.label);
    }
    Ok(target)
}

fn reference(source: wire::DataExpression) -> std::result::Result<ReferenceValue, String> {
    if let Some(wire::data_expression::Kind::Literal(source)) = source.kind {
        let value = dokime_domain::model::values::numeric_value(&value(source)?).ok_or("reference must be numeric")?;
        if !value.is_finite() {
            return Err("reference must be finite".into());
        }
        return Ok(ReferenceValue::Constant(value));
    }
    Ok(ReferenceValue::Expression(expression(source)?))
}

fn filter(source: wire::DocumentFilter) -> std::result::Result<DocumentFilter, String> {
    let operator = match wire::FilterOperator::try_from(source.operator) {
        Ok(wire::FilterOperator::Eq) => FilterOperator::Eq,
        Ok(wire::FilterOperator::Ne) => FilterOperator::Ne,
        Ok(wire::FilterOperator::Gt) => FilterOperator::Gt,
        Ok(wire::FilterOperator::Ge) => FilterOperator::Gte,
        Ok(wire::FilterOperator::Lt) => FilterOperator::Lt,
        Ok(wire::FilterOperator::Le) => FilterOperator::Lte,
        Ok(wire::FilterOperator::Contains) => FilterOperator::Contains,
        Ok(wire::FilterOperator::Exists) => FilterOperator::Exists,
        _ => return Err("filter operator is required".into()),
    };
    Ok(DocumentFilter {
        expression: expression(source.expression.ok_or("filter expression is required")?)?,
        operator,
        value: source.value.map(value).transpose()?,
    })
}

fn value(source: wire::Value) -> std::result::Result<Value, String> {
    Ok(match source.kind.ok_or("value kind is required")? {
        wire::value::Kind::Null(_) => Value::Null,
        wire::value::Kind::Boolean(value) => Value::Bool(value),
        wire::value::Kind::Integer(value) => Value::Integer(value),
        wire::value::Kind::UnsignedInteger(value) => Value::Unsigned(value),
        wire::value::Kind::Number(value) if value.is_finite() => Value::Number(value),
        wire::value::Kind::Text(value) => Value::Text(value),
        wire::value::Kind::List(source) => Value::Array(source.values.into_iter().map(value).collect::<std::result::Result<_, _>>()?),
        wire::value::Kind::Datetime(source) if (0..1_000_000_000).contains(&source.nanos) => Value::Datetime(
            time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(source.seconds) * 1_000_000_000 + i128::from(source.nanos))
                .map_err(|error| error.to_string())?,
        ),
        _ => return Err("unsupported or invalid document value".into()),
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_protocol::builders::Plot;

    #[test]
    fn canonical_documents_preserve_expressions_options_and_nested_sections() {
        let scan = wire::FieldReference::new("scan");
        let surface = Plot::contour(scan.field("temperature").scale(2.0).offset(-1.0))
            .x(scan.field("position_x"))
            .y(scan.field("position_y"))
            .label("Temperature Map")
            .unit("°C")
            .reduce(wire::Reduction::Mean)
            .render(wire::ContourRender::Bands)
            .reference_band(wire::ReferenceBand {
                center: Some(wire::DataExpression {
                    kind: Some(wire::data_expression::Kind::Parameter("target_y".into())),
                    ..Default::default()
                }),
                tolerance: Some(wire::DataExpression {
                    kind: Some(wire::data_expression::Kind::Parameter("band_y".into())),
                    ..Default::default()
                }),
                ..Default::default()
            });
        let source = wire::Document::new("Scan").section(wire::Section::new("Map").section(wire::Section::new("Detail").plot(surface)));
        let admitted = decode_document(&source.encode_to_vec()).unwrap();
        let DocumentNode::Section(section) = &admitted.layout.nodes[0] else { panic!("section") };
        assert!(matches!(section.children[0], DocumentNode::Section(_)));
        let DocumentCardBody::Plot { plot } = &admitted.cards[0].body else { panic!("plot") };
        assert_eq!(plot.plot_type, PlotType::Contour);
        assert_eq!(plot.render, ContourRender::Bands);
        assert_eq!(plot.reduction, Some(Reduction::Mean));
        assert!(
            matches!(&plot.reference_bands[0].center, Some(ReferenceValue::Expression(value)) if value.source == DataSource::Parameter { name: "target_y".into() })
        );
        assert!(
            matches!(&plot.reference_bands[0].tolerance, Some(ReferenceValue::Expression(value)) if value.source == DataSource::Parameter { name: "band_y".into() })
        );
        assert_eq!(
            plot.z_expression.as_ref().unwrap().transforms,
            vec![DataTransform::Scale { factor: 2.0 }, DataTransform::Offset { value: -1.0 }]
        );
        let mut invalid = source;
        let wire::card::Body::Plot(plot) = invalid.sections[0].sections[0].cards[0].body.as_mut().unwrap() else { panic!("plot") };
        plot.kind = 11;
        assert!(decode_document(&invalid.encode_to_vec()).unwrap_err().contains("CONTOUR"));
    }
}
