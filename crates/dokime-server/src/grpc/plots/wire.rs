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
use std::collections::BTreeMap;
// Third Party Imports
use prost::Message;
use serde_json::Value as JsonValue;
use tonic::Status;
// Local Imports
use super::super::enums;
use super::super::value::{self, Budget};
use super::cursor::{bounded_text, expression_stream};
use dokime_app::{
    core::{Identifier, Value},
    document::{
        DocumentFilter, FftOptions, HistogramOptions, PlotDescriptor, PlotInput, ReferenceBand,
        ReferenceLine, SigmaIntervalOptions,
    },
    expressions::{DataExpression, DataSource, DataTransform},
    runtime::query::{
        PlotPayloadCursor, PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind,
        plot_payload_request_for_descriptor,
    },
};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn document_plot_definition(
    descriptor: &PlotDescriptor,
) -> Result<v1::DocumentPlotDefinition, Status> {
    let request = plot_payload_request_for_descriptor(
        descriptor,
        Some(descriptor.input.scope.clone()),
        None,
        descriptor.input.live_window_s,
        None,
    );
    Ok(v1::DocumentPlotDefinition {
        metadata: Some(value::checked_metadata(&descriptor.metadata)?),
        request: Some(plot_request_wire(&request)?),
        presentation: Some(plot_presentation(&descriptor.input)?),
        z_expression: descriptor
            .input
            .z_expression
            .as_ref()
            .map(data_expression_wire)
            .transpose()?,
    })
}

pub(super) fn envelope(
    value: PlotPayloadEnvelope,
    epoch: &str,
    selection: &str,
) -> Result<v1::PlotEnvelope, Status> {
    Ok(v1::PlotEnvelope {
        plot_id: value.plot_id,
        revision: value.revision,
        cursor: Some(plot_cursor_wire(value.cursor, epoch, selection)),
        payload: Some(plot_payload(value.payload)?),
        diagnostics: Some(plot_diagnostics(value.diagnostics)?),
        warnings: value.warnings,
    })
}

pub(super) fn stream_frame(
    kind: PlotPayloadStreamKind,
    value: PlotPayloadEnvelope,
    epoch: &str,
    selection: &str,
) -> Result<v1::PlotStreamFrame, Status> {
    Ok(v1::PlotStreamFrame {
        kind: plot_stream_kind(kind) as i32,
        envelope: Some(envelope(value, epoch, selection)?),
    })
}

pub(crate) fn values(values: &BTreeMap<String, Value>) -> Result<v1::ValueObject, Status> {
    value::collection(values.len())?;
    let mut budget = Budget::new();
    Ok(v1::ValueObject {
        fields: values
            .iter()
            .map(|(key, item)| {
                value::text(key)?;
                Ok((key.clone(), value::encode(item, &mut budget, 0)?))
            })
            .collect::<Result<_, Status>>()?,
    })
}

pub(super) fn ensure_message_budget<M: Message>(
    value: &M,
    limit: usize,
    label: &str,
) -> Result<(), Status> {
    if value.encoded_len() > limit {
        return Err(Status::resource_exhausted(format!(
            "{label} exceeds transport limit"
        )));
    }
    Ok(())
}

