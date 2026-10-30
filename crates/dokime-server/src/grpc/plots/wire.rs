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
use dokime::harmos::domain::Portable;
use tonic::Status;

// Local Imports
use super::super::enums::canonical as enums;
use super::super::value::{Budget, canonical};
use super::cursor::expression_stream;
use crate::grpc::transcode::Transcode;
use dokime::document::DocumentFilter;
use dokime::document::FftOptions;
use dokime::document::HistogramOptions;
use dokime::document::PlotDescriptor;
use dokime::document::PlotInput;
use dokime::document::ReferenceBand;
use dokime::document::ReferenceLine;
use dokime::document::SigmaIntervalOptions;
use dokime::domain::model::identity::Identifier;
use dokime::domain::model::values::Value;
use dokime::expressions::DataExpression;
use dokime::expressions::DataSource;
use dokime::expressions::DataTransform;
use dokime::runtime::query::PlotPayloadEnvelope;
use dokime::runtime::query::PlotPayloadRequest;
use dokime::runtime::query::PlotPayloadStreamKind;
use dokime::runtime::query::declared_plot_request;

/* === Constants === */

const SERVER_PLOT_POINTS: u64 = 100_000;

const GENERATED_AT_BYTES: usize = "0000-00-00T00:00:00.000000000Z".len();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

// Why a plot frame was not built: a transport failure, or a point budget the
// caller can retry under.
pub(super) enum FrameError {
    Status(Status),
    TooManyPoints { maximum: usize },
}

/* === Traits === */

/* === Implementations === */

impl From<Status> for FrameError {
    fn from(status: Status) -> Self {
        Self::Status(status)
    }
}

impl From<dokime::harmos::Error> for FrameError {
    fn from(error: dokime::harmos::Error) -> Self {
        let status = match error {
            dokime::harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted(error.to_string()),
            error => Status::internal(error.to_string()),
        };
        Self::Status(status)
    }
}

impl From<dokime::harmos::storage::Error> for FrameError {
    fn from(error: dokime::harmos::storage::Error) -> Self {
        Self::from(dokime::harmos::Error::from(error))
    }
}

/* === Public Functions === */

pub(crate) fn document_plot_definition(
    descriptor: &PlotDescriptor,
) -> Result<dokime::domain::model::documents::wire::DocumentPlotDefinition, Status> {
    // The declared request, not the mapped query: the live watch sends it back
    // and the query maps it once, exactly as the report maps the descriptor.
    let request = declared_plot_request(descriptor, Some(descriptor.input.scope.clone()), None, descriptor.input.live_window_s);
    Ok(dokime::domain::model::documents::wire::DocumentPlotDefinition {
        metadata: Some(canonical::metadata(&descriptor.metadata)?),
        request: Some(plot_request_wire(&request)?),
        presentation: Some(plot_presentation(&descriptor.input)?),
        z_expression: descriptor.input.z_expression.as_ref().map(data_expression_wire).transpose()?,
    })
}

pub(super) fn envelope(
    value: dokime::harmos::storage::ReadBatch<PlotPayloadEnvelope>, epoch: &str, selection: &str, limit: usize,
) -> Result<dokime::harmos::storage::ReadBatch<dokime::domain::model::plots::wire::PlotEnvelope>, FrameError> {
    stream_frame(PlotPayloadStreamKind::Snapshot, value, epoch, selection, limit)?
        .try_map(|frame| Ok(frame.envelope.expect("constructed plot frame has envelope")))
}

pub(super) fn stream_frame(
    kind: PlotPayloadStreamKind, value: dokime::harmos::storage::ReadBatch<PlotPayloadEnvelope>, epoch: &str, selection: &str, limit: usize,
) -> Result<dokime::harmos::storage::ReadBatch<dokime::domain::model::plots::wire::PlotStreamFrame>, FrameError> {
    value.try_map(|value| {
        let envelope = plot_envelope(value, epoch, selection)?;
        let mut frame = dokime::domain::model::plots::wire::PlotStreamFrame {
            kind: plot_stream_kind(kind),
            envelope: Some(envelope),
        };
        fit_message(&mut frame, |value| value.envelope.as_mut().expect("constructed plot frame has envelope"), limit, true)?;
        Ok(frame)
    })
}

