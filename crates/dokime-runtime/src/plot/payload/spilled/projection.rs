/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Disk-Sorted Telemetry Projection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::storage::query::{
    Accumulator,
    scalar::{Agg, ScalarValue},
};
use rusqlite::params;

// Local Imports
use super::super::{self as payload, PlotPayloadRequest, PlotPoint, PlotSeries};
use super::{
    input::{Read, failure},
    series::{Points, Series, scalar_number},
};
use crate::{expressions::DataExpression, runtime::Result, telemetry::TelemetrySample};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Projection<'a> {
    pub request: &'a PlotPayloadRequest,
    pub inputs: Vec<Input<'a>>,
    pub aggregate: bool,
    pub grouped: bool,
    pub keep_empty_aggregate: bool,
}

pub(super) struct Row<'a> {
    pub expression: usize,
    pub group: &'a str,
    pub sort: f64,
    pub stamp: &'a str,
    pub capture: u64,
    pub point: PlotPoint,
}

/* === Enums === */

#[derive(Clone, Copy)]
pub(super) enum Input<'a> {
    Field(&'a str),
    Expression(&'a DataExpression),
}

/* === Traits === */

/* === Implementations === */

impl<'a> Input<'a> {
    fn value(self, sample: &TelemetrySample) -> Option<f64> {
        match self {
            Self::Field(field) => sample.values.get(field).and_then(payload::numeric_value),
            Self::Expression(expression) => payload::expression_numeric_value(sample, expression),
        }
    }

    fn field(self) -> &'a str {
        match self {
            Self::Field(field) => field,
            Self::Expression(expression) => payload::expression_field_name(expression).unwrap_or("value"),
        }
    }
}

impl<'a> Projection<'a> {
    pub(super) fn telemetry(read: &Read<'_>, request: &'a PlotPayloadRequest) -> Result<Self> {
        reset(read)?;
        let aggregate = request.y_expressions.iter().any(|expression| payload::aggregate_method(expression).is_some())
            && request.x_expression.as_ref().or(request.series_by_expression.as_ref()).is_some();
        let expressions = request.y_expressions.iter().any(payload::is_row_expression);
        let inputs = if aggregate {
            request
                .y_expressions
                .iter()
                .filter(|expression| payload::aggregate_method(expression).is_some())
                .map(Input::Expression)
                .collect()
        } else if expressions {
            request.y_expressions.iter().filter(|expression| payload::is_row_expression(expression)).map(Input::Expression).collect()
        } else {
            request.y_keys.iter().map(|key| Input::Field(key)).collect()
        };
        let grouped = aggregate || if expressions { request.series_by_expression.is_some() } else { request.series_by.is_some() };
        let projection = Self { request, inputs, aggregate, grouped, keep_empty_aggregate: false };
        let mut latest = None;
        read.samples(|sample| {
            if selected(&sample, request) {
                latest = latest.max(payload::parse_timestamp(sample.timestamp.as_str()));
            }
            Ok(())
        })?;
        let cutoff = latest.and_then(|latest| payload::plot_window_cutoff(latest, request.window_s));
        read.samples(|sample| {
            let visible = cutoff.is_none_or(|cutoff| payload::parse_timestamp(sample.timestamp.as_str()).is_none_or(|time| time >= cutoff));
            if !selected(&sample, request) || !visible {
                return Ok(());
            }
            projection.append(read, &sample)
        })?;
        Ok(projection)
    }

