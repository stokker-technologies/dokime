/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native host collection: every stream reads this machine through the
// collectors' own platform probes (native APIs on Windows; nmcli, iw, ping and
// bluetoothctl on Linux).

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use model::telemetry::{TelemetryBatch, TelemetrySample};
use model::telemetry::ValidationMode;
use dokime_host::host_telemetry_contract;

// Local Imports
use crate::support::*;
use dokime::telemetry::TelemetryRow;
use dokime::telemetry::validate_telemetry_sample;
use dokime_domain::model;
use dokime_host::HostTelemetryStream;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn single_sample(batch: &TelemetryBatch) -> &TelemetrySample {
    assert_eq!(batch.samples.len(), 1);
    &batch.samples[0]
}

/* === Tests === */

#[test]
fn host_stream_row() {
    let streams: &[(HostTelemetryStream, &[&str], &[Field])] = &[
        (
            HostTelemetryStream::System,
            &["ok"],
            &[
                Field::Integer("sample_index"),
                Field::Number("cpu_load_percent"),
                Field::Integer("memory_total_bytes"),
                Field::Integer("memory_used_bytes"),
                Field::Integer("temperature_sensor_count"),
            ],
        ),
        (
            HostTelemetryStream::Network,
            &["ok"],
            &[
                Field::Integer("sample_index"),
                Field::Integer("network_interface_count"),
                Field::Number("network_recv_mb_per_second"),
                Field::Number("network_sent_mb_per_second"),
                Field::Integer("network_total_recv_bytes"),
                Field::Integer("network_total_sent_bytes"),
                Field::Integer("local_address_count"),
                Field::Array("local_addresses"),
            ],
        ),
        (HostTelemetryStream::WifiLink, &["ok", "unavailable", "unsupported", "error"], &[Field::Integer("sample_index")]),
        (HostTelemetryStream::WifiLatency, &["ok", "error"], &[Field::Integer("sample_index")]),
        (HostTelemetryStream::Bluetooth, &["ok", "stale", "unsupported", "error"], &[Field::Integer("sample_index")]),
    ];

    for (stream, probe_statuses, fields) in streams {
        let batch = collect_required_native_batch(*stream);
        validate_batch(&batch);

        let sample = single_sample(&batch);
        assert_text_in(sample, "probe_status", probe_statuses);
        for field in *fields {
            assert_field(sample, field);
        }
    }
}

#[test]
fn collected_battery_columns() {
    // Whatever this machine reports — every reading, or none on a host
    // without a battery — is a declared, correctly typed `host.system` column.
    let batch = collect_required_native_batch(HostTelemetryStream::System);
    let contract = host_telemetry_contract(HostTelemetryStream::System);
    let declared = contract.columns.iter().map(|column| column.metadata.identifier.as_str()).collect::<Vec<_>>();
    for sample in batch.samples {
        for key in sample.values.keys().filter(|key| key.starts_with("battery_")) {
            assert!(declared.contains(&key.as_str()), "{key} is not declared");
        }
        validate_telemetry_sample(Some(&contract), &sample, ValidationMode::Strict).unwrap();
        dokime_host::System::from_row(&sample.values).unwrap();
    }
}
