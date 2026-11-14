/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result};

use dokime_protocol::host::PlotPresentation;
use dokime_protocol::host::PlotReferenceBand;
use dokime_protocol::host::PlotReferenceLine;
use dokime_protocol::host::PlotReferenceValue;
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::model::DataExpression;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::GridDefinition;
use dokime_protocol::model::Plot;
use dokime_protocol::model::PlotFftOptions;
use dokime_protocol::model::PlotFilter;
use dokime_protocol::model::PlotHistogramOptions;
use dokime_protocol::model::PlotKind;
use dokime_storage::{ExecutionCut, Store};

use dokime_protocol::model::data_expression::Kind as DataExpressionKind;
use dokime_protocol::model::value::Kind as ValueKind;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn request(plot: &Plot, scope: EvidenceScope) -> Result<PlotRequest> {
    let stream = plot
        .y
        .iter()
        .find_map(stream)
        .or_else(|| plot.z.as_ref().and_then(stream))
        .or_else(|| plot.x.as_ref().and_then(stream))
        .ok_or_else(|| Error::Invalid("Render Plot: Bind at Least One Stored Stream Expression".into()))?;
    Ok(PlotRequest {
        plot_id: plot.metadata.as_ref().map(|metadata| metadata.identifier.clone()),
        plot_type: plot.kind,
        style: plot.style,
        x_scale: plot.x_scale,
        y_scale: plot.y_scale,
        stream,
        scope: Some(scope),
        x_expression: plot.x.clone(),
        y_expressions: plot.y.clone(),
        z_expression: plot.z.clone(),
        series_by_expression: plot.series_by.clone(),
        filters: plot
            .filters
            .iter()
            .map(|filter| PlotFilter {
                expression: filter.expression.clone(),
                operator: filter.operator,
                value: filter.value.clone(),
            })
            .collect(),
        marker_record_types: plot.marker_records.clone(),
        unit: Some(plot.unit.clone()),
        max_points: plot.display_points.map(u64::from),
        histogram: plot.histogram_bins.map(|bins| PlotHistogramOptions { bins: bins as u64, ..Default::default() }),
        fft: (plot.kind == PlotKind::FrequencySeries as i32).then_some(PlotFftOptions {
            sample_rate_hz: plot.sample_rate_hz,
            input_samples: plot.input_samples,
            ..Default::default()
        }),
        arrangement: plot.arrangement,
        pareto: plot.pareto,
        hole: plot.hole,
        xy_interval: plot.interval,
        reduction: plot.reduction,
        render: plot.render,
        grid: plot.grid.as_ref().map(|grid| GridDefinition {
            row: grid.row.clone(),
            column: grid.column.clone(),
            frame: grid.frame.clone(),
            rows: grid.rows.clone(),
            columns: grid.columns.clone(),
            highlights: grid.highlights.clone(),
            summary_label: grid.summary_label.clone(),
        }),
        ..Default::default()
    })
}

pub fn presentation(store: &Store, cut: &ExecutionCut, plot: &Plot) -> Result<PlotPresentation> {
    let value = |expression: Option<&DataExpression>| -> Result<Option<PlotReferenceValue>> {
        let Some(expression) = expression else {
            return Ok(None);
        };
        let value = store.document_value(cut, expression)?;
        let value = match value.kind {
            Some(ValueKind::Number(value)) => value,
            Some(ValueKind::Integer(value)) => value as f64,
            Some(ValueKind::UnsignedInteger(value)) => value as f64,
            _ => return Err(Error::Invalid("Render Plot Reference: Select a Numeric Scalar".into())),
        };
        Ok(Some(PlotReferenceValue { constant: Some(value), expression: None }))
    };
    let mut reference_lines = Vec::new();
    for line in &plot.reference_lines {
        reference_lines.push(PlotReferenceLine {
            value: value(line.value.as_ref())?,
            label: Some(line.label.clone()),
            color: line.color.clone(),
        });
    }
    let mut reference_bands = Vec::new();
    for band in &plot.reference_bands {
        reference_bands.push(PlotReferenceBand {
            min: value(band.min.as_ref())?,
            max: value(band.max.as_ref())?,
            center: value(band.center.as_ref())?,
            tolerance: value(band.tolerance.as_ref())?,
            label: Some(band.label.clone()),
            color: band.color.clone(),
        });
    }
    Ok(PlotPresentation {
        style: plot.style,
        reference_lines,
        reference_bands,
        fft: (plot.kind == PlotKind::FrequencySeries as i32).then_some(PlotFftOptions {
            sample_rate_hz: plot.sample_rate_hz,
            input_samples: plot.input_samples,
            ..Default::default()
        }),
        histogram: plot.histogram_bins.map(|bins| PlotHistogramOptions { bins: bins as u64, ..Default::default() }),
        ..Default::default()
    })
}

/* === Private Functions === */

fn stream(expression: &DataExpression) -> Option<String> {
    match expression.kind.as_ref()? {
        DataExpressionKind::Stream(reference) => Some(reference.identifier.clone()),
        DataExpressionKind::Aggregate(value) => value.input.as_ref().and_then(|input| stream(input)),
        DataExpressionKind::Transform(value) => value.input.as_ref().and_then(|input| stream(input)),
        _ => None,
    }
}

/* === Tests === */
