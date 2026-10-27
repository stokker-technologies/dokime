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
use std::ops::Range;

// Third Party Imports
use harmos::signal::{Aggregates, Series};
use harmos::storage::ReadBatch;
use serde_json::Value as JsonValue;

// Local Imports
use super::projection::{memory, series_entry, text, vector};
use super::{PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotPoint, PlotSeries, groups};
use crate::expressions::{DataExpression, DataSource, DataTransform};
use crate::plot::record_samples::RecordSampleLookup;
use crate::plot::record_timing::{RecordFilterLookup, RecordPairLookup};
use crate::runtime::{Error, Result};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy)]
struct Expression<'a> {
    record_type: &'a str,
    source: Source<'a>,
    expression: &'a DataExpression,
}

#[derive(Clone, Copy)]
struct Candidate<'a> {
    start: Option<&'a RecordEntry>,
    end: &'a RecordEntry,
    value: Option<f64>,
    sample: Option<&'a TelemetrySample>,
}

#[derive(Clone, Copy)]
struct Input<'a> {
    x: X<'a>,
    y: f64,
    group: Option<&'a Value>,
}

struct Row<'a> {
    x: X<'a>,
    y: f64,
    group: Option<Cow<'a, str>>,
    ordinal: usize,
}

struct Plan<'a> {
    expression: Expression<'a>,
    rows: Vec<Row<'a>>,
    groups: Vec<Range<usize>>,
    grouped: bool,
    aggregate: bool,
}

/* === Enums === */

#[derive(Clone, Copy)]
enum Source<'a> {
    Timing { previous: &'a str },
    Telemetry { stream: &'a str, field: &'a str },
}

#[derive(Clone, Copy)]
enum X<'a> {
    Timestamp(&'a str),
    Number(f64),
}

/* === Traits === */

/* === Implementations === */

impl Expression<'_> {
    fn label(&self) -> [&str; 3] {
        match self.source {
            Source::Timing { previous } => [previous, " -> ", self.record_type],
            Source::Telemetry { field, .. } => [field, " at ", self.record_type],
        }
    }

    fn unit<'a>(&self, request: &'a PlotPayloadRequest) -> Option<&'a str> {
        request.unit.as_deref().or_else(|| matches!(self.source, Source::Timing { .. }).then_some("s"))
    }
}

/* === Public Functions === */

