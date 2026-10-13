/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod analyses;
pub(crate) mod builtins;
mod cartesian;
mod types;

/* === Re-Exports === */
pub use analyses::{
    BoxSummary, HistogramBin, HistogramSeries, HistogramStatistics, PlotAnalysis, SigmaBin,
    SigmaSeries,
};
pub use types::{
    DownsampleTechnique, PlotCartesianFrame, PlotCartesianMode, PlotCartesianSeriesFrame,
    PlotComputation, PlotComputedSeries, PlotDomain, PlotEvidence, PlotMarker, PlotPayload,
    PlotPayloadCursor, PlotPayloadDiagnostics, PlotPayloadEnvelope, PlotPayloadRequest,
    PlotPayloadStreamKind, PlotPoint, PlotSeries,
};

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use serde_json::Value as JsonValue;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
// Local Imports
use crate::definition::MaterializedParameter;
use crate::document::{DocumentFilter, FilterOperator, PlotDescriptor, PlotType};
use crate::execution::CycleView;
use crate::plot::record_telemetry::record_telemetry_series;
use crate::plot::record_timing::record_timing_series;
use crate::records::RecordEntry;
use crate::traceability::TraceabilitySnapshot;
use cartesian::cartesian_frame;
use dokime::core::{RecordScope, Value, ValueFormat, datetime_seconds, scope_visible_in};
use dokime::expressions::{DataExpression, DataSource, DataTransform};
use dokime::telemetry::{TelemetryColumn, TelemetrySample};
use harmos::storage::reduce::Agg;

/* === Constants === */
pub(crate) use dokime::document::DEFAULT_PLOT_MAX_POINTS;
const MARKER_COLORS: [&str; 8] = [
    "#ef4444", "#f97316", "#eab308", "#22c55e", "#14b8a6", "#3b82f6", "#8b5cf6", "#ec4899",
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default)]
pub(crate) struct PlotPayloadContext {
    pub x_format: Option<ValueFormat>,
    pub series_format: Option<ValueFormat>,
    pub run_started_at: Option<String>,
    pub cycle_started_at: Option<String>,
}

struct RowExpressionRenderContext<'a> {
    samples: &'a [&'a TelemetrySample],
    x_expression: Option<&'a DataExpression>,
    x_key: &'a str,
    z_index: &'a ZIndex,
    unit: Option<&'a str>,
    max_points: usize,
    technique: DownsampleTechnique,
}

