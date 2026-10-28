/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::support::*;
use dokime_domain::model;
use dokime_host::HostTelemetryStream;
use model::values::Value;

// Native Collection Reports Either Scan Data Or An Explicit Availability State.
#[test]
fn wifi_channel_data() {
    let batch = collect_required_native_batch(HostTelemetryStream::WifiChannelOccupancy);
    validate_batch(&batch);
    assert!(!batch.samples.is_empty(), "stream occupancy batches should contain scan samples");

    let first = &batch.samples[0];
    assert_text_in(first, "scan_status", &["ok", "unavailable", "unsupported", "error"]);
    if first.values.get("scan_status") != Some(&Value::Text("ok".into())) {
        assert!(batch.samples.iter().all(|sample| sample.values.get("scan_status") == first.values.get("scan_status")));
        assert_field(first, &Field::Integer("network_count"));
        assert_field(first, &Field::Integer("scan_network_count"));
        if first.values.get("scan_status") == Some(&Value::Text("error".into())) {
            assert!(matches!(first.values.get("scan_error"), Some(Value::Text(message)) if !message.is_empty()));
        }
        return;
    }

    let scan_timestamp = batch.samples[0].timestamp.clone();
    let Some(Value::Integer(sample_index)) = batch.samples[0].values.get("sample_index") else {
        panic!("expected sample_index integer");
    };
    let mut occupied_samples = 0;
    let mut reported_streams = Vec::new();

    for sample in &batch.samples {
        assert_eq!(sample.timestamp, scan_timestamp, "one occupancy batch should share one scan timestamp");
        assert_text_in(sample, "scan_status", &["ok"]);
        for field in [
            Field::Integer("sample_index"),
            Field::Integer("wifi_channel"),
            Field::Integer("network_count"),
            Field::Integer("scan_network_count"),
            Field::Number("overlap_weight"),
            Field::Integer("same_channel_network_count"),
            Field::Integer("overlapping_network_count"),
            Field::Number("overlap_weighted_network_count"),
        ] {
            assert_field(sample, &field);
        }

        let Some(Value::Integer(wifi_channel)) = sample.values.get("wifi_channel") else {
            panic!("expected wifi_channel integer");
        };
        reported_streams.push(*wifi_channel);

        let Some(Value::Integer(network_count)) = sample.values.get("network_count") else {
            panic!("expected network_count integer");
        };
        let Some(Value::Integer(current_sample_index)) = sample.values.get("sample_index") else {
            panic!("expected sample_index integer");
        };
        assert_eq!(current_sample_index, sample_index, "one occupancy batch should share one scan index");
        if *network_count > 0 {
            occupied_samples += 1;
        }
    }

    assert!(occupied_samples <= batch.samples.len());
    assert!(reported_streams.len() >= 45, "a stream occupancy scan should include dense 2.4 GHz and 5 GHz rows");
    assert!(reported_streams.contains(&1));
    assert!(reported_streams.contains(&14));
    assert!(reported_streams.contains(&32));
    assert!(reported_streams.contains(&177));
}