/* === Private Functions === */
fn plot_request_wire(request: &PlotPayloadRequest) -> Result<v1::PlotRequest, Status> {
    let stream = wire_stream(&request.stream)?;
    value::collection(request.y_keys.len())?;
    let y_keys = request
        .y_keys
        .iter()
        .map(|value| bounded_text(value.clone()))
        .collect::<Result<_, _>>()?;
    value::collection(request.marker_record_types.len())?;
    let marker_record_types = request
        .marker_record_types
        .iter()
        .map(|value| bounded_text(value.clone()))
        .collect::<Result<_, _>>()?;
    Ok(v1::PlotRequest {
        analysis: request
            .analysis
            .as_ref()
            .map(super::analysis::selection_wire)
            .transpose()?,
        plot_id: request
            .plot_id
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
        plot_type: request
            .plot_type
            .as_ref()
            .map_or(v1::PlotType::Unspecified, enums::plot_type) as i32,
        x_scale: enums::axis_scale(&request.x_scale) as i32,
        y_scale: enums::axis_scale(&request.y_scale) as i32,
        stream,
        y_keys,
        marker_record_types,
        filters: request
            .filters
            .iter()
            .map(plot_filter_wire)
            .collect::<Result<_, _>>()?,
        x_key: request
            .x_key
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
        series_by: request
            .series_by
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
        x_expression: request
            .x_expression
            .as_ref()
            .map(data_expression_wire)
            .transpose()?,
        y_expressions: request
            .y_expressions
            .iter()
            .map(data_expression_wire)
            .collect::<Result<_, _>>()?,
        series_by_expression: request
            .series_by_expression
            .as_ref()
            .map(data_expression_wire)
            .transpose()?,
        z_expression: request
            .z_expression
            .as_ref()
            .map(data_expression_wire)
            .transpose()?,
        fft: Some(plot_fft_wire(&request.fft)?),
        interval: Some(plot_interval_wire(&request.interval)?),
        histogram: Some(plot_histogram_wire(&request.histogram)?),
        z_stream: request
            .z_stream
            .as_ref()
            .map(|value| wire_stream(value))
            .transpose()?,
        z_field: request
            .z_field
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
        unit: request
            .unit
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
        max_points: request
            .max_points
            .map(|value| {
                u64::try_from(value)
                    .map_err(|_| Status::internal("plot max_points does not fit wire"))
            })
            .transpose()?,
        window_s: request
            .window_s
            .map(|value| finite_duration(value, "plot window_s"))
            .transpose()?,
        scope: request.scope.as_ref().map(value::scope),
        // A document owner definition is a fresh query selection. Cursors are
        // process-local and are added only when a caller opens a live stream.
        cursor: None,
        technique: enums::downsample_technique(&request.technique) as i32,
    })
}

pub(crate) fn plot_filter_wire(filter: &DocumentFilter) -> Result<v1::PlotFilter, Status> {
    Ok(v1::PlotFilter {
        expression: Some(data_expression_wire(&filter.expression)?),
        operator: enums::filter_operator(&filter.operator) as i32,
        value: filter
            .value
            .as_ref()
            .map(|value| value::encode(value, &mut Budget::new(), 0))
            .transpose()?,
    })
}

pub(crate) fn data_expression_wire(
    expression: &DataExpression,
) -> Result<v1::DataExpression, Status> {
    value::collection(expression.transforms.len())?;
    let source = Some(match &expression.source {
        DataSource::Telemetry { stream, field } => {
            v1::data_expression::Source::Telemetry(v1::TelemetryDataSource {
                stream: expression_stream(stream.clone())?,
                field: bounded_text(field.clone())?,
            })
        }
        DataSource::Parameter { name } => {
            v1::data_expression::Source::Parameter(v1::ParameterDataSource {
                name: bounded_text(name.clone())?,
            })
        }
        DataSource::Dimension { name } => {
            v1::data_expression::Source::Dimension(v1::DimensionDataSource {
                name: bounded_text(name.clone())?,
            })
        }
        DataSource::Record { record_type } => {
            v1::data_expression::Source::Record(v1::RecordDataSource {
                record_type: bounded_text(record_type.clone())?,
            })
        }
        DataSource::Artifact { artifact } => {
            v1::data_expression::Source::Artifact(v1::ArtifactDataSource {
                artifact: bounded_text(artifact.clone())?,
            })
        }
    });
    Ok(v1::DataExpression {
        label: expression
            .label
            .as_ref()
            .map(|label| bounded_text(label.clone()))
            .transpose()?,
        source,
        transforms: expression
            .transforms
            .iter()
            .map(data_transform_wire)
            .collect::<Result<_, _>>()?,
    })
}

