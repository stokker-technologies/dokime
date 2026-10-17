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
use time::OffsetDateTime;
// Local Imports
use super::record_timing::{RecordFilterLookup, scoped_record_time_between};
use crate::plot::{
    DownsampleTechnique, PlotPayloadRequest, PlotPoint, PlotSeries, aggregate_group_points,
    aggregate_method, expression_group_value, finish_plot_series, numeric_value, parse_timestamp,
    record_samples::RecordSampleLookup, retain_window, sample_matches_scope, sample_value,
};
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::execution::RecordScope;
use crate::expressions::{DataExpression, DataSource, DataTransform, RecordTimingRecord};
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
struct RecordTelemetryExpression<'a> {
    label: String,
    record_type: &'a str,
    stream: &'a str,
    field: &'a str,
    expression: &'a DataExpression,
}

#[derive(Clone, Debug)]
struct RecordTelemetryValue<'a> {
    record_timestamp: String,
    record_time: OffsetDateTime,
    sample: &'a TelemetrySample,
    value: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn record_telemetry_series(
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
        .filter_map(record_telemetry_expression)
        .collect::<Vec<_>>();
    if expressions.is_empty() {
        return None;
    }

    let group_expression = request
        .x_expression
        .as_ref()
        .or(request.series_by_expression.as_ref());
    if let Some(group_expression) = group_expression.filter(|_| {
        expressions
            .iter()
            .any(|expression| aggregate_method(expression.expression).is_some())
    }) {
        return Some(grouped_record_telemetry_series(
            telemetry,
            records,
            request,
            group_expression,
            expressions,
            unit,
            max_points,
            technique,
        ));
    }

    Some(plain_record_telemetry_series(
        telemetry,
        records,
        request,
        expressions,
        unit,
        max_points,
        technique,
    ))
}

/* === Private Functions === */
fn plain_record_telemetry_series(
    telemetry: &[TelemetrySample],
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
    expressions: Vec<RecordTelemetryExpression<'_>>,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    if request
        .x_expression
        .as_ref()
        .and_then(record_telemetry_expression)
        .is_some()
        || request.series_by_expression.is_some()
    {
        return contextual_record_telemetry_series(
            telemetry,
            records,
            request,
            expressions,
            unit,
            max_points,
            technique,
        );
    }

    expressions
        .into_iter()
        .map(|expression| {
            let points = record_telemetry_values(telemetry, records, request, &expression)
                .into_iter()
                .map(|value| PlotPoint {
                    x: JsonValue::String(value.record_timestamp),
                    y: Some(value.value),
                    z: None,
                })
                .collect::<Vec<_>>();

            finish_plot_series(
                expression.label.clone(),
                expression.label,
                unit.map(ToOwned::to_owned),
                points,
                max_points,
                technique,
            )
        })
        .collect()
}

