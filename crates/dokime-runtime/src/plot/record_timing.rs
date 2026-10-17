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
use serde_json::Value as JsonValue;
// Local Imports
use crate::document::DocumentFilter;
use crate::document::frame::{Lookup, Side, Strategy, nanos};
use crate::plot::{
    DownsampleTechnique, PlotPayloadRequest, PlotPoint, PlotSeries, aggregate_group_points,
    aggregate_method, expression_group_value, finish_plot_series, numeric_value, parse_timestamp,
    record_samples::RecordSampleLookup, retain_window, sample_matches_scope, sample_value,
};
use crate::records::RecordEntry;
use crate::expressions::{
    DataExpression, DataSource, DataTransform, RecordTimingPoint, RecordTimingRecord,
    record_time_between,
};
use crate::telemetry::TelemetrySample;

/* === Constants === */
pub(crate) const RECORD_CONTEXT_STREAM: &str = "dokime.plot.record_context";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
struct RecordTimingExpression<'a> {
    label: String,
    end_record_type: &'a str,
    previous_record_type: &'a str,
    expression: &'a DataExpression,
}

#[derive(Clone, Debug)]
struct RecordTelemetryAxisExpression<'a> {
    record_type: &'a str,
    stream: &'a str,
    field: &'a str,
}

pub(super) struct RecordFilterLookup<'a> {
    filters: Vec<(&'a DocumentFilter, RecordSampleLookup<'a>)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'a> RecordFilterLookup<'a> {
    pub(super) fn new(telemetry: &'a [TelemetrySample], request: &'a PlotPayloadRequest) -> Self {
        let filters = request
            .filters
            .iter()
            .map(|filter| {
                let stream = match &filter.expression.source {
                    DataSource::Telemetry { stream, .. } => stream.as_str(),
                    DataSource::Dimension { .. } | DataSource::Parameter { .. }
                        if telemetry
                            .iter()
                            .any(|sample| sample.stream.as_str() == RECORD_CONTEXT_STREAM) =>
                    {
                        RECORD_CONTEXT_STREAM
                    }
                    _ => request.stream.as_str(),
                };
                (
                    filter,
                    RecordSampleLookup::for_stream(telemetry, stream, request.scope.as_ref()),
                )
            })
            .collect();

        Self { filters }
    }

    pub(super) fn prepare<'r>(
        &mut self,
        records: impl IntoIterator<Item = &'r RecordEntry> + Clone,
    ) {
        for (_, lookup) in &mut self.filters {
            lookup.prepare(records.clone().into_iter().filter_map(|record| {
                Some((
                    record.scope.as_ref(),
                    parse_timestamp(record.timestamp.as_str())?,
                ))
            }));
        }
    }

    pub(super) fn matches(&self, record: &RecordEntry) -> bool {
        if self.filters.is_empty() {
            return true;
        }
        let Some(timestamp) = parse_timestamp(record.timestamp.as_str()) else {
            return false;
        };

        self.filters.iter().all(|(filter, lookup)| {
            lookup
                .nearest_at_or_before(record.scope.as_ref(), timestamp)
                .is_some_and(|sample| {
                    super::payload::sample_matches_filters(
                        sample.sample,
                        std::slice::from_ref(*filter),
                    )
                })
        })
    }
}