fn data_transform_wire(transform: &DataTransform) -> Result<v1::DataTransform, Status> {
    use v1::data_transform::Kind;
    let kind = match transform {
        DataTransform::Count => Kind::Count(v1::EmptyTransform {}),
        DataTransform::Mean => Kind::Mean(v1::EmptyTransform {}),
        DataTransform::Median => Kind::Median(v1::EmptyTransform {}),
        DataTransform::Min => Kind::Min(v1::EmptyTransform {}),
        DataTransform::Max => Kind::Max(v1::EmptyTransform {}),
        DataTransform::TimeBetween {
            previous_record_type,
        } => Kind::TimeBetween(v1::TimeBetweenTransform {
            previous_record_type: bounded_text(previous_record_type.clone())?,
        }),
        DataTransform::TelemetryAtOrBefore { stream, field } => {
            Kind::TelemetryAtOrBefore(v1::TelemetryAtOrBeforeTransform {
                stream: expression_stream(stream.clone())?,
                field: bounded_text(field.clone())?,
            })
        }
        DataTransform::Detail { name } => Kind::Detail(v1::DetailTransform {
            name: bounded_text(name.clone())?,
        }),
        DataTransform::PerCase => Kind::PerCase(v1::EmptyTransform {}),
    };
    Ok(v1::DataTransform { kind: Some(kind) })
}

fn plot_presentation(input: &PlotInput) -> Result<v1::PlotPresentation, Status> {
    let (x_axis, x_axis_unit) = enums::plot_axis(input.x_axis.as_ref());
    // PlotInput is materialized after layout normalization and intentionally
    // carries no authored group. The document layout remains the grouping
    // authority; an empty group lets clients apply that parent section.
    Ok(v1::PlotPresentation {
        group: String::new(),
        style: enums::plot_style(&input.style) as i32,
        fft: Some(plot_fft_wire(&input.fft)?),
        interval: Some(plot_interval_wire(&input.interval)?),
        histogram: Some(plot_histogram_wire(&input.histogram)?),
        reference_lines: input
            .reference_lines
            .iter()
            .map(plot_reference_line_wire)
            .collect::<Result<_, _>>()?,
        reference_bands: input
            .reference_bands
            .iter()
            .map(plot_reference_band_wire)
            .collect::<Result<_, _>>()?,
        live_window_s: input
            .live_window_s
            .map(|value| finite_duration(value, "plot live_window_s"))
            .transpose()?,
        x_format: value::optional_value_format(input.x_format.as_ref()),
        y_format: value::optional_value_format(input.y_format.as_ref()),
        x_axis,
        x_axis_unit: x_axis_unit.map(bounded_text).transpose()?,
    })
}

fn plot_fft_wire(value: &FftOptions) -> Result<v1::PlotFftOptions, Status> {
    Ok(v1::PlotFftOptions {
        sample_rate_hz: value
            .sample_rate_hz
            .map(|value| finite_nonnegative(value, "plot FFT sample_rate_hz"))
            .transpose()?,
        window: enums::fft_window(&value.window) as i32,
        detrend: enums::fft_detrend(&value.detrend) as i32,
        level_unit: enums::frequency_level_unit(&value.level_unit) as i32,
        include_dc: value.include_dc,
        min_frequency_hz: value
            .min_frequency_hz
            .map(|value| finite_nonnegative(value, "plot FFT min_frequency_hz"))
            .transpose()?,
        max_frequency_hz: value
            .max_frequency_hz
            .map(|value| finite_nonnegative(value, "plot FFT max_frequency_hz"))
            .transpose()?,
    })
}

fn plot_interval_wire(value: &SigmaIntervalOptions) -> Result<v1::PlotIntervalOptions, Status> {
    Ok(v1::PlotIntervalOptions {
        bin_width: finite_nonnegative(value.bin_width, "plot interval bin_width")?,
        sigma: finite_nonnegative(value.sigma, "plot interval sigma")?,
        clip_to_observed: value.clip_to_observed,
    })
}

