/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reduction And Saved Windows Share One Durable Plot History

use super::*;
use crate::service_support::{contracts, evidence};
use dokime_runtime::runtime::query::RuntimePlotQueryMode;

const SAMPLE_BATCH: usize = 1_000;
const MINUTES_PER_DAY: usize = 24 * 60;

pub async fn telemetry_history(samples: usize, display_points: usize, days: usize) {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // Each Batch Continues The Stream In Time, As A Live Source Publishes It.
    for first in (0..samples).step_by(SAMPLE_BATCH) {
        let batch = evidence::samples_from("host.wifi.scan", first, SAMPLE_BATCH, None);
        runtime.publish_telemetry_batch(batch).await.unwrap();
        runtime.storage().flush().await.unwrap();
    }
    let mut request = wifi_request(None, Some(display_points));
    // Markers And Samples Read The Same Saved Storage Cut.
    request.marker_record_types = vec!["test.cycle.started".to_owned()];
    // This runtime's own storage counts its scan, apart from any other runtime in the process.
    let before = runtime.storage_read_counters().data_scan_rows;
    let mut equivalent = request.clone();
    equivalent.plot_id = Some("equivalent".into());
    let envelopes = runtime
        .query_plot_payloads(vec![request, equivalent], RuntimePlotQueryMode::Snapshot)
        .await
        .expect("a large sample set plots within its display points");
    let scanned = runtime.storage_read_counters().data_scan_rows - before;
    assert_eq!(envelopes.len(), 2);
    assert_eq!(envelopes[0].payload.series, envelopes[1].payload.series);
    let series = &envelopes[0].payload.series[0];
    assert_eq!(series.raw_points, samples);
    assert!(series.downsampled);
    assert!(series.points.len() <= display_points);
    // Equivalent Requests Reuse One Evidence Scan, Including Bounded Marker Work.
    let bound = (samples + samples / SAMPLE_BATCH) as u64;
    assert!((samples as u64..=bound).contains(&scanned), "{scanned} rows scanned for {samples} samples");
    let baseline = serde_json::to_vec(runtime.watch_state().latest().state()).unwrap().len();

    let mut window_work = None;
    for day in 0..days {
        save_day(&runtime, samples.div_ceil(60) + day * MINUTES_PER_DAY).await;
        let before = runtime.storage_read_counters();
        let results = runtime.query_plot_payloads(day_windows(), RuntimePlotQueryMode::Snapshot).await.unwrap();
        let after = runtime.storage_read_counters();
        let scanned = after.data_scan_rows - before.data_scan_rows;
        assert_eq!(after.telemetry_anchor_rows - before.telemetry_anchor_rows, 1);
        assert!(scanned <= 16, "scanned {scanned} rows on day {day}");
        if let Some(previous) = window_work {
            assert_eq!(scanned, previous);
        }
        window_work = Some(scanned);
        for result in &results {
            assert_eq!(result.payload.series[0].raw_points, 6);
            assert!(result.payload.series[0].points.len() <= 8);
        }
        assert_eq!(results[0].payload.series, results[1].payload.series);
        let snapshot = serde_json::to_vec(runtime.watch_state().latest().state()).unwrap();
        assert!(snapshot.len() <= baseline + 128 * 1024, "host history must not accumulate in managed snapshots");
    }

    let mut request = wifi_request(None, Some(16));
    request.technique = DownsampleTechnique::MinMax;
    let whole = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
    assert_eq!(whole[0].payload.series[0].raw_points, samples + days * MINUTES_PER_DAY);
    assert!(whole[0].payload.series[0].points.len() <= 16);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

async fn save_day(runtime: &RuntimeHandle, first: usize) {
    for start in (0..MINUTES_PER_DAY).step_by(120) {
        runtime.publish_telemetry_batch(minute_batch(first + start)).await.unwrap();
        runtime.storage().flush().await.unwrap();
        assert_eq!(runtime.storage().status().pending_bytes, 0);
    }
}

// Two Identical Five-Minute Windows Over The Latest Saved Rows.
fn day_windows() -> Vec<PlotPayloadRequest> {
    let mut request = wifi_request(None, Some(8));
    request.plot_type = Some(dokime_runtime::document::PlotType::TimeSeries);
    request.x_key = None;
    request.window_s = Some(5.0 * 60.0);
    request.technique = DownsampleTechnique::MinMax;
    let mut second = request.clone();
    second.plot_id = Some("second".into());
    vec![request, second]
}

// 120 Minute Rows From `first`, Each Carrying Its Minute As Index And Value.
fn minute_batch(first: usize) -> TelemetryBatch {
    const MIDNIGHT: u64 = 1_767_225_600;
    let mut batch = evidence::samples("host.wifi.scan", 120, None);
    for (offset, sample) in batch.samples.iter_mut().enumerate() {
        let minute = first + offset;
        let elapsed = Duration::from_secs(MIDNIGHT + minute as u64 * 60);
        sample.timestamp = Timestamp::from_system_time(std::time::UNIX_EPOCH + elapsed);
        sample.values.insert("sample_index".into(), Value::Integer(minute as i64));
        sample.values.insert("network_count".into(), Value::Integer(minute as i64));
        *sample = contracts::typed_fixture_sample(sample.clone());
    }
    batch
}
