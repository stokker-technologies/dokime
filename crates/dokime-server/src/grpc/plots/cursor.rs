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
use prost::Message;
use sha2::{Digest, Sha256};
use tonic::Status;
// Local Imports
use super::super::enums;
use super::super::value::{self, Budget};
use dokime_app::{
    core::{
        ArtifactId, CaseId, CriterionId, Identifier, RecordScope, SessionId, StepId, SuiteId,
        TemplateId, TestCycleId, TestExecutionId,
    },
    document::{
        AxisScale, DocumentFilter, FftDetrend, FftOptions, FftWindow, FrequencyLevelUnit,
        HistogramOptions, SigmaIntervalOptions,
    },
    expressions::{DataExpression, DataSource, DataTransform},
    runtime::query::{DownsampleTechnique, PlotPayloadCursor, PlotPayloadRequest},
};
use dokime_protocol::v1;

/* === Constants === */
const MAX_PLOT_REQUESTS: usize = 32;
const MAX_PLOT_POINTS: u64 = 100_000;
const MAX_PLOT_TRANSFORMS: usize = 32;

/* === Macros === */
macro_rules! impl_from_uuid {
    ($($ty:ty),+ $(,)?) => {
        $(impl FromUuid for $ty {
            fn from_uuid(value: String) -> Result<Self, dokime_app::ApiError> {
                <$ty>::from_uuid(value)
            }
        })+
    };
}

impl_from_uuid!(
    ArtifactId,
    CriterionId,
    SessionId,
    StepId,
    SuiteId,
    TemplateId,
    TestCycleId,
    TestExecutionId,
);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */
trait FromUuid: Sized {
    fn from_uuid(value: String) -> Result<Self, dokime_app::ApiError>;
}

/* === Implementations === */

/* === Public Functions === */
pub(super) fn bounded_text(value: String) -> Result<String, Status> {
    value::text(&value)?;
    Ok(value)
}

pub(super) fn decode_plot_requests(
    requests: Vec<v1::PlotRequest>,
    epoch: &str,
) -> Result<(Vec<PlotPayloadRequest>, Vec<String>), Status> {
    if requests.len() > MAX_PLOT_REQUESTS {
        return Err(Status::resource_exhausted("plot request exceeds 32 plots"));
    }
    let decoded = requests
        .into_iter()
        .map(|request| {
            let selection = selection_key_wire(&request);
            let request = plot_request(request, epoch, &selection)?;
            Ok((request, selection))
        })
        .collect::<Result<Vec<_>, Status>>()?;
    Ok(decoded.into_iter().unzip())
}

pub(super) fn selection_for_plot<'a>(
    requests: &[PlotPayloadRequest],
    selections: &'a [String],
    plot_id: &str,
) -> Option<&'a str> {
    let index = requests
        .iter()
        .position(|request| request.plot_id.as_deref().unwrap_or(&request.stream) == plot_id)?;
    selections.get(index).map(String::as_str)
}

pub(super) fn expression_stream(value: String) -> Result<String, Status> {
    let value = bounded_text(value)?;
    Identifier::new(value.clone())
        .map_err(|error| Status::invalid_argument(format!("invalid telemetry stream: {error}")))?;
    Ok(value)
}

