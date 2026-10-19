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
use crate::grpc::transcode::Transcode;
use harmos::MemorySize;
use harmos::domain::Portable;
use serde_json::Value as JsonValue;
use tonic::Status;
// Local Imports
use super::super::enums::canonical as enums;
use super::super::value::{self, Budget, canonical};
use super::cursor::{bounded_text, expression_stream};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;
use dokime_runtime::document::DocumentFilter;
use dokime_runtime::document::FftOptions;
use dokime_runtime::document::HistogramOptions;
use dokime_runtime::document::PlotDescriptor;
use dokime_runtime::document::PlotInput;
use dokime_runtime::document::ReferenceBand;
use dokime_runtime::document::ReferenceLine;
use dokime_runtime::document::SigmaIntervalOptions;
use dokime_runtime::expressions::DataExpression;
use dokime_runtime::expressions::DataSource;
use dokime_runtime::expressions::DataTransform;
use dokime_runtime::runtime::query::PlotPayloadCursor;
use dokime_runtime::runtime::query::PlotPayloadEnvelope;
use dokime_runtime::runtime::query::PlotPayloadRequest;
use dokime_runtime::runtime::query::PlotPayloadStreamKind;
use dokime_runtime::runtime::query::plot_payload_request_for_descriptor;

/* === Constants === */
const MESSAGE_HEADROOM_BYTES: usize = 64;
const DECODED_HEADROOM_BYTES: usize = 1024;
const GENERATED_AT_BYTES: usize = "0000-00-00T00:00:00.000000000Z".len();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn document_plot_definition(
    descriptor: &PlotDescriptor,
) -> Result<dokime_domain::model::documents::wire::DocumentPlotDefinition, Status> {
    let request =
        plot_payload_request_for_descriptor(descriptor, Some(descriptor.input.scope.clone()), None, descriptor.input.live_window_s, None);
    Ok(dokime_domain::model::documents::wire::DocumentPlotDefinition {
        metadata: Some(canonical::metadata(&descriptor.metadata)?),
        request: Some(plot_request_wire(&request)?),
        presentation: Some(plot_presentation(&descriptor.input)?),
        z_expression: descriptor.input.z_expression.as_ref().map(data_expression_wire).transpose()?,
    })
}

pub(super) fn envelope(
    value: PlotPayloadEnvelope, epoch: &str, selection: &str, limit: usize,
) -> Result<dokime_domain::model::plots::wire::PlotEnvelope, Status> {
    let envelope = dokime_domain::model::plots::wire::PlotEnvelope {
        plot_id: value.plot_id,
        revision: value.revision,
        cursor: Some(plot_cursor_wire(value.cursor, epoch, selection)),
        payload: Some(plot_payload(value.payload)?),
        diagnostics: Some(plot_diagnostics(value.diagnostics)?),
        warnings: value.warnings,
    };
    fit_message(&envelope, limit)?;
    Ok(envelope)
}

pub(super) fn stream_frame(
    kind: PlotPayloadStreamKind, value: PlotPayloadEnvelope, epoch: &str, selection: &str, limit: usize,
) -> Result<dokime_domain::model::plots::wire::PlotStreamFrame, Status> {
    Ok(dokime_domain::model::plots::wire::PlotStreamFrame {
        kind: plot_stream_kind(kind),
        envelope: Some(envelope(value, epoch, selection, limit)?),
    })
}

pub(crate) fn values(values: &BTreeMap<String, Value>) -> Result<dokime_domain::model::values::wire::Value, Status> {
    canonical::object(values, &mut Budget::new())
}
fn optional_format(
    value: Option<&dokime_domain::model::values::ValueFormat>,
) -> Option<dokime_domain::model::definition::wire::ValueFormat> {
    value.map(canonical::format)
}

pub(super) fn ensure_message_budget<M: Portable>(value: &M, limit: usize, label: &str) -> Result<(), Status> {
    if value.encoded_len() > limit {
        return Err(Status::resource_exhausted(format!("{label} exceeds transport limit")));
    }
    Ok(())
}