#[derive(Clone, Debug)]
struct ZSample {
    scope: Option<RecordScope>,
    timestamp: OffsetDateTime,
    value: f64,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AggregateMethod {
    Count,
    Mean,
    Median,
    Min,
    Max,
}

#[derive(Clone, Debug)]
enum ZIndex {
    None,
    Expression(DataExpression),
    Primary { field: String },
    Stream { samples: Vec<ZSample> },
}

/* === Traits === */

/* === Implementations === */
impl ZIndex {
    fn resolve(&self, sample: &TelemetrySample) -> Option<f64> {
        match self {
            Self::None => None,
            Self::Expression(expression) => expression_numeric_value(sample, expression),
            Self::Primary { field } => sample.values.get(field).and_then(numeric_value),
            Self::Stream { samples, .. } => {
                let sample_time = timestamp(sample)?;
                samples
                    .iter()
                    .rev()
                    .find(|candidate| {
                        candidate.timestamp <= sample_time
                            && crate::evidence::scopes_correlate(
                                candidate.scope.as_ref(),
                                sample.scope.as_ref(),
                            )
                    })
                    .map(|candidate| candidate.value)
            }
        }
    }
}

/* === Public Functions === */
pub(crate) fn build_plot_payload(
    telemetry: &[TelemetrySample],
    records: &[RecordEntry],
    request: PlotPayloadRequest,
    generated_at: &str,
    format_context: &PlotPayloadContext,
) -> PlotPayload {
    let payload = project_plot_payload(telemetry, records, &request, generated_at, format_context);
    if request.analysis.is_some() {
        let mut payload = payload;
        payload.series.clear();
        payload.analysis = Some(PlotAnalysis::Error {
            message: "selected analysis requires an admitted runtime operation".into(),
        });
        return payload;
    }
    finish_plot_payload(payload, &request)
}

pub(crate) fn project_plot_payload(
    telemetry: &[TelemetrySample],
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
    generated_at: &str,
    format_context: &PlotPayloadContext,
) -> PlotPayload {
    let x_key = request
        .x_key
        .clone()
        .unwrap_or_else(|| "timestamp".to_owned());
    // An absent budget means the display-scale default, never unbounded: a
    // full-range ("All") read must not ship every retained raw sample. An
    // explicit `Some(0)` remains the deliberate no-downsampling escape hatch.
    let max_points = request.max_points.unwrap_or(DEFAULT_PLOT_MAX_POINTS);
    let render_max_points = max_points;
    let max_points = 0;
    let technique = request.technique;
    let samples = apply_plot_window(
        telemetry
            .iter()
            .filter(|sample| {
                sample.stream.as_str() == request.stream
                    && sample_matches_scope(sample.scope.as_ref(), request.scope.as_ref())
                    && sample_matches_filters(sample, &request.filters)
            })
            .collect(),
        if request.analysis.is_some()
            || matches!(
                request.plot_type,
                Some(PlotType::Trajectory | PlotType::FrequencySeries)
            )
        {
            "timestamp"
        } else {
            &x_key
        },
        request.window_s,
    );
    let mut z_index = z_index(
        telemetry,
        &request.stream,
        request.z_stream.as_deref(),
        request.z_field.as_deref(),
        request.scope.as_ref(),
    );
    if let Some(expression) = &request.z_expression {
        z_index = match &expression.source {
            DataSource::Telemetry { stream, field } => self::z_index(
                telemetry,
                &request.stream,
                Some(stream),
                Some(field),
                request.scope.as_ref(),
            ),
            _ => ZIndex::Expression(expression.clone()),
        };
    }
    let series = record_timing_series(
        telemetry,
        records,
        request,
        request.unit.as_deref(),
        max_points,
        technique,
    )
    .or_else(|| {
        record_telemetry_series(
            telemetry,
            records,
            request,
            request.unit.as_deref(),
            max_points,
            technique,
        )
    })
    .or_else(|| {
        aggregate_series(
            &samples,
            request,
            &z_index,
            request.unit.as_deref(),
            max_points,
            technique,
        )
    })
    .or_else(|| {
        row_expression_series(
            &samples,
            request,
            &x_key,
            &z_index,
            request.unit.as_deref(),
            max_points,
            technique,
        )
    })
    .unwrap_or_else(|| {
        request.series_by.as_ref().map_or_else(
            || {
                plain_series(
                    &samples,
                    &request.y_keys,
                    &x_key,
                    &z_index,
                    request.unit.as_deref(),
                    max_points,
                    technique,
                )
            },
            |series_by| {
                grouped_series(
                    &samples,
                    &request.y_keys,
                    &x_key,
                    series_by,
                    &z_index,
                    request.unit.as_deref(),
                    max_points,
                    technique,
                )
            },
        )
    });
    let max_points = render_max_points;
    let context = plot_context(request, &samples, telemetry);
    // Every series in one payload reads the same declared y axis, so they
    // share the one resolved winner: a plot override or a column's declared
    // format apply uniformly, not per series.
    let mut series = series;
    for entry in &mut series {
        entry.format = format_context.series_format.clone();
    }

    PlotPayload {
        computed: None,
        stream: request.stream.clone(),
        x_key,
        series_by: request.series_by.clone(),
        context,
        series,
        cartesian: None,
        analysis: None,
        markers: plot_markers(
            records,
            &request.marker_record_types,
            request.scope.as_ref(),
        ),
        max_points,
        window_s: request.window_s,
        generated_at: generated_at.to_owned(),
        x_format: format_context.x_format.clone(),
        run_started_at: format_context.run_started_at.clone(),
        cycle_started_at: format_context.cycle_started_at.clone(),
    }
}

pub(crate) fn finish_plot_payload(
    mut payload: PlotPayload,
    request: &PlotPayloadRequest,
) -> PlotPayload {
    payload.analysis = analyses::analyze(&mut payload.series, request);
    let spectrum = matches!(request.plot_type, Some(PlotType::FrequencySeries));
    if spectrum {
        payload.x_key = "frequency_hz".into();
        payload.x_format = None;
    }
    for entry in &mut payload.series {
        let points = std::mem::take(&mut entry.points);
        let (points, reduced) = if spectrum {
            analyses::reduce_spectrum(points, payload.max_points)
        } else {
            downsample_points(points, payload.max_points, request.technique)
        };
        entry.points = points;
        entry.downsampled = reduced;
        if spectrum {
            entry.format = None;
        }
    }
    payload.cartesian = cartesian_frame(
        &payload.series,
        request.plot_type.as_ref(),
        &request.x_scale,
        &request.y_scale,
    );
    payload
}

pub(crate) fn resolved_format(
    plot: Option<&ValueFormat>,
    column: Option<&TelemetryColumn>,
) -> Option<ValueFormat> {
    plot.cloned()
        .or_else(|| column.and_then(|column| column.format.clone()))
}

pub(crate) fn enrich_plot_samples_with_cycle_context(
    mut telemetry: Vec<TelemetrySample>,
    cycle: &CycleView,
) -> Vec<TelemetrySample> {
    for sample in &mut telemetry {
        let Some(scope) = sample.scope.as_ref() else {
            continue;
        };
        let context = dimension_context_for_scope(cycle, scope);
        for (key, value) in context {
            sample.values.entry(key).or_insert(value);
        }
    }

    telemetry
}

pub(crate) fn enrich_plot_samples_with_traceability_context(
    mut telemetry: Vec<TelemetrySample>,
    traceability: &[TraceabilitySnapshot],
) -> Vec<TelemetrySample> {
    for sample in &mut telemetry {
        let Some(scope) = sample.scope.as_ref() else {
            continue;
        };
        let context = traceability_context_for_scope(traceability, scope);
        for (key, value) in context {
            sample.values.entry(key).or_insert(value);
        }
    }

    telemetry
}

pub(crate) fn normalize_analysis_fields(request: &mut PlotPayloadRequest) {
    if request.analysis.is_none() {
        return;
    }
    let field = |expression: &DataExpression| match &expression.source {
        DataSource::Telemetry { stream, field }
            if stream == &request.stream && expression.transforms.is_empty() =>
        {
            Some(field.clone())
        }
        _ => None,
    };
    let x = request.x_expression.as_ref().and_then(field);
    let y = if request.y_expressions.is_empty() || request.y_expressions.len() > 32 {
        None
    } else {
        request
            .y_expressions
            .iter()
            .map(field)
            .collect::<Option<Vec<_>>>()
    };
    if let Some(x) = x {
        request.x_key = Some(x);
        request.x_expression = None;
    }
    if let Some(y) = y {
        request.y_keys = y;
        request.y_expressions.clear();
    }
}

pub fn plot_payload_request_for_descriptor(
    descriptor: &PlotDescriptor,
    scope: Option<RecordScope>,
    max_points: Option<usize>,
    window_s: Option<f64>,
    cursor: Option<PlotPayloadCursor>,
) -> PlotPayloadRequest {
    let mut request = PlotPayloadRequest {
        analysis: descriptor.input.analysis.clone(),
        plot_id: Some(descriptor.plot_id.to_string()),
        plot_type: Some(descriptor.plot_type.clone()),
        x_scale: descriptor.input.x_scale.clone(),
        y_scale: descriptor.input.y_scale.clone(),
        stream: descriptor_stream(descriptor),
        y_keys: descriptor.input.series_fields.clone(),
        marker_record_types: descriptor.input.marker_record_types.clone(),
        filters: descriptor.input.filters.clone(),
        x_key: descriptor.input.x_field.clone(),
        series_by: descriptor.input.series_by.clone(),
        x_expression: descriptor.input.x_expression.clone(),
        y_expressions: descriptor.input.y_expressions.clone(),
        series_by_expression: descriptor.input.series_by_expression.clone(),
        z_expression: descriptor.input.z_expression.clone(),
        fft: descriptor.input.fft.clone(),
        interval: descriptor.input.interval.clone(),
        histogram: descriptor.input.histogram.clone(),
        z_stream: descriptor.input.z_stream.clone(),
        z_field: descriptor.input.z_field.clone(),
        unit: descriptor.input.unit.clone(),
        x_format: descriptor.input.x_format.clone(),
        y_format: descriptor.input.y_format.clone(),
        max_points: max_points.or(descriptor
            .input
            .display_points
            .map(|points| points as usize)),
        window_s,
        scope,
        cursor,
        technique: descriptor.input.downsample,
    };
    normalize_analysis_fields(&mut request);
    request
}

pub(crate) fn plot_payload_diagnostics(
    payload: &PlotPayload,
    delta: bool,
) -> PlotPayloadDiagnostics {
    if let Some(computed) = &payload.computed {
        return PlotPayloadDiagnostics {
            series_count: computed
                .series
                .iter()
                .map(|series| series.output.outputs.len())
                .sum(),
            point_count: computed
                .series
                .iter()
                .flat_map(|series| {
                    series.output.outputs.iter().map(|output| {
                        series
                            .display_indices
                            .get(&output.key)
                            .map_or(output.values.len(), Vec::len)
                    })
                })
                .sum(),
            raw_point_count: computed
                .series
                .iter()
                .map(|series| series.input_samples)
                .sum(),
            downsampled: computed
                .series
                .iter()
                .any(|series| !series.display_indices.is_empty()),
            delta: false,
            max_points: payload.max_points,
            window_s: payload.window_s,
        };
    }
    PlotPayloadDiagnostics {
        series_count: payload.series.len(),
        point_count: payload
            .series
            .iter()
            .map(|series| series.points.len())
            .sum(),
        raw_point_count: payload.series.iter().map(|series| series.raw_points).sum(),
        downsampled: payload.series.iter().any(|series| series.downsampled),
        delta,
        max_points: payload.max_points,
        window_s: payload.window_s,
    }
}

pub(crate) fn finish_plot_series(
    key: String,
    label: String,
    unit: Option<String>,
    points: Vec<PlotPoint>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> PlotSeries {
    let raw_points = points.len();
    let (points, downsampled) = downsample_points(points, max_points, technique);

    PlotSeries {
        key,
        label,
        unit,
        points,
        raw_points,
        downsampled,
        format: None,
    }
}

pub(crate) fn plot_window_duration(window_s: f64) -> Option<time::Duration> {
    if window_s < 0.0 {
        return None;
    }
    time::Duration::checked_seconds_f64(window_s)
}

pub(crate) fn plot_window_cutoff(
    latest: OffsetDateTime,
    window_s: Option<f64>,
) -> Option<OffsetDateTime> {
    latest.checked_sub(plot_window_duration(window_s?)?)
}

pub(crate) fn retain_window<T>(
    items: &mut Vec<T>,
    window_s: Option<f64>,
    time_of: impl Fn(&T) -> Option<OffsetDateTime>,
) {
    let Some(window_s) = window_s else {
        return;
    };
    let Some(latest) = items.iter().filter_map(&time_of).max() else {
        return;
    };
    let Some(cutoff) = plot_window_cutoff(latest, Some(window_s)) else {
        return;
    };
    items.retain(|item| time_of(item).is_none_or(|time| time >= cutoff));
}

pub(crate) fn aggregate_group_points(
    grouped: BTreeMap<String, Vec<f64>>,
    method: AggregateMethod,
) -> Vec<PlotPoint> {
    let mut sorted_groups = fold_groups(grouped.into_iter().collect(), method);
    sorted_groups.sort_by(|(left, _), (right, _)| compare_series_group(left, right));

    sorted_groups
        .into_iter()
        .map(|(group, y)| PlotPoint {
            x: JsonValue::String(group),
            y,
            z: None,
        })
        .collect()
}

pub(crate) fn aggregate_method(expression: &DataExpression) -> Option<AggregateMethod> {
    expression
        .transforms
        .iter()
        .find_map(|transform| match transform {
            DataTransform::Count => Some(AggregateMethod::Count),
            DataTransform::Mean => Some(AggregateMethod::Mean),
            DataTransform::Median => Some(AggregateMethod::Median),
            DataTransform::Min => Some(AggregateMethod::Min),
            DataTransform::Max => Some(AggregateMethod::Max),
            DataTransform::TimeBetween { .. } => None,
            DataTransform::TelemetryAtOrBefore { .. } => None,
            // Neither folds the values of one series: `detail` selects what is
            // read off a record, and `per_case` widens which executions are
            // read, before any aggregate over the result.
            DataTransform::Detail { .. } => None,
            DataTransform::PerCase => None,
        })
}

pub(crate) fn fold_groups(
    groups: Vec<(String, Vec<f64>)>,
    method: AggregateMethod,
) -> Vec<(String, Option<f64>)> {
    let agg = match method {
        AggregateMethod::Count => Agg::Count,
        AggregateMethod::Mean => Agg::Mean,
        AggregateMethod::Median => Agg::Median,
        AggregateMethod::Min => Agg::Min,
        AggregateMethod::Max => Agg::Max,
    };
    let labels = groups
        .iter()
        .map(|(label, _)| label.clone())
        .collect::<Vec<_>>();
    crate::document::frame::fold_groups(groups, agg)
        .unwrap_or_else(|_| labels.into_iter().map(|label| (label, None)).collect())
}

pub(crate) fn expression_group_value(
    sample: &TelemetrySample,
    expression: &DataExpression,
) -> Option<String> {
    match &expression.source {
        DataSource::Telemetry { stream, field }
            if stream_matches_alias(sample.stream.as_str(), stream) =>
        {
            sample_value(sample, field).and_then(group_value)
        }
        DataSource::Dimension { name: field } | DataSource::Parameter { name: field } => {
            sample_value(sample, field).and_then(group_value)
        }
        DataSource::Telemetry { .. } => None,
        _ => None,
    }
}

pub(crate) fn sample_value<'a>(sample: &'a TelemetrySample, key: &str) -> Option<&'a Value> {
    sample.values.get(key).or_else(|| {
        key.rsplit_once('.')
            .and_then(|(_, field)| sample.values.get(field))
    })
}