fn plot_histogram_wire(value: &HistogramOptions) -> Result<v1::PlotHistogramOptions, Status> {
    Ok(v1::PlotHistogramOptions {
        bins: u64::try_from(value.bins)
            .map_err(|_| Status::internal("plot histogram bins do not fit wire"))?,
        unit: value
            .unit
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
    })
}

fn plot_reference_line_wire(value: &ReferenceLine) -> Result<v1::PlotReferenceLine, Status> {
    Ok(v1::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis) as i32,
        value: value
            .value
            .map(|value| finite_number(value, "plot reference line value"))
            .transpose()?,
        source: value
            .source
            .as_ref()
            .map(data_expression_wire)
            .transpose()?,
        source_scale: finite_number(value.source_scale, "plot reference line source_scale")?,
        label: value
            .label
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
        color: bounded_text(value.color.clone())?,
    })
}

fn plot_reference_band_wire(value: &ReferenceBand) -> Result<v1::PlotReferenceBand, Status> {
    Ok(v1::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis) as i32,
        min: value
            .min
            .map(|value| finite_number(value, "plot reference band min"))
            .transpose()?,
        max: value
            .max
            .map(|value| finite_number(value, "plot reference band max"))
            .transpose()?,
        label: value
            .label
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
        color: bounded_text(value.color.clone())?,
        center: value
            .center
            .as_ref()
            .map(data_expression_wire)
            .transpose()?,
        tolerance: value
            .tolerance
            .as_ref()
            .map(data_expression_wire)
            .transpose()?,
    })
}

fn wire_stream(value: &str) -> Result<String, Status> {
    let value = bounded_text(value.to_owned())?;
    Identifier::new(value.clone())
        .map_err(|error| Status::internal(format!("invalid authored plot stream: {error}")))?;
    Ok(value)
}

fn finite_number(value: f64, label: &str) -> Result<f64, Status> {
    if !value.is_finite() {
        return Err(Status::internal(format!("{label} is not finite")));
    }
    Ok(value)
}

fn finite_nonnegative(value: f64, label: &str) -> Result<f64, Status> {
    let value = finite_number(value, label)?;
    if value < 0.0 {
        return Err(Status::internal(format!("{label} is negative")));
    }
    Ok(value)
}

fn finite_duration(value: f64, label: &str) -> Result<f64, Status> {
    finite_nonnegative(value, label)
}

fn plot_stream_kind(value: PlotPayloadStreamKind) -> v1::PlotStreamKind {
    match value {
        PlotPayloadStreamKind::Snapshot => v1::PlotStreamKind::Snapshot,
        PlotPayloadStreamKind::Append => v1::PlotStreamKind::Append,
        PlotPayloadStreamKind::Replace => v1::PlotStreamKind::Replace,
        PlotPayloadStreamKind::Error => v1::PlotStreamKind::Error,
    }
}

fn plot_cursor_wire(value: PlotPayloadCursor, epoch: &str, selection: &str) -> v1::PlotCursor {
    v1::PlotCursor {
        telemetry_streams: value.telemetry_streams.into_iter().collect(),
        record_sequence: value.record_sequence,
        epoch: epoch.to_owned(),
        selection: selection.to_owned(),
    }
}

fn plot_payload(value: dokime_app::runtime::query::PlotPayload) -> Result<v1::PlotPayload, Status> {
    let context = values(&value.context)?;
    let cartesian = value.cartesian.map(plot_cartesian).transpose()?;
    Ok(v1::PlotPayload {
        computed: value
            .computed
            .map(super::analysis::computation)
            .transpose()?,
        stream: value.stream,
        x_key: value.x_key,
        series_by: value.series_by,
        context: Some(context),
        series: value
            .series
            .into_iter()
            .map(plot_series)
            .collect::<Result<_, _>>()?,
        cartesian,
        markers: value
            .markers
            .into_iter()
            .map(plot_marker)
            .collect::<Result<_, _>>()?,
        max_points: u64::try_from(value.max_points)
            .map_err(|_| Status::internal("plot max_points does not fit wire"))?,
        window_s: value.window_s,
        generated_at: value.generated_at,
        x_format: value::optional_value_format(value.x_format.as_ref()),
        run_started_at: value.run_started_at,
        cycle_started_at: value.cycle_started_at,
        analysis: value.analysis.map(plot_analysis).transpose()?,
    })
}

