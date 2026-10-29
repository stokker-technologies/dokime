/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use harmos::storage::ReadBatch;
use serde_json::Value as JsonValue;

use super::{PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotPoint, PlotSeries};
use crate::runtime::{Error, Result};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::values::Value;

pub(super) fn plain(
    telemetry: &[TelemetrySample], records: crate::records::view::View<'_>, request: &PlotPayloadRequest, generated_at: &str,
    context: &PlotPayloadContext,
) -> Result<ReadBatch<PlotPayload>> {
    let selection = crate::plot::record_samples::PlotSamples::new(telemetry, request);
    let selected = selection.collect();
    let output = harmos::storage::ReadBatch::new(());
    let payload = output.try_map(|_| {
        let series = plain_series(&selected, telemetry, request)?;
        Ok::<_, Error>(super::payload_from_series(&selected, records, request, generated_at, context, series))
    })?;
    Ok(payload)
}

pub(super) fn json_value(value: &Value) -> Result<JsonValue> {
    if !valid_json(value) {
        return Ok(JsonValue::Null);
    }
    json_valid(value)
}

fn plain_series(samples: &[&TelemetrySample], telemetry: &[TelemetrySample], request: &PlotPayloadRequest) -> Result<Vec<PlotSeries>> {
    let x_key = request.x_key.as_deref().unwrap_or("timestamp");
    let z = super::request_z_index(telemetry, request);
    let expressions = request.y_expressions.iter().filter(|expression| super::is_row_expression(expression));
    let expression_count = expressions.clone().count();
    let series_count = if expression_count == 0 { request.y_keys.len() } else { expression_count };
    let mut series = vector(series_count)?;
    if expression_count == 0 {
        for key in &request.y_keys {
            let mut points = vector(samples.len())?;
            for sample in samples {
                points.push(PlotPoint {
                    x: x_value(super::x_input(sample, x_key))?,
                    y: sample.values.get(key).and_then(super::numeric_value),
                    z: z.resolve(sample),
                });
            }
            series.push(series_entry(text(key)?, text(key)?, request.unit.as_deref(), points)?);
        }
        return Ok(series);
    }

    for expression in expressions {
        let selected = samples.iter().filter(|sample| super::expression_numeric_value(sample, expression).is_some());
        let mut points = vector(selected.clone().count())?;
        for sample in selected {
            let x = super::expression_x_input(sample, request.x_expression.as_ref(), x_key);
            points.push(PlotPoint {
                x: x_value(x)?,
                y: super::expression_numeric_value(sample, expression),
                z: z.resolve(sample),
            });
        }
        let label = super::expression_field_name(expression).unwrap_or("value");
        series.push(series_entry(text(label)?, text(label)?, request.unit.as_deref(), points)?);
    }
    Ok(series)
}

pub(super) fn series_entry(key: String, label: String, unit: Option<&str>, points: Vec<PlotPoint>) -> Result<PlotSeries> {
    let raw_points = points.len();
    Ok(PlotSeries {
        key,
        label,
        unit: unit.map(text).transpose()?,
        points,
        raw_points,
        downsampled: false,
        format: None,
    })
}

pub(super) fn x_value(input: super::XInput<'_>) -> Result<JsonValue> {
    match input {
        super::XInput::Missing => Ok(JsonValue::Null),
        super::XInput::Timestamp(value) => Ok(JsonValue::String(text(value)?)),
        super::XInput::Value(value) => json_value(value),
    }
}

pub(super) fn vector<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| memory())?;
    Ok(values)
}

pub(super) fn text(value: &str) -> Result<String> {
    let mut text = String::new();
    text.try_reserve_exact(value.len()).map_err(|_| memory())?;
    text.push_str(value);
    Ok(text)
}

fn valid_json(value: &Value) -> bool {
    match value {
        Value::Number(value) => value.is_finite(),
        Value::Datetime(value) => crate::storage::datetime_memory(*value).is_ok(),
        Value::Array(values) => values.iter().all(valid_json),
        Value::Object(values) => values.values().all(valid_json),
        _ => true,
    }
}

fn json_valid(value: &Value) -> Result<JsonValue> {
    Ok(match value {
        Value::Null => JsonValue::Null,
        Value::Bool(value) => (*value).into(),
        Value::Integer(value) => (*value).into(),
        Value::Unsigned(value) => (*value).into(),
        Value::Number(value) => JsonValue::from(*value),
        Value::Text(value) => text(value)?.into(),
        Value::Datetime(value) => crate::storage::encode_datetime(*value)?.into(),
        Value::Array(values) => {
            let mut output = vector(values.len())?;
            for value in values {
                output.push(json_valid(value)?);
            }
            JsonValue::Array(output)
        }
        Value::Object(values) => {
            let mut output = serde_json::Map::new();
            for (name, value) in values {
                output.insert(text(name)?, json_valid(value)?);
            }
            JsonValue::Object(output)
        }
    })
}

pub(super) fn memory() -> Error {
    harmos::Error::ResourceExhausted { resource: "initial plot projection" }.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn exact_json_projection_preserves_recursive_value_semantics() {
        let datetime =
            time::OffsetDateTime::parse("2026-10-25T12:34:56.123+02:00", &time::format_description::well_known::Rfc3339).unwrap();
        let array = Value::Array(vec![Value::Null, Value::Bool(true), Value::Integer(i64::MIN), Value::Unsigned(u64::MAX)]);
        let values = BTreeMap::from([
            ("array".into(), array),
            ("datetime".into(), Value::Datetime(datetime)),
            ("text".into(), Value::Text("quoted \" text".into())),
            ("zero".into(), Value::Number(-0.0)),
        ]);
        let value = Value::Object(values);
        assert_eq!(json_value(&value).unwrap(), value.to_plain_json().unwrap());
    }

    #[test]
    fn invalid_nested_numbers_replace_the_entire_json_value() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let value = Value::Array(vec![Value::Text("prefix".into()), Value::Number(invalid)]);
            assert!(value.to_plain_json().is_err());
            assert_eq!(json_value(&value).unwrap(), JsonValue::Null);
        }
    }
}
