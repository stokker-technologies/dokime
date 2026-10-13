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

// Local Imports
use crate::grpc::value::{Budget, canonical};
use dokime_app::{
    core::{Metadata, RecordScope, Value, ValueFormat},
    document::{
        DocumentCardBody, DocumentDefinition, DocumentFilter, DocumentNode, PlotAxisKind,
        PlotDefinition, TelemetrySummaryDefinition,
    },
    expressions::{DataExpression, DataSource, DataTransform},
};
use dokime_contract::operations::{common, documents, history, plots, value as value_wire};
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(in crate::grpc) fn memory(
    source: &DocumentDefinition,
    scope: &RecordScope,
) -> harmos::Result<usize> {
    let (layout_count, layout_heap) = layout(&source.layout.nodes)?;
    let mut bytes = std::mem::size_of::<history::HistoricalDocument>()
        .saturating_add(source.document_id.as_str().len())
        .saturating_add(metadata_heap(&source.metadata)?)
        .saturating_add(scope_heap(scope))
        .saturating_add(slots::<documents::DocumentLayoutNode>(layout_count))
        .saturating_add(layout_heap)
        .saturating_add(slots::<history::HistoricalDocumentCard>(source.cards.len()));
    for card in &source.cards {
        bytes = bytes
            .saturating_add(metadata_heap(&card.metadata)?)
            .saturating_add(filters::<history::HistoricalDocumentFilter>(&card.filters)?);
        let body = match &card.body {
            DocumentCardBody::Text { markdown } => markdown.len(),
            DocumentCardBody::Table { rows } => {
                let mut heap = slots::<documents::DocumentTableRow>(rows.len());
                let mut budget = Budget::new();
                for row in rows {
                    heap = heap
                        .saturating_add(card.metadata.identifier.as_str().len())
                        .saturating_add(
                            canonical::object_memory(row, &mut budget).map_err(failure)?
                                - std::mem::size_of::<value_wire::Value>(),
                        );
                }
                heap
            }
            DocumentCardBody::TelemetrySummary { summary } => summary_heap(summary)?,
            DocumentCardBody::Plot { plot } => plot_heap(plot, scope)?,
            _ => 0,
        };
        bytes = bytes.saturating_add(body);
    }
    if bytes == usize::MAX {
        return Err(harmos::Error::ResourceExhausted {
            resource: "historical document construction",
        });
    }
    Ok(bytes)
}

pub(super) fn layout_count(nodes: &[DocumentNode]) -> usize {
    nodes.iter().fold(0usize, |count, node| {
        let children = match node {
            DocumentNode::Section(section) => layout_count(&section.children),
            DocumentNode::Card(_) => 0,
        };
        count.saturating_add(1).saturating_add(children)
    })
}

/* === Private Functions === */

fn slots<T>(length: usize) -> usize {
    length
        .saturating_mul(4)
        .saturating_mul(std::mem::size_of::<T>())
}

fn optional(source: &Option<String>) -> usize {
    source.as_ref().map_or(0, String::len)
}

fn scope_heap(source: &RecordScope) -> usize {
    canonical::scope_memory(source).map_or(usize::MAX, |memory| {
        memory - std::mem::size_of::<common::EvidenceScope>()
    })
}

fn metadata_heap(source: &Metadata) -> harmos::Result<usize> {
    Ok(canonical::metadata_memory(source).map_err(failure)?
        - std::mem::size_of::<common::Metadata>())
}

fn value_heap(source: &Value) -> harmos::Result<usize> {
    Ok(
        canonical::memory(source, &mut Budget::new(), 0).map_err(failure)?
            - std::mem::size_of::<value_wire::Value>(),
    )
}

fn layout(nodes: &[DocumentNode]) -> harmos::Result<(usize, usize)> {
    let mut count = 0usize;
    let mut bytes = 0usize;
    for node in nodes {
        count = count.saturating_add(1);
        match node {
            DocumentNode::Card(card) => {
                bytes = bytes.saturating_add(card.card_id.as_str().len());
            }
            DocumentNode::Section(section) => {
                let (children, heap) = layout(&section.children)?;
                count = count.saturating_add(children);
                bytes = bytes
                    .saturating_add(std::mem::size_of::<documents::DocumentSection>())
                    .saturating_add(section.node_id.as_str().len())
                    .saturating_add(metadata_heap(&section.metadata)?)
                    .saturating_add(heap);
                if let Some(repeat) = &section.repeat {
                    bytes = bytes
                        .saturating_add(expression_heap(&repeat.value))
                        .saturating_add(repeat.label_template.len())
                        .saturating_add(optional(&repeat.id_template))
                        .saturating_add(filters::<plots::PlotFilter>(&repeat.filters)?);
                    if let Some(order) = &repeat.order_by {
                        bytes = bytes.saturating_add(expression_heap(order));
                    }
                }
            }
        }
    }
    Ok((count, bytes))
}