/* === Public Functions === */
pub(crate) fn record_timing_series(
    telemetry: &[TelemetrySample],
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Option<Vec<PlotSeries>> {
    let expressions = request
        .y_expressions
        .iter()
        .filter_map(record_timing_expression)
        .collect::<Vec<_>>();
    if expressions.is_empty() {
        return None;
    }
    let record_inputs = records
        .iter()
        .filter(|record| sample_matches_scope(record.scope.as_ref(), request.scope.as_ref()))
        .map(|record| RecordTimingRecord {
            record_type: record.record_type.as_str(),
            sequence: record.sequence,
            timestamp: &record.timestamp,
        })
        .collect::<Vec<_>>();
    let records_by_sequence = records
        .iter()
        .map(|record| (record.sequence, record))
        .collect::<BTreeMap<_, _>>();

    let group_expression = request
        .x_expression
        .as_ref()
        .or(request.series_by_expression.as_ref());
    if let Some(group_expression) = group_expression.filter(|_| {
        expressions
            .iter()
            .any(|expression| aggregate_method(expression.expression).is_some())
    }) {
        return Some(grouped_record_timing_series(
            telemetry,
            &record_inputs,
            &records_by_sequence,
            request,
            group_expression,
            expressions,
            unit,
            max_points,
            technique,
        ));
    }

    let series = expressions
        .into_iter()
        .flat_map(|expression| {
            let mut timing_points = timing_points_for_expression(
                telemetry,
                request,
                &record_inputs,
                &records_by_sequence,
                &expression,
            );
            retain_window(&mut timing_points, request.window_s, |point| {
                parse_timestamp(point.ended_at.as_str())
            });

            raw_record_timing_series(
                telemetry,
                &records_by_sequence,
                request,
                expression,
                timing_points,
                unit,
                max_points,
                technique,
            )
        })
        .collect::<Vec<_>>();

    Some(series)
}

/* === Private Functions === */
fn record_timing_expression(expression: &DataExpression) -> Option<RecordTimingExpression<'_>> {
    let DataSource::Record { record_type } = &expression.source else {
        return None;
    };
    expression.transforms.iter().find_map(|transform| {
        let DataTransform::TimeBetween {
            previous_record_type,
        } = transform
        else {
            return None;
        };
        Some(RecordTimingExpression {
            label: format!("{previous_record_type} -> {record_type}"),
            end_record_type: record_type,
            previous_record_type,
            expression,
        })
    })
}

