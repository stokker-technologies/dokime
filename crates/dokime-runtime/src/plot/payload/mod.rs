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
mod groups;
#[cfg(feature = "storage")]
mod projection;
#[cfg(feature = "storage")]
mod records;
#[cfg(feature = "storage")]
mod retained;
#[cfg(feature = "storage")]
pub(crate) mod spilled;
mod types;

/* === Re-Exports === */

pub use analyses::{
    BoxSummary, HistogramBin, HistogramSeries, HistogramStatistics, PlotAnalysis, SigmaBin, SigmaSeries, WifiBandScan, WifiBandWindow,
    WifiChannelCount, WifiScan,
};
#[cfg(feature = "storage")]
pub(crate) use retained::{
    finish as finish_retained_plot_payload, frame_owned as finish_plot_frame, project as project_retained_plot_payload,
};
pub use types::{
    DownsampleTechnique, PlotCartesianFrame, PlotCartesianMode, PlotCartesianSeriesFrame, PlotComputation, PlotComputedSeries, PlotDomain,
    PlotEvidence, PlotMarker, PlotPayload, PlotPayloadDiagnostics, PlotPayloadEnvelope, PlotPayloadRequest, PlotPayloadStreamKind,
    PlotPoint, PlotSeries, PlotSeriesSummary, PlotWindow,
};

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::Aggregate;
use serde_json::Value as JsonValue;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

// Local Imports
use crate::document::{DocumentFilter, FilterOperator, PlotDescriptor, PlotType};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::plot::record_timing::RECORD_CONTEXT_STREAM;
use crate::telemetry::{TelemetryColumn, TelemetrySample};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::scope_visible_in;
use dokime_domain::model::traceability::TraceabilitySnapshot;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueFormat;
use dokime_domain::model::values::datetime_seconds;

/* === Constants === */

const MARKER_COLORS: [&str; 8] = ["#ef4444", "#f97316", "#eab308", "#22c55e", "#14b8a6", "#3b82f6", "#8b5cf6", "#ec4899"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub(crate) struct PlotPayloadContext<'a> {
    pub default_max_points: usize,
    pub x_format: Option<&'a ValueFormat>,
    pub series_format: Option<&'a ValueFormat>,
    pub run_started_at: Option<&'a str>,
    pub cycle_started_at: Option<&'a str>,
}

#[derive(Clone, Debug)]
struct ZSample<'a> {
    scope: Option<&'a RecordScope>,
    timestamp: OffsetDateTime,
    value: f64,
    ordinal: usize,
}

/* === Enums === */