pub(super) fn project(
    telemetry: &[TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest, generated_at: &str,
    context: &PlotPayloadContext,
) -> Result<Option<ReadBatch<PlotPayload>>> {
    let timing = request.y_expressions.iter().any(|expression| timing_expression(expression).is_some());
    let expressions = request
        .y_expressions
        .iter()
        .filter_map(|expression| if timing { timing_expression(expression) } else { telemetry_expression(expression) });
    let count = expressions.clone().count();
    if count == 0 {
        return Ok(None);
    }
    let group = request.x_expression.as_ref().or(request.series_by_expression.as_ref());
    let aggregate = group.is_some() && expressions.clone().any(|expression| super::aggregate_method(expression.expression).is_some());
    let group = if aggregate { group } else { request.series_by_expression.as_ref() };
    let expressions = expressions.filter(|expression| !aggregate || super::aggregate_method(expression.expression).is_some());
    let count = expressions.clone().count();
    let filters = RecordFilterLookup::try_new(telemetry, request)?;
    let owner = harmos::storage::ReadBatch::new(());
    let mut plans = owner.try_map(|_| vector::<ReadBatch<Plan<'_>>>(count))?;

    for expression in expressions {
        let plan = plan(telemetry, records, request, expression, group, aggregate, &filters)?;
        // Only Borrowed Descriptors And Their Numeric Group Labels Survive The
        // Lookup Pass. The Candidate And Correlation Claims End In That Pass.
        plans = plans.try_map(|mut plans| {
            plans.push(plan);
            Ok::<_, Error>(plans)
        })?;
    }
    let selected = crate::plot::record_samples::PlotSamples::new(telemetry, request);
    let selected = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(selected.collect()))?;
    // This Covers Our Exact Aggregate Input Vector. Harmos' Private
    // Percentile/Sort Workspace Has No Public Planner And Remains Separate.
    let output = harmos::storage::ReadBatch::new(()).try_map(|_| {
        let series = build(&plans, request)?;
        Ok::<_, Error>(super::payload_from_series(telemetry, &selected, records, request, generated_at, context, series))
    })?;
    Ok(Some(output))
}

/* === Private Functions === */

#[allow(clippy::too_many_arguments)]
fn plan<'a>(
    telemetry: &'a [TelemetrySample], records: crate::records::view::View<'a>, request: &'a PlotPayloadRequest, expression: Expression<'a>,
    group: Option<&'a DataExpression>, aggregate: bool, filters: &RecordFilterLookup<'_>,
) -> Result<ReadBatch<Plan<'a>>> {
    let axis = request.x_expression.as_ref().and_then(telemetry_expression);
    let contextual = !aggregate && (axis.is_some() || group.is_some());
    let scoped = records.iter().filter(|record| super::sample_matches_scope(record.scope.as_ref(), request.scope.as_ref()));
    let previous = match expression.source {
        Source::Timing { previous } => Some(previous),
        Source::Telemetry { .. } if contextual => {
            axis.filter(|axis| axis.record_type != expression.record_type).map(|axis| axis.record_type)
        }
        _ => None,
    };
    let pairs = previous.map(|previous| RecordPairLookup::try_new(scoped.clone(), previous)).transpose()?;
    let group_context = group.is_some() && !(aggregate && matches!(expression.source, Source::Telemetry { .. }));
    let y = lookup(telemetry, request, expression)?;
    let x = axis.filter(|_| contextual).map(|axis| lookup(telemetry, request, axis)).transpose()?.flatten();
    let group_lookup = group_context
        .then(|| RecordSampleLookup::try_for_stream_matching(telemetry, &request.stream, request.scope.as_ref(), |_| true))
        .transpose()?;
    let candidates = if let Some(pairs) = &pairs {
        let selected = pairs.pairs(expression.record_type).filter(|pair| filters.matches(pair.end)).map(|pair| Candidate {
            start: Some(pair.start),
            end: pair.end,
            value: matches!(expression.source, Source::Timing { .. }).then_some(pair.seconds),
            sample: None,
        });
        collect(selected)?
    } else {
        let selected = scoped.filter(|record| record.record_type.as_str() == expression.record_type && filters.matches(record));
        let selected = selected.filter_map(|record| {
            let mut candidate = Candidate { start: axis.map(|_| record), end: record, value: None, sample: None };
            if !contextual {
                let (sample, value) = telemetry_value(&candidate, expression, y.as_ref()?)?;
                candidate.sample = Some(sample);
                candidate.value = Some(value);
            }
            Some(candidate)
        });
        collect(selected)?
    };
    let latest = candidates.iter().filter_map(|candidate| super::parse_timestamp(candidate.end.timestamp.as_str())).max();
    let cutoff = latest.and_then(|latest| super::plot_window_cutoff(latest, request.window_s));
    let inputs = candidates
        .iter()
        .filter(|candidate| {
            cutoff.is_none_or(|cutoff| super::parse_timestamp(candidate.end.timestamp.as_str()).is_none_or(|time| time >= cutoff))
        })
        .filter_map(|candidate| input(candidate, expression, axis, group, aggregate, y.as_ref(), x.as_ref(), group_lookup.as_ref()));
    let rows = harmos::storage::ReadBatch::new(()).try_map(|_| {
        let mut rows = vector(inputs.clone().count())?;
        for (ordinal, input) in inputs.enumerate() {
            let group = input.group.map(groups::key).transpose()?.flatten();
            rows.push(Row { x: input.x, y: input.y, group, ordinal });
        }
        rows.sort_unstable_by(|left, right| left.group.cmp(&right.group).then(left.ordinal.cmp(&right.ordinal)));
        Ok::<_, Error>(Plan { expression, rows, groups: Vec::new(), grouped: group.is_some(), aggregate })
    })?;
    let count = usize::from(!rows.rows.is_empty()) + rows.rows.windows(2).filter(|pair| pair[0].group != pair[1].group).count();
    rows.try_map(|mut plan| {
        let mut ranges = vector(count)?;
        let mut start = 0;
        for at in 1..plan.rows.len() {
            if plan.rows[at - 1].group != plan.rows[at].group {
                ranges.push(start..at);
                start = at;
            }
        }
        if !plan.rows.is_empty() {
            ranges.push(start..plan.rows.len());
        }
        ranges.sort_unstable_by(|left, right| {
            super::compare_series_group(
                plan.rows[left.start].group.as_deref().unwrap_or(""),
                plan.rows[right.start].group.as_deref().unwrap_or(""),
            )
        });
        plan.groups = ranges;
        Ok(plan)
    })
}

fn collect<'a>(candidates: impl Iterator<Item = Candidate<'a>> + Clone) -> Result<ReadBatch<Vec<Candidate<'a>>>> {
    let count = candidates.clone().count();
    harmos::storage::ReadBatch::new(()).try_map(|_| {
        let mut rows = vector(count)?;
        rows.extend(candidates);
        Ok::<_, Error>(rows)
    })
}

fn lookup<'a>(
    telemetry: &'a [TelemetrySample], request: &PlotPayloadRequest, expression: Expression<'_>,
) -> Result<Option<RecordSampleLookup<'a>>> {
    let Source::Telemetry { stream, field } = expression.source else { return Ok(None) };
    Ok(Some(RecordSampleLookup::try_for_stream_matching(telemetry, stream, request.scope.as_ref(), |sample| {
        super::sample_value(sample, field).and_then(super::numeric_value).is_some()
    })?))
}

