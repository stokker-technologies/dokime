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
pub(crate) use crate::streams::AggregateOutput;

/* === Imports === */
// Standard Library Imports
use std::borrow::Cow;
// Third Party Imports
use harmos::Aggregate;
use harmos::signal::{Aggregates, Series};

// Local Imports
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueType;
use crate::telemetry::{TelemetryContract, TelemetrySample};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn project_samples(
    contracts: &[TelemetryContract],
    samples: &[TelemetrySample],
) -> Vec<AggregateOutput> {
    let mut outputs = Vec::new();
    for contract in contracts {
        let stream_samples = samples
            .iter()
            .filter(|sample| sample.stream == contract.stream)
            .collect::<Vec<_>>();
        for column in
            contract.normalized().columns.into_iter().filter(|column| {
                matches!(column.value_type, ValueType::Integer | ValueType::Number)
            })
        {
            let values = stream_samples
                .iter()
                .filter_map(|sample| sample.values.get(column.metadata.identifier.as_str()))
                .filter_map(|value| match value {
                    Value::Integer(value) => Some(*value as f64),
                    Value::Unsigned(value) => Some(*value as f64),
                    Value::Number(value) => Some(*value),
                    _ => None,
                })
                .filter(|value| value.is_finite())
                .collect::<Vec<_>>();
            outputs.extend(project_field(
                &contract.stream,
                column.metadata.identifier.as_str(),
                &values,
            ));
        }
    }
    outputs
}

/* === Private Functions === */
fn project_field(stream: &Identifier, field: &str, values: &[f64]) -> Vec<AggregateOutput> {
    let (Some(first), Some(last)) = (values.first(), values.last()) else {
        return Vec::new();
    };
    let mut series = Series::new(values);
    series.sort();
    // `first`, `last` and `max_abs` read the capture order and magnitude, which
    // no member of the vocabulary names; every statistic is the Harmos one.
    let statistic = |aggregate: Aggregate| {
        series
            .aggregate(aggregate)
            .map(|value| (Cow::Owned(aggregate.name()), value))
    };
    let max_abs = values
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max);
    let statistics = [
        statistic(Aggregate::Count),
        Some((Cow::Borrowed("first"), *first)),
        Some((Cow::Borrowed("last"), *last)),
        statistic(Aggregate::Min),
        statistic(Aggregate::Max),
        statistic(Aggregate::Mean),
        statistic(Aggregate::Median),
        statistic(Aggregate::Std),
        statistic(Aggregate::Rms),
        Some((Cow::Borrowed("max_abs"), max_abs)),
        statistic(Aggregate::Sum),
        statistic(Aggregate::Percentile(95)),
        statistic(Aggregate::Percentile(99)),
    ];
    statistics
        .into_iter()
        .flatten()
        .map(|(stat, value)| output(stream, field, &stat, value))
        .collect()
}

fn output(stream: &Identifier, field: &str, stat: &str, value: f64) -> AggregateOutput {
    let key = valid_identifier(&format!(
        "telemetry.{}.values.{field}.{stat}",
        stream.as_str()
    ));
    AggregateOutput {
        metadata: {
            let mut metadata = Metadata::definition(
                Identifier::new(&key).expect("aggregate keys are sanitized"),
                format!(
                    "{} {} {}",
                    stream,
                    field.replace('_', " "),
                    stat.replace('_', " ")
                ),
                Some(format!("Telemetry aggregate {stat} for {stream} {field}.")),
                ["output", "telemetry", "aggregate"],
            );
            metadata.created_at = Some(time::OffsetDateTime::now_utc());
            metadata
        },
        value: Value::Number(value),
        key,
    }
}

fn valid_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/* === Tests === */
