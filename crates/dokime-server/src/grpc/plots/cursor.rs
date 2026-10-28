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
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use super::super::enums::canonical as enums;
use super::super::value::{self, Budget, canonical};
use dokime::document::AxisScale;
use dokime::document::DocumentFilter;
use dokime::document::FftDetrend;
use dokime::document::FftOptions;
use dokime::document::FftWindow;
use dokime::document::FrequencyLevelUnit;
use dokime::document::HistogramOptions;
use dokime::document::SigmaIntervalOptions;
use dokime::domain::model::execution::RecordScope;
use dokime::domain::model::identity::Identifier;
use dokime::expressions::DataExpression;
use dokime::expressions::DataSource;
use dokime::expressions::DataTransform;
use dokime::runtime::query::DownsampleTechnique;
use dokime::runtime::query::PlotPayloadRequest;
use dokime::runtime::query::PlotWindow;

/* === Constants === */

const SERVER_PLOT_POINTS: u64 = 100_000;
const SERVER_PLOT_REQUESTS: u64 = 32;
const SERVER_PLOT_TRANSFORMS: u64 = 32;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn decode_plot_requests(
    requests: Vec<dokime::domain::model::plots::wire::PlotRequest>,
) -> Result<(Vec<PlotPayloadRequest>, Vec<String>), Status> {
    if requests.len() > SERVER_PLOT_REQUESTS as usize {
        return Err(Status::resource_exhausted("plot request exceeds 32 plots"));
    }
    let decoded = requests
        .into_iter()
        .map(|request| {
            let selection = selection_key_wire(&request);
            let request = plot_request(request)?;
            Ok((request, selection))
        })
        .collect::<Result<Vec<_>, Status>>()?;
    Ok(decoded.into_iter().unzip())
}

pub(super) fn selection_for_plot<'a>(requests: &[PlotPayloadRequest], selections: &'a [String], plot_id: &str) -> Option<&'a str> {
    let index = requests.iter().position(|request| request.plot_id.as_deref().unwrap_or(&request.stream) == plot_id)?;
    selections.get(index).map(String::as_str)
}

pub(super) fn expression_stream(value: String) -> Result<String, Status> {
    Identifier::new(value.clone()).map_err(|error| Status::invalid_argument(format!("invalid telemetry stream: {error}")))?;
    Ok(value)
}

pub(crate) fn record_scope(scope: dokime::domain::model::identity::wire::EvidenceScope) -> Result<RecordScope, Status> {
    canonical::decoded_scope(&scope)
}

/* === Private Functions === */