/* === Private Functions === */
fn fit_message(envelope: &dokime_domain::model::plots::wire::PlotEnvelope, limit: usize) -> Result<(), Status> {
    let encoded_budget = limit.saturating_sub(MESSAGE_HEADROOM_BYTES);
    let decoded_budget = super::super::config::output_stream_bytes(limit).saturating_sub(DECODED_HEADROOM_BYTES);
    let encoded = envelope.wire_len()?;
    let decoded = envelope.memory_size();
    if encoded <= encoded_budget && decoded <= decoded_budget {
        return Ok(());
    }
    let exhausted = || Status::resource_exhausted("plot response exceeds transport limit");
    let Some(payload) = &envelope.payload else {
        return Err(exhausted());
    };
    let count = payload.series.iter().map(|series| series.points.len()).sum::<usize>();
    // Points and the cartesian frame aligned with them grow together; the
    // rest of the envelope is the same whatever the points. Its generation
    // time is charged at its longest, so every delivery of this plot names
    // the same bound however its trailing zeros fall.
    let mut bare = envelope.clone();
    if let Some(payload) = &mut bare.payload {
        payload.cartesian = None;
        for series in &mut payload.series {
            series.points = Vec::new();
        }
        let generated = payload.generated_at.len().max(GENERATED_AT_BYTES);
        payload.generated_at = "0".repeat(generated);
    }
    let series = payload.series.len().max(1);
    let fits = |total: usize, fixed: usize, budget: usize| {
        (count > 0 && fixed < budget).then(|| {
            let per_point = total.saturating_sub(fixed).div_ceil(count).max(1);
            (budget - fixed) / per_point / series
        })
    };
    let (Some(by_encoding), Some(by_decoding)) =
        (fits(encoded, bare.wire_len()?, encoded_budget), fits(decoded, bare.memory_size(), decoded_budget))
    else {
        return Err(exhausted());
    };
    let maximum = by_encoding.min(by_decoding);
    Err(Status::invalid_argument(format!(
        "plot {} cannot take {} points in one {limit}-byte message; at most {maximum} points fit",
        envelope.plot_id, payload.max_points
    )))
}

fn plot_request_wire(request: &PlotPayloadRequest) -> Result<dokime_domain::model::plots::wire::PlotRequest, Status> {
    let stream = wire_stream(&request.stream)?;
    value::collection(request.y_keys.len())?;
    let y_keys = request.y_keys.iter().map(|value| bounded_text(value.clone())).collect::<Result<_, _>>()?;
    value::collection(request.marker_record_types.len())?;
    let marker_record_types = request.marker_record_types.iter().map(|value| bounded_text(value.clone())).collect::<Result<_, _>>()?;
    Ok(dokime_domain::model::plots::wire::PlotRequest {
        analysis: request.analysis.as_ref().map(super::analysis::selection_wire).transpose()?,
        plot_id: request.plot_id.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
        plot_type: request.plot_type.as_ref().map_or(dokime_domain::model::plots::wire::PlotType::Unspecified, enums::plot_type),
        x_scale: enums::axis_scale(&request.x_scale),
        y_scale: enums::axis_scale(&request.y_scale),
        stream,
        y_keys,
        marker_record_types,
        filters: request.filters.iter().map(plot_filter_wire).collect::<Result<_, _>>()?,
        x_key: request.x_key.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
        series_by: request.series_by.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
        x_expression: request.x_expression.as_ref().map(data_expression_wire).transpose()?,
        y_expressions: request.y_expressions.iter().map(data_expression_wire).collect::<Result<_, _>>()?,
        series_by_expression: request.series_by_expression.as_ref().map(data_expression_wire).transpose()?,
        z_expression: request.z_expression.as_ref().map(data_expression_wire).transpose()?,
        fft: Some(plot_fft_wire(&request.fft)?),
        interval: Some(plot_interval_wire(&request.interval)?),
        histogram: Some(plot_histogram_wire(&request.histogram)?),
        z_stream: request.z_stream.as_ref().map(|value| wire_stream(value)).transpose()?,
        z_field: request.z_field.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
        unit: request.unit.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
        max_points: request
            .max_points
            .map(|value| u64::try_from(value).map_err(|_| Status::internal("plot max_points does not fit wire")))
            .transpose()?,
        window_s: request.window_s.map(|value| finite_duration(value, "plot window_s")).transpose()?,
        scope: request.scope.as_ref().map(canonical::scope).transpose()?,
        // A document owner definition is a fresh query selection. Cursors are
        // process-local and are added only when a caller opens a live stream.
        cursor: None,
        technique: enums::downsample_technique(&request.technique),
        summary_window: request
            .summary_window
            .map(|window| dokime_domain::model::plots::wire::PlotWindow { from: window.from, to: window.to }),
    })
}