fn telemetry_value<'a>(
    candidate: &Candidate<'a>, expression: Expression<'_>, lookup: &RecordSampleLookup<'a>,
) -> Option<(&'a TelemetrySample, f64)> {
    let Source::Telemetry { field, .. } = expression.source else { return None };
    let time = super::parse_timestamp(candidate.end.timestamp.as_str())?;
    let sample = lookup.nearest_at_or_before(candidate.end.scope.as_ref(), time)?.sample;
    Some((sample, super::sample_value(sample, field).and_then(super::numeric_value)?))
}

#[allow(clippy::too_many_arguments)]
fn input<'a>(
    candidate: &Candidate<'a>, expression: Expression<'_>, axis: Option<Expression<'_>>, group: Option<&DataExpression>, aggregate: bool,
    y_lookup: Option<&RecordSampleLookup<'a>>, x_lookup: Option<&RecordSampleLookup<'a>>, group_lookup: Option<&RecordSampleLookup<'a>>,
) -> Option<Input<'a>> {
    let (sample, y) = match candidate.value {
        Some(value) => (candidate.sample, value),
        None => {
            let (sample, value) = telemetry_value(candidate, expression, y_lookup?)?;
            (Some(sample), value)
        }
    };
    let x = if let Some(axis) = axis.filter(|_| !aggregate) {
        let record = match expression.source {
            Source::Timing { previous } if axis.record_type == previous => candidate.start?,
            Source::Timing { .. } if axis.record_type != expression.record_type => return None,
            Source::Timing { .. } => candidate.end,
            Source::Telemetry { .. } => candidate.start?,
        };
        let candidate = Candidate { start: None, end: record, value: None, sample: None };
        X::Number(telemetry_value(&candidate, axis, x_lookup?)?.1)
    } else {
        X::Timestamp(candidate.end.timestamp.as_str())
    };
    let group = match group {
        Some(group) => {
            let sample = if aggregate && matches!(expression.source, Source::Telemetry { .. }) {
                sample?
            } else {
                let time = super::parse_timestamp(candidate.end.timestamp.as_str())?;
                group_lookup?.nearest_at_or_before(candidate.end.scope.as_ref(), time)?.sample
            };
            let value = super::expression_group_input(sample, group)?;
            if !groups::is_key(value) {
                return None;
            }
            Some(value)
        }
        None => None,
    };
    Some(Input { x, y, group })
}

fn build(plans: &[ReadBatch<Plan<'_>>], request: &PlotPayloadRequest) -> Result<Vec<PlotSeries>> {
    let count = plans.iter().try_fold(0usize, |count, plan| {
        let series = if plan.aggregate || !plan.grouped { 1 } else { plan.groups.len() };
        count.checked_add(series).ok_or_else(memory)
    })?;
    let mut series = vector(count)?;
    for plan in plans {
        if plan.aggregate {
            let mut points = vector(plan.groups.len())?;
            let method = super::aggregate_method(plan.expression.expression).expect("selected aggregate");
            for group in &plan.groups {
                let mut values = vector(group.len())?;
                values.extend(plan.rows[group.clone()].iter().map(|row| row.y));
                let y = Series::from(values).aggregate(method);
                let label = plan.rows[group.start].group.as_deref().unwrap_or("");
                points.push(PlotPoint { x: JsonValue::String(text(label)?), y, z: None });
            }
            series.push(series_entry(
                groups::join(&plan.expression.label())?,
                groups::join(&plan.expression.label())?,
                plan.expression.unit(request),
                points,
            )?);
            continue;
        }
        if !plan.grouped {
            let points = points(&plan.rows)?;
            series.push(series_entry(
                groups::join(&plan.expression.label())?,
                groups::join(&plan.expression.label())?,
                plan.expression.unit(request),
                points,
            )?);
            continue;
        }
        for group in &plan.groups {
            let label = plan.rows[group.start].group.as_deref().unwrap_or("");
            series.push(series_entry(text(label)?, text(label)?, plan.expression.unit(request), points(&plan.rows[group.clone()])?)?);
        }
    }
    Ok(series)
}

fn points(rows: &[Row<'_>]) -> Result<Vec<PlotPoint>> {
    let mut points = vector(rows.len())?;
    for row in rows {
        let x = match row.x {
            X::Timestamp(value) => JsonValue::String(text(value)?),
            X::Number(value) => JsonValue::from(value),
        };
        points.push(PlotPoint { x, y: Some(row.y), z: None });
    }
    Ok(points)
}

fn timing_expression(expression: &DataExpression) -> Option<Expression<'_>> {
    let DataSource::Record { record_type } = &expression.source else { return None };
    expression.transforms.iter().find_map(|transform| {
        let DataTransform::TimeBetween { previous_record_type } = transform else { return None };
        Some(Expression { record_type, source: Source::Timing { previous: previous_record_type }, expression })
    })
}

fn telemetry_expression(expression: &DataExpression) -> Option<Expression<'_>> {
    let DataSource::Record { record_type } = &expression.source else { return None };
    expression.transforms.iter().find_map(|transform| {
        let DataTransform::TelemetryAtOrBefore { stream, field } = transform else { return None };
        Some(Expression { record_type, source: Source::Telemetry { stream, field }, expression })
    })
}

/* === Tests === */
