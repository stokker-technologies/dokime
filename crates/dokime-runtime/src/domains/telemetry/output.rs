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

// Third Party Imports
// Local Imports
use crate::telemetry::TelemetryContract;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn project_saved(
    session: &crate::storage::Manager, local: &crate::storage::sqlite::State, scope: &dokime_domain::model::execution::RecordScope,
    contracts: &[TelemetryContract],
) -> crate::runtime::Result<Vec<AggregateOutput>> {
    let execution = scope
        .test_execution_id
        .as_ref()
        .ok_or_else(|| crate::runtime::Error::Operation("telemetry outputs require an execution".into()))?;
    // The Raw Rows And Their Summaries Share One Durable Transaction.
    let through = session.cursor_accepted();
    session.flush_until(&through).await?;
    let summaries = local.numeric_summaries(execution.to_string()).await?;
    let mut outputs = Vec::new();
    for contract in contracts {
        for column in contract.normalized().columns {
            if !matches!(column.value_type, ValueType::Integer | ValueType::Number) {
                continue;
            }
            let field = column.metadata.identifier.as_str();
            let Some(summary) = summaries.iter().find(|summary| summary.stream == contract.stream.as_str() && summary.field == field)
            else {
                continue;
            };
            outputs.extend(summary.statistics.iter().map(|(name, value)| output(&contract.stream, field, name, *value)));
        }
    }
    Ok(outputs)
}

/* === Private Functions === */

fn output(stream: &Identifier, field: &str, stat: &str, value: f64) -> AggregateOutput {
    let key = valid_identifier(&format!("telemetry.{}.values.{field}.{stat}", stream.as_str()));
    AggregateOutput {
        metadata: {
            let mut metadata = Metadata::definition(
                Identifier::new(&key).expect("aggregate keys are sanitized"),
                format!("{} {} {}", stream, field.replace('_', " "), stat.replace('_', " ")),
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
        .map(|character| if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') { character } else { '_' })
        .collect()
}

/* === Tests === */