pub(crate) fn plot_filter_wire(filter: &DocumentFilter) -> Result<dokime_domain::model::plots::wire::PlotFilter, Status> {
    Ok(dokime_domain::model::plots::wire::PlotFilter {
        expression: Some(data_expression_wire(&filter.expression)?),
        operator: enums::filter_operator(&filter.operator),
        value: filter.value.as_ref().map(|value| canonical::encode(value, &mut Budget::new(), 0)).transpose()?,
    })
}

pub(crate) fn data_expression_wire(expression: &DataExpression) -> Result<dokime_domain::model::plots::wire::DataExpression, Status> {
    value::collection(expression.transforms.len())?;
    let source = Some(match &expression.source {
        DataSource::Telemetry { stream, field } => {
            dokime_domain::model::plots::wire::DataExpressionSource::Telemetry(dokime_domain::model::plots::wire::TelemetryDataSource {
                stream: expression_stream(stream.clone())?,
                field: bounded_text(field.clone())?,
            })
        }
        DataSource::Parameter { name } => {
            dokime_domain::model::plots::wire::DataExpressionSource::Parameter(dokime_domain::model::plots::wire::ParameterDataSource {
                name: bounded_text(name.clone())?,
            })
        }
        DataSource::Dimension { name } => {
            dokime_domain::model::plots::wire::DataExpressionSource::Dimension(dokime_domain::model::plots::wire::DimensionDataSource {
                name: bounded_text(name.clone())?,
            })
        }
        DataSource::Record { record_type } => {
            dokime_domain::model::plots::wire::DataExpressionSource::Record(dokime_domain::model::plots::wire::RecordDataSource {
                record_type: bounded_text(record_type.clone())?,
            })
        }
        DataSource::Artifact { artifact } => {
            dokime_domain::model::plots::wire::DataExpressionSource::Artifact(dokime_domain::model::plots::wire::ArtifactDataSource {
                artifact: bounded_text(artifact.clone())?,
            })
        }
    });
    Ok(dokime_domain::model::plots::wire::DataExpression {
        label: expression.label.as_ref().map(|label| bounded_text(label.clone())).transpose()?,
        source,
        transforms: expression.transforms.iter().map(data_transform_wire).collect::<Result<_, _>>()?,
    })
}

fn data_transform_wire(transform: &DataTransform) -> Result<dokime_domain::model::plots::wire::DataTransform, Status> {
    use dokime_domain::model::plots::wire::DataTransformKind as Kind;
    let kind = match transform {
        DataTransform::Aggregate(aggregate) => {
            Kind::Aggregate(dokime_domain::model::plots::wire::AggregateTransform { aggregate: *aggregate })
        }
        DataTransform::TimeBetween { previous_record_type } => Kind::TimeBetween(dokime_domain::model::plots::wire::TimeBetweenTransform {
            previous_record_type: bounded_text(previous_record_type.clone())?,
        }),
        DataTransform::TelemetryAtOrBefore { stream, field } => {
            Kind::TelemetryAtOrBefore(dokime_domain::model::plots::wire::TelemetryAtOrBeforeTransform {
                stream: expression_stream(stream.clone())?,
                field: bounded_text(field.clone())?,
            })
        }
        DataTransform::Detail { name } => {
            Kind::Detail(dokime_domain::model::plots::wire::DetailTransform { name: bounded_text(name.clone())? })
        }
        DataTransform::PerCase => Kind::PerCase(dokime_domain::model::plots::wire::EmptyTransform {}),
    };
    Ok(dokime_domain::model::plots::wire::DataTransform { kind: Some(kind) })
}