pub(crate) fn values(values: &BTreeMap<String, Value>) -> Result<dokime::domain::model::values::wire::Value, Status> {
    canonical::object(values, &mut Budget::new()?)
}

pub(super) fn ensure_message_budget<M: Portable>(value: &M, limit: usize, label: &str) -> Result<(), Status> {
    if value.wire_len().map_err(Status::from)? > limit {
        return Err(Status::resource_exhausted(format!("{label} exceeds transport limit")));
    }
    Ok(())
}

pub(crate) fn plot_filter_wire(filter: &DocumentFilter) -> Result<dokime::domain::model::plots::wire::PlotFilter, Status> {
    Ok(dokime::domain::model::plots::wire::PlotFilter {
        expression: Some(data_expression_wire(&filter.expression)?),
        operator: enums::filter_operator(&filter.operator),
        value: filter.value.as_ref().map(|value| canonical::encode(value, &mut Budget::new()?, 0)).transpose()?,
    })
}

pub(crate) fn data_expression_wire(expression: &DataExpression) -> Result<dokime::domain::model::plots::wire::DataExpression, Status> {
    let source =
        Some(match &expression.source {
            DataSource::Telemetry { stream, field } => dokime::domain::model::plots::wire::DataExpressionSource::Telemetry(
                dokime::domain::model::plots::wire::TelemetryDataSource {
                    stream: expression_stream(stream.clone())?,
                    field: field.clone(),
                },
            ),
            DataSource::Parameter { name } => dokime::domain::model::plots::wire::DataExpressionSource::Parameter(
                dokime::domain::model::plots::wire::ParameterDataSource { name: name.clone() },
            ),
            DataSource::Dimension { name } => dokime::domain::model::plots::wire::DataExpressionSource::Dimension(
                dokime::domain::model::plots::wire::DimensionDataSource { name: name.clone() },
            ),
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
        });
    Ok(dokime::domain::model::plots::wire::DataExpression {
        label: expression.label.clone(),
        source,
        transforms: expression.transforms.iter().map(data_transform_wire).collect::<Result<_, _>>()?,
    })
}

/* === Private Functions === */

fn plot_envelope(
    value: PlotPayloadEnvelope, epoch: &str, selection: &str,
) -> Result<dokime::domain::model::plots::wire::PlotEnvelope, Status> {
    Ok(dokime::domain::model::plots::wire::PlotEnvelope {
        plot_id: value.plot_id,
        revision: value.revision,
        epoch: epoch.to_owned(),
        selection: selection.to_owned(),
        payload: Some(plot_payload(value.payload)?),
        diagnostics: Some(plot_diagnostics(value.diagnostics)?),
        warnings: value.warnings,
    })
}

fn optional_format(
    value: Option<&dokime::domain::model::values::ValueFormat>,
) -> Option<dokime::domain::model::definition::wire::ValueFormat> {
    value.map(canonical::value_format)
}

