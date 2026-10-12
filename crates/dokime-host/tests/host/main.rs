/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "../support/mod.rs"]
pub mod support;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_contract::telemetry::{TelemetryBatch, TelemetrySample};
use dokime_host::HostTelemetryStream;
use support::*;

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

pub(crate) fn host_stream_row() {
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
        (
            HostTelemetryStream::Wifi,
            &["ok", "unavailable", "unsupported", "error"],
            &[Field::Integer("sample_index")],
        ),
        (
            HostTelemetryStream::WifiLatency,
            &["ok", "error"],
            &[Field::Integer("sample_index")],
        ),
        (
            HostTelemetryStream::Bluetooth,
            &["ok", "stale", "unsupported", "error"],
            &[Field::Integer("sample_index")],
        ),
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
fn domain_contract() {
    {
        eprintln!("Scenario: host_stream_row");
        host_stream_row();
    }
}