fn plot_presentation(input: &PlotInput) -> Result<dokime_domain::model::documents::wire::PlotPresentation, Status> {
    let (x_axis, x_axis_unit) = enums::plot_axis(input.x_axis.as_ref());
    Ok(dokime_domain::model::documents::wire::PlotPresentation {
        style: enums::plot_style(&input.style),
        fft: Some(plot_fft_wire(&input.fft)?),
        interval: Some(plot_interval_wire(&input.interval)?),
        histogram: Some(plot_histogram_wire(&input.histogram)?),
        reference_lines: input.reference_lines.iter().map(plot_reference_line_wire).collect::<Result<_, _>>()?,
        reference_bands: input.reference_bands.iter().map(plot_reference_band_wire).collect::<Result<_, _>>()?,
        live_window_s: input.live_window_s.map(|value| finite_duration(value, "plot live_window_s")).transpose()?,
        x_format: optional_format(input.x_format.as_ref()),
        y_format: optional_format(input.y_format.as_ref()),
        x_axis,
        x_axis_unit: x_axis_unit.map(bounded_text).transpose()?,
    })
}

fn plot_fft_wire(value: &FftOptions) -> Result<dokime_domain::model::plots::wire::PlotFftOptions, Status> {
    Ok(dokime_domain::model::plots::wire::PlotFftOptions {
        sample_rate_hz: value.sample_rate_hz.map(|value| finite_nonnegative(value, "plot FFT sample_rate_hz")).transpose()?,
        window: enums::fft_window(&value.window),
        detrend: enums::fft_detrend(&value.detrend),
        level_unit: enums::frequency_level_unit(&value.level_unit),
        include_dc: value.include_dc,
        min_frequency_hz: value.min_frequency_hz.map(|value| finite_nonnegative(value, "plot FFT min_frequency_hz")).transpose()?,
        max_frequency_hz: value.max_frequency_hz.map(|value| finite_nonnegative(value, "plot FFT max_frequency_hz")).transpose()?,
    })
}

fn plot_interval_wire(value: &SigmaIntervalOptions) -> Result<dokime_domain::model::plots::wire::PlotIntervalOptions, Status> {
    Ok(dokime_domain::model::plots::wire::PlotIntervalOptions {
        bin_width: finite_nonnegative(value.bin_width, "plot interval bin_width")?,
        sigma: finite_nonnegative(value.sigma, "plot interval sigma")?,
        clip_to_observed: value.clip_to_observed,
    })
}

fn plot_histogram_wire(value: &HistogramOptions) -> Result<dokime_domain::model::plots::wire::PlotHistogramOptions, Status> {
    Ok(dokime_domain::model::plots::wire::PlotHistogramOptions {
        bins: u64::try_from(value.bins).map_err(|_| Status::internal("plot histogram bins do not fit wire"))?,
        unit: value.unit.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
    })
}

fn plot_reference_line_wire(value: &ReferenceLine) -> Result<dokime_domain::model::documents::wire::PlotReferenceLine, Status> {
    Ok(dokime_domain::model::documents::wire::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis),
        value: value.value.map(|value| finite_number(value, "plot reference line value")).transpose()?,
        source: value.source.as_ref().map(data_expression_wire).transpose()?,
        source_scale: finite_number(value.source_scale, "plot reference line source_scale")?,
        label: value.label.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
        color: bounded_text(value.color.clone())?,
    })
}

fn plot_reference_band_wire(value: &ReferenceBand) -> Result<dokime_domain::model::documents::wire::PlotReferenceBand, Status> {
    Ok(dokime_domain::model::documents::wire::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis),
        min: value.min.map(|value| finite_number(value, "plot reference band min")).transpose()?,
        max: value.max.map(|value| finite_number(value, "plot reference band max")).transpose()?,
        label: value.label.as_ref().map(|value| bounded_text(value.clone())).transpose()?,
        color: bounded_text(value.color.clone())?,
        center: value.center.as_ref().map(data_expression_wire).transpose()?,
        tolerance: value.tolerance.as_ref().map(data_expression_wire).transpose()?,
    })
}