    fn append(&self, read: &Read<'_>, sample: &TelemetrySample) -> Result<()> {
        let group = if self.aggregate {
            self.request
                .x_expression
                .as_ref()
                .or(self.request.series_by_expression.as_ref())
                .and_then(|expression| payload::expression_group_input(sample, expression))
        } else if self.inputs.iter().any(|input| matches!(input, Input::Expression(_))) {
            self.request.series_by_expression.as_ref().and_then(|expression| payload::expression_group_input(sample, expression))
        } else {
            self.request.series_by.as_deref().and_then(|field| payload::sample_value(sample, field))
        };
        let group = group.map(payload::groups::key).transpose()?.flatten();
        if self.grouped && group.is_none() {
            return Ok(());
        }
        let z = z_value(read, sample, self.request)?;
        let x_key = self.request.x_key.as_deref().unwrap_or("timestamp");
        let time_order = self.request.analysis.is_some()
            || matches!(self.request.plot_type, Some(crate::document::PlotType::Trajectory | crate::document::PlotType::FrequencySeries));
        let sort = if time_order || x_key == "timestamp" { 0.0 } else { payload::x_sort_value(sample, x_key) };
        for (index, input) in self.inputs.iter().copied().enumerate() {
            let y = input.value(sample);
            if !self.aggregate && matches!(input, Input::Expression(_)) && y.is_none() {
                continue;
            }
            let x = match input {
                Input::Field(_) => payload::x_input(sample, x_key),
                Input::Expression(_) => payload::expression_x_input(sample, self.request.x_expression.as_ref(), x_key),
            };
            let point = PlotPoint { x: payload::projection::x_value(x)?, y, z };
            insert(
                read,
                Row {
                    expression: index,
                    group: group.as_deref().unwrap_or(""),
                    sort,
                    stamp: sample.timestamp.as_str(),
                    capture: sample.sequence,
                    point,
                },
            )?;
        }
        Ok(())
    }

