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

// Third Party Imports

// Local Imports
use dokime_domain::model;
use dokime_host::{Host, HostTelemetryStream, host_telemetry_contracts};
use dokime_runtime::telemetry::validate_telemetry_sample;
use model::telemetry::{TelemetryBatch, TelemetrySample, ValidationMode};
use model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
pub enum Field {
    Integer(&'static str),
    Number(&'static str),
    Array(&'static str),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn collect_required_native_batch(stream: HostTelemetryStream) -> TelemetryBatch {
    Host::default()
        .collect_stream(stream)
        .expect("native host collection should return a structured result")
        .unwrap_or_else(|| panic!("native host collection for {} should produce a telemetry sample", stream.as_str()))
}

pub fn validate_batch(batch: &TelemetryBatch) {
    let contract = host_telemetry_contracts()
        .into_iter()
        .find(|contract| contract.stream == batch.stream)
        .unwrap_or_else(|| panic!("missing built-in telemetry contract for {}", batch.stream.as_str()));

    for sample in &batch.samples {
        validate_telemetry_sample(Some(&contract), sample, ValidationMode::Strict)
            .expect("host telemetry sample should match the strict built-in contract");
    }
}

pub fn assert_text_in(sample: &TelemetrySample, key: &str, expected: &[&str]) {
    let Some(Value::Text(value)) = sample.values.get(key) else {
        panic!("expected text field {key}");
    };

    assert!(expected.contains(&value.as_str()), "unexpected {key} value {value:?}; expected one of {expected:?}");
}

pub fn assert_field(sample: &TelemetrySample, field: &Field) {
    match field {
        Field::Integer(key) => {
            let Some(Value::Integer(value)) = sample.values.get(*key) else {
                panic!("expected integer field {key}");
            };
            assert!(*value >= 0, "expected {key} to be non-negative");
        }
        Field::Number(key) => {
            let Some(Value::Number(value)) = sample.values.get(*key) else {
                panic!("expected number field {key}");
            };
            assert!(value.is_finite(), "expected {key} to be finite");
        }
        Field::Array(key) => {
            let Some(Value::Array(_)) = sample.values.get(*key) else {
                panic!("expected array field {key}");
            };
        }
    }
}

/* === Private Functions === */

/* === Tests === */
