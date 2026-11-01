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
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{self, Write};
use std::ops::Range;

// Third Party Imports
use harmos::signal::{Aggregates, Series};
use harmos::storage::ReadBatch;

// Local Imports
use super::projection::{memory, series_entry, text, vector, x_value};
use super::{PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotPoint, PlotSeries};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::runtime::Result;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::values::Value;

/* === Constants === */

// Even The Exact Decimal Expansion Of The Smallest Binary Subnormal Fits:
// Its Fraction Has MANTISSA_DIGITS - MIN_EXP Places, Plus Sign And "0.".
const SCALAR_TEXT_BYTES: usize = (f64::MANTISSA_DIGITS as i32 - f64::MIN_EXP) as usize + 3;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ScalarText {
    bytes: [u8; SCALAR_TEXT_BYTES],
    len: usize,
}

struct TextLength(usize);

struct GroupRow<'a> {
    key: Cow<'a, str>,
    sample: &'a TelemetrySample,
    ordinal: usize,
}

struct GroupRows<'a> {
    selected: Vec<&'a TelemetrySample>,
    rows: Vec<GroupRow<'a>>,
    groups: Vec<Range<usize>>,
}

/* === Enums === */

#[derive(Clone, Copy)]
enum Source<'a> {
    Field(&'a str),
    Expression(&'a DataExpression),
}

#[derive(Clone, Copy)]
enum Operation<'a> {
    Rows(Source<'a>),
    Aggregate(&'a DataExpression),
}

#[derive(Clone, Copy)]
enum Input<'a> {
    Field(&'a str),
    Expression(&'a DataExpression),
}

#[derive(Clone)]
enum Inputs<'a> {
    Fields(std::slice::Iter<'a, String>),
    Expressions(std::slice::Iter<'a, DataExpression>),
}

/* === Traits === */

/* === Implementations === */

impl Source<'_> {
    fn input(self, sample: &TelemetrySample) -> Option<&Value> {
        match self {
            Self::Field(field) => super::sample_value(sample, field),
            Self::Expression(expression) => super::expression_group_input(sample, expression),
        }
    }
}

impl Input<'_> {
    fn value(self, sample: &TelemetrySample) -> Option<f64> {
        match self {
            Self::Field(field) => sample.values.get(field).and_then(super::series_value),
            Self::Expression(expression) => super::expression_numeric_value(sample, expression),
        }
    }
}

impl<'a> Iterator for Inputs<'a> {
    type Item = Input<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Fields(fields) => fields.next().map(|field| Input::Field(field.as_str())),
            Self::Expressions(expressions) => expressions.find(|expression| super::is_row_expression(expression)).map(Input::Expression),
        }
    }
}

impl GroupRows<'_> {
    fn count(&self, group: &Range<usize>, input: Input<'_>) -> usize {
        match input {
            Input::Field(_) => group.len(),
            Input::Expression(_) => self.rows[group.clone()].iter().filter(|row| input.value(row.sample).is_some()).count(),
        }
    }
}

impl ScalarText {
    fn new() -> Self {
        Self { bytes: [0; SCALAR_TEXT_BYTES], len: 0 }
    }

    fn format(&mut self, value: impl fmt::Display) -> &str {
        write!(self, "{value}").expect("primitive scalar formatting fits its binary representation bound");
        std::str::from_utf8(&self.bytes[..self.len]).expect("primitive scalar formatting is ASCII")
    }
}