fn fit_message<T: Portable>(
    message: &mut T, select: fn(&mut T) -> &mut dokime::domain::model::plots::wire::PlotEnvelope, limit: usize, streamed: bool,
) -> Result<(), FrameError> {
    let encoded_budget = limit;
    let size = |message: &T| -> Result<usize, Status> {
        if streamed { crate::grpc::content::output_frame_len(message) } else { message.wire_len().map_err(Status::from) }
    };
    let encoded = size(message)?;
    if encoded <= encoded_budget {
        return Ok(());
    }
    let exhausted = || Status::resource_exhausted("plot response exceeds transport limit");
    let envelope = select(message);
    let Some(payload) = &envelope.payload else {
        return Err(exhausted().into());
    };
    let count = payload.series.iter().try_fold(0usize, |count, series| count.checked_add(series.points.len())).ok_or_else(exhausted)?;
    let series = payload.series.len().max(1);
    // Points and the cartesian frame aligned with them grow together; the
    // rest of the envelope is the same whatever the points. Its generation
    // time is charged at its longest, so every delivery of this plot names
    // the same bound however its trailing zeros fall.
    // A Failing Projection Is Never Delivered; Release Its Points In Place.
    if let Some(payload) = &mut envelope.payload {
        payload.cartesian = None;
        for series in &mut payload.series {
            series.points = Vec::new();
        }
        let generated = payload.generated_at.len().max(GENERATED_AT_BYTES);
        payload.generated_at = "0".repeat(generated);
    }
    let requested = envelope.payload.as_ref().expect("failing plot has payload").max_points;
    let fits = |total: usize, fixed: usize, budget: usize| {
        (count > 0 && fixed < budget).then(|| {
            let per_point = total.saturating_sub(fixed).div_ceil(count).max(1);
            (budget - fixed) / per_point / series
        })
    };
    let Some(maximum) = fits(encoded, size(message)?, encoded_budget) else {
        return Err(exhausted().into());
    };
    if maximum == 0 || (requested != 0 && maximum as u64 >= requested) {
        return Err(exhausted().into());
    }
    Err(FrameError::TooManyPoints { maximum })
}

fn plot_request_wire(request: &PlotPayloadRequest) -> Result<dokime::domain::model::plots::wire::PlotRequest, Status> {
    let stream = wire_stream(&request.stream)?;
    let y_keys = request.y_keys.to_vec();
    let marker_record_types = request.marker_record_types.to_vec();
    Ok(dokime::domain::model::plots::wire::PlotRequest {
        analysis: request.analysis.as_ref().map(super::analysis::selection_wire).transpose()?,
        plot_id: request.plot_id.clone(),
        plot_type: request.plot_type.as_ref().map_or(dokime::domain::model::plots::wire::PlotType::Unspecified, enums::plot_type),
        x_scale: enums::axis_scale(&request.x_scale),
        y_scale: enums::axis_scale(&request.y_scale),
        stream,
        y_keys,
        marker_record_types,
        filters: request.filters.iter().map(plot_filter_wire).collect::<Result<_, _>>()?,
        x_key: request.x_key.clone(),
        series_by: request.series_by.clone(),
        x_expression: request.x_expression.as_ref().map(data_expression_wire).transpose()?,
        y_expressions: request.y_expressions.iter().map(data_expression_wire).collect::<Result<_, _>>()?,
        series_by_expression: request.series_by_expression.as_ref().map(data_expression_wire).transpose()?,
        z_expression: request.z_expression.as_ref().map(data_expression_wire).transpose()?,
        fft: Some(plot_fft_wire(&request.fft)?),
        interval: Some(plot_interval_wire(&request.interval)?),
        histogram: Some(plot_histogram_wire(&request.histogram)?),
        z_stream: request.z_stream.as_deref().map(wire_stream).transpose()?,
        z_field: request.z_field.clone(),
        unit: request.unit.clone(),
        max_points: request
            .max_points
            .map(|value| u64::try_from(value).map_err(|_| Status::internal("plot max_points does not fit wire")))
            .transpose()?,
        window_s: request.window_s.map(|value| finite_duration(value, "plot window_s")).transpose()?,
        scope: request.scope.as_ref().map(canonical::scope).transpose()?,
        technique: enums::downsample_technique(&request.technique),
        summary_window: request
            .summary_window
            .map(|window| dokime::domain::model::plots::wire::PlotWindow { from: window.from, to: window.to }),
    })
}