fn contextual_record_telemetry_series(
    telemetry: &[TelemetrySample],
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
    expressions: Vec<RecordTelemetryExpression<'_>>,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    let x_expression = request
        .x_expression
        .as_ref()
        .and_then(record_telemetry_expression);
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

    expressions
        .into_iter()
        .flat_map(|expression| {
            contextual_record_telemetry_series_for_expression(
                telemetry,
                records,
                &record_inputs,
                &records_by_sequence,
                request,
                &expression,
                x_expression.as_ref(),
                unit,
                max_points,
                technique,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn contextual_record_telemetry_series_for_expression(
    telemetry: &[TelemetrySample],
    records: &[RecordEntry],
    record_inputs: &[RecordTimingRecord<'_>],
    records_by_sequence: &BTreeMap<u64, &RecordEntry>,
    request: &PlotPayloadRequest,
    expression: &RecordTelemetryExpression<'_>,
    x_expression: Option<&RecordTelemetryExpression<'_>>,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    let mut y_lookup = record_telemetry_lookup(telemetry, request, expression);
    let mut x_lookup =
        x_expression.map(|expression| record_telemetry_lookup(telemetry, request, expression));
    let mut group_lookup =
        RecordSampleLookup::for_stream(telemetry, &request.stream, request.scope.as_ref());
    let mut grouped = BTreeMap::<String, Vec<PlotPoint>>::new();
    let mut plain = Vec::new();

    let mut pairs = paired_records(
        records,
        record_inputs,
        records_by_sequence,
        expression,
        x_expression,
    );
    let mut filters = RecordFilterLookup::new(telemetry, request);
    pairs.retain(|(_, record)| sample_matches_scope(record.scope.as_ref(), request.scope.as_ref()));
    filters.prepare(pairs.iter().map(|(_, record)| *record));
    pairs.retain(|(_, record)| filters.matches(record));
    retain_window(&mut pairs, request.window_s, |(_, record)| {
        parse_timestamp(record.timestamp.as_str())
    });
    fn probe(record: &RecordEntry) -> Option<(Option<&RecordScope>, OffsetDateTime)> {
        Some((
            record.scope.as_ref(),
            parse_timestamp(record.timestamp.as_str())?,
        ))
    }
    y_lookup.prepare(pairs.iter().filter_map(|(_, record)| probe(record)));
    if let Some(lookup) = x_lookup.as_mut() {
        lookup.prepare(pairs.iter().filter_map(|(record, _)| probe((*record)?)));
    }
    if request.series_by_expression.is_some() {
        group_lookup.prepare(pairs.iter().filter_map(|(_, record)| probe(record)));
    }

    for (x_record, y_record) in pairs {
        let Some(y_time) = parse_timestamp(y_record.timestamp.as_str()) else {
            continue;
        };
        let Some(y_sample) = y_lookup.nearest_at_or_before(y_record.scope.as_ref(), y_time) else {
            continue;
        };
        let Some(y) = sample_value(y_sample.sample, expression.field).and_then(numeric_value)
        else {
            continue;
        };
        let x = if let (Some(x_expression), Some(x_lookup), Some(x_record)) =
            (x_expression, x_lookup.as_ref(), x_record)
        {
            let Some(x_time) = parse_timestamp(x_record.timestamp.as_str()) else {
                continue;
            };
            let Some(x_sample) = x_lookup.nearest_at_or_before(x_record.scope.as_ref(), x_time)
            else {
                continue;
            };
            sample_value(x_sample.sample, x_expression.field)
                .and_then(numeric_value)
                .map(JsonValue::from)
                .unwrap_or(JsonValue::Null)
        } else {
            JsonValue::String(y_record.timestamp.to_string())
        };
        let point = PlotPoint {
            x,
            y: Some(y),
            z: None,
        };

        if let Some(group_expression) = request.series_by_expression.as_ref() {
            let Some(group_sample) =
                group_lookup.nearest_at_or_before(y_record.scope.as_ref(), y_time)
            else {
                continue;
            };
            let Some(group) = expression_group_value(group_sample.sample, group_expression) else {
                continue;
            };
            grouped.entry(group).or_default().push(point);
        } else {
            plain.push(point);
        }
    }

    if request.series_by_expression.is_none() {
        return vec![finish_plot_series(
            expression.label.clone(),
            expression.label.clone(),
            unit.map(ToOwned::to_owned),
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
                unit.map(ToOwned::to_owned),
                points,
                max_points,
                technique,
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn grouped_record_telemetry_series(
    telemetry: &[TelemetrySample],
    records: &[RecordEntry],
    request: &PlotPayloadRequest,
    group_expression: &DataExpression,
    expressions: Vec<RecordTelemetryExpression<'_>>,
    unit: Option<&str>,
    max_points: usize,
    technique: DownsampleTechnique,
) -> Vec<PlotSeries> {
    expressions
        .into_iter()
        .filter_map(|expression| {
            let method = aggregate_method(expression.expression)?;
            let mut grouped = BTreeMap::<String, Vec<f64>>::new();
            for value in record_telemetry_values(telemetry, records, request, &expression) {
                let Some(group) = expression_group_value(value.sample, group_expression) else {
                    continue;
                };
                grouped.entry(group).or_default().push(value.value);
            }

            Some(finish_plot_series(
                expression.label.clone(),
                expression.label,
                unit.map(ToOwned::to_owned),
                aggregate_group_points(grouped, method),
                max_points,
                technique,
            ))
        })
        .collect()
}

fn record_telemetry_values<'a>(
    telemetry: &'a [TelemetrySample],
    records: &'a [RecordEntry],
    request: &PlotPayloadRequest,
    expression: &RecordTelemetryExpression<'_>,
) -> Vec<RecordTelemetryValue<'a>> {
    let mut samples = RecordSampleLookup::for_stream_matching(
        telemetry,
        expression.stream,
        request.scope.as_ref(),
        |sample| {
            sample_value(sample, expression.field)
                .and_then(numeric_value)
                .is_some()
        },
    );
    let mut filters = RecordFilterLookup::new(telemetry, request);
    let typed = records
        .iter()
        .filter(|record| record.record_type.as_str() == expression.record_type)
        .collect::<Vec<_>>();
    filters.prepare(typed.iter().copied());
    let selected = typed
        .into_iter()
        .filter(|record| filters.matches(record))
        .filter(|record| sample_matches_scope(record.scope.as_ref(), request.scope.as_ref()))
        .collect::<Vec<_>>();
    samples.prepare(selected.iter().filter_map(|record| {
        Some((
            record.scope.as_ref(),
            parse_timestamp(record.timestamp.as_str())?,
        ))
    }));
    let mut values = selected
        .into_iter()
        .filter_map(|record| {
            let record_time = parse_timestamp(record.timestamp.as_str())?;
            let sample = samples.nearest_at_or_before(record.scope.as_ref(), record_time)?;
            let value = sample_value(sample.sample, expression.field).and_then(numeric_value)?;
            Some(RecordTelemetryValue {
                record_timestamp: record.timestamp.to_string(),
                record_time,
                sample: sample.sample,
                value,
            })
        })
        .collect::<Vec<_>>();

    retain_window(&mut values, request.window_s, |value| {
        Some(value.record_time)
    });
    values
}

fn record_telemetry_lookup<'a>(
    telemetry: &'a [TelemetrySample],
    request: &PlotPayloadRequest,
    expression: &RecordTelemetryExpression<'_>,
) -> RecordSampleLookup<'a> {
    RecordSampleLookup::for_stream_matching(
        telemetry,
        expression.stream,
        request.scope.as_ref(),
        |sample| {
            sample_value(sample, expression.field)
                .and_then(numeric_value)
                .is_some()
        },
    )
}

fn paired_records<'a>(
    records: &'a [RecordEntry],
    record_inputs: &[RecordTimingRecord<'_>],
    records_by_sequence: &BTreeMap<u64, &'a RecordEntry>,
    y_expression: &RecordTelemetryExpression<'_>,
    x_expression: Option<&RecordTelemetryExpression<'_>>,
) -> Vec<(Option<&'a RecordEntry>, &'a RecordEntry)> {
    let Some(x_expression) = x_expression else {
        return records
            .iter()
            .filter(|record| record.record_type.as_str() == y_expression.record_type)
            .map(|record| (None, record))
            .collect();
    };
    if x_expression.record_type == y_expression.record_type {
        return records
            .iter()
            .filter(|record| record.record_type.as_str() == y_expression.record_type)
            .map(|record| (Some(record), record))
            .collect();
    }

    scoped_record_time_between(
        record_inputs,
        records_by_sequence,
        y_expression.record_type,
        x_expression.record_type,
    )
    .into_iter()
    .filter_map(|point| {
        let start = records_by_sequence.get(&point.start_sequence).copied()?;
        let end = records_by_sequence.get(&point.end_sequence).copied()?;
        Some((Some(start), end))
    })
    .collect()
}

fn record_telemetry_expression(
    expression: &DataExpression,
) -> Option<RecordTelemetryExpression<'_>> {
    let DataSource::Record { record_type } = &expression.source else {
        return None;
    };
    expression.transforms.iter().find_map(|transform| {
        let DataTransform::TelemetryAtOrBefore { stream, field } = transform else {
            return None;
        };
        Some(RecordTelemetryExpression {
            label: format!("{field} at {record_type}"),
            record_type,
            stream,
            field,
            expression,
        })
    })
}

/* === Tests === */
