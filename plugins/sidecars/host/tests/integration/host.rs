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
// BlueZ D-Bus on Linux).

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use model::telemetry::ValidationMode;
use model::telemetry::{TelemetryBatch, TelemetrySample};
use plugin_host::host_telemetry_contract;

// Local Imports
use crate::support::*;
use dokime::telemetry::TelemetryRow;
use dokime::telemetry::validate_telemetry_sample;
use dokime_domain::model;
use plugin_host::HostTelemetryStream;

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

#[dokime_tracing::trace_test]
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
                Field::Number("memory_used_gib"),
                Field::Number("memory_total_gib"),
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

#[dokime_tracing::trace_test]
#[test]
fn system_memory_snapshot() {
    let batch = collect_required_native_batch(HostTelemetryStream::System);
    validate_batch(&batch);
    let sample = single_sample(&batch);
    let row = plugin_host::System::from_row(&sample.values).unwrap();
    let used = row.memory_used_gib.unwrap();
    let total = row.memory_total_gib.unwrap();
    assert_eq!(used, row.memory_used_bytes.unwrap() as f64 / 1_073_741_824.0);
    assert_eq!(total, row.memory_total_bytes.unwrap() as f64 / 1_073_741_824.0);
    if total > 0.0 {
        assert_eq!(row.memory_pressure_percent, Some(used / total * 100.0));
    } else {
        assert_eq!(row.memory_pressure_percent, None);
    }
}

#[dokime_tracing::trace_test]
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
        plugin_host::System::from_row(&sample.values).unwrap();
    }
}

#[dokime_tracing::trace_test]
#[test]
fn executable_declares_host() {
    let executable = std::env::var("NEXTEST_BIN_EXE_plugin-host")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_plugin-host").map(str::to_owned))
        .expect("workspace host executable");
    let executable = dokime::__harmos::sidecar::Executable::at(executable).expect("the host executable declares itself");
    let declared = executable.declaration();
    assert_eq!(declared.id, "plugin_host");
    assert_eq!(declared.version.to_string(), env!("CARGO_PKG_VERSION"));
    let streams = declared.streams.iter().map(|stream| stream.id.as_str()).collect::<std::collections::BTreeSet<_>>();
    let expected = plugin_host::host_stream_identifiers();
    let mut expected = expected.iter().map(|id| id.as_str()).collect::<std::collections::BTreeSet<_>>();
    expected.insert(<plugin_host::host::cpu::load::LoadControl as dokime::StreamSpec>::ID);
    assert_eq!(streams, expected);
    let operations = declared.operations.iter().map(|operation| operation.id.as_str()).collect::<std::collections::BTreeSet<_>>();
    assert!(operations.contains(<plugin_host::host::cpu::load::LoadHold as dokime::harmos::operations::OperationDeclaration>::ID));
    assert!(operations.contains("host.network.radio.set"));
    assert!(operations.contains("host.network.adapter.set"));
    assert!(operations.contains("host.power.plan.set"));
}
