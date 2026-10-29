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
use dokime_host::{HostOptions, HostTelemetryStream, host_telemetry_contracts};
use dokime::telemetry::validate_telemetry_sample;
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
    use dokime_host::collectors::{bluetooth::BluetoothCollector, network::NetworkCollector, system::SystemCollector, wifi::WifiCollector};
    let options = HostOptions::default();
    let mut wifi = WifiCollector::standalone(options.clone());
    let rows = match stream {
        HostTelemetryStream::System => vec![SystemCollector::new().collect().unwrap()],
        HostTelemetryStream::Network => vec![NetworkCollector::new().collect().unwrap()],
        HostTelemetryStream::Wifi => vec![wifi.collect_link_with_options(&options).unwrap()],
        HostTelemetryStream::WifiLatency => vec![wifi.collect_latency_with_options(&options).unwrap()],
        HostTelemetryStream::WifiChannelOccupancy => wifi.collect_channel_occupancy_with_options(&options).unwrap(),
        HostTelemetryStream::Bluetooth => vec![BluetoothCollector::new(options).collect().unwrap()],
    };
    let samples = rows
        .into_iter()
        .map(|row| {
            let mut sample = TelemetrySample::new(dokime::model::identity::new_id(), stream.identifier(), 1, row.timestamp, "host");
            sample.values = row.values;
            sample.values.insert("timestamp".into(), Value::from(sample.timestamp.clone()));
            sample
        })
        .collect();
    TelemetryBatch::new(dokime::model::identity::new_id(), stream.identifier(), samples)
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
