/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Record And Telemetry Correlation For Plot Projection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::super::{self as payload, PlotPayloadRequest, PlotPoint};
use super::{
    input::Read,
    projection::{self, Input, Projection, Row},
    series::Series,
};
use crate::{
    expressions::{DataExpression, DataSource, DataTransform},
    runtime::Result,
    telemetry::TelemetrySample,
};
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy)]
struct Expression<'a> {
    kind: &'a str,
    source: Source<'a>,
    expression: &'a DataExpression,
}

struct Plan<'a> {
    request: &'a PlotPayloadRequest,
    expression: Expression<'a>,
    axis: Option<Expression<'a>>,
    group: Option<&'a DataExpression>,
    aggregate: bool,
    previous: Option<&'a str>,
    contextual: bool,
    context_stream: bool,
}

struct Candidate {
    start: Option<RecordEntry>,
    end: RecordEntry,
    sample: Option<TelemetrySample>,
    value: Option<f64>,
}

/* === Enums === */

#[derive(Clone, Copy)]
enum Source<'a> {
    Timing(&'a str),
    Telemetry { stream: &'a str, field: &'a str },
}

/* === Traits === */

/* === Implementations === */

impl Plan<'_> {
    fn candidate(&self, read: &Read<'_>, ordinal: i64, record: RecordEntry) -> Result<Option<Candidate>> {
        if record.record_type.as_str() != self.expression.kind
            || !payload::sample_matches_scope(record.scope.as_ref(), self.request.scope.as_ref())
        {
            return Ok(None);
        }
        if !self.filters(read, &record)? {
            return Ok(None);
        }
        let mut candidate = Candidate {
            start: None,
            end: record,
            sample: None,
            value: None,
        };
        if let Some(previous) = self.previous {
            let Some(start) = read.previous(previous, ordinal, &candidate.end, self.request.scope.as_ref())? else {
                return Ok(None);
            };
            if matches!(self.expression.source, Source::Timing(_)) {
                let Some(seconds) = start.timestamp.millis_until(&candidate.end.timestamp) else {
                    return Ok(None);
                };
                candidate.value = Some(seconds.0 as f64 / 1000.0);
            }
            candidate.start = Some(start);
        } else {
            candidate.start = self.axis.map(|_| candidate.end.clone());
            if !self.contextual {
                let Some((sample, value)) = self.telemetry(read, self.expression, &candidate.end)? else {
                    return Ok(None);
                };
                candidate.sample = Some(sample);
                candidate.value = Some(value);
            }
        }
        Ok(Some(candidate))
    }

    fn filters(&self, read: &Read<'_>, record: &RecordEntry) -> Result<bool> {
        for filter in &self.request.filters {
            let stream = match &filter.expression.source {
                DataSource::Telemetry { stream, .. } => stream.as_str(),
                DataSource::Dimension { .. } | DataSource::Parameter { .. } if self.context_stream => crate::plot::RECORD_CONTEXT_STREAM,
                _ => &self.request.stream,
            };
            let sample = read.nearest(stream, record.scope.as_ref(), record.timestamp.as_str(), true, |sample| {
                payload::sample_matches_scope(sample.scope.as_ref(), self.request.scope.as_ref())
            })?;
            let matches = sample.is_some_and(|sample| payload::sample_matches_filters(&sample, std::slice::from_ref(filter)));
            if !matches {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn telemetry(&self, read: &Read<'_>, expression: Expression<'_>, record: &RecordEntry) -> Result<Option<(TelemetrySample, f64)>> {
        let Source::Telemetry { stream, field } = expression.source else {
            return Ok(None);
        };
        let sample = read.nearest(stream, record.scope.as_ref(), record.timestamp.as_str(), true, |sample| {
            payload::sample_matches_scope(sample.scope.as_ref(), self.request.scope.as_ref())
                && payload::sample_value(sample, field).and_then(payload::numeric_value).is_some()
        })?;
        Ok(sample.and_then(|sample| {
            let value = payload::sample_value(&sample, field).and_then(payload::numeric_value)?;
            Some((sample, value))
        }))
    }

    fn append(&self, read: &Read<'_>, ordinal: u64, candidate: Candidate) -> Result<()> {
        let (sample, y) = match candidate.value {
            Some(value) => (candidate.sample, value),
            None => {
                let Some((sample, value)) = self.telemetry(read, self.expression, &candidate.end)? else {
                    return Ok(());
                };
                (Some(sample), value)
            }
        };
        let x = if let Some(axis) = self.axis.filter(|_| !self.aggregate) {
            let record = match self.expression.source {
                Source::Timing(previous) if axis.kind == previous => candidate.start.as_ref(),
                Source::Timing(_) if axis.kind != self.expression.kind => None,
                Source::Timing(_) => Some(&candidate.end),
                Source::Telemetry { .. } => candidate.start.as_ref(),
            };
            let Some(record) = record else {
                return Ok(());
            };
            let Some((_, value)) = self.telemetry(read, axis, record)? else {
                return Ok(());
            };
            serde_json::Value::from(value)
        } else {
            serde_json::Value::String(candidate.end.timestamp.as_str().into())
        };
        let group = if let Some(group) = self.group {
            let sample = if self.aggregate && matches!(self.expression.source, Source::Telemetry { .. }) {
                sample
            } else {
                read.nearest(&self.request.stream, candidate.end.scope.as_ref(), candidate.end.timestamp.as_str(), true, |sample| {
                    payload::sample_matches_scope(sample.scope.as_ref(), self.request.scope.as_ref())
                })?
            };
            let Some(sample) = sample else {
                return Ok(());
            };
            let Some(value) = payload::expression_group_input(&sample, group) else {
                return Ok(());
            };
            let Some(key) = payload::groups::key(value)? else {
                return Ok(());
            };
            key.into_owned()
        } else {
            String::new()
        };
        projection::insert(
            read,
            Row {
                expression: 0,
                group: &group,
                sort: 0.0,
                stamp: "",
                capture: ordinal,
                point: PlotPoint { x, y: Some(y), z: None },
                exact: None,
            },
        )
    }
}

/* === Public Functions === */

pub(super) fn project(read: &Read<'_>, request: &PlotPayloadRequest, mut visit: impl FnMut(Series) -> Result<()>) -> Result<bool> {
    let timing = request.y_expressions.iter().any(|expression| expression_for(expression, true).is_some());
    let expressions: Vec<_> = request.y_expressions.iter().filter_map(|expression| expression_for(expression, timing)).collect();
    if expressions.is_empty() {
        return Ok(false);
    }
    let group = request.x_expression.as_ref().or(request.series_by_expression.as_ref());
    let aggregate = group.is_some() && expressions.iter().any(|expression| payload::aggregate_method(expression.expression).is_some());
    let group = if aggregate { group } else { request.series_by_expression.as_ref() };
    let mut context_stream = false;
    read.samples(|sample| {
        context_stream |= sample.stream.as_str() == crate::plot::RECORD_CONTEXT_STREAM;
        Ok(())
    })?;
    for expression in expressions.into_iter().filter(|expression| !aggregate || payload::aggregate_method(expression.expression).is_some())
    {
        projection::reset(read)?;
        let axis = request.x_expression.as_ref().and_then(|expression| expression_for(expression, false));
        let contextual = !aggregate && (axis.is_some() || group.is_some());
        let previous = match expression.source {
            Source::Timing(previous) => Some(previous),
            Source::Telemetry { .. } if contextual => axis.filter(|axis| axis.kind != expression.kind).map(|axis| axis.kind),
            _ => None,
        };
        let plan = Plan {
            request,
            expression,
            axis,
            group,
            aggregate,
            previous,
            contextual,
            context_stream,
        };
        let mut latest = None;
        read.records(previous.is_some(), |ordinal, record| {
            if let Some(candidate) = plan.candidate(read, ordinal, record)? {
                latest = latest.max(payload::parse_timestamp(candidate.end.timestamp.as_str()));
            }
            Ok(())
        })?;
        let cutoff = latest.and_then(|latest| payload::plot_window_cutoff(latest, request.window_s));
        let mut order = 0;
        read.records(previous.is_some(), |ordinal, record| {
            let Some(candidate) = plan.candidate(read, ordinal, record)? else {
                return Ok(());
            };
            let time = payload::parse_timestamp(candidate.end.timestamp.as_str());
            if cutoff.is_some_and(|cutoff| time.is_some_and(|time| time < cutoff)) {
                return Ok(());
            }
            order += 1;
            plan.append(read, order, candidate)
        })?;
        let projected = Projection {
            request,
            inputs: vec![Input::Expression(expression.expression)],
            aggregate,
            grouped: group.is_some(),
            keep_empty_aggregate: true,
        };
        projected.series(read, |mut series| {
            if !aggregate && group.is_some() {
                series.metadata.key = series.metadata.label.clone();
            } else {
                let label = match expression.source {
                    Source::Timing(previous) => format!("{previous} -> {}", expression.kind),
                    Source::Telemetry { field, .. } => format!("{field} at {}", expression.kind),
                };
                series.metadata.key = label.clone();
                series.metadata.label = label;
            }
            if matches!(expression.source, Source::Timing(_)) && series.metadata.unit.is_none() {
                series.metadata.unit = Some("s".into());
            }
            visit(series)
        })?;
    }
    Ok(true)
}

/* === Private Functions === */

fn expression_for(expression: &DataExpression, timing: bool) -> Option<Expression<'_>> {
    let DataSource::Record { record_type } = &expression.source else {
        return None;
    };
    let source = expression.transforms.iter().find_map(|transform| match transform {
        DataTransform::TimeBetween { previous_record_type } if timing => Some(Source::Timing(previous_record_type)),
        DataTransform::TelemetryAtOrBefore { stream, field } if !timing => Some(Source::Telemetry { stream, field }),
        _ => None,
    })?;
    Some(Expression { kind: record_type, source, expression })
}

/* === Tests === */