    pub(super) fn series(&self, read: &Read<'_>, mut visit: impl FnMut(Series) -> Result<()>) -> Result<()> {
        for (index, input) in self.inputs.iter().copied().enumerate() {
            if !self.grouped {
                let mut points = Points::new()?;
                rows(read, index, "", |point| points.push(point))?;
                visit(points.finish(metadata(input.field(), input.field(), self.request))?)?;
                continue;
            }
            let mut groups = read
                .connection
                .prepare("SELECT grp FROM projected WHERE expression=?1 GROUP BY grp ORDER BY group_number IS NULL,group_number,grp")
                .map_err(failure)?;
            let mut groups = groups.query(params![index as i64]).map_err(failure)?;
            let mut aggregates = Points::new()?;
            let mut count = 0;
            while let Some(row) = groups.next().map_err(failure)? {
                let group: String = row.get(0).map_err(failure)?;
                count += 1;
                if self.aggregate {
                    let Input::Expression(expression) = input else { unreachable!("aggregate inputs are expressions") };
                    let method = payload::aggregate_method(expression).expect("selected aggregate");
                    let mut accumulator = Accumulator::new(Agg::Of(method));
                    let mut capture = 0;
                    let mut z = None;
                    rows(read, index, &group, |point| {
                        if let Some(y) = point.y {
                            accumulator.push(capture, &ScalarValue::Float(y))?;
                        }
                        capture += 1;
                        z = point.z;
                        Ok(())
                    })?;
                    let y = scalar_number(accumulator.finish(&mut || Ok(()))?);
                    aggregates.push(PlotPoint { x: group.into(), y, z })?;
                } else {
                    let mut points = Points::new()?;
                    rows(read, index, &group, |point| points.push(point))?;
                    let (key, label) = match input {
                        Input::Field(_) if self.request.y_keys.len() == 1 => (group.clone(), group.clone()),
                        Input::Field(field) => (format!("{field}:{group}"), format!("{field} {group}")),
                        Input::Expression(_) => (format!("{}:{group}", input.field()), group.clone()),
                    };
                    visit(points.finish(metadata(&key, &label, self.request))?)?;
                }
            }
            if self.aggregate && (count != 0 || self.keep_empty_aggregate) {
                visit(aggregates.finish(metadata(input.field(), input.field(), self.request))?)?;
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn reset(read: &Read<'_>) -> Result<()> {
    read.connection
        .execute_batch(
            "DROP TABLE IF EXISTS projected;
         CREATE TABLE projected (
             ordinal INTEGER PRIMARY KEY, expression INTEGER NOT NULL, grp TEXT NOT NULL, group_number REAL,
             sort BLOB NOT NULL, stamp TEXT NOT NULL, capture BLOB NOT NULL, x TEXT NOT NULL, y BLOB, z BLOB
         );
         CREATE INDEX projected_order ON projected(expression,grp,sort,stamp,capture,ordinal);",
        )
        .map_err(failure)
}

pub(super) fn metadata(key: &str, label: &str, request: &PlotPayloadRequest) -> PlotSeries {
    PlotSeries {
        key: key.into(),
        label: label.into(),
        unit: request.unit.clone(),
        points: Vec::new(),
        raw_points: 0,
        downsampled: false,
        format: None,
    }
}

pub(super) fn insert(read: &Read<'_>, row: Row<'_>) -> Result<()> {
    let Row { expression, group, sort, stamp, capture, point } = row;
    let group_number = group.parse::<f64>().ok().filter(|value| value.is_finite());
    let bits = sort.to_bits();
    let order = if bits >> 63 == 0 { bits ^ (1u64 << 63) } else { !bits };
    let order = order.to_be_bytes();
    let capture = capture.to_be_bytes();
    let x = serde_json::to_string(&point.x).map_err(failure)?;
    let y = point.y.map(f64::to_le_bytes);
    let z = point.z.map(f64::to_le_bytes);
    read.connection
        .execute(
            "INSERT INTO projected(expression,grp,group_number,sort,stamp,capture,x,y,z) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                expression as i64,
                group,
                group_number,
                order.as_slice(),
                stamp,
                capture.as_slice(),
                x,
                y.as_ref().map(|bytes| bytes.as_slice()),
                z.as_ref().map(|bytes| bytes.as_slice())
            ],
        )
        .map_err(failure)?;
    Ok(())
}

pub(super) fn rows(read: &Read<'_>, expression: usize, group: &str, mut visit: impl FnMut(PlotPoint) -> Result<()>) -> Result<()> {
    let mut statement = read
        .connection
        .prepare("SELECT x,y,z FROM projected WHERE expression=?1 AND grp=?2 ORDER BY sort,stamp,capture,ordinal")
        .map_err(failure)?;
    let mut rows = statement.query(params![expression as i64, group]).map_err(failure)?;
    while let Some(row) = rows.next().map_err(failure)? {
        let x: String = row.get(0).map_err(failure)?;
        let x = serde_json::from_str(&x).map_err(failure)?;
        let point = PlotPoint { x, y: number(row.get(1).map_err(failure)?)?, z: number(row.get(2).map_err(failure)?)? };
        visit(point)?;
    }
    Ok(())
}

/* === Private Functions === */

pub(super) fn selected(sample: &TelemetrySample, request: &PlotPayloadRequest) -> bool {
    sample.stream.as_str() == request.stream
        && payload::sample_matches_scope(sample.scope.as_ref(), request.scope.as_ref())
        && payload::sample_matches_filters(sample, &request.filters)
}

fn z_value(read: &Read<'_>, sample: &TelemetrySample, request: &PlotPayloadRequest) -> Result<Option<f64>> {
    let (stream, field) = match request.z_expression.as_ref() {
        Some(expression) => match &expression.source {
            crate::expressions::DataSource::Telemetry { stream, field } => (stream.as_str(), field.as_str()),
            _ => return Ok(payload::expression_numeric_value(sample, expression)),
        },
        None => {
            let Some(field) = request.z_field.as_deref() else {
                return Ok(None);
            };
            (request.z_stream.as_deref().unwrap_or(&request.stream), field)
        }
    };
    if payload::stream_matches_alias(&request.stream, stream) {
        return Ok(sample.values.get(field).and_then(payload::numeric_value));
    }
    let other = read.nearest(stream, sample.scope.as_ref(), sample.timestamp.as_str(), false, |candidate| {
        crate::evidence_runtime::scopes_correlate(candidate.scope.as_ref(), sample.scope.as_ref())
            && payload::sample_matches_scope(candidate.scope.as_ref(), request.scope.as_ref())
            && payload::sample_value(candidate, field).and_then(payload::numeric_value).is_some()
    })?;
    Ok(other.and_then(|sample| payload::sample_value(&sample, field).and_then(payload::numeric_value)))
}

/* === Tests === */

fn number(bytes: Option<Vec<u8>>) -> Result<Option<f64>> {
    bytes
        .map(|bytes| {
            let bytes = bytes.try_into().map_err(|_| failure("invalid projected number"))?;
            Ok(f64::from_le_bytes(bytes))
        })
        .transpose()
}
