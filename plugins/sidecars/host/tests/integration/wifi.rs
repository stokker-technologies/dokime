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
use model::values::Value;

// Local Imports
use crate::support::*;
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

/* === Tests === */

// Native Collection Reports Either Scanned Networks Or An Explicit Availability State.
#[test]
fn wifi_scan_rows() {
    let batch = collect_required_native_batch(HostTelemetryStream::WifiScan);
    validate_batch(&batch);
    assert!(!batch.samples.is_empty(), "a scan batch contains at least its status row");
    let first = &batch.samples[0];
    assert_text_in(first, "scan_status", &["ok", "cached", "unavailable", "error"]);
    assert_field(first, &Field::Integer("scan_network_count"));
    if !matches!(first.values.get("scan_status"), Some(Value::Text(status)) if status == "ok" || status == "cached") {
        assert_eq!(batch.samples.len(), 1, "an empty or failed scan is one status row");
        return;
    }
    let total = batch.samples.len() as i64;
    let associated = batch.samples.iter().filter(|sample| sample.values.get("associated") == Some(&Value::Bool(true))).count();
    assert!(associated <= 1, "the host associates with at most one network");
    for sample in &batch.samples {
        assert_eq!(sample.timestamp, first.timestamp, "one scan shares one timestamp");
        assert_eq!(sample.values.get("sample_index"), first.values.get("sample_index"));
        assert_eq!(sample.values.get("scan_network_count"), Some(&Value::Integer(total)));
        assert_field(sample, &Field::Integer("network_count"));
        assert!(matches!(sample.values.get("associated"), Some(Value::Bool(_))));
    }
}

// A Link Sample States Whether The Host Is Associated.
#[test]
fn wifi_link_association() {
    let batch = collect_required_native_batch(HostTelemetryStream::WifiLink);
    validate_batch(&batch);
    let sample = &batch.samples[0];
    if sample.values.get("probe_status") == Some(&Value::Text("ok".into())) {
        assert!(matches!(sample.values.get("associated"), Some(Value::Bool(_))));
    }
}

// A Missing Reply Is Loss, Never A Failed Collection.
#[test]
fn wifi_latency_loss() {
    let batch = collect_required_native_batch(HostTelemetryStream::WifiLatency);
    validate_batch(&batch);
    let sample = &batch.samples[0];
    if sample.values.get("probe_status") == Some(&Value::Text("ok".into())) {
        assert_field(sample, &Field::Number("wifi_packet_loss_percent"));
    }
}
