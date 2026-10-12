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
use std::cmp::Ordering;
// Third Party Imports

// Local Imports
use dokime::core::{Identifier, Metadata, Value, ValueType};
use dokime::telemetry::{TelemetryContract, TelemetrySample};

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
    if values.is_empty() {
        return Vec::new();
    }
    let count = values.len() as f64;
    let sum = values.iter().sum::<f64>();
    let mean = sum / count;
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / count;
    let rms = (values.iter().map(|value| value * value).sum::<f64>() / count).sqrt();
    let mut ordered = values.to_vec();
    ordered.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
    let statistics = [
        ("count", count),
        ("first", values[0]),
        ("last", values[values.len() - 1]),
        ("min", ordered[0]),
        ("max", ordered[ordered.len() - 1]),
        ("mean", mean),
        ("median", percentile(&ordered, 50)),
        ("std", variance.sqrt()),
        ("rms", rms),
        (
            "max_abs",
            values
                .iter()
                .map(|value| value.abs())
                .fold(0.0_f64, f64::max),
        ),
        ("sum", sum),
        ("p95", percentile(&ordered, 95)),
        ("p99", percentile(&ordered, 99)),
    ];
    statistics
        .into_iter()
        .map(|(stat, value)| output(stream, field, stat, value))
        .collect()
}

fn percentile(ordered: &[f64], rank: usize) -> f64 {
    let index = (ordered.len().saturating_sub(1) * rank + 50) / 100;
    ordered[index]
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