#[allow(clippy::too_many_arguments)]
fn raw_record_timing_series(
    telemetry: &[TelemetrySample],
    records_by_sequence: &BTreeMap<u64, &RecordEntry>,
    request: &PlotPayloadRequest,
    expression: RecordTimingExpression<'_>,
    timing_points: Vec<RecordTimingPoint>,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    let x_axis = request
        .x_expression
        .as_ref()
        .and_then(record_telemetry_axis_expression);
    let group_expression = request.series_by_expression.as_ref();
    if x_axis.is_none() && group_expression.is_none() {
        let points = timing_points
            .into_iter()
            .map(|point| PlotPoint {
                x: JsonValue::String(point.ended_at.to_string()),
                y: Some(point.duration.0 as f64 / 1_000.0),
                z: None,
            })
            .collect::<Vec<_>>();
        return vec![finish_plot_series(
            expression.label.clone(),
            expression.label,
            unit.map(ToOwned::to_owned).or_else(|| Some("s".to_owned())),
            points,
            max_points,
            technique,
        )];
    }

    let x_lookup = x_axis.as_ref().map(|axis| {
        RecordSampleLookup::for_stream_matching(
            telemetry,
            axis.stream,
            request.scope.as_ref(),
            |sample| {
                sample_value(sample, axis.field)
                    .and_then(numeric_value)
                    .is_some()
            },
        )
    });
    let mut x_lookup = x_lookup;
    let mut group_lookup =
        RecordSampleLookup::for_stream(telemetry, &request.stream, request.scope.as_ref());
    let records = |sequence: &u64| records_by_sequence.get(sequence).copied();
    if let (Some(axis), Some(lookup)) = (&x_axis, x_lookup.as_mut()) {
        lookup.prepare(timing_points.iter().filter_map(|point| {
            let record = timing_axis_record(
                &expression,
                axis,
                records(&point.start_sequence)?,
                records(&point.end_sequence)?,
            )?;
            Some((
                record.scope.as_ref(),
                parse_timestamp(record.timestamp.as_str())?,
            ))
        }));
    }
    if group_expression.is_some() {
        group_lookup.prepare(timing_points.iter().filter_map(|point| {
            Some((
                records(&point.end_sequence)?.scope.as_ref(),
                parse_timestamp(point.ended_at.as_str())?,
            ))
        }));
    }
    let mut grouped = BTreeMap::<String, Vec<PlotPoint>>::new();
    let mut plain = Vec::new();

    for point in timing_points {
        let Some(start_record) = records_by_sequence.get(&point.start_sequence) else {
            continue;
        };
        let Some(end_record) = records_by_sequence.get(&point.end_sequence) else {
            continue;
        };
        let Some(end_time) = parse_timestamp(point.ended_at.as_str()) else {
            continue;
        };
        let x = if let (Some(axis), Some(lookup)) = (&x_axis, &x_lookup) {
            let Some(axis_record) = timing_axis_record(&expression, axis, start_record, end_record)
            else {
                continue;
            };
            let Some(axis_time) = parse_timestamp(axis_record.timestamp.as_str()) else {
                continue;
            };
            let Some(sample) = lookup.nearest_at_or_before(axis_record.scope.as_ref(), axis_time)
            else {
                continue;
            };
            sample_value(sample.sample, axis.field)
                .and_then(numeric_value)
                .map(JsonValue::from)
                .unwrap_or(JsonValue::Null)
        } else {
            JsonValue::String(point.ended_at.to_string())
        };
        let plot_point = PlotPoint {
            x,
            y: Some(point.duration.0 as f64 / 1_000.0),
            z: None,
        };
        if let Some(group_expression) = group_expression {
            let Some(sample) =
                group_lookup.nearest_at_or_before(end_record.scope.as_ref(), end_time)
            else {
                continue;
            };
            let Some(group) = expression_group_value(sample.sample, group_expression) else {
                continue;
            };
            grouped.entry(group).or_default().push(plot_point);
        } else {
            plain.push(plot_point);
        }
    }

    if group_expression.is_none() {
        return vec![finish_plot_series(
            expression.label.clone(),
            expression.label,
            unit.map(ToOwned::to_owned).or_else(|| Some("s".to_owned())),
            plain,
            max_points,
            technique,
        )];
    }

    grouped
        .into_iter()
        .map(|(group, points)| {
            finish_plot_series(
                group.clone(),
                group,
                unit.map(ToOwned::to_owned).or_else(|| Some("s".to_owned())),
                points,
                max_points,
                technique,
            )
        })
        .collect()
}

fn record_telemetry_axis_expression(
    expression: &DataExpression,
) -> Option<RecordTelemetryAxisExpression<'_>> {
    let DataSource::Record { record_type } = &expression.source else {
        return None;
    };
    expression.transforms.iter().find_map(|transform| {
        let DataTransform::TelemetryAtOrBefore { stream, field } = transform else {
            return None;
        };
        Some(RecordTelemetryAxisExpression {
            record_type,
            stream,
            field,
        })
    })
}