pub(crate) fn record_scope(scope: v1::EvidenceScope) -> Result<RecordScope, Status> {
    macro_rules! parse {
        ($field:ident, $ty:ty, $label:literal) => {
            scope
                .$field
                .map(|value| parse_id::<$ty>(value, $label))
                .transpose()?
        };
    }
    let origin = match scope.origin.and_then(|origin| origin.kind) {
        None | Some(v1::evidence_origin::Kind::Default(true)) => {
            dokime_app::core::EvidenceOrigin::Default
        }
        Some(v1::evidence_origin::Kind::Initialization(true)) => {
            dokime_app::core::EvidenceOrigin::Initialization
        }
        Some(v1::evidence_origin::Kind::Cleanup(true)) => dokime_app::core::EvidenceOrigin::Cleanup,
        Some(v1::evidence_origin::Kind::StepAttemptId(id)) => {
            dokime_app::core::EvidenceOrigin::Step {
                attempt_id: dokime_app::core::StepAttemptId::from_uuid(id)
                    .map_err(|error| Status::invalid_argument(error.to_string()))?,
            }
        }
        _ => return Err(Status::invalid_argument("invalid evidence origin")),
    };
    if origin != dokime_app::core::EvidenceOrigin::Default && scope.execution_id.is_none() {
        return Err(Status::invalid_argument(
            "evidence origin requires an execution owner",
        ));
    }
    Ok(RecordScope {
        origin,
        session_id: parse!(session_id, SessionId, "session_id"),
        test_cycle_id: scope
            .cycle_id
            .map(|value| parse_id::<TestCycleId>(value, "cycle_id"))
            .transpose()?,
        test_case_id: scope
            .case_id
            .map(|value| {
                CaseId::from_wire(value)
                    .map_err(|error| Status::invalid_argument(format!("invalid case_id: {error}")))
            })
            .transpose()?,
        test_execution_id: parse!(execution_id, TestExecutionId, "execution_id"),
        step_id: parse!(step_id, StepId, "step_id"),
        criterion_id: parse!(criterion_id, CriterionId, "criterion_id"),
        template_id: parse!(template_id, TemplateId, "template_id"),
        suite_id: parse!(suite_id, SuiteId, "suite_id"),
    })
}

/* === Private Functions === */
fn plot_cursor(cursor: v1::PlotCursor) -> PlotPayloadCursor {
    PlotPayloadCursor {
        telemetry_streams: cursor.telemetry_streams.into_iter().collect(),
        record_sequence: cursor.record_sequence,
    }
}

fn plot_request(
    request: v1::PlotRequest,
    epoch: &str,
    selection: &str,
) -> Result<PlotPayloadRequest, Status> {
    if let Some(cursor) = request.cursor.as_ref() {
        if cursor.epoch != epoch {
            return Err(Status::failed_precondition(
                "plot cursor belongs to another runtime epoch",
            ));
        }
        if cursor.selection != selection {
            return Err(Status::failed_precondition(
                "plot cursor belongs to another plot selection",
            ));
        }
    }
    value::text(&request.stream)?;
    if request.stream.is_empty() {
        return Err(Status::invalid_argument("plot stream is required"));
    }
    Identifier::new(request.stream.clone())
        .map_err(|error| Status::invalid_argument(format!("invalid plot stream: {error}")))?;
    value::collection(request.y_keys.len())?;
    value::collection(request.marker_record_types.len())?;
    value::collection(request.filters.len())?;
    for item in request
        .y_keys
        .iter()
        .chain(request.marker_record_types.iter())
    {
        value::text(item)?;
    }
    let plot_type = enums::plot_type_from_wire(request.plot_type)?;
    // An unset scale or technique is the default, not a refusal.
    let x_scale = enums::axis_scale_from_wire(request.x_scale)?.unwrap_or(AxisScale::Linear);
    let y_scale = enums::axis_scale_from_wire(request.y_scale)?.unwrap_or(AxisScale::Linear);
    let technique = enums::downsample_technique_from_wire(request.technique)?
        .unwrap_or(DownsampleTechnique::Raw);
    let max_points = request
        .max_points
        .map(|value| {
            if value > MAX_PLOT_POINTS {
                Err(Status::resource_exhausted("plot max_points exceeds 100000"))
            } else {
                usize::try_from(value)
                    .map_err(|_| Status::out_of_range("plot max_points is not supported"))
            }
        })
        .transpose()?;
    if let Some(window_s) = request.window_s
        && (!window_s.is_finite() || window_s < 0.0)
    {
        return Err(Status::invalid_argument(
            "plot window_s must be finite and non-negative",
        ));
    }

    Ok(PlotPayloadRequest {
        analysis: request
            .analysis
            .map(super::analysis::selection)
            .transpose()?,
        plot_id: request.plot_id.map(bounded_text).transpose()?,
        plot_type,
        x_scale,
        y_scale,
        stream: request.stream,
        y_keys: request.y_keys,
        marker_record_types: request.marker_record_types,
        filters: request
            .filters
            .into_iter()
            .map(plot_filter)
            .collect::<Result<_, _>>()?,
        x_key: request.x_key.map(bounded_text).transpose()?,
        series_by: request.series_by.map(bounded_text).transpose()?,
        x_expression: request.x_expression.map(data_expression).transpose()?,
        y_expressions: request
            .y_expressions
            .into_iter()
            .map(data_expression)
            .collect::<Result<_, _>>()?,
        series_by_expression: request
            .series_by_expression
            .map(data_expression)
            .transpose()?,
        z_expression: request.z_expression.map(data_expression).transpose()?,
        fft: request
            .fft
            .map(fft_options)
            .transpose()?
            .unwrap_or_default(),
        interval: request
            .interval
            .map(interval_options)
            .transpose()?
            .unwrap_or_default(),
        histogram: request
            .histogram
            .map(histogram_options)
            .transpose()?
            .unwrap_or_default(),
        z_stream: request.z_stream.map(bounded_text).transpose()?,
        z_field: request.z_field.map(bounded_text).transpose()?,
        unit: request.unit.map(bounded_text).transpose()?,
        // A raw client request names no plot declaration, so there is no
        // authored override to carry; the resolved winner stays a column
        // format alone, folded in wherever the payload is assembled.
        x_format: None,
        y_format: None,
        max_points,
        window_s: request.window_s,
        scope: request.scope.map(record_scope).transpose()?,
        cursor: request.cursor.map(plot_cursor),
        technique,
    })
}