fn data_transform_wire(transform: &DataTransform) -> Result<dokime::domain::model::plots::wire::DataTransform, Status> {
    use dokime::domain::model::plots::wire::DataTransformKind as Kind;
    let kind = match transform {
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
                stream: expression_stream(stream.clone())?,
                field: field.clone(),
            })
        }
        DataTransform::Detail { name } => Kind::Detail(dokime::domain::model::plots::wire::DetailTransform { name: name.clone() }),
        DataTransform::PerCase => Kind::PerCase(dokime::domain::model::plots::wire::EmptyTransform {}),
    };
    Ok(dokime::domain::model::plots::wire::DataTransform { kind: Some(kind) })
}

fn plot_presentation(input: &PlotInput) -> Result<dokime::domain::model::documents::wire::PlotPresentation, Status> {
    let (x_axis, x_axis_unit) = enums::plot_axis(input.x_axis.as_ref());
    Ok(dokime::domain::model::documents::wire::PlotPresentation {
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
        x_axis_unit,
    })
}

fn plot_fft_wire(value: &FftOptions) -> Result<dokime::domain::model::plots::wire::PlotFftOptions, Status> {
    Ok(dokime::domain::model::plots::wire::PlotFftOptions {
        sample_rate_hz: value.sample_rate_hz.map(|value| finite_nonnegative(value, "plot FFT sample_rate_hz")).transpose()?,
        window: enums::fft_window(&value.window),
        detrend: enums::fft_detrend(&value.detrend),
        level_unit: enums::frequency_level_unit(&value.level_unit),
        include_dc: value.include_dc,
        min_frequency_hz: value.min_frequency_hz.map(|value| finite_nonnegative(value, "plot FFT min_frequency_hz")).transpose()?,
        max_frequency_hz: value.max_frequency_hz.map(|value| finite_nonnegative(value, "plot FFT max_frequency_hz")).transpose()?,
    })
}

fn plot_interval_wire(value: &SigmaIntervalOptions) -> Result<dokime::domain::model::plots::wire::PlotIntervalOptions, Status> {
    Ok(dokime::domain::model::plots::wire::PlotIntervalOptions {
        bin_width: finite_nonnegative(value.bin_width, "plot interval bin_width")?,
        sigma: finite_nonnegative(value.sigma, "plot interval sigma")?,
        clip_to_observed: value.clip_to_observed,
    })
}

fn plot_histogram_wire(value: &HistogramOptions) -> Result<dokime::domain::model::plots::wire::PlotHistogramOptions, Status> {
    Ok(dokime::domain::model::plots::wire::PlotHistogramOptions {
        bins: u64::try_from(value.bins).map_err(|_| Status::internal("plot histogram bins do not fit wire"))?,
        unit: value.unit.clone(),
    })
}

fn plot_reference_line_wire(value: &ReferenceLine) -> Result<dokime::domain::model::documents::wire::PlotReferenceLine, Status> {
    Ok(dokime::domain::model::documents::wire::PlotReferenceLine {
        axis: enums::reference_axis(&value.axis),
        value: value.value.map(|value| finite_number(value, "plot reference line value")).transpose()?,
        source: value.source.as_ref().map(data_expression_wire).transpose()?,
        source_scale: finite_number(value.source_scale, "plot reference line source_scale")?,
        label: value.label.clone(),
        color: value.color.clone(),
    })
}

fn plot_reference_band_wire(value: &ReferenceBand) -> Result<dokime::domain::model::documents::wire::PlotReferenceBand, Status> {
    Ok(dokime::domain::model::documents::wire::PlotReferenceBand {
        axis: enums::reference_axis(&value.axis),
        min: value.min.map(|value| finite_number(value, "plot reference band min")).transpose()?,
        max: value.max.map(|value| finite_number(value, "plot reference band max")).transpose()?,
        label: value.label.clone(),
        color: value.color.clone(),
        center: value.center.as_ref().map(data_expression_wire).transpose()?,
        tolerance: value.tolerance.as_ref().map(data_expression_wire).transpose()?,
    })
}