impl Write for ScalarText {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl Write for TextLength {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(fmt::Error)?;
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn project(
    telemetry: &[TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest, generated_at: &str,
    context: &PlotPayloadContext,
) -> Result<Option<ReadBatch<PlotPayload>>> {
    let Some(operation) = operation(request) else {
        return Ok(None);
    };

    let selection = crate::plot::record_samples::PlotSamples::new(telemetry, request);
    let selected = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>(selection.collect()))?;
    let (source, filter_rows) = match operation {
        Operation::Rows(source) => (source, true),
        Operation::Aggregate(expression) => (Source::Expression(expression), false),
    };
    let plan = rows(selected, source, request, filter_rows)?;
    // All Final Keys And Points Are Constructed Anew; No Plan Allocation Moves
    // Into The Payload. Release Its Descriptors And Numeric Keys After Building.
    let output = harmos::storage::ReadBatch::new(());
    let output = output.try_map(|_| {
        let series = match operation {
            Operation::Rows(_) => build_series(&plan, telemetry, request)?,
            Operation::Aggregate(_) => build_aggregates(&plan, telemetry, request)?,
        };
        Ok::<_, crate::runtime::Error>(super::payload_from_series(&plan.selected, records, request, generated_at, context, series))
    })?;
    Ok(Some(output))
}

pub(super) fn compare(left: &Value, right: &Value) -> Option<Ordering> {
    if let (Some(left), Some(right)) = (borrowed(left), borrowed(right)) {
        return Some(left.cmp(right));
    }
    let mut left_scratch = ScalarText::new();
    let mut right_scratch = ScalarText::new();
    let left = scalar(left, &mut left_scratch)?;
    let right = scalar(right, &mut right_scratch)?;
    Some(left.cmp(right))
}

pub(super) fn contains(actual: &str, expected: &Value) -> bool {
    if let Some(expected) = borrowed(expected) {
        return actual.contains(expected);
    }
    let mut scratch = ScalarText::new();
    let expected = scalar(expected, &mut scratch);
    match expected {
        Some(expected) => actual.contains(expected),
        None => false,
    }
}

/* === Private Functions === */

fn operation(request: &PlotPayloadRequest) -> Option<Operation<'_>> {
    // Record Correlation Has Priority Over Telemetry Aggregates In The Shared Projection.
    let record = request.y_expressions.iter().any(|expression| {
        matches!(expression.source, DataSource::Record { .. })
            && expression
                .transforms
                .iter()
                .any(|transform| matches!(transform, DataTransform::TimeBetween { .. } | DataTransform::TelemetryAtOrBefore { .. }))
    });
    if record {
        return None;
    }
    let aggregate = request.y_expressions.iter().any(|expression| super::aggregate_method(expression).is_some());
    let group = request.x_expression.as_ref().or(request.series_by_expression.as_ref());
    if let Some(group) = group.filter(|_| aggregate) {
        return Some(Operation::Aggregate(group));
    }
    if !request.y_expressions.iter().any(super::is_row_expression) {
        return request.series_by.as_deref().map(|field| Operation::Rows(Source::Field(field)));
    }
    request.series_by_expression.as_ref().map(|expression| Operation::Rows(Source::Expression(expression)))
}

fn inputs(request: &PlotPayloadRequest) -> Inputs<'_> {
    if request.y_expressions.iter().any(super::is_row_expression) {
        Inputs::Expressions(request.y_expressions.iter())
    } else {
        Inputs::Fields(request.y_keys.iter())
    }
}

fn group_input<'a>(sample: &'a TelemetrySample, source: Source<'_>, request: &PlotPayloadRequest, filter_rows: bool) -> Option<&'a Value> {
    let unused = filter_rows
        && request.y_expressions.iter().any(super::is_row_expression)
        && !request
            .y_expressions
            .iter()
            .filter(|expression| super::is_row_expression(expression))
            .any(|expression| super::expression_numeric_value(sample, expression).is_some());
    if unused {
        return None;
    }
    source.input(sample)
}