fn nonnegative(value: f64, label: &str) -> Result<f64, Status> {
    if !value.is_finite() || value < 0.0 {
        return Err(Status::invalid_argument(format!(
            "{label} must be finite and non-negative"
        )));
    }

    Ok(value)
}

fn fft_options(value: v1::PlotFftOptions) -> Result<FftOptions, Status> {
    let defaults = FftOptions::default();
    Ok(FftOptions {
        sample_rate_hz: value
            .sample_rate_hz
            .map(|v| nonnegative(v, "FFT sample rate"))
            .transpose()?,
        window: match v1::FftWindow::try_from(value.window) {
            Ok(v1::FftWindow::Unspecified) => defaults.window,
            Ok(v1::FftWindow::Hann) => FftWindow::Hann,
            Ok(v1::FftWindow::Rectangular) => FftWindow::Rectangular,
            Err(_) => return Err(Status::invalid_argument("invalid FFT window")),
        },
        detrend: match v1::FftDetrend::try_from(value.detrend) {
            Ok(v1::FftDetrend::Unspecified) => defaults.detrend,
            Ok(v1::FftDetrend::None) => FftDetrend::None,
            Ok(v1::FftDetrend::Mean) => FftDetrend::Mean,
            Err(_) => return Err(Status::invalid_argument("invalid FFT detrend")),
        },
        level_unit: match v1::FrequencyLevelUnit::try_from(value.level_unit) {
            Ok(v1::FrequencyLevelUnit::Unspecified) => defaults.level_unit,
            Ok(v1::FrequencyLevelUnit::Linear) => FrequencyLevelUnit::Linear,
            Ok(v1::FrequencyLevelUnit::Db) => FrequencyLevelUnit::Db,
            Ok(v1::FrequencyLevelUnit::Dbu) => FrequencyLevelUnit::Dbu,
            Ok(v1::FrequencyLevelUnit::Dbm) => FrequencyLevelUnit::Dbm,
            Err(_) => return Err(Status::invalid_argument("invalid FFT level unit")),
        },
        include_dc: value.include_dc,
        min_frequency_hz: value
            .min_frequency_hz
            .map(|v| nonnegative(v, "FFT minimum frequency"))
            .transpose()?,
        max_frequency_hz: value
            .max_frequency_hz
            .map(|v| nonnegative(v, "FFT maximum frequency"))
            .transpose()?,
    })
}

fn interval_options(value: v1::PlotIntervalOptions) -> Result<SigmaIntervalOptions, Status> {
    Ok(SigmaIntervalOptions {
        bin_width: nonnegative(value.bin_width, "interval bin width")?,
        sigma: nonnegative(value.sigma, "interval sigma")?,
        clip_to_observed: value.clip_to_observed,
    })
}