fn plot_request(request: dokime::domain::model::plots::wire::PlotRequest) -> Result<PlotPayloadRequest, Status> {
    if request.stream.is_empty() {
        return Err(Status::invalid_argument("plot stream is required"));
    }
    Identifier::new(request.stream.clone()).map_err(|error| Status::invalid_argument(format!("invalid plot stream: {error}")))?;
    value::collection(request.y_keys.len())?;
    value::collection(request.marker_record_types.len())?;
    value::collection(request.filters.len())?;
    let plot_type = enums::plot_type_from_wire(request.plot_type)?;
    // An unset scale or technique is the default, not a refusal.
    let x_scale = enums::axis_scale_from_wire(request.x_scale)?.unwrap_or(AxisScale::Linear);
    let y_scale = enums::axis_scale_from_wire(request.y_scale)?.unwrap_or(AxisScale::Linear);
    let technique = enums::downsample_technique_from_wire(request.technique)?.unwrap_or(DownsampleTechnique::Auto);
    let max_points = request
        .max_points
        .map(|value| {
            if value > SERVER_PLOT_POINTS {
                Err(Status::invalid_argument(format!("plot max_points must not exceed {}", SERVER_PLOT_POINTS)))
            } else {
                usize::try_from(value).map_err(|_| Status::out_of_range("plot max_points is not supported"))
            }
        })
        .transpose()?;
    if let Some(window_s) = request.window_s
        && (!window_s.is_finite() || window_s < 0.0)
    {
        return Err(Status::invalid_argument("plot window_s must be finite and non-negative"));
    }

    let summary_window = request
        .summary_window
        .map(|window| {
            if window.from.is_finite() && window.to.is_finite() && window.from <= window.to {
                Ok(PlotWindow { from: window.from, to: window.to })
            } else {
                Err(Status::invalid_argument("plot summary_window must be a finite range with from <= to"))
            }
        })
        .transpose()?;

    Ok(PlotPayloadRequest {
        analysis: request.analysis.map(super::analysis::selection).transpose()?,
        plot_id: request.plot_id,
        plot_type,
        x_scale,
        y_scale,
        stream: request.stream,
        y_keys: request.y_keys,
        marker_record_types: request.marker_record_types,
        filters: request.filters.into_iter().map(plot_filter).collect::<Result<_, _>>()?,
        x_key: request.x_key,
        series_by: request.series_by,
        x_expression: request.x_expression.map(data_expression).transpose()?,
        y_expressions: request.y_expressions.into_iter().map(data_expression).collect::<Result<_, _>>()?,
        series_by_expression: request.series_by_expression.map(data_expression).transpose()?,
        z_expression: request.z_expression.map(data_expression).transpose()?,
        fft: request.fft.map(fft_options).transpose()?.unwrap_or_default(),
        interval: request.interval.map(interval_options).transpose()?.unwrap_or_default(),
        histogram: request.histogram.map(histogram_options).transpose()?.unwrap_or_default(),
        z_stream: request.z_stream,
        z_field: request.z_field,
        unit: request.unit,
        // A raw client request names no plot declaration, so there is no
        // authored override to carry; the resolved winner stays a column
        // format alone, folded in wherever the payload is assembled.
        x_format: None,
        y_format: None,
        max_points,
        window_s: request.window_s,
        scope: request.scope.map(record_scope).transpose()?,
        technique,
        summary_window,
    })
}

fn nonnegative(value: f64, label: &str) -> Result<f64, Status> {
    if !value.is_finite() || value < 0.0 {
        return Err(Status::invalid_argument(format!("{label} must be finite and non-negative")));
    }

    Ok(value)
}

fn fft_options(value: dokime::domain::model::plots::wire::PlotFftOptions) -> Result<FftOptions, Status> {
    let defaults = FftOptions::default();
    Ok(FftOptions {
        sample_rate_hz: value.sample_rate_hz.map(|v| nonnegative(v, "FFT sample rate")).transpose()?,
        window: match value.window {
            dokime::domain::model::plots::wire::FftWindow::Unspecified => defaults.window,
            dokime::domain::model::plots::wire::FftWindow::Hann => FftWindow::Hann,
            dokime::domain::model::plots::wire::FftWindow::Rectangular => FftWindow::Rectangular,
        },
        detrend: match value.detrend {
            dokime::domain::model::plots::wire::FftDetrend::Unspecified => defaults.detrend,
            dokime::domain::model::plots::wire::FftDetrend::None => FftDetrend::None,
            dokime::domain::model::plots::wire::FftDetrend::Mean => FftDetrend::Mean,
        },
        level_unit: match value.level_unit {
            dokime::domain::model::plots::wire::FrequencyLevelUnit::Unspecified => defaults.level_unit,
            dokime::domain::model::plots::wire::FrequencyLevelUnit::Linear => FrequencyLevelUnit::Linear,
            dokime::domain::model::plots::wire::FrequencyLevelUnit::Db => FrequencyLevelUnit::Db,
            dokime::domain::model::plots::wire::FrequencyLevelUnit::Dbu => FrequencyLevelUnit::Dbu,
            dokime::domain::model::plots::wire::FrequencyLevelUnit::Dbm => FrequencyLevelUnit::Dbm,
        },
        include_dc: value.include_dc,
        min_frequency_hz: value.min_frequency_hz.map(|v| nonnegative(v, "FFT minimum frequency")).transpose()?,
        max_frequency_hz: value.max_frequency_hz.map(|v| nonnegative(v, "FFT maximum frequency")).transpose()?,
    })
}