#[derive(Clone, Debug)]
enum ZIndex<'a> {
    None,
    Expression(&'a DataExpression),
    Primary { field: &'a str },
    Stream { samples: Vec<ZSample<'a>> },
}

#[derive(Clone, Copy)]
enum XInput<'a> {
    Missing,
    Timestamp(&'a str),
    Value(&'a Value),
}

/* === Traits === */

/* === Implementations === */

impl ZIndex<'_> {
    fn resolve(&self, sample: &TelemetrySample) -> Option<f64> {
        match self {
            Self::None => None,
            Self::Expression(expression) => expression_numeric_value(sample, expression),
            Self::Primary { field } => sample.values.get(*field).and_then(numeric_value),
            Self::Stream { samples, .. } => {
                let sample_time = timestamp(sample)?;
                samples
                    .iter()
                    .rev()
                    .find(|candidate| {
                        candidate.timestamp <= sample_time
                            && crate::evidence_runtime::scopes_correlate(candidate.scope, sample.scope.as_ref())
                    })
                    .map(|candidate| candidate.value)
            }
        }
    }
}

/* === Public Functions === */

pub(crate) fn plot_streams(request: &PlotPayloadRequest) -> Vec<String> {
    let mut count = 0;
    visit_plot_streams(request, |_| count += 1);
    let mut streams = Vec::with_capacity(count);
    visit_plot_streams(request, |name| streams.push(name.to_owned()));
    streams.sort_unstable();
    streams.dedup();
    streams
}

fn visit_plot_streams(request: &PlotPayloadRequest, mut visit: impl FnMut(&str)) {
    visit(&request.stream);
    visit(RECORD_CONTEXT_STREAM);
    if let Some(stream) = &request.z_stream {
        visit(stream);
    }
    let expressions = request
        .x_expression
        .iter()
        .chain(&request.y_expressions)
        .chain(&request.series_by_expression)
        .chain(&request.z_expression)
        .chain(request.filters.iter().map(|filter| &filter.expression));
    for expression in expressions {
        if let DataSource::Telemetry { stream, .. } = &expression.source {
            visit(stream);
        }
        for transform in &expression.transforms {
            if let DataTransform::TelemetryAtOrBefore { stream, .. } = transform {
                visit(stream);
            }
        }
    }
}

fn request_z_index<'a>(telemetry: &'a [TelemetrySample], request: &'a PlotPayloadRequest) -> ZIndex<'a> {
    match request.z_expression.as_ref() {
        Some(expression) => match &expression.source {
            DataSource::Telemetry { stream, field } => {
                self::z_index(telemetry, &request.stream, Some(stream), Some(field), request.scope.as_ref())
            }
            _ => ZIndex::Expression(expression),
        },
        None => z_index(telemetry, &request.stream, request.z_stream.as_deref(), request.z_field.as_deref(), request.scope.as_ref()),
    }
}

fn payload_from_series(
    samples: &[&TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest, generated_at: &str,
    format_context: &PlotPayloadContext, series: Vec<PlotSeries>,
) -> PlotPayload {
    let context = plot_context(request, samples);
    // Every series in one payload reads the same declared y axis, so they
    // share the one resolved winner: a plot override or a column's declared
    // format apply uniformly, not per series.
    let mut series = series;
    for entry in &mut series {
        entry.format = format_context.series_format.cloned();
    }

    PlotPayload {
        computed: None,
        stream: request.stream.clone(),
        x_key: request.x_key.as_deref().unwrap_or("timestamp").to_owned(),
        series_by: request.series_by.clone(),
        context,
        series,
        cartesian: None,
        analysis: None,
        markers: plot_markers(records, &request.marker_record_types, request.scope.as_ref()),
        max_points: request.max_points.unwrap_or(format_context.default_max_points),
        window_s: request.window_s,
        generated_at: generated_at.to_owned(),
        x_format: format_context.x_format.cloned(),
        run_started_at: format_context.run_started_at.map(str::to_owned),
        cycle_started_at: format_context.cycle_started_at.map(str::to_owned),
        summaries: Vec::new(),
    }
}

pub(crate) fn resolved_format<'a>(plot: Option<&'a ValueFormat>, column: Option<&'a TelemetryColumn>) -> Option<&'a ValueFormat> {
    plot.or_else(|| column.and_then(|column| column.format.as_ref()))
}

pub(crate) fn enrich_plot_samples_with_cycle_context(telemetry: &mut [TelemetrySample], execution: &TestExecution) {
    for sample in telemetry {
        if sample.scope.is_none() {
            continue;
        }
        let context = dimension_context_for_scope(execution);
        for (key, value) in context {
            sample.values.entry(key).or_insert(value);
        }
    }
}

pub(crate) fn enrich_plot_samples_with_traceability_context(telemetry: &mut [TelemetrySample], traceability: &[TraceabilitySnapshot]) {
    for sample in telemetry {
        let Some(scope) = sample.scope.as_ref() else {
            continue;
        };
        let context = traceability_context_for_scope(traceability, scope);
        for (key, value) in context {
            sample.values.entry(key).or_insert(value);
        }
    }
}

pub(crate) fn normalize_histogram_value(request: &mut PlotPayloadRequest) {
    if request.plot_type != Some(PlotType::Histogram) {
        return;
    }
    let expression = request.x_expression.take();
    let mut expressions = Vec::with_capacity(usize::from(expression.is_some()));
    expressions.extend(expression);
    request.y_expressions = expressions;
    let key = request.x_key.take().filter(|key| key != "timestamp" && request.y_expressions.is_empty());
    let mut keys = Vec::with_capacity(usize::from(key.is_some()));
    keys.extend(key);
    request.y_keys = keys;
    request.y_format = request.x_format.take();
}

pub(crate) fn normalize_analysis_fields(request: &mut PlotPayloadRequest) {
    if request.analysis.is_none() {
        return;
    }
    let field = |expression: &DataExpression| match &expression.source {
        DataSource::Telemetry { stream, field } if stream == &request.stream && expression.transforms.is_empty() => Some(field.clone()),
        _ => None,
    };
    let x = request.x_expression.as_ref().and_then(field);
    let y = if request.y_expressions.is_empty() {
        None
    } else {
        let mut keys = Vec::with_capacity(request.y_expressions.len());
        let mut valid = true;
        for expression in &request.y_expressions {
            let Some(key) = field(expression) else {
                valid = false;
                break;
            };
            keys.push(key);
        }
        valid.then_some(keys)
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

/// The query a descriptor resolves to: its declared request, mapped once
/// through [`canonical_plot_query`], the same mapping a live query applies to
/// the declared request a client sends back.
pub fn plot_payload_request_for_descriptor(
    descriptor: &PlotDescriptor, scope: Option<RecordScope>, max_points: Option<usize>, window_s: Option<f64>,
) -> PlotPayloadRequest {
    canonical_plot_query(declared_plot_request(descriptor, scope, max_points, window_s))
}

/// The query-time mapping of a declared request: a histogram's declared x
/// becomes the binned series, and simple analysis fields become field keys.
/// It runs exactly once per query; a mapped request is not a declaration.
pub(crate) fn canonical_plot_query(mut request: PlotPayloadRequest) -> PlotPayloadRequest {
    normalize_histogram_value(&mut request);
    normalize_analysis_fields(&mut request);
    request
}

/// A descriptor's request as the document declares it, before the query-time
/// mapping: what a document definition carries to a client, and what the
/// client's live watch sends back.
pub fn declared_plot_request(
    descriptor: &PlotDescriptor, scope: Option<RecordScope>, max_points: Option<usize>, window_s: Option<f64>,
) -> PlotPayloadRequest {
    PlotPayloadRequest {
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
        max_points: max_points.or(descriptor.input.display_points.map(|points| points as usize)),
        window_s,
        scope,
        technique: descriptor.input.downsample,
        summary_window: None,
    }
}

pub(crate) fn plot_payload_diagnostics(payload: &PlotPayload) -> PlotPayloadDiagnostics {
    if let Some(computed) = &payload.computed {
        return PlotPayloadDiagnostics {
            series_count: computed.series.iter().map(|series| series.output.outputs.len()).sum(),
            point_count: computed
                .series
                .iter()
                .flat_map(|series| {
                    series.output.outputs.iter().map(|output| series.display_indices.get(&output.key).map_or(output.values.len(), Vec::len))
                })
                .sum(),
            raw_point_count: computed.series.iter().map(|series| series.input_samples).sum(),
            downsampled: computed.series.iter().any(|series| !series.display_indices.is_empty()),
            max_points: payload.max_points,
            window_s: payload.window_s,
        };
    }
    PlotPayloadDiagnostics {
        series_count: payload.series.len(),
        point_count: payload.series.iter().map(|series| series.points.len()).sum(),
        raw_point_count: payload.series.iter().map(|series| series.raw_points).sum(),
        downsampled: payload.series.iter().any(|series| series.downsampled),
        max_points: payload.max_points,
        window_s: payload.window_s,
    }
}

pub(crate) fn plot_window_duration(window_s: f64) -> Option<time::Duration> {
    if window_s < 0.0 {
        return None;
    }
    time::Duration::checked_seconds_f64(window_s)
}

pub(crate) fn plot_window_cutoff(latest: OffsetDateTime, window_s: Option<f64>) -> Option<OffsetDateTime> {
    latest.checked_sub(plot_window_duration(window_s?)?)
}

pub(crate) fn aggregate_method(expression: &DataExpression) -> Option<Aggregate> {
    expression.transforms.iter().find_map(|transform| match transform {
        DataTransform::Aggregate(aggregate) => Some(*aggregate),
        DataTransform::TimeBetween { .. } => None,
        DataTransform::TelemetryAtOrBefore { .. } => None,
        // Neither folds the values of one series: `detail` selects what is
        // read off a record, and `per_case` widens which executions are
        // read, before any aggregate over the result.
        DataTransform::Detail { .. } => None,
        DataTransform::PerCase => None,
    })
}

fn expression_group_input<'a>(sample: &'a TelemetrySample, expression: &DataExpression) -> Option<&'a Value> {
    match &expression.source {
        DataSource::Telemetry { stream, field } if stream_matches_alias(sample.stream.as_str(), stream) => sample_value(sample, field),
        DataSource::Dimension { name: field } | DataSource::Parameter { name: field } => sample_value(sample, field),
        DataSource::Telemetry { .. } => None,
        _ => None,
    }
}

pub(crate) fn sample_value<'a>(sample: &'a TelemetrySample, key: &str) -> Option<&'a Value> {
    sample.values.get(key).or_else(|| key.rsplit_once('.').and_then(|(_, field)| sample.values.get(field)))
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
    let left_number = left.parse::<f64>().ok().filter(|value| value.is_finite());
    let right_number = right.parse::<f64>().ok().filter(|value| value.is_finite());
    match (left_number, right_number) {
        (Some(left_number), Some(right_number)) => {
            left_number.partial_cmp(&right_number).expect("finite group numbers compare").then_with(|| left.cmp(right))
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.cmp(right),
    }
}

pub(crate) fn stream_matches_alias(actual: &str, declared: &str) -> bool {
    actual == declared
        || actual.strip_suffix(declared).is_some_and(|prefix| prefix.ends_with('.'))
        || declared.strip_suffix(actual).is_some_and(|prefix| prefix.ends_with('.'))
}

pub(crate) fn sample_matches_scope(sample_scope: Option<&RecordScope>, filter: Option<&RecordScope>) -> bool {
    filter.is_none_or(|filter| scope_visible_in(sample_scope, filter))
}

pub(crate) fn sample_matches_filters(sample: &TelemetrySample, filters: &[DocumentFilter]) -> bool {
    filters.iter().all(|filter| sample_matches_filter(sample, filter))
}

/* === Private Functions === */

fn dimension_context_for_scope(execution: &TestExecution) -> BTreeMap<String, Value> {
    let mut context = BTreeMap::new();
    visit_dimension_context(execution, |key, value| insert_context_value(&mut context, key, value));
    context
}

fn visit_dimension_context(execution: &TestExecution, mut visit: impl FnMut(&str, &Value)) {
    for (key, value) in &execution.snapshot.case().parameters {
        visit(key, value);
    }
    // Match The Sorted Effective Map Without Cloning It.
    let mut inputs = execution.snapshot.parameters().iter().peekable();
    let mut outputs = execution.output_parameters.iter().peekable();
    loop {
        let next = match (inputs.peek(), outputs.peek()) {
            (Some((left, _)), Some((right, _))) => match left.cmp(right) {
                std::cmp::Ordering::Less => inputs.next(),
                std::cmp::Ordering::Equal => {
                    inputs.next();
                    outputs.next()
                }
                std::cmp::Ordering::Greater => outputs.next(),
            },
            (Some(_), None) => inputs.next(),
            (None, Some(_)) => outputs.next(),
            (None, None) => break,
        };
        if let Some((key, parameter)) = next {
            visit(key, &parameter.value);
        }
    }
}
fn traceability_context_for_scope(traceability: &[TraceabilitySnapshot], scope: &RecordScope) -> BTreeMap<String, Value> {
    let mut context = BTreeMap::new();

    for snapshot in traceability {
        if !scope_visible_in(Some(&snapshot.scope), scope) {
            continue;
        }
        for entry in &snapshot.entries {
            insert_context_value(&mut context, entry.metadata.identifier.as_str(), &entry.value);
        }
    }

    context
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

pub(super) fn sort_plot_samples(samples: &mut [&TelemetrySample], x_key: &str) {
    // Every Reference Comes From The Same Input Slice. Address Order Is Its
    // Capture Order, Preserving Stable Ties Without A Sort Scratch Allocation.
    let capture_order =
        |left: &&TelemetrySample, right: &&TelemetrySample| std::ptr::from_ref(*left).addr().cmp(&std::ptr::from_ref(*right).addr());
    samples.sort_unstable_by(|left, right| left.timestamp.cmp(&right.timestamp).then_with(|| capture_order(left, right)));
    if x_key != "timestamp" {
        samples.sort_unstable_by(|left, right| {
            x_sort_value(left, x_key)
                .total_cmp(&x_sort_value(right, x_key))
                .then_with(|| left.timestamp.cmp(&right.timestamp))
                .then_with(|| capture_order(left, right))
        });
    }
}

fn descriptor_stream(descriptor: &PlotDescriptor) -> String {
    descriptor
        .input
        .stream
        .clone()
        .or_else(|| telemetry_stream_for_expression(descriptor.input.x_expression.as_ref()))
        .or_else(|| descriptor.input.y_expressions.iter().find_map(|expression| telemetry_stream_for_expression(Some(expression))))
        .or_else(|| telemetry_stream_for_expression(descriptor.input.series_by_expression.as_ref()))
        .or_else(|| descriptor.input.available_streams.first().map(ToString::to_string))
        .unwrap_or_else(|| "default".to_owned())
}

fn telemetry_stream_for_expression(expression: Option<&DataExpression>) -> Option<String> {
    match &expression?.source {
        DataSource::Telemetry { stream, .. } => Some(stream.clone()),
        _ => None,
    }
}

fn is_row_expression(expression: &DataExpression) -> bool {
    matches!(expression.source, DataSource::Telemetry { .. } | DataSource::Dimension { .. } | DataSource::Parameter { .. })
        && expression.transforms.is_empty()
}

fn expression_x_input<'a>(sample: &'a TelemetrySample, expression: Option<&DataExpression>, x_key: &str) -> XInput<'a> {
    let Some(expression) = expression else {
        return x_input(sample, x_key);
    };
    match &expression.source {
        DataSource::Telemetry { stream, field } if stream_matches_alias(sample.stream.as_str(), stream) => {
            if field == "timestamp" {
                XInput::Timestamp(sample.timestamp.as_str())
            } else {
                sample_value(sample, field).map_or(XInput::Missing, XInput::Value)
            }
        }
        DataSource::Dimension { name } | DataSource::Parameter { name } => {
            sample_value(sample, name).map_or(XInput::Missing, XInput::Value)
        }
        _ => x_input(sample, x_key),
    }
}

fn expression_numeric_value(sample: &TelemetrySample, expression: &DataExpression) -> Option<f64> {
    match &expression.source {
        DataSource::Telemetry { stream, field } if stream_matches_alias(sample.stream.as_str(), stream) => {
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

fn plot_context(request: &PlotPayloadRequest, samples: &[&TelemetrySample]) -> BTreeMap<String, Value> {
    if request.plot_type.as_ref() != Some(&PlotType::WifiChannelOccupancy) {
        return BTreeMap::new();
    }

    let mut context = BTreeMap::new();
    for (key, value) in plot_context_values(samples) {
        copy_context_value(&mut context, key, value);
    }
    context
}

fn plot_context_values<'a>(samples: &[&'a TelemetrySample]) -> [(&'static str, Option<&'a Value>); 3] {
    let occupancy_values = samples.last().map(|sample| &sample.values);

    [
        ("active_wifi_channel", occupancy_values.and_then(|values| values.get("active_wifi_channel"))),
        ("active_wifi_band", occupancy_values.and_then(|values| values.get("active_wifi_band"))),
        ("active_wifi_ssid", occupancy_values.and_then(|values| values.get("active_wifi_ssid"))),
    ]
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

fn sample_matches_filter(sample: &TelemetrySample, filter: &DocumentFilter) -> bool {
    let actual = filter_expression_value(sample, &filter.expression);
    match filter.operator {
        FilterOperator::Exists => actual.is_some_and(traceable_context_value),
        FilterOperator::Eq => filter.value.as_ref().is_some_and(|expected| actual.is_some_and(|actual| values_equal(actual, expected))),
        FilterOperator::Ne => filter.value.as_ref().is_none_or(|expected| actual.is_none_or(|actual| !values_equal(actual, expected))),
        FilterOperator::Gt => {
            compare_filter_value(actual, filter.value.as_ref()).is_some_and(|ordering| matches!(ordering, std::cmp::Ordering::Greater))
        }
        FilterOperator::Gte => compare_filter_value(actual, filter.value.as_ref())
            .is_some_and(|ordering| matches!(ordering, std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
        FilterOperator::Lt => {
            compare_filter_value(actual, filter.value.as_ref()).is_some_and(|ordering| matches!(ordering, std::cmp::Ordering::Less))
        }
        FilterOperator::Lte => compare_filter_value(actual, filter.value.as_ref())
            .is_some_and(|ordering| matches!(ordering, std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
        FilterOperator::Contains => {
            filter.value.as_ref().is_some_and(|expected| actual.is_some_and(|actual| value_contains(actual, expected)))
        }
    }
}

fn filter_expression_value<'a>(sample: &'a TelemetrySample, expression: &DataExpression) -> Option<&'a Value> {
    match &expression.source {
        DataSource::Telemetry { stream, field } if stream_matches_alias(sample.stream.as_str(), stream) => sample_value(sample, field),
        DataSource::Dimension { name } | DataSource::Parameter { name } => sample_value(sample, name),
        _ => None,
    }
}

fn values_equal(actual: &Value, expected: &Value) -> bool {
    if let (Some(actual), Some(expected)) = (numeric_value(actual), numeric_value(expected)) {
        return (actual - expected).abs() <= f64::EPSILON;
    }
    actual == expected
}

fn compare_filter_value(actual: Option<&Value>, expected: Option<&Value>) -> Option<std::cmp::Ordering> {
    let actual = actual?;
    let expected = expected?;
    match (numeric_value(actual), numeric_value(expected)) {
        (Some(actual), Some(expected)) => actual.partial_cmp(&expected),
        _ => groups::compare(actual, expected),
    }
}

fn value_contains(actual: &Value, expected: &Value) -> bool {
    match actual {
        Value::Text(actual) => groups::contains(actual, expected),
        Value::Array(values) => values.iter().any(|candidate| values_equal(candidate, expected)),
        _ => false,
    }
}

fn plot_markers(records: crate::records::view::View<'_>, marker_record_types: &[String], scope: Option<&RecordScope>) -> Vec<PlotMarker> {
    if marker_record_types.is_empty() {
        return Vec::new();
    }
    let selected = records
        .iter()
        .filter(|record| marker_record_types.iter().any(|value| record.record_type.as_str() == value))
        .filter(|record| sample_matches_scope(record.scope.as_ref(), scope));
    let mut markers = Vec::with_capacity(selected.clone().count());
    markers.extend(selected.map(|record| {
        let record_type = record.record_type.as_str().to_owned();
        PlotMarker {
            x: JsonValue::String(record.timestamp.as_str().to_owned()),
            label: record.message.clone().unwrap_or_else(|| record_type.clone()),
            color: marker_color(&record_type),
            record_type,
        }
    }));
    markers
}

fn z_index<'a>(
    telemetry: &'a [TelemetrySample], primary_stream: &str, z_stream: Option<&str>, z_field: Option<&'a str>, scope: Option<&RecordScope>,
) -> ZIndex<'a> {
    let Some(field) = z_field else {
        return ZIndex::None;
    };
    let stream = z_stream.unwrap_or(primary_stream);
    if stream_matches_alias(primary_stream, stream) {
        return ZIndex::Primary { field };
    }
    let selected = z_samples(telemetry, stream, field, scope);
    let mut samples = Vec::with_capacity(selected.clone().count());
    samples.extend(selected);
    samples.sort_unstable_by(|left, right| left.timestamp.cmp(&right.timestamp).then(left.ordinal.cmp(&right.ordinal)));
    ZIndex::Stream { samples }
}

fn z_samples<'a>(
    telemetry: &'a [TelemetrySample], stream: &str, field: &str, scope: Option<&RecordScope>,
) -> impl Iterator<Item = ZSample<'a>> + Clone {
    telemetry
        .iter()
        .enumerate()
        .filter(move |(_, sample)| {
            stream_matches_alias(sample.stream.as_str(), stream) && sample_matches_scope(sample.scope.as_ref(), scope)
        })
        .filter_map(move |(ordinal, sample)| {
            Some(ZSample {
                scope: sample.scope.as_ref(),
                timestamp: timestamp(sample)?,
                value: sample_value(sample, field).and_then(numeric_value)?,
                ordinal,
            })
        })
}

fn x_input<'a>(sample: &'a TelemetrySample, x_key: &str) -> XInput<'a> {
    if x_key == "timestamp" {
        return XInput::Timestamp(sample.timestamp.as_str());
    }
    sample.values.get(x_key).map_or(XInput::Missing, XInput::Value)
}

