/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) use crate::streams::AggregateOutput;

use harmos::Aggregate;
use harmos::storage::query::{
    Accumulator,
    scalar::{Agg, ScalarValue},
};

use crate::telemetry::TelemetryContract;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueType;

struct AggregateCancellation(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Drop for AggregateCancellation {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }
}

pub(crate) async fn project_saved(
    session: &crate::storage::Manager, local: &crate::storage::sqlite::State, settings: &harmos::config::SchemaSettings,
    scope: &dokime_domain::model::execution::RecordScope, contracts: &[TelemetryContract],
) -> crate::runtime::Result<Vec<AggregateOutput>> {
    let at = session.cursor_saved();
    let codecs = crate::runtime::storage::telemetry::saved_codecs(session, &at, None, settings, |id, _| {
        contracts.iter().any(|contract| id.strip_prefix("dokime.telemetry.") == Some(contract.stream.as_str()))
    })
    .await?;
    let mut outputs = Vec::new();
    for contract in contracts {
        let mut columns = contract.normalized().columns;
        columns.retain(|column| matches!(column.value_type, ValueType::Integer | ValueType::Number));
        if columns.is_empty() {
            continue;
        }
        // The Saved Cursor Is Immutable: Reuse Its First Bounded Page Across Fields.
        let execution = scope.test_execution_id.as_ref().map(ToString::to_string);
        let stream = Some(contract.stream.to_string());
        let first_page = local.telemetry_page(at.clone(), 0, execution.clone(), stream.clone()).await?;
        let first_page = decode_page(first_page, &codecs)?;
        if first_page.is_empty() {
            continue;
        }
        let mut end = None;
        for column in columns {
            let field = column.metadata.identifier.as_str();
            let statistics = [
                Aggregate::Count,
                Aggregate::Min,
                Aggregate::Max,
                Aggregate::Mean,
                Aggregate::Median,
                Aggregate::Std,
                Aggregate::Rms,
                Aggregate::Sum,
                Aggregate::Percentile(95),
                Aggregate::Percentile(99),
            ];
            let mut accumulators = statistics.map(|statistic| Accumulator::new(Agg::Of(statistic)));
            let mut first = None;
            let mut last = None;
            let mut max_abs = 0f64;
            let mut after = 0;
            let mut page = first_page.as_slice();
            let mut next_page;
            loop {
                for (position, sample) in page {
                    after = *position;
                    if !dokime_domain::model::execution::scope_visible_in(sample.scope.as_ref(), scope) {
                        continue;
                    }
                    let value = match sample.values.get(field) {
                        Some(Value::Integer(value)) => Some(*value as f64),
                        Some(Value::Unsigned(value)) => Some(*value as f64),
                        Some(Value::Number(value)) => Some(*value),
                        _ => None,
                    };
                    let Some(value) = value.filter(|value| value.is_finite()) else {
                        continue;
                    };
                    first.get_or_insert(value);
                    last = Some(value);
                    max_abs = max_abs.max(value.abs());
                    for accumulator in &mut accumulators {
                        accumulator.push(after, &ScalarValue::Float(value))?;
                    }
                }
                tokio::task::yield_now().await;
                if end == Some(after) {
                    break;
                }
                let rows = local.telemetry_page(at.clone(), after, execution.clone(), stream.clone()).await?;
                next_page = decode_page(rows, &codecs)?;
                if next_page.is_empty() {
                    end = Some(after);
                    break;
                }
                page = &next_page;
            }
            let (Some(first), Some(last)) = (first, last) else {
                continue;
            };
            let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let cancellation = AggregateCancellation(cancelled.clone());
            let values = tokio::task::spawn_blocking(move || -> crate::runtime::Result<Vec<(String, Option<f64>)>> {
                let mut values = Vec::new();
                for (statistic, accumulator) in statistics.into_iter().zip(accumulators) {
                    let value = accumulator.finish(&mut || {
                        if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                            return Err(harmos::storage::Error::InvalidInput {
                                field: "telemetry aggregate",
                                reason: "cancelled",
                            });
                        }
                        Ok(())
                    })?;
                    let value = match value {
                        ScalarValue::Float(value) => Some(value),
                        ScalarValue::Unsigned(value) => Some(value as f64),
                        _ => None,
                    };
                    values.push((statistic.name(), value));
                }
                Ok(values)
            })
            .await??;
            drop(cancellation);
            if let Some(value) = values[0].1 {
                outputs.push(output(&contract.stream, field, &values[0].0, value));
            }
            outputs.push(output(&contract.stream, field, "first", first));
            outputs.push(output(&contract.stream, field, "last", last));
            for (name, value) in &values[1..7] {
                if let Some(value) = value {
                    outputs.push(output(&contract.stream, field, name, *value));
                }
            }
            outputs.push(output(&contract.stream, field, "max_abs", max_abs));
            for (name, value) in &values[7..] {
                if let Some(value) = value {
                    outputs.push(output(&contract.stream, field, name, *value));
                }
            }
        }
    }
    Ok(outputs)
}

fn decode_page(
    rows: Vec<harmos::storage::StoredRow>, codecs: &[crate::storage::telemetry::TypedCodec],
) -> crate::runtime::Result<Vec<(u64, crate::telemetry::TelemetrySample)>> {
    rows.into_iter()
        .map(|row| {
            let codec = codecs
                .iter()
                .find(|codec| codec.schema() == &row.record.schema)
                .ok_or_else(|| crate::runtime::Error::Operation("saved telemetry codec is missing".into()))?;
            let entry = codec.decode_stored(&row)?;
            Ok((row.record.position.sequence, entry.sample))
        })
        .collect()
}

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