pub(crate) fn downsample_points(
    points: Vec<PlotPoint>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> (Vec<PlotPoint>, bool) {
    if points.len() <= max_points || max_points == 0 {
        return (points, false);
    }
    if max_points == 1 {
        return stride_downsample(points, max_points);
    }
    let technique = match technique {
        DownsampleTechnique::Auto | DownsampleTechnique::Unspecified => {
            if points.len() <= max_points.saturating_mul(4) {
                DownsampleTechnique::Lttb
            } else {
                DownsampleTechnique::MinMax
            }
        }
        technique => technique,
    };
    match technique {
        DownsampleTechnique::Raw => stride_downsample(points, max_points),
        DownsampleTechnique::Auto
        | DownsampleTechnique::Unspecified
        | DownsampleTechnique::MinMax
        | DownsampleTechnique::Lttb => {
            // The numeric techniques need a clean `(index, y)` scalar series; fall
            // back to stride for any series they can't model (a missing `y`, a
            // degenerate axis, an LTTB threshold below three).
            match numeric_downsample(&points, max_points, technique) {
                Some(reduced) => (reduced, true),
                None => stride_downsample(points, max_points),
            }
        }
    }
}

pub(crate) fn numeric_value(value: &Value) -> Option<f64> {
    match value {
        Value::Integer(value) => Some(*value as f64),
        Value::Unsigned(value) => Some(*value as f64),
        Value::Number(value) if value.is_finite() => Some(*value),
        _ => None,
    }
}

pub(crate) fn parse_timestamp(value: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).ok()
}

