/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Saved-Telemetry Plot Paths, Shared By Their Integration Guards And Full-Scale System Stories.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::prelude::*;
use dokime_runtime::runtime::query::{DownsampleTechnique, PlotPayloadRequest, RuntimePlotQueryMode};

/* === Constants === */

const SAMPLE_BATCH: usize = 1_000;

const MINUTES_PER_DAY: usize = 24 * 60;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn register_plot_suite(runtime: &RuntimeHandle) {
    let definition = TestSuiteDefinition {
        suite_id: "plot.query.suite".to_owned(),
        tags: Vec::new(),
        workflow: None,
        suite: plot_query_suite(),
        record_contracts: Vec::new(),
        telemetry_contracts: vec![wifi_occupancy_contract()],
    };
    runtime.set_suites("plot-query", dokime_runtime::definition::SuiteSet::new([definition])).await.unwrap();
}

pub fn plot_query_suite() -> TestSuite {
    TestSuite {
        metadata: Metadata::new(Identifier::new("plot.query.suite").unwrap(), "Plot Query Suite", None),
        tags: Vec::new(),
        parameter_groups: Vec::new(),
        test_case_templates: Vec::new(),
    }
}

pub fn wifi_request(scope: Option<RecordScope>, max_points: Option<usize>) -> PlotPayloadRequest {
    PlotPayloadRequest {
        plot_id: Some("wifi.network_count".to_owned()),
        plot_type: None,
        x_scale: Default::default(),
        y_scale: Default::default(),
        stream: "host.wifi.channel_occupancy".to_owned(),
        y_keys: vec!["network_count".to_owned()],
        marker_record_types: Vec::new(),
        filters: Vec::new(),
        x_key: Some("sample_index".to_owned()),
        series_by: None,
        x_expression: None,
        y_expressions: Vec::new(),
        series_by_expression: None,
        z_stream: None,
        z_field: None,
        unit: Some("count".to_owned()),
        x_format: None,
        y_format: None,
        max_points,
        window_s: None,
        scope,
        technique: DownsampleTechnique::Raw,
        ..Default::default()
    }
}

// A plot over `samples` saved rows reads each saved row once and draws within its display points.
pub async fn sample_plot(samples: usize, display_points: usize) {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    for _ in 0..samples / SAMPLE_BATCH {
        runtime.publish_telemetry_batch(super::evidence::samples("host.wifi.channel_occupancy", SAMPLE_BATCH, None)).await.unwrap();
        runtime.storage().flush().await.unwrap();
    }
    let mut request = wifi_request(None, Some(display_points));
    // Markers And Samples Read The Same Saved Storage Cut.
    request.marker_record_types = vec!["test.cycle.started".to_owned()];
    // This runtime's own storage counts its scan, apart from any other runtime in the process.
    let before = runtime.storage_read_counters().data_scan_rows;
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .expect("a large sample set plots within its display points");
    let scanned = runtime.storage_read_counters().data_scan_rows - before;
    let series = &envelopes[0].payload.series[0];
    assert_eq!(series.raw_points, samples);
    assert!(series.downsampled);
    assert!(series.points.len() <= display_points);
    // One Read Of Every Saved Row Plus At Most One Marker Row Per Saved Batch (Measured: 5,003 Of
    // 5,000 And 50,012 Of 50,000), Never A Second Pass.
    let bound = (samples + samples / SAMPLE_BATCH) as u64;
    assert!((samples as u64..=bound).contains(&scanned), "{scanned} rows scanned for {samples} samples");
}

// `days` of minute-rate host history: every day's saved windows stay bounded, managed
// snapshots never accumulate the history, and the whole history reads back exactly.
pub async fn saved_windows(days: usize) {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    let baseline = serde_json::to_vec(runtime.watch_state().latest().state()).unwrap().len();

    for day in 0..days {
        save_day(&runtime, day).await;
        let results = runtime.query_plot_payloads(day_windows(), RuntimePlotQueryMode::Snapshot).await.unwrap();
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
    assert_eq!(whole[0].payload.series[0].raw_points, days * MINUTES_PER_DAY);
    assert!(whole[0].payload.series[0].points.len() <= 16);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

async fn save_day(runtime: &RuntimeHandle, day: usize) {
    for start in (0..MINUTES_PER_DAY).step_by(120) {
        runtime.publish_telemetry_batch(minute_batch(day * MINUTES_PER_DAY + start)).await.unwrap();
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
    let mut batch = super::evidence::samples("host.wifi.channel_occupancy", 120, None);
    for (offset, sample) in batch.samples.iter_mut().enumerate() {
        let minute = first + offset;
        let elapsed = Duration::from_secs(MIDNIGHT + minute as u64 * 60);
        sample.timestamp = Timestamp::from_system_time(std::time::UNIX_EPOCH + elapsed);
        sample.values.insert("sample_index".into(), Value::Integer(minute as i64));
        sample.values.insert("network_count".into(), Value::Integer(minute as i64));
        *sample = super::contracts::typed_fixture_sample(sample.clone());
    }
    batch
}

/* === Tests === */