fn timing_axis_record<'a>(
    timing: &RecordTimingExpression<'_>,
    axis: &RecordTelemetryAxisExpression<'_>,
    start_record: &'a RecordEntry,
    end_record: &'a RecordEntry,
) -> Option<&'a RecordEntry> {
    if axis.record_type == timing.end_record_type {
        Some(end_record)
    } else if axis.record_type == timing.previous_record_type {
        Some(start_record)
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn grouped_record_timing_series(
    telemetry: &[TelemetrySample],
    record_inputs: &[RecordTimingRecord<'_>],
    records_by_sequence: &BTreeMap<u64, &RecordEntry>,
    request: &PlotPayloadRequest,
    group_expression: &DataExpression,
    expressions: Vec<RecordTimingExpression<'_>>,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    let mut samples =
        RecordSampleLookup::for_stream(telemetry, &request.stream, request.scope.as_ref());

    expressions
        .into_iter()
        .filter_map(|expression| {
            let method = aggregate_method(expression.expression)?;
            let mut timing_points = timing_points_for_expression(
                telemetry,
                request,
                record_inputs,
                records_by_sequence,
                &expression,
            );
            retain_window(&mut timing_points, request.window_s, |point| {
                parse_timestamp(point.ended_at.as_str())
            });
            samples.prepare(timing_points.iter().filter_map(|point| {
                Some((
                    records_by_sequence.get(&point.end_sequence)?.scope.as_ref(),
                    parse_timestamp(point.ended_at.as_str())?,
                ))
            }));

            let mut grouped = BTreeMap::<String, Vec<f64>>::new();
            for point in timing_points {
                let Some(end_record) = records_by_sequence.get(&point.end_sequence) else {
                    continue;
                };
                let Some(end_time) = parse_timestamp(point.ended_at.as_str()) else {
                    continue;
                };
                let Some(sample) =
                    samples.nearest_at_or_before(end_record.scope.as_ref(), end_time)
                else {
                    continue;
                };
                let Some(group) = expression_group_value(sample.sample, group_expression) else {
                    continue;
                };
                grouped
                    .entry(group)
                    .or_default()
                    .push(point.duration.0 as f64 / 1_000.0);
            }

            Some(finish_plot_series(
                expression.label.clone(),
                expression.label,
                unit.map(ToOwned::to_owned).or_else(|| Some("s".to_owned())),
                aggregate_group_points(grouped, method),
                max_points,
                technique,
            ))
        })
        .collect()
}

fn timing_points_for_expression(
    telemetry: &[TelemetrySample],
    request: &PlotPayloadRequest,
    record_inputs: &[RecordTimingRecord<'_>],
    records_by_sequence: &BTreeMap<u64, &RecordEntry>,
    expression: &RecordTimingExpression<'_>,
) -> Vec<RecordTimingPoint> {
    let mut filters = RecordFilterLookup::new(telemetry, request);
    let points = scoped_record_time_between(
        record_inputs,
        records_by_sequence,
        expression.end_record_type,
        expression.previous_record_type,
    );
    filters.prepare(
        points
            .iter()
            .filter_map(|point| records_by_sequence.get(&point.end_sequence).copied()),
    );
    points
        .into_iter()
        .filter(|point| {
            records_by_sequence
                .get(&point.end_sequence)
                .is_some_and(|record| filters.matches(record))
        })
        .collect()
}

pub(super) fn scoped_record_time_between(
    record_inputs: &[RecordTimingRecord<'_>],
    records_by_sequence: &BTreeMap<u64, &RecordEntry>,
    end_record_type: &str,
    previous_record_type: &str,
) -> Vec<RecordTimingPoint> {
    let mut records = record_inputs.to_vec();
    records.sort_by_key(|record| record.sequence);
    let entries = records
        .iter()
        .filter_map(|record| Some((*record, *records_by_sequence.get(&record.sequence)?)))
        .collect::<Vec<_>>();
    fn side<'a>(ordinal: usize, entry: &'a RecordEntry) -> Side<'a> {
        Side {
            ordinal,
            scope: entry.scope.as_ref(),
            time: nanos(&entry.timestamp),
            present: true,
        }
    }
    let starts = Lookup::new(
        entries
            .iter()
            .enumerate()
            .filter(|(_, (record, _))| record.record_type == previous_record_type)
            .map(|(ordinal, (_, entry))| side(ordinal, entry)),
        Strategy::Previous,
    );
    let ends = entries
        .iter()
        .enumerate()
        .filter(|(_, (record, _))| record.record_type == end_record_type)
        .collect::<Vec<_>>();
    let probes = ends
        .iter()
        .map(|(ordinal, (_, entry))| side(*ordinal, entry))
        .collect::<Vec<_>>();
    ends.iter()
        .zip(starts.find_many(&probes))
        .filter_map(|((_, (record, _)), start)| Some((*record, entries[start?].0)))
        .flat_map(|(end, start)| {
            record_time_between([start, end], end_record_type, previous_record_type)
        })
        .collect()
}

/* === Tests === */