fn interval_options(value: dokime::domain::model::plots::wire::PlotIntervalOptions) -> Result<SigmaIntervalOptions, Status> {
    Ok(SigmaIntervalOptions {
        bin_width: nonnegative(value.bin_width, "interval bin width")?,
        sigma: nonnegative(value.sigma, "interval sigma")?,
        clip_to_observed: value.clip_to_observed,
    })
}

fn histogram_options(value: dokime::domain::model::plots::wire::PlotHistogramOptions) -> Result<HistogramOptions, Status> {
    if value.bins > SERVER_PLOT_POINTS {
        return Err(Status::resource_exhausted("histogram bins exceeds 100000"));
    }

    Ok(HistogramOptions {
        bins: usize::try_from(value.bins).map_err(|_| Status::out_of_range("histogram bins is not supported"))?,
        unit: value.unit,
    })
}

fn selection_key_wire(request: &dokime::domain::model::plots::wire::PlotRequest) -> String {
    Sha256::digest(dokime::harmos::Message::encode_to_vec(request)).iter().map(|byte| format!("{byte:02x}")).collect::<String>()
}

fn plot_filter(filter: dokime::domain::model::plots::wire::PlotFilter) -> Result<DocumentFilter, Status> {
    let expression =
        filter.expression.ok_or_else(|| Status::invalid_argument("plot filter expression is required")).and_then(data_expression)?;
    let operator =
        enums::filter_operator_from_wire(filter.operator)?.ok_or_else(|| Status::invalid_argument("plot filter operator is required"))?;
    let value = filter.value.map(|value| canonical::decode(value, &mut Budget::new()?, 0)).transpose()?;
    Ok(DocumentFilter { expression, operator, value })
}

fn data_expression(expression: dokime::domain::model::plots::wire::DataExpression) -> Result<DataExpression, Status> {
    if expression.transforms.len() > SERVER_PLOT_TRANSFORMS as usize {
        return Err(Status::resource_exhausted("plot expression exceeds 32 transforms"));
    }
    let source = match expression.source.ok_or_else(|| Status::invalid_argument("data expression source is required"))? {
        dokime::domain::model::plots::wire::DataExpressionSource::Telemetry(source) => DataSource::Telemetry {
            stream: expression_stream(source.stream)?,
            field: source.field,
        },
        dokime::domain::model::plots::wire::DataExpressionSource::Parameter(source) => DataSource::Parameter { name: source.name },
        dokime::domain::model::plots::wire::DataExpressionSource::Dimension(source) => DataSource::Dimension { name: source.name },
        dokime::domain::model::plots::wire::DataExpressionSource::Record(source) => DataSource::Record { record_type: source.record_type },
        dokime::domain::model::plots::wire::DataExpressionSource::Artifact(source) => DataSource::Artifact { artifact: source.artifact },
    };
    let transforms = expression.transforms.into_iter().map(data_transform).collect::<Result<_, _>>()?;
    Ok(DataExpression {
        label: expression.label,
        source,
        transforms,
    })
}

fn data_transform(transform: dokime::domain::model::plots::wire::DataTransform) -> Result<DataTransform, Status> {
    let kind = transform.kind.ok_or_else(|| Status::invalid_argument("data transform kind is required"))?;
    Ok(match kind {
        dokime::domain::model::plots::wire::DataTransformKind::Aggregate(value) => {
            if !value.aggregate.answerable() {
                return Err(Status::invalid_argument("data transform aggregate percentile rank exceeds 100"));
            }
            DataTransform::Aggregate(value.aggregate)
        }
        dokime::domain::model::plots::wire::DataTransformKind::TimeBetween(value) => DataTransform::TimeBetween {
            previous_record_type: value.previous_record_type,
        },
        dokime::domain::model::plots::wire::DataTransformKind::TelemetryAtOrBefore(value) => DataTransform::TelemetryAtOrBefore {
            stream: expression_stream(value.stream)?,
            field: value.field,
        },
        dokime::domain::model::plots::wire::DataTransformKind::Detail(value) => DataTransform::Detail { name: value.name },
        dokime::domain::model::plots::wire::DataTransformKind::PerCase(_) => DataTransform::PerCase,
    })
}

/* === Tests === */