fn wire_stream(value: &str) -> Result<String, Status> {
    let value = value.to_owned();
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

fn plot_stream_kind(value: PlotPayloadStreamKind) -> dokime::domain::model::plots::wire::PlotStreamKind {
    match value {
        PlotPayloadStreamKind::Snapshot => dokime::domain::model::plots::wire::PlotStreamKind::Snapshot,
        PlotPayloadStreamKind::Replace => dokime::domain::model::plots::wire::PlotStreamKind::Replace,
        PlotPayloadStreamKind::Error => dokime::domain::model::plots::wire::PlotStreamKind::Error,
    }
}

fn mapped<S, T>(values: Vec<S>, mut map: impl FnMut(S) -> Result<T, Status>) -> Result<Vec<T>, Status> {
    let mut output = canonical::slots(values.len())?;
    for value in values {
        output.push(map(value)?);
    }
    Ok(output)
}

fn plot_payload(value: dokime::runtime::query::PlotPayload) -> Result<dokime::domain::model::plots::wire::PlotPayload, Status> {
    let maximum = SERVER_PLOT_POINTS as usize;
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
    Ok(dokime::domain::model::plots::wire::PlotPayload {
        computed: value.computed.map(super::analysis::computation).transpose()?,
        stream: value.stream,
        x_key: value.x_key,
        series_by: value.series_by,
        context: Some(context),
        series: mapped(value.series, plot_series)?,
        cartesian,
        markers: mapped(value.markers, plot_marker)?,
        max_points: u64::try_from(value.max_points).map_err(|_| Status::internal("plot max_points does not fit wire"))?,
        window_s: value.window_s,
        generated_at: value.generated_at,
        x_format: optional_format(value.x_format.as_ref()),
        run_started_at: value.run_started_at,
        cycle_started_at: value.cycle_started_at,
        analysis: value.analysis.map(plot_analysis).transpose()?,
        summaries: mapped(value.summaries, plot_series_summary)?,
    })
}

fn plot_series_summary(
    value: dokime::runtime::query::PlotSeriesSummary,
) -> Result<dokime::domain::model::plots::wire::PlotSeriesSummary, Status> {
    let number = |value: Option<f64>| value.map(|value| finite_number(value, "plot series summary")).transpose();
    Ok(dokime::domain::model::plots::wire::PlotSeriesSummary {
        series_id: value.series_id,
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
                Ok::<_, Status>(dokime::domain::model::plots::wire::PlotWindow {
                    from: finite_number(window.from, "plot summary window")?,
                    to: finite_number(window.to, "plot summary window")?,
                })
            })
            .transpose()?,
    })
}