fn x_sort_value(sample: &TelemetrySample, x_key: &str) -> f64 {
    sample.values.get(x_key).and_then(|value| numeric_value(value).or_else(|| datetime_seconds(value))).unwrap_or(f64::NAN)
}

fn timestamp(sample: &TelemetrySample) -> Option<OffsetDateTime> {
    parse_timestamp(sample.timestamp.as_str())
}

fn marker_color(record_type: &str) -> String {
    marker_color_text(record_type).to_owned()
}

fn marker_color_text(record_type: &str) -> &'static str {
    let hash = record_type.bytes().fold(0usize, |acc, byte| acc.wrapping_mul(31).wrapping_add(usize::from(byte)));
    MARKER_COLORS[hash % MARKER_COLORS.len()]
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentSelection, Plot, plot_descriptor_for_definition, plot_input_for_definition};
    use dokime_domain::model::expressions::telemetry;

    #[test]
    fn analysis_fields_normalize_without_a_second_output_limit() {
        let fields = (0..33).map(|index| format!("field_{index}")).collect::<Vec<_>>();
        let request = PlotPayloadRequest {
            analysis: Some(crate::analysis::Selection {
                id: "fixture.analysis".into(),
                version: 1,
                parameters: BTreeMap::new(),
            }),
            stream: "fixture.stream".into(),
            y_expressions: fields.iter().map(|field| telemetry("fixture.stream", field)).collect(),
            ..Default::default()
        };
        let normalized = canonical_plot_query(request);
        assert_eq!(normalized.y_keys, fields);
        assert!(normalized.y_expressions.is_empty());
    }
    // The obc startup suite's histogram declares its binned value on x. The
    // report maps the descriptor once; the live view receives the declared
    // request in its document and sends it back, and the live query maps that
    // once. Both bin the same input, so neither reports "none was declared".
    #[test]
    fn histogram_binning_inputs() {
        let mut plot = Plot::histogram("Voltage Distribution");
        plot.id("obc.startup.voltage_distribution");
        plot.unit("V");
        plot.x(telemetry("obc.startup", "voltage_v"));
        let input = plot_input_for_definition(&plot, &[], 0, &DocumentSelection::default());
        let descriptor = plot_descriptor_for_definition(input, plot.metadata.clone());

        let report = plot_payload_request_for_descriptor(&descriptor, None, None, None);
        let declared = declared_plot_request(&descriptor, None, None, None);
        let live = canonical_plot_query(declared.clone());

        assert!(declared.x_expression.is_some(), "the document carries the declared x, not the mapped request");
        assert_eq!((&live.x_key, &live.y_keys, &live.y_expressions), (&report.x_key, &report.y_keys, &report.y_expressions));
        assert!(!report.y_keys.is_empty() || !report.y_expressions.is_empty(), "the declared x is the binned series");
        assert!(matches!(analyses::analyze(&mut [], &live), Some(PlotAnalysis::Histogram { .. })), "the live query bins");
        assert!(matches!(analyses::analyze(&mut [], &report), Some(PlotAnalysis::Histogram { .. })), "the report bins");
    }
}