fn rows<'a>(
    selected: ReadBatch<Vec<&'a TelemetrySample>>, source: Source<'_>, request: &PlotPayloadRequest, filter_rows: bool,
) -> Result<ReadBatch<GroupRows<'a>>> {
    let count = selected.len();
    let rows = selected.try_map(|selected| {
        let mut rows = vector(count)?;
        for (ordinal, sample) in selected.iter().enumerate() {
            let Some(value) = group_input(sample, source, request, filter_rows) else { continue };
            let Some(key) = key(value)? else { continue };
            rows.push(GroupRow { key, sample, ordinal });
        }
        rows.sort_unstable_by(|left, right| left.key.cmp(&right.key).then(left.ordinal.cmp(&right.ordinal)));
        Ok::<_, crate::runtime::Error>(GroupRows { selected, rows, groups: Vec::new() })
    })?;

    let count = usize::from(!rows.rows.is_empty()) + rows.rows.windows(2).filter(|pair| pair[0].key != pair[1].key).count();
    rows.try_map(|mut plan| {
        let mut groups = vector(count)?;
        let mut start = 0;
        for at in 1..plan.rows.len() {
            if plan.rows[at - 1].key != plan.rows[at].key {
                groups.push(start..at);
                start = at;
            }
        }
        if !plan.rows.is_empty() {
            groups.push(start..plan.rows.len());
        }
        groups.sort_unstable_by(|left, right| super::compare_series_group(&plan.rows[left.start].key, &plan.rows[right.start].key));
        plan.groups = groups;
        Ok(plan)
    })
}

fn point_input<'a>(sample: &'a TelemetrySample, input: Input<'_>, request: &PlotPayloadRequest) -> super::XInput<'a> {
    let field = request.x_key.as_deref().unwrap_or("timestamp");
    match input {
        Input::Field(_) => super::x_input(sample, field),
        Input::Expression(_) => super::expression_x_input(sample, request.x_expression.as_ref(), field),
    }
}

fn build_series(plan: &GroupRows<'_>, telemetry: &[TelemetrySample], request: &PlotPayloadRequest) -> Result<Vec<PlotSeries>> {
    let count = inputs(request).try_fold(0usize, |count, input| {
        let groups = plan.groups.iter().filter(|group| plan.count(group, input) != 0).count();
        count.checked_add(groups).ok_or_else(memory)
    })?;
    let mut series = vector(count)?;
    let z = super::request_z_index(telemetry, request);
    for input in inputs(request) {
        for group in &plan.groups {
            let count = plan.count(group, input);
            if count == 0 {
                continue;
            }
            let mut points = vector(count)?;
            for row in &plan.rows[group.clone()] {
                let y = input.value(row.sample);
                if matches!(input, Input::Expression(_)) && y.is_none() {
                    continue;
                }
                points.push(PlotPoint {
                    x: x_value(point_input(row.sample, input, request))?,
                    y,
                    z: z.resolve(row.sample),
                });
            }
            let (key, label) = labels(input, &plan.rows[group.start].key, request.y_keys.len());
            series.push(series_entry(join(&key)?, join(&label)?, request.unit.as_deref(), points)?);
        }
    }
    Ok(series)
}

fn build_aggregates(plan: &GroupRows<'_>, telemetry: &[TelemetrySample], request: &PlotPayloadRequest) -> Result<Vec<PlotSeries>> {
    if plan.groups.is_empty() {
        return Ok(Vec::new());
    }
    let selected = request.y_expressions.iter().filter(|expression| super::aggregate_method(expression).is_some());
    let mut series = vector(selected.clone().count())?;
    let z = super::request_z_index(telemetry, request);
    for expression in selected {
        let method = super::aggregate_method(expression).expect("selected aggregate expression");
        let mut points = vector(plan.groups.len())?;
        for group in &plan.groups {
            let mut values = vector(plan.count(group, Input::Expression(expression)))?;
            values.extend(plan.rows[group.clone()].iter().filter_map(|row| super::expression_numeric_value(row.sample, expression)));
            let y = Series::from(values).aggregate(method);
            let last = &plan.rows[group.end - 1];
            points.push(PlotPoint {
                x: serde_json::Value::String(text(&last.key)?),
                y,
                z: z.resolve(last.sample),
            });
        }
        let label = super::expression_field_name(expression).unwrap_or("value");
        series.push(series_entry(text(label)?, text(label)?, request.unit.as_deref(), points)?);
    }
    Ok(series)
}

fn labels<'a>(input: Input<'a>, group: &'a str, fields: usize) -> ([&'a str; 3], [&'a str; 3]) {
    match input {
        Input::Field(_) if fields == 1 => ([group, "", ""], [group, "", ""]),
        Input::Field(field) => ([field, ":", group], [field, " ", group]),
        Input::Expression(expression) => ([super::expression_field_name(expression).unwrap_or("value"), ":", group], [group, "", ""]),
    }
}