pub(crate) fn compare_series_group(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left), Ok(right)) => left
            .partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => left.cmp(right),
    }
}

pub(crate) fn stream_matches_alias(actual: &str, declared: &str) -> bool {
    actual == declared
        || actual
            .strip_suffix(declared)
            .is_some_and(|prefix| prefix.ends_with('.'))
        || declared
            .strip_suffix(actual)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

pub(crate) fn sample_matches_scope(
    sample_scope: Option<&RecordScope>,
    filter: Option<&RecordScope>,
) -> bool {
    filter.is_none_or(|filter| scope_visible_in(sample_scope, filter))
}

/* === Private Functions === */
fn dimension_context_for_scope(cycle: &CycleView, scope: &RecordScope) -> BTreeMap<String, Value> {
    let mut context = BTreeMap::new();
    let execution = if let Some(id) = &scope.test_execution_id {
        cycle
            .test_executions
            .iter()
            .find(|execution| &execution.test_execution_id == id)
    } else {
        scope.test_case_id.as_ref().and_then(|case_id| {
            cycle
                .test_executions
                .iter()
                .rev()
                .find(|execution| &execution.test_case_id == case_id)
        })
    };

    if let Some(execution) = execution {
        for (key, value) in &execution.snapshot.case().parameters {
            insert_context_value(&mut context, key, value);
        }
        for (key, parameter) in &execution.effective_parameters() {
            insert_materialized_context_value(&mut context, key, parameter);
        }
    }

    context
}

fn traceability_context_for_scope(
    traceability: &[TraceabilitySnapshot],
    scope: &RecordScope,
) -> BTreeMap<String, Value> {
    let mut context = BTreeMap::new();

    for snapshot in traceability {
        if !scope_visible_in(Some(&snapshot.scope), scope) {
            continue;
        }
        for entry in &snapshot.entries {
            insert_context_value(
                &mut context,
                entry.metadata.identifier.as_str(),
                &entry.value,
            );
        }
    }

    context
}

fn insert_materialized_context_value(
    context: &mut BTreeMap<String, Value>,
    key: &str,
    parameter: &MaterializedParameter,
) {
    insert_context_value(context, key, &parameter.value);
}

fn insert_context_value(context: &mut BTreeMap<String, Value>, key: &str, value: &Value) {
    if !traceable_context_value(value) {
        return;
    }
    context.insert(key.to_owned(), value.clone());
    if let Some(alias) = key.rsplit('.').next()
        && alias != key
    {
        context.insert(alias.to_owned(), value.clone());
    }
}

fn apply_plot_window<'a>(
    mut samples: Vec<&'a TelemetrySample>,
    x_key: &str,
    window_s: Option<f64>,
) -> Vec<&'a TelemetrySample> {
    samples.sort_by(|left, right| left.timestamp.cmp(&right.timestamp));
    retain_window(&mut samples, window_s, |sample| timestamp(sample));
    if x_key != "timestamp" {
        samples.sort_by(|left, right| {
            x_sort_value(left, x_key).total_cmp(&x_sort_value(right, x_key))
        });
    }
    samples
}