fn plot_analysis(
    value: dokime_app::runtime::query::PlotAnalysis,
) -> Result<v1::PlotAnalysis, Status> {
    use dokime_app::runtime::query::PlotAnalysis;
    use v1::plot_analysis::Result as Analysis;

    let count = |value| {
        u64::try_from(value).map_err(|_| Status::internal("analysis count does not fit wire"))
    };
    let number = |value| finite_number(value, "plot analysis number");
    let result = match value {
        PlotAnalysis::Error { message } => Analysis::Error(v1::PlotAnalysisError {
            message: bounded_text(message)?,
        }),
        PlotAnalysis::Histogram { series } => Analysis::Histogram(v1::PlotHistogramAnalysis {
            series: series
                .into_iter()
                .map(|series| {
                    Ok(v1::PlotHistogramSeries {
                        key: bounded_text(series.key)?,
                        count: count(series.count)?,
                        statistics: series
                            .statistics
                            .map(|statistics| {
                                Ok::<_, Status>(v1::PlotHistogramStatistics {
                                    mean: number(statistics.mean)?,
                                    median: number(statistics.median)?,
                                    std: number(statistics.std)?,
                                    rms: number(statistics.rms)?,
                                    min: number(statistics.min)?,
                                    max: number(statistics.max)?,
                                })
                            })
                            .transpose()?,
                        bins: series
                            .bins
                            .into_iter()
                            .map(|bin| {
                                Ok(v1::PlotHistogramBin {
                                    lower: number(bin.lower)?,
                                    upper: number(bin.upper)?,
                                    count: count(bin.count)?,
                                })
                            })
                            .collect::<Result<_, Status>>()?,
                    })
                })
                .collect::<Result<_, Status>>()?,
        }),
        PlotAnalysis::BoxSummary { series } => Analysis::BoxSummary(v1::PlotBoxSummaryAnalysis {
            series: series
                .into_iter()
                .map(|series| {
                    Ok(v1::PlotBoxSummary {
                        key: bounded_text(series.key)?,
                        label: bounded_text(series.label)?,
                        count: count(series.count)?,
                        min: number(series.min)?,
                        q1: number(series.q1)?,
                        median: number(series.median)?,
                        q3: number(series.q3)?,
                        max: number(series.max)?,
                        lower_whisker: number(series.lower_whisker)?,
                        upper_whisker: number(series.upper_whisker)?,
                        outliers: series
                            .outliers
                            .into_iter()
                            .map(number)
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, Status>>()?,
        }),
        PlotAnalysis::SigmaInterval { series } => {
            Analysis::SigmaInterval(v1::PlotSigmaIntervalAnalysis {
                series: series
                    .into_iter()
                    .map(|series| {
                        Ok(v1::PlotSigmaSeries {
                            key: bounded_text(series.key)?,
                            bins: series
                                .bins
                                .into_iter()
                                .map(|bin| {
                                    Ok(v1::PlotSigmaBin {
                                        x: number(bin.x)?,
                                        count: count(bin.count)?,
                                        mean: number(bin.mean)?,
                                        lower: number(bin.lower)?,
                                        upper: number(bin.upper)?,
                                    })
                                })
                                .collect::<Result<_, Status>>()?,
                        })
                    })
                    .collect::<Result<_, Status>>()?,
            })
        }
    };

    Ok(v1::PlotAnalysis {
        result: Some(result),
    })
}

fn json_value(value: JsonValue) -> Result<Value, Status> {
    Ok(match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(value) => Value::Bool(value),
        JsonValue::Number(value) => {
            if let Some(value) = value.as_i64() {
                Value::Integer(value)
            } else if let Some(value) = value.as_u64() {
                Value::Unsigned(value)
            } else {
                let value = value
                    .as_f64()
                    .ok_or_else(|| Status::internal("plot point number is not representable"))?;
                if !value.is_finite() {
                    return Err(Status::internal("plot point number is not finite"));
                }
                Value::Number(value)
            }
        }
        JsonValue::String(value) => Value::Text(value),
        JsonValue::Array(values) => Value::Array(
            values
                .into_iter()
                .map(json_value)
                .collect::<Result<_, _>>()?,
        ),
        JsonValue::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| Ok((key, json_value(value)?)))
                .collect::<Result<_, Status>>()?,
        ),
    })
}

fn plot_cartesian(
    value: dokime_app::runtime::query::PlotCartesianFrame,
) -> Result<v1::PlotCartesianFrame, Status> {
    Ok(v1::PlotCartesianFrame {
        mode: match value.mode {
            dokime_app::runtime::query::PlotCartesianMode::Aligned => {
                v1::PlotCartesianMode::Aligned
            }
            dokime_app::runtime::query::PlotCartesianMode::Faceted => {
                v1::PlotCartesianMode::Faceted
            }
        } as i32,
        series: value
            .series
            .into_iter()
            .map(|series| v1::PlotCartesianSeriesFrame {
                x: series.x,
                y: series
                    .y
                    .into_iter()
                    .map(|value| v1::NullableNumber { value })
                    .collect(),
            })
            .collect(),
        x_domain: value.x_domain.map(plot_domain),
        y_domain: value.y_domain.map(plot_domain),
        point_count: u64::try_from(value.point_count)
            .map_err(|_| Status::internal("plot point count does not fit wire"))?,
        signature: value.signature,
    })
}

fn plot_domain(value: dokime_app::runtime::query::PlotDomain) -> v1::PlotDomain {
    v1::PlotDomain {
        min: value.min,
        max: value.max,
    }
}

fn plot_diagnostics(
    value: dokime_app::runtime::query::PlotPayloadDiagnostics,
) -> Result<v1::PlotPayloadDiagnostics, Status> {
    Ok(v1::PlotPayloadDiagnostics {
        series_count: u64::try_from(value.series_count)
            .map_err(|_| Status::internal("plot series count does not fit wire"))?,
        point_count: u64::try_from(value.point_count)
            .map_err(|_| Status::internal("plot point count does not fit wire"))?,
        raw_point_count: u64::try_from(value.raw_point_count)
            .map_err(|_| Status::internal("plot raw point count does not fit wire"))?,
        downsampled: value.downsampled,
        delta: value.delta,
        max_points: u64::try_from(value.max_points)
            .map_err(|_| Status::internal("plot max_points does not fit wire"))?,
        window_s: value.window_s,
    })
}

fn plot_series(value: dokime_app::runtime::query::PlotSeries) -> Result<v1::PlotSeries, Status> {
    Ok(v1::PlotSeries {
        key: value.key,
        label: value.label,
        unit: value.unit,
        points: value
            .points
            .into_iter()
            .map(plot_point)
            .collect::<Result<_, _>>()?,
        raw_points: u64::try_from(value.raw_points)
            .map_err(|_| Status::internal("plot raw point count does not fit wire"))?,
        downsampled: value.downsampled,
        format: value::optional_value_format(value.format.as_ref()),
    })
}

fn plot_point(value: dokime_app::runtime::query::PlotPoint) -> Result<v1::PlotPoint, Status> {
    Ok(v1::PlotPoint {
        x: Some(value::encode(&json_value(value.x)?, &mut Budget::new(), 0)?),
        y: value.y,
        z: value.z,
    })
}

fn plot_marker(value: dokime_app::runtime::query::PlotMarker) -> Result<v1::PlotMarker, Status> {
    Ok(v1::PlotMarker {
        x: Some(value::encode(&json_value(value.x)?, &mut Budget::new(), 0)?),
        label: value.label,
        record_type: value.record_type,
        color: value.color,
    })
}

/* === Tests === */
