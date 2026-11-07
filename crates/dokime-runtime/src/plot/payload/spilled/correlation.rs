/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Record and Telemetry Correlation for Plot Projection

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
};
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy)]
struct Expression<'a> {
    kind: &'a str,
    previous: &'a str,
    expression: &'a DataExpression,
}

struct Plan<'a> {
    request: &'a PlotPayloadRequest,
    expression: Expression<'a>,
    group: Option<&'a DataExpression>,
    context_stream: bool,
}

struct Candidate {
    end: RecordEntry,
    seconds: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Plan<'_> {
    fn candidate(&self, read: &Read<'_>, ordinal: i64, record: RecordEntry) -> Result<Option<Candidate>> {
        if record.record_type.as_str() != self.expression.kind
            || !self.request.scope.as_ref().is_none_or(|scope| record.is_visible_in(scope))
        {
            return Ok(None);
        }
        if !self.filters(read, &record)? {
            return Ok(None);
        }
        let Some(start) = read.previous(self.expression.previous, ordinal, &record, self.request.scope.as_ref())? else {
            return Ok(None);
        };
        let Some(seconds) = start.timestamp.millis_until(&record.timestamp) else {
            return Ok(None);
        };
        Ok(Some(Candidate {
            end: record,
            seconds: seconds.0 as f64 / 1000.0,
        }))
    }

    fn filters(&self, read: &Read<'_>, record: &RecordEntry) -> Result<bool> {
        for filter in &self.request.filters {
            let stream = match &filter.expression.source {
                DataSource::Telemetry { stream, .. } => stream.as_str(),
                DataSource::Dimension { .. } | DataSource::Parameter { .. } if self.context_stream => crate::plot::RECORD_CONTEXT_STREAM,
                _ => &self.request.stream,
            };
            let sample = read.nearest(stream, record.scope.as_ref(), record.timestamp.as_str(), true, |sample| {
                payload::sample_matches_scope(sample, self.request.scope.as_ref())
            })?;
            let matches = sample.is_some_and(|sample| payload::sample_matches_filters(&sample, std::slice::from_ref(filter)));
            if !matches {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn append(&self, read: &Read<'_>, ordinal: u64, candidate: Candidate) -> Result<()> {
        let group = if let Some(group) = self.group {
            let sample =
                read.nearest(&self.request.stream, candidate.end.scope.as_ref(), candidate.end.timestamp.as_str(), true, |sample| {
                    payload::sample_matches_scope(sample, self.request.scope.as_ref())
                })?;
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
                point: PlotPoint {
                    x: serde_json::Value::String(candidate.end.timestamp.as_str().into()),
                    y: Some(candidate.seconds),
                    z: None,
                },
                exact: None,
            },
        )
    }
}

/* === Public Functions === */

pub(super) fn project(read: &Read<'_>, request: &PlotPayloadRequest, mut visit: impl FnMut(Series) -> Result<()>) -> Result<bool> {
    let expressions: Vec<_> = request.y_expressions.iter().filter_map(timing_expression).collect();
    if expressions.is_empty() {
        return Ok(false);
    }
    let group = request.x_expression.as_ref().or(request.series_by_expression.as_ref());
    let aggregate = group.is_some() && expressions.iter().any(|expression| payload::aggregate_method(expression.expression).is_some());
    let group = if aggregate { group } else { request.series_by_expression.as_ref() };
    let expressions: Vec<_> =
        expressions.into_iter().filter(|expression| !aggregate || payload::aggregate_method(expression.expression).is_some()).collect();
    let inputs = expressions.len();
    let mut context_stream = false;
    read.samples(|sample| {
        context_stream |= sample.stream.as_str() == crate::plot::RECORD_CONTEXT_STREAM;
        Ok(())
    })?;
    for expression in expressions {
        projection::reset(read)?;
        let plan = Plan {
            request,
            expression,
            group,
            context_stream,
        };
        let mut latest = None;
        read.records(true, |ordinal, record| {
            if let Some(candidate) = plan.candidate(read, ordinal, record)? {
                latest = latest.max(payload::parse_timestamp(candidate.end.timestamp.as_str()));
            }
            Ok(())
        })?;
        let cutoff = latest.and_then(|latest| payload::plot_window_cutoff(latest, request.window_s));
        let mut order = 0;
        read.records(true, |ordinal, record| {
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
        let name = format!("{} -> {}", expression.previous, expression.kind);
        projected.series(read, |mut series| {
            if !aggregate && group.is_some() {
                // A Single-Input Projection Labels Each Trace By Its Bare Group.
                let (key, label) = payload::groups::trace_names(&name, &series.metadata.label, inputs, false);
                let (key, label) = (payload::groups::join(&key)?, payload::groups::join(&label)?);
                series.metadata.key = key;
                series.metadata.label = label;
            } else {
                series.metadata.key = name.clone();
                series.metadata.label = name.clone();
            }
            if series.metadata.unit.is_none() {
                series.metadata.unit = Some("s".into());
            }
            visit(series)
        })?;
    }
    Ok(true)
}

/* === Private Functions === */

fn timing_expression(expression: &DataExpression) -> Option<Expression<'_>> {
    let DataSource::Record { record_type } = &expression.source else {
        return None;
    };
    let previous = expression.transforms.iter().find_map(|transform| match transform {
        DataTransform::TimeBetween { previous_record_type } => Some(previous_record_type.as_str()),
        _ => None,
    })?;
    Some(Expression { kind: record_type, previous, expression })
}

/* === Tests === */