fn plot_analysis(value: dokime::runtime::query::PlotAnalysis) -> Result<dokime::domain::model::plots::wire::PlotAnalysis, Status> {
    use dokime::domain::model::plots::wire;
    use dokime::runtime::query::PlotAnalysis;
    use wire::PlotAnalysisResult as Analysis;
    let count = |value| u64::try_from(value).map_err(|_| Status::internal("analysis count does not fit wire"));
    let number = |value| finite_number(value, "plot analysis number");
    let result = match value {
        PlotAnalysis::Error { message } => Analysis::Error(wire::PlotAnalysisError { message }),
        PlotAnalysis::Histogram { series } => Analysis::Histogram(wire::PlotHistogramAnalysis {
            series: mapped(series, |series| {
                Ok(wire::PlotHistogramSeries {
                    key: series.key,
                    count: count(series.count)?,
                    statistics: series
                        .statistics
                        .map(|statistics| {
                            Ok::<_, Status>(wire::PlotHistogramStatistics {
                                mean: number(statistics.mean)?,
                                median: number(statistics.median)?,
                                std: number(statistics.std)?,
                                rms: number(statistics.rms)?,
                                min: number(statistics.min)?,
                                max: number(statistics.max)?,
                            })
                        })
                        .transpose()?,
                    bins: mapped(series.bins, |bin| {
                        Ok(wire::PlotHistogramBin {
                            lower: number(bin.lower)?,
                            upper: number(bin.upper)?,
                            count: count(bin.count)?,
                        })
                    })?,
                })
            })?,
        }),
        PlotAnalysis::BoxSummary { series, total } => Analysis::BoxSummary(wire::PlotBoxSummaryAnalysis {
            total: count(total)?,
            series: mapped(series, |series| {
                Ok(wire::PlotBoxSummary {
                    key: series.key,
                    label: series.label,
                    count: count(series.count)?,
                    min: number(series.min)?,
                    q1: number(series.q1)?,
                    median: number(series.median)?,
                    q3: number(series.q3)?,
                    max: number(series.max)?,
                    lower_whisker: number(series.lower_whisker)?,
                    upper_whisker: number(series.upper_whisker)?,
                    outliers: mapped(series.outliers, number)?,
                })
            })?,
        }),
        PlotAnalysis::SigmaInterval { series } => Analysis::SigmaInterval(wire::PlotSigmaIntervalAnalysis {
            series: mapped(series, |series| {
                Ok(wire::PlotSigmaSeries {
                    key: series.key,
                    raw_points: count(series.raw_points)?,
                    bins: mapped(series.bins, |bin| {
                        Ok(wire::PlotSigmaBin {
                            x: number(bin.x)?,
                            count: count(bin.count)?,
                            mean: number(bin.mean)?,
                            lower: number(bin.lower)?,
                            upper: number(bin.upper)?,
                        })
                    })?,
                })
            })?,
        }),
        PlotAnalysis::WifiOccupancy {
            channels,
            occupied_channels,
            bands,
            scans,
        } => Analysis::WifiOccupancy(wifi_occupancy(channels, occupied_channels, bands, scans)?),
    };
    Ok(wire::PlotAnalysis { result: Some(result) })
}

fn wifi_occupancy(
    channels: Vec<i64>, occupied_channels: Vec<i64>, bands: Vec<dokime::runtime::query::WifiBandWindow>,
    scans: Vec<dokime::runtime::query::WifiScan>,
) -> Result<dokime::domain::model::plots::wire::PlotWifiOccupancyAnalysis, Status> {
    use dokime::domain::model::plots::wire;
    let count = |value| u64::try_from(value).map_err(|_| Status::internal("wifi occupancy count does not fit wire"));
    Ok(wire::PlotWifiOccupancyAnalysis {
        channels,
        occupied_channels,
        bands: mapped(bands, |band| {
            Ok(wire::PlotWifiBandWindow {
                band: band.band,
                occupied_channels: count(band.occupied_channels)?,
            })
        })?,
        scans: mapped(scans, |scan| {
            Ok(wire::PlotWifiScan {
                time: scan.time,
                channels: mapped(scan.channels, |channel| {
                    Ok(wire::PlotWifiChannelCount {
                        channel: channel.channel,
                        count: count(channel.count)?,
                    })
                })?,
                bands: mapped(scan.bands, |band| {
                    Ok(wire::PlotWifiBandScan {
                        band: band.band,
                        occupied_channels: count(band.occupied_channels)?,
                        total_networks: count(band.total_networks)?,
                        peak_count: count(band.peak_count)?,
                    })
                })?,
                occupied_channels: count(scan.occupied_channels)?,
                total_networks: count(scan.total_networks)?,
                peak_count: count(scan.peak_count)?,
            })
        })?,
    })
}

fn plot_cartesian(
    value: dokime::runtime::query::PlotCartesianFrame,
) -> Result<dokime::domain::model::plots::wire::PlotCartesianFrame, Status> {
    Ok(dokime::domain::model::plots::wire::PlotCartesianFrame {
        mode: match value.mode {
            dokime::runtime::query::PlotCartesianMode::Aligned => dokime::domain::model::plots::wire::PlotCartesianMode::Aligned,
            dokime::runtime::query::PlotCartesianMode::Faceted => dokime::domain::model::plots::wire::PlotCartesianMode::Faceted,
        },
        series: mapped(value.series, |series| {
            Ok(dokime::domain::model::plots::wire::PlotCartesianSeriesFrame {
                x: series.x,
                y: mapped(series.y, |value| Ok(dokime::domain::model::plots::wire::NullableNumber { value }))?,
            })
        })?,
        x_domain: value.x_domain.map(plot_domain),
        y_domain: value.y_domain.map(plot_domain),
        point_count: u64::try_from(value.point_count).map_err(|_| Status::internal("plot point count does not fit wire"))?,
        signature: value.signature,
    })
}