fn expression_heap(source: &DataExpression) -> usize {
    let source_heap = match &source.source {
        DataSource::Telemetry { stream, field } => stream.len().saturating_add(field.len()),
        DataSource::Parameter { name } | DataSource::Dimension { name } => name.len(),
        DataSource::Record { record_type } => record_type.len(),
        DataSource::Artifact { artifact } => artifact.len(),
    };
    source.transforms.iter().fold(
        source_heap
            .saturating_add(optional(&source.label))
            .saturating_add(slots::<plots::DataTransform>(source.transforms.len())),
        |bytes, transform| {
            bytes.saturating_add(match transform {
                DataTransform::TimeBetween {
                    previous_record_type,
                } => previous_record_type.len(),
                DataTransform::TelemetryAtOrBefore { stream, field } => {
                    stream.len().saturating_add(field.len())
                }
                DataTransform::Detail { name } => name.len(),
                _ => 0,
            })
        },
    )
}

fn filters<T>(source: &[DocumentFilter]) -> harmos::Result<usize> {
    let mut bytes = slots::<T>(source.len());
    for filter in source {
        bytes = bytes.saturating_add(expression_heap(&filter.expression));
        if let Some(value) = &filter.value {
            bytes = bytes.saturating_add(value_heap(value)?);
        }
    }
    Ok(bytes)
}

fn summary_heap(source: &TelemetrySummaryDefinition) -> harmos::Result<usize> {
    let mut bytes = slots::<history::HistoricalSummaryGroup>(source.groups.len());
    for group in &source.groups {
        bytes = bytes
            .saturating_add(metadata_heap(&group.metadata)?)
            .saturating_add(slots::<history::HistoricalSummaryTile>(group.tiles.len()));
        for tile in &group.tiles {
            bytes = bytes
                .saturating_add(metadata_heap(&tile.metadata)?)
                .saturating_add(expression_heap(&tile.value))
                .saturating_add(filters::<history::HistoricalDocumentFilter>(&tile.filters)?)
                .saturating_add(optional(&tile.unit))
                .saturating_add(slots::<history::HistoricalSummaryCondition>(
                    tile.conditions.len(),
                ));
            for condition in &tile.conditions {
                bytes = bytes.saturating_add(value_heap(&condition.value)?);
            }
        }
    }
    Ok(bytes)
}

fn format_heap(source: Option<&ValueFormat>) -> usize {
    match source {
        Some(ValueFormat::AbsoluteTime { pattern }) => pattern.len(),
        _ => 0,
    }
}

fn plot_heap(source: &PlotDefinition, scope: &RecordScope) -> harmos::Result<usize> {
    let mut bytes = metadata_heap(&source.metadata)?
        .saturating_add(source.metadata.identifier.as_str().len())
        .saturating_add(optional(&source.stream))
        .saturating_add(optional(&source.x))
        .saturating_add(optional(&source.z_stream))
        .saturating_add(optional(&source.z_field))
        .saturating_add(optional(&source.series_by))
        .saturating_add(optional(&source.unit))
        .saturating_add(scope_heap(scope))
        .saturating_add(source.group.len())
        .saturating_add(optional(&source.histogram.unit)) // Request Histogram.
        .saturating_add(optional(&source.histogram.unit)) // Presentation Histogram.
        .saturating_add(format_heap(source.x_format.as_ref()))
        .saturating_add(format_heap(source.y_format.as_ref()))
        .saturating_add(filters::<plots::PlotFilter>(&source.filters)?);
    if let Some(PlotAxisKind::Elapsed { unit }) = &source.x_axis {
        bytes = bytes.saturating_add(unit.len());
    }
    if let Some(analysis) = &source.analysis {
        bytes = bytes.saturating_add(analysis.id.len()).saturating_add(
            canonical::object_memory(&analysis.parameters, &mut Budget::new()).map_err(failure)?,
        );
    }
    for strings in [&source.y, &source.marker_record_types] {
        bytes = bytes.saturating_add(slots::<String>(strings.len()));
        for text in strings {
            bytes = bytes.saturating_add(text.len());
        }
    }
    for expression in [
        &source.x_expression,
        &source.series_by_expression,
        &source.z_expression,
    ]
    .into_iter()
    .flatten()
    {
        bytes = bytes.saturating_add(expression_heap(expression));
    }
    // Authored Z Is Retained In Both The Request And The Document Definition.
    if let Some(expression) = &source.z_expression {
        bytes = bytes.saturating_add(expression_heap(expression));
    }
    bytes = bytes.saturating_add(slots::<plots::DataExpression>(source.y_expressions.len()));
    for expression in &source.y_expressions {
        bytes = bytes.saturating_add(expression_heap(expression));
    }
    bytes = bytes.saturating_add(slots::<documents::PlotReferenceLine>(
        source.reference_lines.len(),
    ));
    for line in &source.reference_lines {
        bytes = bytes
            .saturating_add(optional(&line.label))
            .saturating_add(line.color.len());
        if let Some(expression) = &line.source {
            bytes = bytes.saturating_add(expression_heap(expression));
        }
    }
    bytes = bytes.saturating_add(slots::<documents::PlotReferenceBand>(
        source.reference_bands.len(),
    ));
    for band in &source.reference_bands {
        bytes = bytes
            .saturating_add(optional(&band.label))
            .saturating_add(band.color.len());
        for expression in [&band.center, &band.tolerance].into_iter().flatten() {
            bytes = bytes.saturating_add(expression_heap(expression));
        }
    }
    Ok(bytes)
}

fn failure(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return harmos::Error::ResourceExhausted {
            resource: "canonical historical document graph",
        };
    }
    harmos::Error::backend(
        "count canonical historical document graph",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */
