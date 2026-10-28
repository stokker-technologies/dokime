/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::document::{DocumentSelection, PlotDefinition, PlotDescriptor, PlotInput, PlotType};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

pub fn plot_input_for_definition(
    plot: &PlotDefinition, telemetry: &[TelemetrySample], marker_count: usize, selection: &DocumentSelection,
) -> PlotInput {
    let mut matching = Vec::with_capacity(telemetry.len());
    matching.extend(telemetry.iter().filter(|sample| sample_has_plot_fields(sample, plot)).map(|sample| &sample.stream));
    if matching.is_empty() && plot_uses_telemetry(plot) {
        matching.extend(telemetry.iter().map(|sample| &sample.stream));
    }
    matching.sort_unstable();
    matching.dedup();
    let available_streams: Vec<Identifier> = matching.into_iter().cloned().collect();
    let stream = plot.stream.as_ref().map(|stream| resolved_plot_stream(stream, &available_streams));

    PlotInput {
        plot_id: plot.metadata.identifier.clone(),
        scope: selection.scope.clone(),
        plot_type: plot.plot_type.clone(),
        x_field: plot.x.clone(),
        series_fields: plot.y.clone(),
        z_stream: plot.z_stream.clone(),
        z_field: plot.z_field.clone(),
        stream,
        series_by: plot.series_by.clone(),
        x_expression: plot.x_expression.clone(),
        y_expressions: plot.y_expressions.clone(),
        z_expression: plot.z_expression.clone(),
        series_by_expression: plot.series_by_expression.clone(),
        filters: plot.filters.clone(),
        unit: plot.unit.clone(),
        style: plot.style.clone(),
        x_scale: plot.x_scale.clone(),
        y_scale: plot.y_scale.clone(),
        fft: plot.fft.clone(),
        analysis: plot.analysis.clone(),
        interval: plot.interval.clone(),
        histogram: plot.histogram.clone(),
        marker_record_types: plot.marker_record_types.clone(),
        reference_lines: plot.reference_lines.clone(),
        reference_bands: plot.reference_bands.clone(),
        live_window_s: plot.live_window_s,
        display_points: plot.display_points,
        downsample: plot.downsample,
        x_format: plot.x_format.clone(),
        y_format: plot.y_format.clone(),
        x_axis: plot.x_axis.clone(),
        available_streams,
        marker_count,
    }
}

pub fn plot_descriptor_for_definition(input: PlotInput, metadata: Metadata) -> PlotDescriptor {
    PlotDescriptor {
        plot_id: input.plot_id.clone(),
        metadata,
        plot_type: input.plot_type.clone(),
        input,
    }
}

pub(crate) fn stream_matches_alias(actual: &str, declared: &str) -> bool {
    actual == declared || actual.strip_suffix(declared).is_some_and(|prefix| prefix.ends_with('.'))
}

fn sample_has_plot_fields(sample: &TelemetrySample, plot: &PlotDefinition) -> bool {
    if plot.stream.as_ref().is_some_and(|stream| !stream_matches_alias(sample.stream.as_str(), stream)) {
        return false;
    }

    if has_telemetry_expression(plot) {
        expression_sample_has_plot_fields(sample, plot)
    } else {
        field_sample_has_plot_fields(sample, plot)
    }
}

fn field_sample_has_plot_fields(sample: &TelemetrySample, plot: &PlotDefinition) -> bool {
    // A histogram's one value is its x; the count on y is the host's.
    let has_y_field = plot.plot_type == PlotType::Histogram || plot.y.iter().any(|field| sample.values.contains_key(field));
    let has_x_field = plot.x.as_ref().is_none_or(|field| field == "timestamp" || sample.values.contains_key(field));
    let has_z_field = plot.z_field.as_ref().is_none_or(|field| {
        let z_stream = plot.z_stream.as_ref().or(plot.stream.as_ref());
        z_stream.is_some_and(|stream| !stream_matches_alias(sample.stream.as_str(), stream)) || sample.values.contains_key(field)
    });

    has_y_field && has_x_field && has_z_field
}

fn expression_sample_has_plot_fields(sample: &TelemetrySample, plot: &PlotDefinition) -> bool {
    let has_y_field = plot.plot_type == PlotType::Histogram
        || plot.y_expressions.iter().any(|expression| telemetry_expression_matches(sample, expression));
    let has_x_field = plot.x_expression.as_ref().is_none_or(|expression| telemetry_expression_matches(sample, expression));
    let has_z_field = plot.z_expression.as_ref().is_none_or(|expression| telemetry_expression_matches(sample, expression));

    has_y_field && has_x_field && has_z_field
}

fn has_telemetry_expression(plot: &PlotDefinition) -> bool {
    plot.x_expression.as_ref().is_some_and(is_telemetry_expression)
        || plot.y_expressions.iter().any(is_telemetry_expression)
        || plot.z_expression.as_ref().is_some_and(is_telemetry_expression)
        || plot.series_by_expression.as_ref().is_some_and(is_telemetry_expression)
        || plot.filters.iter().any(|filter| is_telemetry_expression(&filter.expression))
}

fn plot_uses_telemetry(plot: &PlotDefinition) -> bool {
    has_telemetry_expression(plot)
        || plot.stream.is_some()
        || plot.x.is_some()
        || !plot.y.is_empty()
        || plot.z_field.is_some()
        || plot.z_stream.is_some()
        || plot.series_by.is_some()
}

fn is_telemetry_expression(expression: &DataExpression) -> bool {
    matches!(expression.source, DataSource::Telemetry { .. })
        || expression.transforms.iter().any(|transform| matches!(transform, DataTransform::TelemetryAtOrBefore { .. }))
}

fn telemetry_expression_matches(sample: &TelemetrySample, expression: &DataExpression) -> bool {
    match &expression.source {
        DataSource::Telemetry { stream, field } => {
            stream_matches_alias(sample.stream.as_str(), stream) && (field == "timestamp" || sample.values.contains_key(field))
        }
        DataSource::Record { .. } => expression.transforms.iter().any(|transform| {
            let DataTransform::TelemetryAtOrBefore { stream, field } = transform else {
                return false;
            };
            stream_matches_alias(sample.stream.as_str(), stream) && sample.values.contains_key(field)
        }),
        DataSource::Parameter { .. } | DataSource::Dimension { .. } | DataSource::Artifact { .. } => true,
    }
}

fn resolved_plot_stream(stream: &str, available_streams: &[Identifier]) -> String {
    available_streams
        .iter()
        .find(|candidate| stream_matches_alias(candidate.as_str(), stream))
        .map(|candidate| candidate.as_str().to_owned())
        .unwrap_or_else(|| stream.to_owned())
}