fn plot_domain(value: dokime::runtime::query::PlotDomain) -> dokime::domain::model::plots::wire::PlotDomain {
    dokime::domain::model::plots::wire::PlotDomain { min: value.min, max: value.max }
}

fn plot_diagnostics(
    value: dokime::runtime::query::PlotPayloadDiagnostics,
) -> Result<dokime::domain::model::plots::wire::PlotPayloadDiagnostics, Status> {
    Ok(dokime::domain::model::plots::wire::PlotPayloadDiagnostics {
        series_count: u64::try_from(value.series_count).map_err(|_| Status::internal("plot series count does not fit wire"))?,
        point_count: u64::try_from(value.point_count).map_err(|_| Status::internal("plot point count does not fit wire"))?,
        raw_point_count: u64::try_from(value.raw_point_count).map_err(|_| Status::internal("plot raw point count does not fit wire"))?,
        downsampled: value.downsampled,
        max_points: u64::try_from(value.max_points).map_err(|_| Status::internal("plot max_points does not fit wire"))?,
        window_s: value.window_s,
    })
}

fn plot_series(value: dokime::runtime::query::PlotSeries) -> Result<dokime::domain::model::plots::wire::PlotSeries, Status> {
    Ok(dokime::domain::model::plots::wire::PlotSeries {
        key: value.key,
        label: value.label,
        unit: value.unit,
        points: mapped(value.points, plot_point)?,
        raw_points: u64::try_from(value.raw_points).map_err(|_| Status::internal("plot raw point count does not fit wire"))?,
        downsampled: value.downsampled,
        format: optional_format(value.format.as_ref()),
    })
}

fn plot_point(value: dokime::runtime::query::PlotPoint) -> Result<dokime::domain::model::plots::wire::PlotPoint, Status> {
    Ok(dokime::domain::model::plots::wire::PlotPoint {
        x: Some(canonical::encode_json(value.x, &mut Budget::new()?, 0)?),
        y: value.y,
        z: value.z,
    })
}

fn plot_marker(value: dokime::runtime::query::PlotMarker) -> Result<dokime::domain::model::plots::wire::PlotMarker, Status> {
    Ok(dokime::domain::model::plots::wire::PlotMarker {
        x: Some(canonical::encode_json(value.x, &mut Budget::new()?, 0)?),
        label: value.label,
        record_type: value.record_type,
        color: value.color,
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::domain::model::plots::wire::{PlotEnvelope, PlotStreamFrame, PlotStreamKind};

    #[test]
    fn complete_stream_frame_limits() {
        let mut frame = PlotStreamFrame {
            kind: PlotStreamKind::Error,
            envelope: Some(PlotEnvelope {
                plot_id: "temperature".to_owned(),
                revision: 1,
                epoch: "epoch".to_owned(),
                selection: "selection".to_owned(),
                payload: None,
                diagnostics: None,
                warnings: vec!["unavailable".to_owned()],
            }),
        };
        let select: fn(&mut PlotStreamFrame) -> &mut PlotEnvelope = |frame| frame.envelope.as_mut().expect("fixture envelope");
        let encoded = crate::grpc::content::output_frame_len(&frame).unwrap();
        assert!(fit_message(&mut frame, select, encoded, true).is_ok());
        assert!(matches!(fit_message(&mut frame, select, encoded - 1, true), Err(FrameError::Status(_))));
    }
}
