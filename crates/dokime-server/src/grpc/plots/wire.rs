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
use harmos::Protobuf;
use serde_json::Value as JsonValue;
use tonic::Status;
// Local Imports
use super::super::enums::canonical as enums;
use super::super::value::{self, Budget, canonical};
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
) -> Result<dokime_contract::operations::documents::DocumentPlotDefinition, Status> {
    let request = plot_payload_request_for_descriptor(
        descriptor,
        Some(descriptor.input.scope.clone()),
        None,
        descriptor.input.live_window_s,
        None,
    );
    Ok(
        dokime_contract::operations::documents::DocumentPlotDefinition {
            metadata: Some(canonical::metadata(&descriptor.metadata)?),
            request: Some(plot_request_wire(&request)?),
            presentation: Some(plot_presentation(&descriptor.input)?),
            z_expression: descriptor
                .input
                .z_expression
                .as_ref()
                .map(data_expression_wire)
                .transpose()?,
        },
    )
}

pub(super) fn envelope(
    value: PlotPayloadEnvelope,
    epoch: &str,
    selection: &str,
) -> Result<dokime_contract::operations::plots::PlotEnvelope, Status> {
    Ok(dokime_contract::operations::plots::PlotEnvelope {
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
) -> Result<dokime_contract::operations::plots::PlotStreamFrame, Status> {
    Ok(dokime_contract::operations::plots::PlotStreamFrame {
        kind: plot_stream_kind(kind),
        envelope: Some(envelope(value, epoch, selection)?),
    })
}

pub(crate) fn values(
    values: &BTreeMap<String, Value>,
) -> Result<dokime_contract::operations::value::Value, Status> {
    canonical::object(values, &mut Budget::new())
}
fn optional_format(
    value: Option<&dokime_app::core::ValueFormat>,
) -> Option<dokime_contract::operations::schema::ValueFormat> {
    value.map(canonical::format)
}

pub(super) fn ensure_message_budget<M: Protobuf>(
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
fn plot_request_wire(
    request: &PlotPayloadRequest,
) -> Result<dokime_contract::operations::plots::PlotRequest, Status> {
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
    Ok(dokime_contract::operations::plots::PlotRequest {
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
        plot_type: request.plot_type.as_ref().map_or(
            dokime_contract::operations::plots::PlotType::Unspecified,
            enums::plot_type,
        ),
        x_scale: enums::axis_scale(&request.x_scale),
        y_scale: enums::axis_scale(&request.y_scale),
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
        scope: request.scope.as_ref().map(canonical::scope).transpose()?,
        // A document owner definition is a fresh query selection. Cursors are
        // process-local and are added only when a caller opens a live stream.
        cursor: None,
        technique: enums::downsample_technique(&request.technique),
    })
}

pub(crate) fn plot_filter_wire(
    filter: &DocumentFilter,
) -> Result<dokime_contract::operations::plots::PlotFilter, Status> {
    Ok(dokime_contract::operations::plots::PlotFilter {
        expression: Some(data_expression_wire(&filter.expression)?),
        operator: enums::filter_operator(&filter.operator),
        value: filter
            .value
            .as_ref()
            .map(|value| canonical::encode(value, &mut Budget::new(), 0))
            .transpose()?,
    })
}

pub(crate) fn data_expression_wire(
    expression: &DataExpression,
) -> Result<dokime_contract::operations::plots::DataExpression, Status> {
    value::collection(expression.transforms.len())?;
    let source = Some(match &expression.source {
        DataSource::Telemetry { stream, field } => {
            dokime_contract::operations::plots::DataExpressionSource::Telemetry(
                dokime_contract::operations::plots::TelemetryDataSource {
                    stream: expression_stream(stream.clone())?,
                    field: bounded_text(field.clone())?,
                },
            )
        }
        DataSource::Parameter { name } => {
            dokime_contract::operations::plots::DataExpressionSource::Parameter(
                dokime_contract::operations::plots::ParameterDataSource {
                    name: bounded_text(name.clone())?,
                },
            )
        }
        DataSource::Dimension { name } => {
            dokime_contract::operations::plots::DataExpressionSource::Dimension(
                dokime_contract::operations::plots::DimensionDataSource {
                    name: bounded_text(name.clone())?,
                },
            )
        }
        DataSource::Record { record_type } => {
            dokime_contract::operations::plots::DataExpressionSource::Record(
                dokime_contract::operations::plots::RecordDataSource {
                    record_type: bounded_text(record_type.clone())?,
                },
            )
        }
        DataSource::Artifact { artifact } => {
            dokime_contract::operations::plots::DataExpressionSource::Artifact(
                dokime_contract::operations::plots::ArtifactDataSource {
                    artifact: bounded_text(artifact.clone())?,
                },
            )
        }
    });
    Ok(dokime_contract::operations::plots::DataExpression {
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

fn data_transform_wire(
    transform: &DataTransform,
) -> Result<dokime_contract::operations::plots::DataTransform, Status> {
    use dokime_contract::operations::plots::DataTransformKind as Kind;
    let kind = match transform {
        DataTransform::Aggregate(aggregate) => {
            Kind::Aggregate(dokime_contract::operations::plots::AggregateTransform {
                aggregate: aggregate.name(),
            })
        }
        DataTransform::TimeBetween {
            previous_record_type,
        } => Kind::TimeBetween(dokime_contract::operations::plots::TimeBetweenTransform {
            previous_record_type: bounded_text(previous_record_type.clone())?,
        }),
        DataTransform::TelemetryAtOrBefore { stream, field } => Kind::TelemetryAtOrBefore(
            dokime_contract::operations::plots::TelemetryAtOrBeforeTransform {
                stream: expression_stream(stream.clone())?,
                field: bounded_text(field.clone())?,
            },
        ),
        DataTransform::Detail { name } => {
            Kind::Detail(dokime_contract::operations::plots::DetailTransform {
                name: bounded_text(name.clone())?,
            })
        }
        DataTransform::PerCase => {
            Kind::PerCase(dokime_contract::operations::plots::EmptyTransform {})
        }
    };
    Ok(dokime_contract::operations::plots::DataTransform { kind: Some(kind) })
}

fn plot_presentation(
    input: &PlotInput,
) -> Result<dokime_contract::operations::documents::PlotPresentation, Status> {
    let (x_axis, x_axis_unit) = enums::plot_axis(input.x_axis.as_ref());
    Ok(dokime_contract::operations::documents::PlotPresentation {
        style: enums::plot_style(&input.style),
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
        x_format: optional_format(input.x_format.as_ref()),
        y_format: optional_format(input.y_format.as_ref()),
        x_axis,
        x_axis_unit: x_axis_unit.map(bounded_text).transpose()?,
    })
}

fn plot_fft_wire(
    value: &FftOptions,
) -> Result<dokime_contract::operations::plots::PlotFftOptions, Status> {
    Ok(dokime_contract::operations::plots::PlotFftOptions {
        sample_rate_hz: value
            .sample_rate_hz
            .map(|value| finite_nonnegative(value, "plot FFT sample_rate_hz"))
            .transpose()?,
        window: enums::fft_window(&value.window),
        detrend: enums::fft_detrend(&value.detrend),
        level_unit: enums::frequency_level_unit(&value.level_unit),
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

fn plot_interval_wire(
    value: &SigmaIntervalOptions,
) -> Result<dokime_contract::operations::plots::PlotIntervalOptions, Status> {
    Ok(dokime_contract::operations::plots::PlotIntervalOptions {
        bin_width: finite_nonnegative(value.bin_width, "plot interval bin_width")?,
        sigma: finite_nonnegative(value.sigma, "plot interval sigma")?,
        clip_to_observed: value.clip_to_observed,
    })
}

fn plot_histogram_wire(
    value: &HistogramOptions,
) -> Result<dokime_contract::operations::plots::PlotHistogramOptions, Status> {
    Ok(dokime_contract::operations::plots::PlotHistogramOptions {
        bins: u64::try_from(value.bins)
            .map_err(|_| Status::internal("plot histogram bins do not fit wire"))?,
        unit: value
            .unit
            .as_ref()
            .map(|value| bounded_text(value.clone()))
            .transpose()?,
    })
}

fn plot_reference_line_wire(
    value: &ReferenceLine,
) -> Result<dokime_contract::operations::documents::PlotReferenceLine, Status> {
    Ok(dokime_contract::operations::documents::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis),
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

fn plot_reference_band_wire(
    value: &ReferenceBand,
) -> Result<dokime_contract::operations::documents::PlotReferenceBand, Status> {
    Ok(dokime_contract::operations::documents::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis),
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

fn plot_stream_kind(
    value: PlotPayloadStreamKind,
) -> dokime_contract::operations::plots::PlotStreamKind {
    match value {
        PlotPayloadStreamKind::Snapshot => {
            dokime_contract::operations::plots::PlotStreamKind::Snapshot
        }
        PlotPayloadStreamKind::Append => dokime_contract::operations::plots::PlotStreamKind::Append,
        PlotPayloadStreamKind::Replace => {
            dokime_contract::operations::plots::PlotStreamKind::Replace
        }
        PlotPayloadStreamKind::Error => dokime_contract::operations::plots::PlotStreamKind::Error,
    }
}

fn plot_cursor_wire(
    value: PlotPayloadCursor,
    epoch: &str,
    selection: &str,
) -> dokime_contract::operations::plots::PlotCursor {
    dokime_contract::operations::plots::PlotCursor {
        telemetry_streams: value
            .telemetry_streams
            .into_iter()
            .map(|(key, value)| {
                dokime_contract::operations::plots::PlotCursorTelemetryStreamsEntry { key, value }
            })
            .collect(),
        record_sequence: value.record_sequence,
        epoch: epoch.to_owned(),
        selection: selection.to_owned(),
    }
}

fn plot_payload(
    value: dokime_app::runtime::query::PlotPayload,
) -> Result<dokime_contract::operations::plots::PlotPayload, Status> {
    let maximum = super::cursor::MAX_PLOT_POINTS as usize;
    if value
        .series
        .iter()
        .any(|series| series.points.len() > maximum)
        || value.cartesian.as_ref().is_some_and(|frame| {
            frame
                .series
                .iter()
                .any(|series| series.x.len().max(series.y.len()) > maximum)
        })
        || value.computed.as_ref().is_some_and(|computed| {
            computed.series.iter().any(|series| {
                series
                    .output
                    .outputs
                    .iter()
                    .any(|output| output.axis.len().max(output.values.len()) > maximum)
            })
        })
    {
        return Err(Status::resource_exhausted(
            "plot exceeds retained point limit",
        ));
    }
    let context = values(&value.context)?;
    let cartesian = value.cartesian.map(plot_cartesian).transpose()?;
    Ok(dokime_contract::operations::plots::PlotPayload {
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
        x_format: optional_format(value.x_format.as_ref()),
        run_started_at: value.run_started_at,
        cycle_started_at: value.cycle_started_at,
        analysis: value.analysis.map(plot_analysis).transpose()?,
    })
}

fn plot_analysis(
    value: dokime_app::runtime::query::PlotAnalysis,
) -> Result<dokime_contract::operations::plots::PlotAnalysis, Status> {
    use dokime_app::runtime::query::PlotAnalysis;
    use dokime_contract::operations::plots::PlotAnalysisResult as Analysis;

    let count = |value| {
        u64::try_from(value).map_err(|_| Status::internal("analysis count does not fit wire"))
    };
    let number = |value| finite_number(value, "plot analysis number");
    let result = match value {
        PlotAnalysis::Error { message } => Analysis::Error(dokime_contract::operations::plots::PlotAnalysisError {
            message: bounded_text(message)?,
        }),
        PlotAnalysis::Histogram { series } => Analysis::Histogram(dokime_contract::operations::plots::PlotHistogramAnalysis {
            series: series
                .into_iter()
                .map(|series| {
                    Ok(dokime_contract::operations::plots::PlotHistogramSeries {
                        key: bounded_text(series.key)?,
                        count: count(series.count)?,
                        statistics: series
                            .statistics
                            .map(|statistics| {
                                Ok::<_, Status>(dokime_contract::operations::plots::PlotHistogramStatistics {
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
                                Ok(dokime_contract::operations::plots::PlotHistogramBin {
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
        PlotAnalysis::BoxSummary { series } => Analysis::BoxSummary(dokime_contract::operations::plots::PlotBoxSummaryAnalysis {
            series: series
                .into_iter()
                .map(|series| {
                    Ok(dokime_contract::operations::plots::PlotBoxSummary {
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
            Analysis::SigmaInterval(dokime_contract::operations::plots::PlotSigmaIntervalAnalysis {
                series: series
                    .into_iter()
                    .map(|series| {
                        Ok(dokime_contract::operations::plots::PlotSigmaSeries {
                            key: bounded_text(series.key)?,
                            bins: series
                                .bins
                                .into_iter()
                                .map(|bin| {
                                    Ok(dokime_contract::operations::plots::PlotSigmaBin {
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

    Ok(dokime_contract::operations::plots::PlotAnalysis {
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
) -> Result<dokime_contract::operations::plots::PlotCartesianFrame, Status> {
    Ok(dokime_contract::operations::plots::PlotCartesianFrame {
        mode: match value.mode {
            dokime_app::runtime::query::PlotCartesianMode::Aligned => {
                dokime_contract::operations::plots::PlotCartesianMode::Aligned
            }
            dokime_app::runtime::query::PlotCartesianMode::Faceted => {
                dokime_contract::operations::plots::PlotCartesianMode::Faceted
            }
        },
        series: value
            .series
            .into_iter()
            .map(
                |series| dokime_contract::operations::plots::PlotCartesianSeriesFrame {
                    x: series.x,
                    y: series
                        .y
                        .into_iter()
                        .map(|value| dokime_contract::operations::plots::NullableNumber { value })
                        .collect(),
                },
            )
            .collect(),
        x_domain: value.x_domain.map(plot_domain),
        y_domain: value.y_domain.map(plot_domain),
        point_count: u64::try_from(value.point_count)
            .map_err(|_| Status::internal("plot point count does not fit wire"))?,
        signature: value.signature,
    })
}

fn plot_domain(
    value: dokime_app::runtime::query::PlotDomain,
) -> dokime_contract::operations::plots::PlotDomain {
    dokime_contract::operations::plots::PlotDomain {
        min: value.min,
        max: value.max,
    }
}

fn plot_diagnostics(
    value: dokime_app::runtime::query::PlotPayloadDiagnostics,
) -> Result<dokime_contract::operations::plots::PlotPayloadDiagnostics, Status> {
    Ok(dokime_contract::operations::plots::PlotPayloadDiagnostics {
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

fn plot_series(
    value: dokime_app::runtime::query::PlotSeries,
) -> Result<dokime_contract::operations::plots::PlotSeries, Status> {
    Ok(dokime_contract::operations::plots::PlotSeries {
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
        format: optional_format(value.format.as_ref()),
    })
}

fn plot_point(
    value: dokime_app::runtime::query::PlotPoint,
) -> Result<dokime_contract::operations::plots::PlotPoint, Status> {
    Ok(dokime_contract::operations::plots::PlotPoint {
        x: Some(canonical::encode(
            &json_value(value.x)?,
            &mut Budget::new(),
            0,
        )?),
        y: value.y,
        z: value.z,
    })
}

fn plot_marker(
    value: dokime_app::runtime::query::PlotMarker,
) -> Result<dokime_contract::operations::plots::PlotMarker, Status> {
    Ok(dokime_contract::operations::plots::PlotMarker {
        x: Some(canonical::encode(
            &json_value(value.x)?,
            &mut Budget::new(),
            0,
        )?),
        label: value.label,
        record_type: value.record_type,
        color: value.color,
    })
}

/* === Tests === */