fn histogram_options(value: v1::PlotHistogramOptions) -> Result<HistogramOptions, Status> {
    if value.bins > MAX_PLOT_POINTS {
        return Err(Status::resource_exhausted("histogram bins exceeds 100000"));
    }

    Ok(HistogramOptions {
        bins: usize::try_from(value.bins)
            .map_err(|_| Status::out_of_range("histogram bins is not supported"))?,
        unit: value.unit.map(bounded_text).transpose()?,
    })
}

fn selection_key_wire(request: &v1::PlotRequest) -> String {
    let mut request = request.clone();
    request.cursor = None;
    Sha256::digest(request.encode_to_vec())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn plot_filter(filter: v1::PlotFilter) -> Result<DocumentFilter, Status> {
    let expression = filter
        .expression
        .ok_or_else(|| Status::invalid_argument("plot filter expression is required"))
        .and_then(data_expression)?;
    let operator = enums::filter_operator_from_wire(filter.operator)?
        .ok_or_else(|| Status::invalid_argument("plot filter operator is required"))?;
    let value = filter
        .value
        .map(|value| value::decode(value, &mut Budget::new(), 0))
        .transpose()?;
    Ok(DocumentFilter {
        expression,
        operator,
        value,
    })
}

fn data_expression(expression: v1::DataExpression) -> Result<DataExpression, Status> {
    if expression.transforms.len() > MAX_PLOT_TRANSFORMS {
        return Err(Status::resource_exhausted(
            "plot expression exceeds 32 transforms",
        ));
    }
    let source = match expression
        .source
        .ok_or_else(|| Status::invalid_argument("data expression source is required"))?
    {
        v1::data_expression::Source::Telemetry(source) => DataSource::Telemetry {
            stream: expression_stream(source.stream)?,
            field: bounded_text(source.field)?,
        },
        v1::data_expression::Source::Parameter(source) => DataSource::Parameter {
            name: bounded_text(source.name)?,
        },
        v1::data_expression::Source::Dimension(source) => DataSource::Dimension {
            name: bounded_text(source.name)?,
        },
        v1::data_expression::Source::Record(source) => DataSource::Record {
            record_type: bounded_text(source.record_type)?,
        },
        v1::data_expression::Source::Artifact(source) => DataSource::Artifact {
            artifact: bounded_text(source.artifact)?,
        },
    };
    let transforms = expression
        .transforms
        .into_iter()
        .map(data_transform)
        .collect::<Result<_, _>>()?;
    Ok(DataExpression {
        label: expression.label.map(bounded_text).transpose()?,
        source,
        transforms,
    })
}

fn data_transform(transform: v1::DataTransform) -> Result<DataTransform, Status> {
    let kind = transform
        .kind
        .ok_or_else(|| Status::invalid_argument("data transform kind is required"))?;
    Ok(match kind {
        v1::data_transform::Kind::Count(_) => DataTransform::Count,
        v1::data_transform::Kind::Mean(_) => DataTransform::Mean,
        v1::data_transform::Kind::Median(_) => DataTransform::Median,
        v1::data_transform::Kind::Min(_) => DataTransform::Min,
        v1::data_transform::Kind::Max(_) => DataTransform::Max,
        v1::data_transform::Kind::TimeBetween(value) => DataTransform::TimeBetween {
            previous_record_type: bounded_text(value.previous_record_type)?,
        },
        v1::data_transform::Kind::TelemetryAtOrBefore(value) => {
            DataTransform::TelemetryAtOrBefore {
                stream: expression_stream(value.stream)?,
                field: bounded_text(value.field)?,
            }
        }
        v1::data_transform::Kind::Detail(value) => DataTransform::Detail {
            name: bounded_text(value.name)?,
        },
        v1::data_transform::Kind::PerCase(_) => DataTransform::PerCase,
    })
}

fn parse_id<T>(value: String, label: &str) -> Result<T, Status>
where
    T: FromUuid,
{
    if uuid::Uuid::parse_str(&value)
        .map(|value| value.is_nil())
        .unwrap_or(true)
    {
        return Err(Status::invalid_argument(format!("invalid {label}")));
    }
    T::from_uuid(value)
        .map_err(|error| Status::invalid_argument(format!("invalid {label}: {error}")))
}

/* === Tests === */