fn descriptor_stream(descriptor: &PlotDescriptor) -> String {
    descriptor
        .input
        .stream
        .clone()
        .or_else(|| telemetry_stream_for_expression(descriptor.input.x_expression.as_ref()))
        .or_else(|| {
            descriptor
                .input
                .y_expressions
                .iter()
                .find_map(|expression| telemetry_stream_for_expression(Some(expression)))
        })
        .or_else(|| telemetry_stream_for_expression(descriptor.input.series_by_expression.as_ref()))
        .or_else(|| {
            descriptor
                .input
                .available_streams
                .first()
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| "default".to_owned())
}

fn telemetry_stream_for_expression(expression: Option<&DataExpression>) -> Option<String> {
    match &expression?.source {
        DataSource::Telemetry { stream, .. } => Some(stream.clone()),
        _ => None,
    }
}

fn plain_series(
    samples: &[&TelemetrySample],
    y_keys: &[String],
    x_key: &str,
    z_index: &ZIndex,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    y_keys
        .iter()
        .map(|key| {
            let points = samples
                .iter()
                .map(|sample| PlotPoint {
                    x: x_value(sample, x_key),
                    y: sample.values.get(key).and_then(numeric_value),
                    z: z_index.resolve(sample),
                })
                .collect::<Vec<_>>();

            finish_plot_series(
                key.clone(),
                key.clone(),
                unit.map(ToOwned::to_owned),
                points,
                max_points,
                technique,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn grouped_series(
    samples: &[&TelemetrySample],
    y_keys: &[String],
    x_key: &str,
    series_by: &str,
    z_index: &ZIndex,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    let mut groups: BTreeMap<String, Vec<&TelemetrySample>> = BTreeMap::new();
    samples.iter().for_each(|sample| {
        if let Some(value) = sample_value(sample, series_by).and_then(group_value) {
            groups.entry(value).or_default().push(sample);
        }
    });
    let mut sorted_groups = groups.into_iter().collect::<Vec<_>>();
    sorted_groups.sort_by(|(left, _), (right, _)| compare_series_group(left, right));

    y_keys
        .iter()
        .flat_map(|key| {
            sorted_groups.iter().map(move |(group_key, group_samples)| {
                let label = if y_keys.len() == 1 {
                    group_key.clone()
                } else {
                    format!("{key} {group_key}")
                };
                let points = group_samples
                    .iter()
                    .map(|sample| PlotPoint {
                        x: x_value(sample, x_key),
                        y: sample.values.get(key).and_then(numeric_value),
                        z: z_index.resolve(sample),
                    })
                    .collect::<Vec<_>>();
                let series_key = if y_keys.len() == 1 {
                    group_key.clone()
                } else {
                    format!("{key}:{group_key}")
                };

                finish_plot_series(
                    series_key,
                    label,
                    unit.map(ToOwned::to_owned),
                    points,
                    max_points,
                    technique,
                )
            })
        })
        .collect()
}

fn aggregate_series(
    samples: &[&TelemetrySample],
    request: &PlotPayloadRequest,
    z_index: &ZIndex,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Option<Vec<PlotSeries>> {
    let group_expression = request
        .x_expression
        .as_ref()
        .or(request.series_by_expression.as_ref())?;
    let y_expressions = request
        .y_expressions
        .iter()
        .filter_map(|expression| {
            let method = aggregate_method(expression)?;
            let label = telemetry_field_name(expression)
                .or_else(|| expression_field_name(expression))
                .unwrap_or("value")
                .to_owned();
            Some((label, expression, method))
        })
        .collect::<Vec<_>>();
    if y_expressions.is_empty() {
        return None;
    }

    let mut grouped: BTreeMap<String, Vec<&TelemetrySample>> = BTreeMap::new();
    for sample in samples {
        if let Some(group) = expression_group_value(sample, group_expression) {
            grouped.entry(group).or_default().push(sample);
        }
    }
    if grouped.is_empty() {
        return Some(Vec::new());
    }

    let mut sorted_groups = grouped.into_iter().collect::<Vec<_>>();
    sorted_groups.sort_by(|(left, _), (right, _)| compare_series_group(left, right));

    let series = y_expressions
        .into_iter()
        .map(|(label, expression, method)| {
            let folded = fold_groups(
                sorted_groups
                    .iter()
                    .map(|(group, group_samples)| {
                        let values = group_samples
                            .iter()
                            .filter_map(|sample| expression_numeric_value(sample, expression))
                            .collect::<Vec<_>>();
                        (group.clone(), values)
                    })
                    .collect(),
                method,
            );
            let points = sorted_groups
                .iter()
                .zip(folded)
                .map(|((group, group_samples), (_, y))| {
                    let representative = group_samples.last().copied();
                    PlotPoint {
                        x: JsonValue::String(group.clone()),
                        y,
                        z: representative.and_then(|sample| z_index.resolve(sample)),
                    }
                })
                .collect::<Vec<_>>();

            finish_plot_series(
                label.clone(),
                label,
                unit.map(ToOwned::to_owned),
                points,
                max_points,
                technique,
            )
        })
        .collect::<Vec<_>>();
    Some(series)
}

fn row_expression_series(
    samples: &[&TelemetrySample],
    request: &PlotPayloadRequest,
    x_key: &str,
    z_index: &ZIndex,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Option<Vec<PlotSeries>> {
    let y_expressions = request
        .y_expressions
        .iter()
        .filter(|expression| is_row_expression(expression))
        .collect::<Vec<_>>();
    if y_expressions.is_empty() {
        return None;
    }

    let series = y_expressions
        .into_iter()
        .flat_map(|expression| {
            let label = expression_field_name(expression)
                .unwrap_or("value")
                .to_owned();
            let context = RowExpressionRenderContext {
                samples,
                x_expression: request.x_expression.as_ref(),
                x_key,
                z_index,
                unit,
                max_points,
                technique,
            };
            if let Some(series_by) = request.series_by_expression.as_ref() {
                grouped_row_expression_series(expression, series_by, &label, &context)
            } else {
                vec![plain_row_expression_series(expression, &label, &context)]
            }
        })
        .collect::<Vec<_>>();

    Some(series)
}

fn is_row_expression(expression: &DataExpression) -> bool {
    matches!(
        expression.source,
        DataSource::Telemetry { .. } | DataSource::Dimension { .. } | DataSource::Parameter { .. }
    ) && expression.transforms.is_empty()
}

fn plain_row_expression_series(
    expression: &DataExpression,
    label: &str,
    context: &RowExpressionRenderContext<'_>,
) -> PlotSeries {
    let points = context
        .samples
        .iter()
        .filter_map(|sample| {
            Some(PlotPoint {
                x: expression_x_value(sample, context.x_expression, context.x_key),
                y: Some(expression_numeric_value(sample, expression)?),
                z: context.z_index.resolve(sample),
            })
        })
        .collect::<Vec<_>>();

    finish_plot_series(
        label.to_owned(),
        label.to_owned(),
        context.unit.map(ToOwned::to_owned),
        points,
        context.max_points,
        context.technique,
    )
}

fn grouped_row_expression_series(
    expression: &DataExpression,
    series_by: &DataExpression,
    label: &str,
    context: &RowExpressionRenderContext<'_>,
) -> Vec<PlotSeries> {
    let mut grouped = BTreeMap::<String, Vec<PlotPoint>>::new();
    for sample in context.samples {
        let Some(group) = expression_group_value(sample, series_by) else {
            continue;
        };
        let Some(y) = expression_numeric_value(sample, expression) else {
            continue;
        };
        grouped.entry(group).or_default().push(PlotPoint {
            x: expression_x_value(sample, context.x_expression, context.x_key),
            y: Some(y),
            z: context.z_index.resolve(sample),
        });
    }

    grouped
        .into_iter()
        .map(|(group, points)| {
            finish_plot_series(
                format!("{label}:{group}"),
                group,
                context.unit.map(ToOwned::to_owned),
                points,
                context.max_points,
                context.technique,
            )
        })
        .collect()
}

fn expression_x_value(
    sample: &TelemetrySample,
    expression: Option<&DataExpression>,
    x_key: &str,
) -> JsonValue {
    let Some(expression) = expression else {
        return x_value(sample, x_key);
    };
    match &expression.source {
        DataSource::Telemetry { stream, field }
            if stream_matches_alias(sample.stream.as_str(), stream) =>
        {
            if field == "timestamp" {
                JsonValue::String(sample.timestamp.to_string())
            } else {
                value_to_json(sample_value(sample, field))
            }
        }
        DataSource::Dimension { name } | DataSource::Parameter { name } => {
            value_to_json(sample_value(sample, name))
        }
        _ => x_value(sample, x_key),
    }
}

fn value_to_json(value: Option<&Value>) -> JsonValue {
    value.map(plot_json_value).unwrap_or(JsonValue::Null)
}

fn plot_json_value(value: &Value) -> JsonValue {
    match value {
        Value::Datetime(timestamp) => JsonValue::String(crate::core::datetime_text(*timestamp)),
        value => value.to_plain_json().unwrap_or(JsonValue::Null),
    }
}

fn expression_numeric_value(sample: &TelemetrySample, expression: &DataExpression) -> Option<f64> {
    match &expression.source {
        DataSource::Telemetry { stream, field }
            if stream_matches_alias(sample.stream.as_str(), stream) =>
        {
            sample_value(sample, field).and_then(numeric_value)
        }
        DataSource::Dimension { name: field } | DataSource::Parameter { name: field } => {
            sample_value(sample, field).and_then(numeric_value)
        }
        DataSource::Telemetry { .. } => None,
        _ => None,
    }
}

fn expression_field_name(expression: &DataExpression) -> Option<&str> {
    match &expression.source {
        DataSource::Telemetry { field, .. } => Some(field),
        DataSource::Dimension { name } | DataSource::Parameter { name } => Some(name),
        DataSource::Record { record_type } => Some(record_type),
        DataSource::Artifact { artifact } => Some(artifact),
    }
}

fn telemetry_field_name(expression: &DataExpression) -> Option<&str> {
    match &expression.source {
        DataSource::Telemetry { field, .. } => Some(field),
        _ => None,
    }
}

fn plot_context(
    request: &PlotPayloadRequest,
    samples: &[&TelemetrySample],
    telemetry: &[TelemetrySample],
) -> BTreeMap<String, Value> {
    if request.plot_type.as_ref() != Some(&PlotType::WifiChannelOccupancy) {
        return BTreeMap::new();
    }

    let mut context = BTreeMap::new();
    let occupancy_values = samples.last().map(|sample| &sample.values);
    let wifi_values = telemetry
        .iter()
        .rev()
        .find(|sample| sample.stream.as_str() == "host.wifi")
        .map(|sample| &sample.values);

    copy_context_value(
        &mut context,
        "active_wifi_channel",
        occupancy_values
            .and_then(|values| values.get("active_wifi_channel"))
            .or_else(|| wifi_values.and_then(|values| values.get("active_wifi_channel")))
            .or_else(|| wifi_values.and_then(|values| values.get("wifi_channel"))),
    );
    copy_context_value(
        &mut context,
        "active_wifi_band",
        occupancy_values
            .and_then(|values| values.get("active_wifi_band"))
            .or_else(|| wifi_values.and_then(|values| values.get("active_wifi_band")))
            .or_else(|| wifi_values.and_then(|values| values.get("wifi_band"))),
    );
    copy_context_value(
        &mut context,
        "active_wifi_ssid",
        occupancy_values
            .and_then(|values| values.get("active_wifi_ssid"))
            .or_else(|| wifi_values.and_then(|values| values.get("wifi_ssid"))),
    );

    context
}

fn copy_context_value(context: &mut BTreeMap<String, Value>, key: &str, value: Option<&Value>) {
    if let Some(value) = value.filter(|value| traceable_context_value(value)) {
        context.insert(key.to_owned(), value.clone());
    }
}

fn traceable_context_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Text(value) => !value.trim().is_empty(),
        _ => true,
    }
}

pub(crate) fn sample_matches_filters(sample: &TelemetrySample, filters: &[DocumentFilter]) -> bool {
    filters
        .iter()
        .all(|filter| sample_matches_filter(sample, filter))
}

fn sample_matches_filter(sample: &TelemetrySample, filter: &DocumentFilter) -> bool {
    let actual = filter_expression_value(sample, &filter.expression);
    match filter.operator {
        FilterOperator::Exists => actual.is_some_and(traceable_context_value),
        FilterOperator::Eq => filter
            .value
            .as_ref()
            .is_some_and(|expected| actual.is_some_and(|actual| values_equal(actual, expected))),
        FilterOperator::Ne => filter
            .value
            .as_ref()
            .is_none_or(|expected| actual.is_none_or(|actual| !values_equal(actual, expected))),
        FilterOperator::Gt => compare_filter_value(actual, filter.value.as_ref())
            .is_some_and(|ordering| matches!(ordering, std::cmp::Ordering::Greater)),
        FilterOperator::Gte => {
            compare_filter_value(actual, filter.value.as_ref()).is_some_and(|ordering| {
                matches!(
                    ordering,
                    std::cmp::Ordering::Greater | std::cmp::Ordering::Equal
                )
            })
        }
        FilterOperator::Lt => compare_filter_value(actual, filter.value.as_ref())
            .is_some_and(|ordering| matches!(ordering, std::cmp::Ordering::Less)),
        FilterOperator::Lte => {
            compare_filter_value(actual, filter.value.as_ref()).is_some_and(|ordering| {
                matches!(
                    ordering,
                    std::cmp::Ordering::Less | std::cmp::Ordering::Equal
                )
            })
        }
        FilterOperator::Contains => filter
            .value
            .as_ref()
            .is_some_and(|expected| actual.is_some_and(|actual| value_contains(actual, expected))),
    }
}

fn filter_expression_value<'a>(
    sample: &'a TelemetrySample,
    expression: &DataExpression,
) -> Option<&'a Value> {
    match &expression.source {
        DataSource::Telemetry { stream, field }
            if stream_matches_alias(sample.stream.as_str(), stream) =>
        {
            sample_value(sample, field)
        }
        DataSource::Dimension { name } | DataSource::Parameter { name } => {
            sample_value(sample, name)
        }
        _ => None,
    }
}

fn values_equal(actual: &Value, expected: &Value) -> bool {
    if let (Some(actual), Some(expected)) = (numeric_value(actual), numeric_value(expected)) {
        return (actual - expected).abs() <= f64::EPSILON;
    }
    actual == expected
}

fn compare_filter_value(
    actual: Option<&Value>,
    expected: Option<&Value>,
) -> Option<std::cmp::Ordering> {
    let actual = actual?;
    let expected = expected?;
    match (numeric_value(actual), numeric_value(expected)) {
        (Some(actual), Some(expected)) => actual.partial_cmp(&expected),
        _ => match (group_value(actual), group_value(expected)) {
            (Some(actual), Some(expected)) => Some(actual.cmp(&expected)),
            _ => None,
        },
    }
}

fn value_contains(actual: &Value, expected: &Value) -> bool {
    match actual {
        Value::Text(actual) => {
            group_value(expected).is_some_and(|expected| actual.contains(expected.as_str()))
        }
        Value::Array(values) => values
            .iter()
            .any(|candidate| values_equal(candidate, expected)),
        _ => false,
    }
}

fn stride_downsample(points: Vec<PlotPoint>, max_points: usize) -> (Vec<PlotPoint>, bool) {
    let indices = harmos_signal::decimate(points.len(), max_points);
    let mut selected = indices.into_iter().peekable();
    let sampled = points
        .into_iter()
        .enumerate()
        .filter_map(|(index, point)| {
            (selected.peek() == Some(&index)).then(|| {
                selected.next();
                point
            })
        })
        .collect();
    (sampled, true)
}

fn numeric_downsample(
    points: &[PlotPoint],
    max_points: usize,
    technique: DownsampleTechnique,
) -> Option<Vec<PlotPoint>> {
    let values = points
        .iter()
        .map(|point| point.y.filter(|value| value.is_finite()))
        .collect::<Option<Vec<f64>>>()?;
    let mut indices = match technique {
        DownsampleTechnique::MinMax => harmos_signal::extrema(&values, (max_points / 2).max(1)),
        DownsampleTechnique::Lttb => {
            let x = points
                .iter()
                .map(analyses::point_x)
                .collect::<Option<Vec<_>>>()?;
            if max_points < 3 {
                return None;
            }
            harmos_signal::lttb(&x, &values, max_points)
        }
        DownsampleTechnique::Raw | DownsampleTechnique::Auto | DownsampleTechnique::Unspecified => {
            return None;
        }
    };

    // The algorithms emit coordinates in ascending order, but min/max within a
    // bucket can repeat or invert; normalize to ascending, unique indices.
    indices.sort_unstable();
    indices.dedup();
    Some(
        indices
            .into_iter()
            .filter_map(|index| points.get(index).cloned())
            .collect(),
    )
}

fn plot_markers(
    records: &[RecordEntry],
    marker_record_types: &[String],
    scope: Option<&RecordScope>,
) -> Vec<PlotMarker> {
    if marker_record_types.is_empty() {
        return Vec::new();
    }
    records
        .iter()
        .filter(|record| {
            marker_record_types
                .iter()
                .any(|value| record.record_type.as_str() == value)
        })
        .filter(|record| sample_matches_scope(record.scope.as_ref(), scope))
        .map(|record| {
            let record_type = record.record_type.to_string();
            PlotMarker {
                x: JsonValue::String(record.timestamp.to_string()),
                label: record
                    .message
                    .clone()
                    .unwrap_or_else(|| record_type.clone()),
                color: marker_color(&record_type),
                record_type,
            }
        })
        .collect()
}

fn z_index(
    telemetry: &[TelemetrySample],
    primary_stream: &str,
    z_stream: Option<&str>,
    z_field: Option<&str>,
    scope: Option<&RecordScope>,
) -> ZIndex {
    let Some(field) = z_field else {
        return ZIndex::None;
    };
    let stream = z_stream.unwrap_or(primary_stream);
    if stream_matches_alias(primary_stream, stream) {
        return ZIndex::Primary {
            field: field.to_owned(),
        };
    }
    let mut samples = telemetry
        .iter()
        .filter(|sample| {
            stream_matches_alias(sample.stream.as_str(), stream)
                && sample_matches_scope(sample.scope.as_ref(), scope)
        })
        .filter_map(|sample| {
            Some(ZSample {
                scope: sample.scope.clone(),
                timestamp: timestamp(sample)?,
                value: sample_value(sample, field).and_then(numeric_value)?,
            })
        })
        .collect::<Vec<_>>();
    samples.sort_by_key(|sample| sample.timestamp);

    ZIndex::Stream { samples }
}

fn x_value(sample: &TelemetrySample, x_key: &str) -> JsonValue {
    if x_key == "timestamp" {
        return JsonValue::String(sample.timestamp.to_string());
    }
    sample
        .values
        .get(x_key)
        .map(plot_json_value)
        .unwrap_or(JsonValue::Null)
}

fn x_sort_value(sample: &TelemetrySample, x_key: &str) -> f64 {
    sample
        .values
        .get(x_key)
        .and_then(|value| numeric_value(value).or_else(|| datetime_seconds(value)))
        .unwrap_or(f64::NAN)
}

fn group_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Text(value) if value.trim().is_empty() => None,
        Value::Text(value) => Some(value.clone()),
        Value::Integer(value) => Some(value.to_string()),
        Value::Unsigned(value) => Some(value.to_string()),
        Value::Number(value) if value.is_finite() => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn timestamp(sample: &TelemetrySample) -> Option<OffsetDateTime> {
    parse_timestamp(sample.timestamp.as_str())
}

fn marker_color(record_type: &str) -> String {
    let hash = record_type.bytes().fold(0usize, |acc, byte| {
        acc.wrapping_mul(31).wrapping_add(usize::from(byte))
    });
    MARKER_COLORS[hash % MARKER_COLORS.len()].to_owned()
}

/* === Tests === */