fn wire_stream(value: &str) -> Result<String, Status> {
    let value = bounded_text(value.to_owned())?;
    Identifier::new(value.clone()).map_err(|error| Status::internal(format!("invalid authored plot stream: {error}")))?;
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

fn plot_stream_kind(value: PlotPayloadStreamKind) -> dokime_domain::model::plots::wire::PlotStreamKind {
    match value {
        PlotPayloadStreamKind::Snapshot => dokime_domain::model::plots::wire::PlotStreamKind::Snapshot,
        PlotPayloadStreamKind::Append => dokime_domain::model::plots::wire::PlotStreamKind::Append,
        PlotPayloadStreamKind::Replace => dokime_domain::model::plots::wire::PlotStreamKind::Replace,
        PlotPayloadStreamKind::Error => dokime_domain::model::plots::wire::PlotStreamKind::Error,
    }
}

fn plot_cursor_wire(value: PlotPayloadCursor, epoch: &str, selection: &str) -> dokime_domain::model::plots::wire::PlotCursor {
    dokime_domain::model::plots::wire::PlotCursor {
        telemetry_streams: value
            .telemetry_streams
            .into_iter()
            .map(|(key, value)| dokime_domain::model::plots::wire::PlotCursorTelemetryStreamsEntry { key, value })
            .collect(),
        record_sequence: value.record_sequence,
        epoch: epoch.to_owned(),
        selection: selection.to_owned(),
    }
}

fn plot_payload(value: dokime_runtime::runtime::query::PlotPayload) -> Result<dokime_domain::model::plots::wire::PlotPayload, Status> {
    let maximum = super::cursor::MAX_PLOT_POINTS as usize;
    if value.series.iter().any(|series| series.points.len() > maximum)
        || value.cartesian.as_ref().is_some_and(|frame| frame.series.iter().any(|series| series.x.len().max(series.y.len()) > maximum))
        || value.computed.as_ref().is_some_and(|computed| {
            computed
                .series
                .iter()
                .any(|series| series.output.outputs.iter().any(|output| output.axis.len().max(output.values.len()) > maximum))
        })
    {
        return Err(Status::resource_exhausted("plot exceeds retained point limit"));
    }
    let context = values(&value.context)?;
    let cartesian = value.cartesian.map(plot_cartesian).transpose()?;
    Ok(dokime_domain::model::plots::wire::PlotPayload {
        computed: value.computed.map(super::analysis::computation).transpose()?,
        stream: value.stream,
        x_key: value.x_key,
        series_by: value.series_by,
        context: Some(context),
        series: value.series.into_iter().map(plot_series).collect::<Result<_, _>>()?,
        cartesian,
        markers: value.markers.into_iter().map(plot_marker).collect::<Result<_, _>>()?,
        max_points: u64::try_from(value.max_points).map_err(|_| Status::internal("plot max_points does not fit wire"))?,
        window_s: value.window_s,
        generated_at: value.generated_at,
        x_format: optional_format(value.x_format.as_ref()),
        run_started_at: value.run_started_at,
        cycle_started_at: value.cycle_started_at,
        analysis: value.analysis.map(plot_analysis).transpose()?,
        summaries: value.summaries.into_iter().map(plot_series_summary).collect::<Result<_, _>>()?,
    })
}

fn plot_series_summary(
    value: dokime_runtime::runtime::query::PlotSeriesSummary,
) -> Result<dokime_domain::model::plots::wire::PlotSeriesSummary, Status> {
    let number = |value: Option<f64>| value.map(|value| finite_number(value, "plot series summary")).transpose();
    Ok(dokime_domain::model::plots::wire::PlotSeriesSummary {
        series_id: bounded_text(value.series_id)?,
        count: u64::try_from(value.count).map_err(|_| Status::internal("plot summary count does not fit wire"))?,
        min: number(value.min)?,
        max: number(value.max)?,
        mean: number(value.mean)?,
        median: number(value.median)?,
        std: number(value.std)?,
        rms: number(value.rms)?,
        window: value
            .window
            .map(|window| {
                Ok::<_, Status>(dokime_domain::model::plots::wire::PlotWindow {
                    from: finite_number(window.from, "plot summary window")?,
                    to: finite_number(window.to, "plot summary window")?,
                })
            })
            .transpose()?,
    })
}

fn plot_analysis(value: dokime_runtime::runtime::query::PlotAnalysis) -> Result<dokime_domain::model::plots::wire::PlotAnalysis, Status> {
    use dokime_domain::model::plots::wire::PlotAnalysisResult as Analysis;
    use dokime_runtime::runtime::query::PlotAnalysis;

    let count = |value| u64::try_from(value).map_err(|_| Status::internal("analysis count does not fit wire"));
    let number = |value| finite_number(value, "plot analysis number");
    let result = match value {
        PlotAnalysis::Error { message } => {
            Analysis::Error(dokime_domain::model::plots::wire::PlotAnalysisError { message: bounded_text(message)? })
        }
        PlotAnalysis::Histogram { series } => Analysis::Histogram(dokime_domain::model::plots::wire::PlotHistogramAnalysis {
            series: series
                .into_iter()
                .map(|series| {
                    Ok(dokime_domain::model::plots::wire::PlotHistogramSeries {
                        key: bounded_text(series.key)?,
                        count: count(series.count)?,
                        statistics: series
                            .statistics
                            .map(|statistics| {
                                Ok::<_, Status>(dokime_domain::model::plots::wire::PlotHistogramStatistics {
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
                                Ok(dokime_domain::model::plots::wire::PlotHistogramBin {
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
        PlotAnalysis::BoxSummary { series, total } => Analysis::BoxSummary(dokime_domain::model::plots::wire::PlotBoxSummaryAnalysis {
            total: count(total)?,
            series: series
                .into_iter()
                .map(|series| {
                    Ok(dokime_domain::model::plots::wire::PlotBoxSummary {
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
                        outliers: series.outliers.into_iter().map(number).collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, Status>>()?,
        }),
        PlotAnalysis::SigmaInterval { series } => Analysis::SigmaInterval(dokime_domain::model::plots::wire::PlotSigmaIntervalAnalysis {
            series: series
                .into_iter()
                .map(|series| {
                    Ok(dokime_domain::model::plots::wire::PlotSigmaSeries {
                        key: bounded_text(series.key)?,
                        raw_points: count(series.raw_points)?,
                        bins: series
                            .bins
                            .into_iter()
                            .map(|bin| {
                                Ok(dokime_domain::model::plots::wire::PlotSigmaBin {
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
        }),
        PlotAnalysis::WifiOccupancy { channels, occupied_channels, bands, scans } => {
            Analysis::WifiOccupancy(wifi_occupancy(channels, occupied_channels, bands, scans)?)
        }
    };

    Ok(dokime_domain::model::plots::wire::PlotAnalysis { result: Some(result) })
}

fn wifi_occupancy(
    channels: Vec<i64>, occupied_channels: Vec<i64>, bands: Vec<dokime_runtime::runtime::query::WifiBandWindow>,
    scans: Vec<dokime_runtime::runtime::query::WifiScan>,
) -> Result<dokime_domain::model::plots::wire::PlotWifiOccupancyAnalysis, Status> {
    use dokime_domain::model::plots::wire::PlotWifiBandScan;
    use dokime_domain::model::plots::wire::PlotWifiBandWindow;
    use dokime_domain::model::plots::wire::PlotWifiChannelCount;
    use dokime_domain::model::plots::wire::PlotWifiOccupancyAnalysis;
    use dokime_domain::model::plots::wire::PlotWifiScan;

    let count = |value| u64::try_from(value).map_err(|_| Status::internal("wifi occupancy count does not fit wire"));
    Ok(PlotWifiOccupancyAnalysis {
        channels,
        occupied_channels,
        bands: bands
            .into_iter()
            .map(|band| Ok(PlotWifiBandWindow { band: bounded_text(band.band)?, occupied_channels: count(band.occupied_channels)? }))
            .collect::<Result<_, Status>>()?,
        scans: scans
            .into_iter()
            .map(|scan| {
                Ok(PlotWifiScan {
                    time: bounded_text(scan.time)?,
                    channels: scan
                        .channels
                        .into_iter()
                        .map(|channel| Ok(PlotWifiChannelCount { channel: channel.channel, count: count(channel.count)? }))
                        .collect::<Result<_, Status>>()?,
                    bands: scan
                        .bands
                        .into_iter()
                        .map(|band| {
                            Ok(PlotWifiBandScan {
                                band: bounded_text(band.band)?,
                                occupied_channels: count(band.occupied_channels)?,
                                total_networks: count(band.total_networks)?,
                                peak_count: count(band.peak_count)?,
                            })
                        })
                        .collect::<Result<_, Status>>()?,
                    occupied_channels: count(scan.occupied_channels)?,
                    total_networks: count(scan.total_networks)?,
                    peak_count: count(scan.peak_count)?,
                })
            })
            .collect::<Result<_, Status>>()?,
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
                let value = value.as_f64().ok_or_else(|| Status::internal("plot point number is not representable"))?;
                if !value.is_finite() {
                    return Err(Status::internal("plot point number is not finite"));
                }
                Value::Number(value)
            }
        }
        JsonValue::String(value) => Value::Text(value),
        JsonValue::Array(values) => Value::Array(values.into_iter().map(json_value).collect::<Result<_, _>>()?),
        JsonValue::Object(values) => {
            Value::Object(values.into_iter().map(|(key, value)| Ok((key, json_value(value)?))).collect::<Result<_, Status>>()?)
        }
    })
}

fn plot_cartesian(
    value: dokime_runtime::runtime::query::PlotCartesianFrame,
) -> Result<dokime_domain::model::plots::wire::PlotCartesianFrame, Status> {
    Ok(dokime_domain::model::plots::wire::PlotCartesianFrame {
        mode: match value.mode {
            dokime_runtime::runtime::query::PlotCartesianMode::Aligned => dokime_domain::model::plots::wire::PlotCartesianMode::Aligned,
            dokime_runtime::runtime::query::PlotCartesianMode::Faceted => dokime_domain::model::plots::wire::PlotCartesianMode::Faceted,
        },
        series: value
            .series
            .into_iter()
            .map(|series| dokime_domain::model::plots::wire::PlotCartesianSeriesFrame {
                x: series.x,
                y: series.y.into_iter().map(|value| dokime_domain::model::plots::wire::NullableNumber { value }).collect(),
            })
            .collect(),
        x_domain: value.x_domain.map(plot_domain),
        y_domain: value.y_domain.map(plot_domain),
        point_count: u64::try_from(value.point_count).map_err(|_| Status::internal("plot point count does not fit wire"))?,
        signature: value.signature,
    })
}

fn plot_domain(value: dokime_runtime::runtime::query::PlotDomain) -> dokime_domain::model::plots::wire::PlotDomain {
    dokime_domain::model::plots::wire::PlotDomain { min: value.min, max: value.max }
}

fn plot_diagnostics(
    value: dokime_runtime::runtime::query::PlotPayloadDiagnostics,
) -> Result<dokime_domain::model::plots::wire::PlotPayloadDiagnostics, Status> {
    Ok(dokime_domain::model::plots::wire::PlotPayloadDiagnostics {
        series_count: u64::try_from(value.series_count).map_err(|_| Status::internal("plot series count does not fit wire"))?,
        point_count: u64::try_from(value.point_count).map_err(|_| Status::internal("plot point count does not fit wire"))?,
        raw_point_count: u64::try_from(value.raw_point_count).map_err(|_| Status::internal("plot raw point count does not fit wire"))?,
        downsampled: value.downsampled,
        delta: value.delta,
        max_points: u64::try_from(value.max_points).map_err(|_| Status::internal("plot max_points does not fit wire"))?,
        window_s: value.window_s,
    })
}

fn plot_series(value: dokime_runtime::runtime::query::PlotSeries) -> Result<dokime_domain::model::plots::wire::PlotSeries, Status> {
    Ok(dokime_domain::model::plots::wire::PlotSeries {
        key: value.key,
        label: value.label,
        unit: value.unit,
        points: value.points.into_iter().map(plot_point).collect::<Result<_, _>>()?,
        raw_points: u64::try_from(value.raw_points).map_err(|_| Status::internal("plot raw point count does not fit wire"))?,
        downsampled: value.downsampled,
        format: optional_format(value.format.as_ref()),
    })
}

fn plot_point(value: dokime_runtime::runtime::query::PlotPoint) -> Result<dokime_domain::model::plots::wire::PlotPoint, Status> {
    Ok(dokime_domain::model::plots::wire::PlotPoint {
        x: Some(canonical::encode(&json_value(value.x)?, &mut Budget::new(), 0)?),
        y: value.y,
        z: value.z,
    })
}

fn plot_marker(value: dokime_runtime::runtime::query::PlotMarker) -> Result<dokime_domain::model::plots::wire::PlotMarker, Status> {
    Ok(dokime_domain::model::plots::wire::PlotMarker {
        x: Some(canonical::encode(&json_value(value.x)?, &mut Budget::new(), 0)?),
        label: value.label,
        record_type: value.record_type,
        color: value.color,
    })
}

/* === Tests === */