fn parts_length(parts: &[&str]) -> Result<usize> {
    parts.iter().try_fold(0usize, |length, part| length.checked_add(part.len()).ok_or_else(memory))
}

pub(super) fn join(parts: &[&str]) -> Result<String> {
    let mut value = String::new();
    value.try_reserve_exact(parts_length(parts)?).map_err(|_| memory())?;
    for part in parts {
        value.push_str(part);
    }
    Ok(value)
}

pub(super) fn is_key(value: &Value) -> bool {
    borrowed(value).is_some()
        || matches!(value, Value::Integer(_) | Value::Unsigned(_))
        || matches!(value, Value::Number(value) if value.is_finite())
}

pub(super) fn key(value: &Value) -> Result<Option<Cow<'_, str>>> {
    if let Some(value) = borrowed(value) {
        return Ok(Some(Cow::Borrowed(value)));
    }
    let value = match value {
        Value::Integer(value) => try_display_owned(value)?,
        Value::Unsigned(value) => try_display_owned(value)?,
        Value::Number(value) if value.is_finite() => try_display_owned(value)?,
        _ => return Ok(None),
    };
    Ok(Some(Cow::Owned(value)))
}

fn display_length(value: impl fmt::Display) -> usize {
    let mut length = TextLength(0);
    write!(&mut length, "{value}").expect("primitive scalar length fits address space");
    length.0
}

fn try_display_owned(value: impl fmt::Display) -> Result<String> {
    let mut output = String::new();
    output.try_reserve_exact(display_length(&value)).map_err(|_| memory())?;
    write!(&mut output, "{value}").expect("String formatting is infallible");
    Ok(output)
}

fn borrowed(value: &Value) -> Option<&str> {
    match value {
        Value::Text(value) if !value.trim().is_empty() => Some(value),
        Value::Bool(true) => Some("true"),
        Value::Bool(false) => Some("false"),
        _ => None,
    }
}

fn scalar<'a>(value: &'a Value, scratch: &'a mut ScalarText) -> Option<&'a str> {
    match value {
        Value::Integer(value) => Some(scratch.format(value)),
        Value::Unsigned(value) => Some(scratch.format(value)),
        Value::Number(value) if value.is_finite() => Some(scratch.format(value)),
        value => borrowed(value),
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_text_matches_existing_display_even_at_float_extremes() {
        for number in [-0.0, f64::MAX, f64::MIN, f64::MIN_POSITIVE, f64::from_bits(1)] {
            assert_eq!(key(&Value::Number(number)).unwrap().as_deref(), Some(number.to_string().as_str()));
        }
        assert_eq!(key(&Value::Integer(i64::MIN)).unwrap().as_deref(), Some(i64::MIN.to_string().as_str()));
        assert_eq!(key(&Value::Unsigned(u64::MAX)).unwrap().as_deref(), Some(u64::MAX.to_string().as_str()));
        assert_eq!(key(&Value::Bool(true)).unwrap().as_deref(), Some("true"));
    }

    #[test]
    fn borrowed_comparisons_preserve_group_text_rules() {
        assert_eq!(compare(&Value::Integer(12), &Value::Text("2".into())), Some(Ordering::Less));
        assert_eq!(compare(&Value::Bool(false), &Value::Text("false".into())), Some(Ordering::Equal));
        assert_eq!(compare(&Value::Text(" ".into()), &Value::Text("value".into())), None);
        assert!(contains("prefix12suffix", &Value::Integer(12)));
        assert!(!contains("anything", &Value::Number(f64::NAN)));
    }

    #[test]
    fn group_order_is_total_across_numeric_and_text_labels() {
        let mut labels = ["2", "1a", "10", "1.0", "1", "NaN", "inf", "+0", "-0"];
        labels.sort_unstable_by(|left, right| super::super::compare_series_group(left, right));
        assert_eq!(labels, ["+0", "-0", "1", "1.0", "2", "10", "1a", "NaN", "inf"]);
    }
}
