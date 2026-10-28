/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::*;
use crate::service_support::plots::{plot_query_suite, register_plot_suite, wifi_request};
use dokime_runtime::runtime::query::{
    DownsampleTechnique, PlotAnalysis, PlotDomain, PlotPayloadRequest, PlotSeriesSummary, PlotWindow, RuntimePlotQueryMode, WifiBandScan,
    WifiBandWindow, WifiChannelCount,
};

const PLOT_POINTS: usize = 400;

// snapshot query with all window uses all samples before downsampling.

#[tokio::test]
async fn snapshot_query_samples() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 30, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();

    let envelopes = runtime.query_plot_payloads(vec![wifi_request(None, Some(5))], RuntimePlotQueryMode::Snapshot).await.unwrap();

    assert_eq!(envelopes.len(), 1);
    assert_eq!(envelopes[0].payload.series[0].raw_points, 30);
    assert_eq!(envelopes[0].payload.series[0].points.len(), 5);
    assert!(envelopes[0].payload.series[0].downsampled);

    for (budget, expected) in [(10, DownsampleTechnique::Lttb), (5, DownsampleTechnique::MinMax)] {
        let mut automatic = wifi_request(None, Some(budget));
        automatic.technique = DownsampleTechnique::Auto;
        let mut explicit = automatic.clone();
        explicit.technique = expected;
        let automatic = runtime.query_plot_payloads(vec![automatic], RuntimePlotQueryMode::Snapshot).await.unwrap();
        let explicit = runtime.query_plot_payloads(vec![explicit], RuntimePlotQueryMode::Snapshot).await.unwrap();
        assert_eq!(automatic[0].payload.series, explicit[0].payload.series);
    }
    let raw = runtime.query_plot_payloads(vec![wifi_request(None, Some(0))], RuntimePlotQueryMode::Snapshot).await.unwrap();
    assert_eq!(raw[0].payload.series[0].points.len(), 30);
    assert!(!raw[0].payload.series[0].downsampled);
    let default = runtime.query_plot_payloads(vec![wifi_request(None, None)], RuntimePlotQueryMode::Snapshot).await.unwrap();
    assert_eq!(default[0].payload.max_points, 2_000_usize);
}

#[tokio::test]
async fn batch_replacement_requests() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 4, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut first = wifi_request(None, Some(10));
    first.plot_id = Some("first".to_owned());
    let mut second = wifi_request(None, Some(10));
    second.plot_id = Some("second".to_owned());

    let envelopes = runtime.query_plot_payloads(vec![first, second], RuntimePlotQueryMode::Replace).await.unwrap();

    assert_eq!(envelopes.iter().map(|envelope| envelope.plot_id.as_str()).collect::<Vec<_>>(), vec!["first", "second"]);
    assert_eq!(envelopes[0].payload.series[0].raw_points, 4);
    assert_eq!(envelopes[1].payload.series[0].raw_points, 4);
    assert_eq!(envelopes[0].payload.series, envelopes[1].payload.series);
    assert_eq!(envelopes[1].revision, runtime.storage().cursor_saved().sequence);
}

// a time series plot summarizes each series over the exact rows of its window,
// whatever the display budget, for the whole window, a trailing window, a
// Zoomed Ranges And Complete Replacement Frames.
#[tokio::test]
async fn series_window_summaries() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // `network_count` is the sample index, one sample per second from midnight.
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 10, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let midnight = 1_767_225_600.0;
    let mut whole = wifi_request(None, Some(3));
    whole.plot_type = Some(dokime_runtime::document::PlotType::TimeSeries);
    whole.x_key = None;
    whole.technique = DownsampleTechnique::MinMax;
    let mut trailing = whole.clone();
    trailing.window_s = Some(4.0);
    let mut zoomed = whole.clone();
    zoomed.summary_window = Some(PlotWindow { from: midnight + 2.0, to: midnight + 4.0 });

    let envelopes = runtime.query_plot_payloads(vec![whole.clone(), trailing, zoomed], RuntimePlotQueryMode::Snapshot).await.unwrap();

    // The whole window: 0..=9, though the plot draws three points.
    assert!(envelopes[0].payload.series[0].points.len() <= 3);
    assert_summary(&envelopes[0].payload.summaries, 10, [0.0, 9.0, 4.5, 4.5, 8.25_f64.sqrt(), 28.5_f64.sqrt()], [midnight, midnight + 9.0]);
    // The trailing four seconds: 5..=9.
    assert_summary(
        &envelopes[1].payload.summaries,
        5,
        [5.0, 9.0, 7.0, 7.0, 2.0_f64.sqrt(), 51.0_f64.sqrt()],
        [midnight + 5.0, midnight + 9.0],
    );
    // The zoomed range: 2..=4, over the window the request named.
    assert_summary(
        &envelopes[2].payload.summaries,
        3,
        [2.0, 4.0, 3.0, 3.0, (2.0_f64 / 3.0).sqrt(), (29.0_f64 / 3.0).sqrt()],
        [midnight + 2.0, midnight + 4.0],
    );

    // A Replacement Summarizes The Same Complete Stored Window.
    let replacement = whole.clone();
    let replaced = runtime.query_plot_payloads(vec![replacement], RuntimePlotQueryMode::Replace).await.unwrap();
    assert_eq!(replaced[0].payload.series[0].raw_points, 10);
    assert_eq!(replaced[0].payload.summaries, envelopes[0].payload.summaries);

    // Narrowing Does Not Destroy Older Stored Samples; Widening And All Requery Them.
    for (window_s, count, minimum) in [(Some(2.0), 3, 7.0), (Some(8.0), 9, 1.0), (None, 10, 0.0)] {
        let request = PlotPayloadRequest { window_s, ..whole.clone() };
        let result = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Replace).await.unwrap();
        assert_eq!(result[0].payload.series[0].raw_points, count);
        assert_eq!(result[0].payload.summaries[0].count, count);
        assert_eq!(result[0].payload.summaries[0].min, Some(minimum));
    }
}

// a wifi occupancy plot carries its per-scan and per-band counts: readings
// sharing a whole second form one scan, a channel keeping its largest reading.
#[tokio::test]
async fn wifi_occupancy_scans() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // (second, channel, networks); 1005 encodes 6 GHz channel 5.
    let readings = [(0, 1, 2), (0, 6, 3), (0, 36, 0), (1, 6, 1), (1, 6, 5), (1, 36, 4), (1, 1005, 2), (2, 1, 0), (2, 6, 0)];
    let stream = identifier("host.wifi.channel_occupancy");
    let samples = readings
        .iter()
        .enumerate()
        .map(|(index, (second, channel, networks))| {
            let mut sample = TelemetrySample::new(
                dokime_runtime::model::identity::new_id(),
                stream.clone(),
                0,
                Timestamp::new(format!("2026-01-01T00:00:0{second}.{index}Z")),
                "host.fixture",
            );
            sample.values.insert("sample_index".to_owned(), Value::Integer(index as i64));
            sample.values.insert("wifi_channel".to_owned(), Value::Integer(*channel));
            sample.values.insert("network_count".to_owned(), Value::Integer(*networks));
            crate::service_support::contracts::typed_fixture_sample(sample)
        })
        .collect();
    runtime.publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream, samples)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut request = wifi_request(None, Some(0));
    request.plot_type = Some(dokime_runtime::document::PlotType::WifiChannelOccupancy);
    request.x_key = Some("timestamp".to_owned());
    request.series_by = Some("wifi_channel".to_owned());

    let envelopes = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();

    let Some(PlotAnalysis::WifiOccupancy {
        channels,
        occupied_channels,
        bands,
        scans,
    }) = &envelopes[0].payload.analysis
    else {
        panic!("no wifi occupancy: {:?}", envelopes[0].payload.analysis);
    };
    assert_eq!(channels, &[1, 6, 36, 1005]);
    assert_eq!(occupied_channels, &[1, 6, 36, 1005]);
    assert_eq!(bands, &[band_window("2.4 GHz", 2), band_window("5 GHz", 1), band_window("6 GHz", 1),]);
    let totals = scans.iter().map(|scan| (scan.occupied_channels, scan.total_networks, scan.peak_count)).collect::<Vec<_>>();
    assert_eq!(totals, [(2, 5, 3), (3, 11, 5), (0, 0, 0)]);
    assert_eq!(scans[1].channels, [(6, 5), (36, 4), (1005, 2)].map(|(channel, count)| WifiChannelCount { channel, count }));
    assert_eq!(scans[1].bands, [band_scan("2.4 GHz", 1, 5, 5), band_scan("5 GHz", 1, 4, 4), band_scan("6 GHz", 1, 2, 2),]);
    assert_eq!(scans[0].bands, [band_scan("2.4 GHz", 2, 5, 3), band_scan("5 GHz", 0, 0, 0)]);
    assert!(scans[0].time.starts_with("2026-01-01T00:00:00"));
    assert_eq!(scans[2].channels, [WifiChannelCount { channel: 1, count: 0 }, WifiChannelCount { channel: 6, count: 0 }]);
    assert_eq!(scans[2].bands, [band_scan("2.4 GHz", 0, 0, 0)]);
}

// an xy interval plot's analysis counts the samples its bins summarize.
#[tokio::test]
async fn interval_raw_points() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // `sample_index` and `network_count` both run 0..=9.
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 10, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut request = wifi_request(None, Some(0));
    request.plot_type = Some(dokime_runtime::document::PlotType::XyInterval);
    request.interval.bin_width = 4.0;
    request.interval.sigma = 1.0;

    let envelopes = runtime.query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot).await.unwrap();

    let Some(PlotAnalysis::SigmaInterval { series }) = &envelopes[0].payload.analysis else {
        panic!("no interval: {:?}", envelopes[0].payload.analysis);
    };
    assert_eq!(series[0].bins.iter().map(|bin| bin.count).collect::<Vec<_>>(), [4, 4, 2]);
    assert_eq!(series[0].raw_points, 10);
    assert_eq!(series[0].bins.iter().map(|bin| bin.x).collect::<Vec<_>>(), [2.0, 6.0, 10.0]);
    assert_eq!(series[0].bins.iter().map(|bin| bin.mean).collect::<Vec<_>>(), [1.5, 5.5, 8.5]);
    let sigma = 1.25_f64.sqrt();
    assert!((series[0].bins[0].lower - (1.5 - sigma)).abs() < 1e-12);
    assert!((series[0].bins[0].upper - (1.5 + sigma)).abs() < 1e-12);

    request.interval.sigma = 10.0;
    request.interval.clip_to_observed = true;
    let clipped = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let Some(PlotAnalysis::SigmaInterval { series }) = &clipped[0].payload.analysis else {
        panic!("no interval");
    };
    assert_eq!(series[0].bins.iter().map(|bin| (bin.lower, bin.upper)).collect::<Vec<_>>(), [(0.0, 3.0), (4.0, 7.0), (8.0, 9.0)]);
}

// Replacement Frames Carry The Complete Cartesian Window.
#[tokio::test]
async fn replacement_frame_domains() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // `network_count` is the sample index, one sample per second from midnight.
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 10, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let midnight = 1_767_225_600.0;
    let mut whole = wifi_request(None, Some(0));
    whole.plot_type = Some(dokime_runtime::document::PlotType::TimeSeries);
    whole.x_key = None;
    let replacement = whole.clone();

    let snapshot = runtime.query_plot_payloads(vec![whole], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let replaced = runtime.query_plot_payloads(vec![replacement], RuntimePlotQueryMode::Replace).await.unwrap();

    let snapshot = snapshot[0].payload.cartesian.as_ref().unwrap();
    let frame = replaced[0].payload.cartesian.as_ref().unwrap();
    assert_eq!(frame.series[0].x.len(), 10);
    assert_eq!(frame.x_domain, Some(PlotDomain { min: midnight, max: midnight + 9.0 }));
    assert_eq!(frame.y_domain, Some(PlotDomain { min: 0.0, max: 9.0 }));
    assert_eq!(frame.point_count, 10);
    assert_eq!((&frame.x_domain, &frame.y_domain, frame.point_count), (&snapshot.x_domain, &snapshot.y_domain, snapshot.point_count));
}

#[tokio::test]
async fn execution_plots_samples() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    register_plot_suite(&runtime).await;
    let wide_keys = [
        "elapsed_s",
        "distance_m",
        "position_x_m",
        "position_y_m",
        "speed_m_per_s",
        "velocity_x_m_per_s",
        "velocity_y_m_per_s",
        "target_reacquire_time_s",
        "connection_acquired_time_s",
    ];
    let mut contract = crate::service_support::contracts::tracking_contract();
    contract.stream = Identifier::new("fixture.wide").unwrap();
    let mut exact_contract = crate::service_support::contracts::fixture_telemetry_contract();
    exact_contract.stream = Identifier::new("fixture.exact").unwrap();
    for key in ["sample_index", "ordinal"] {
        exact_contract.columns.push(
            dokime_runtime::telemetry::TelemetryColumn::new(
                Metadata::new(Identifier::new(key).unwrap(), key, None),
                dokime_runtime::ValueType::Integer,
                dokime_runtime::telemetry::TelemetryRole::Value,
            )
            .optional(),
        );
    }
    let mut empty_contract = exact_contract.clone();
    empty_contract.stream = Identifier::new("fixture.empty").unwrap();
    let mut wide_suite = plot_query_suite();
    wide_suite.metadata = Metadata::new(Identifier::new("wide.plot.suite").unwrap(), "Wide Plot Suite", None);
    runtime
        .set_suites(
            "wide-plot",
            dokime_runtime::definition::SuiteSet::new([TestSuiteDefinition {
                suite_id: "wide.plot.suite".into(),
                tags: Vec::new(),
                workflow: None,
                suite: wide_suite,
                record_contracts: Vec::new(),
                telemetry_contracts: vec![contract, exact_contract, empty_contract],
            }]),
        )
        .await
        .unwrap();
    let mut suite = fixture_suite(&["passed"]);
    let mut document = dokime_runtime::document::DocumentDefinition::new("default", "Plot Settings");
    let mut section = dokime_runtime::document::DocumentSection::new("Settings");
    let mut plot = dokime_runtime::document::Plot::time_series("Captured Plot");
    plot.id("captured.plot");
    plot.stream = Some("host.wifi.channel_occupancy".into());
    plot.y = vec!["network_count".into(), "wifi_channel".into()];
    section.plot(plot);
    let mut wide = dokime_runtime::document::Plot::time_series("Wide Plot");
    wide.id("wide.plot");
    wide.stream = Some("fixture.wide".into());
    wide.y = wide_keys.iter().map(|key| (*key).into()).collect();
    section.plot(wide);
    let mut sparse = dokime_runtime::document::Plot::xy_scatter("Sparse Plot");
    sparse.id("sparse.plot");
    sparse.stream = Some("fixture.wide".into());
    sparse.x = Some("elapsed_s".into());
    sparse.y = vec!["connection_loss_distance_m".into()];
    section.plot(sparse);
    for (id, field, label) in [("distance.plot", "distance_m", "distance.selected"), ("position.plot", "position_x_m", "position.selected")]
    {
        let mut projection = dokime_runtime::document::Plot::xy_scatter(id);
        projection.id(id);
        projection.stream = Some("fixture.wide".into());
        projection.x = Some("elapsed_s".into());
        projection.y(dokime_runtime::expressions::telemetry("fixture.wide", field).label(label));
        section.plot(projection);
    }
    let mut histogram = dokime_runtime::document::Plot::histogram("Histogram");
    histogram.id("histogram.plot");
    histogram.stream = Some("fixture.wide".into());
    histogram.x = Some("distance_m".into());
    histogram.histogram.bins = 2;
    section.plot(histogram);
    let mut exact = dokime_runtime::document::Plot::time_series("Exact Integers");
    exact.id("exact.plot");
    exact.stream = Some("fixture.exact".into());
    exact.y = vec!["sample_index".into()];
    section.plot(exact);
    let mut missing_x = dokime_runtime::document::Plot::xy_scatter("Independent Window");
    missing_x.id("missing-x.plot");
    missing_x.stream = Some("fixture.exact".into());
    missing_x.x = Some("ordinal".into());
    missing_x.y = vec!["sample_index".into()];
    section.plot(missing_x);
    for (id, stream, field) in [
        ("fractional.plot", "fixture.wide", "connection_acquired_distance_m"),
        ("constant.plot", "fixture.exact", "ordinal"),
        ("empty.plot", "fixture.empty", "value"),
    ] {
        let mut plot = dokime_runtime::document::Plot::histogram(id);
        plot.id(id);
        plot.stream = Some(stream.into());
        plot.x = Some(field.into());
        plot.histogram.bins = 2;
        section.plot(plot);
    }
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite).await;
    runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &cycle, CycleStatus::Running).await;
    let execution = running.test_executions[0].scope.clone();
    let execution_id = execution.test_execution_id.as_ref().unwrap().clone();
    let document_id = dokime_runtime::DocumentId::from_identifier("default").unwrap();
    let plot_id = Identifier::new("captured.plot").unwrap();
    runtime.update_plot_settings(&cycle, &execution_id, &document_id, &plot_id, Some(25), DownsampleTechnique::MinMax).await.unwrap();
    let snapshot = runtime.watch_state().latest();
    let definition = &snapshot.state().executions[&execution_id].documents[&document_id];
    let saved_plot = definition
        .cards
        .iter()
        .map(|card| &card.body)
        .chain(definition.sections.iter().flat_map(|section| section.cards.iter().map(|card| &card.body)))
        .find_map(|body| match body {
            dokime_runtime::document::DocumentCardBody::Plot { plot } if plot.metadata.identifier == plot_id => Some(plot),
            _ => None,
        })
        .unwrap();
    assert_eq!(saved_plot.display_points, Some(25));
    assert_eq!(saved_plot.downsample, DownsampleTechnique::MinMax);
    drop(snapshot);
    for (scope, count) in [(None, 7), (Some(RecordScope::default().with_cycle(cycle.clone())), 5), (Some(execution.clone()), 2)] {
        let receipt = runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", count, scope)).await.unwrap();
        runtime.storage().flush().await.unwrap();
        assert_eq!(receipt.accepted, count as u64);
        assert_eq!(receipt.dropped, 0);
    }

    let mut wide_batch = samples("tracking", 3, Some(execution.clone()));
    wide_batch.stream = Identifier::new("fixture.wide").unwrap();
    for (row, sample) in wide_batch.samples.iter_mut().enumerate() {
        for (index, key) in wide_keys.iter().enumerate() {
            sample.values.insert((*key).into(), Value::Number((index * 10 + row) as f64));
        }
        sample.values.insert("elapsed_s".into(), Value::Number([2.0, 0.0, 1.0][row]));
        sample.values.insert("connection_acquired_distance_m".into(), Value::Number([0.1, 0.2, 0.3][row]));
        if row != 1 {
            sample.values.insert("connection_loss_distance_m".into(), Value::Number(100.0 + row as f64));
        }
        *sample = crate::service_support::contracts::typed_fixture_sample(sample.clone());
        sample.stream = wide_batch.stream.clone();
    }
    let receipt = runtime.publish_telemetry_batch(wide_batch).await.unwrap();
    runtime.storage().flush().await.unwrap();
    assert_eq!(receipt.accepted, 3);
    assert_eq!(receipt.dropped, 0);
    let mut exact_batch = samples("host.wifi.channel_occupancy", 2, Some(execution.clone()));
    exact_batch.stream = Identifier::new("fixture.exact").unwrap();
    for (index, sample) in exact_batch.samples.iter_mut().enumerate() {
        sample.stream = Identifier::new("fixture.telemetry").unwrap();
        sample.values.clear();
        sample.values.insert("timestamp".into(), Value::from(sample.timestamp.clone()));
        sample.values.insert("value".into(), Value::Number(index as f64));
        sample.values.insert("sample_index".into(), Value::Integer(9_007_199_254_740_993 + index as i64));
        if index == 1 {
            sample.values.insert("ordinal".into(), Value::Unsigned(7));
        }
        *sample = crate::service_support::contracts::typed_fixture_sample(sample.clone());
        sample.stream = exact_batch.stream.clone();
    }
    let receipt = runtime.publish_telemetry_batch(exact_batch).await.unwrap();
    runtime.storage().flush().await.unwrap();
    assert_eq!(receipt.accepted, 2);
    assert_eq!(receipt.dropped, 0);
    let envelopes =
        runtime.query_plot_payloads(vec![wifi_request(Some(execution), Some(100))], RuntimePlotQueryMode::Snapshot).await.unwrap();

    assert_eq!(envelopes[0].payload.series[0].raw_points, 2);
    assert_eq!(envelopes[0].payload.series[0].points.len(), 2);
    runtime.storage().flush().await.unwrap();
    let mut opened = runtime.open_execution_report(&execution_id).await.unwrap();
    let report = crate::service_support::reports::parse(
        std::str::from_utf8(&crate::service_support::reports::opened_bytes(&mut opened).await).unwrap(),
    );
    let current = &report["case_runs"][0]["plots"]["captured.plot"];
    let series = current["series"].as_array().unwrap();
    assert_eq!(series.len(), 2);
    for (series, key, values) in [(&series[0], "network_count", [0.0, 1.0]), (&series[1], "wifi_channel", [6.0, 6.0])] {
        assert_eq!(series["key"], key);
        assert_eq!(series["raw_points"], 2);
        let points = series["points"].as_array().unwrap();
        assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).collect::<Vec<_>>(), values);
    }
    assert_eq!(current["summaries"][0]["count"], 2);
    assert_eq!(current["summaries"][0]["mean"], 0.5);
    assert_eq!(current["summaries"][1]["count"], 2);
    assert_eq!(current["summaries"][1]["mean"], 6.0);
    let plots = &report["case_runs"][0]["plots"];
    let wide = &plots["wide.plot"];
    assert_eq!(wide["series"].as_array().unwrap().len(), wide_keys.len());
    for (index, key) in wide_keys.iter().enumerate() {
        let series = &wide["series"][index];
        assert_eq!(series["key"], *key);
        assert_eq!(series["raw_points"], 3);
        let expected = if index == 0 { [2.0, 0.0, 1.0] } else { [(index * 10) as f64, (index * 10 + 1) as f64, (index * 10 + 2) as f64] };
        let points = series["points"].as_array().unwrap();
        assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).collect::<Vec<_>>(), expected);
        let summary = &wide["summaries"][index];
        assert_eq!(summary["series_id"], *key);
        assert_eq!(summary["count"], 3);
        let base = (index * 10) as f64;
        for (name, expected) in [
            ("min", base),
            ("max", base + 2.0),
            ("mean", base + 1.0),
            ("median", base + 1.0),
            ("std", (2.0_f64 / 3.0).sqrt()),
            ("rms", ((base.powi(2) + (base + 1.0).powi(2) + (base + 2.0).powi(2)) / 3.0).sqrt()),
        ] {
            let actual = summary[name].as_f64().unwrap();
            assert!((actual - expected).abs() < 1e-9, "{key} {name}: {actual} != {expected}");
        }
        assert_eq!(summary["window"], serde_json::json!({"from": 1_767_225_600.0, "to": 1_767_225_602.0}));
    }
    let sparse = plots["sparse.plot"]["series"][0]["points"].as_array().unwrap();
    assert_eq!(sparse.len(), 3);
    for (x, y) in [(0.0, None), (1.0, Some(102.0)), (2.0, Some(100.0))] {
        let point = sparse.iter().find(|point| point["x"].as_f64() == Some(x)).unwrap();
        assert_eq!(point["y"].as_f64(), y);
    }
    let definitions = report["case_runs"][0]["documents"]["default"]["definition"]["cards"].as_array().unwrap();
    for (id, field, label, expected) in [
        ("distance.plot", "distance_m", "distance.selected", [11.0, 12.0, 10.0]),
        ("position.plot", "position_x_m", "position.selected", [21.0, 22.0, 20.0]),
    ] {
        let series = plots[id]["series"].as_array().unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0]["key"], field);
        assert_eq!(series[0]["label"], field);
        assert_eq!(series[0]["raw_points"], 3);
        let points = series[0]["points"].as_array().unwrap();
        assert_eq!(points.iter().map(|point| point["x"].as_f64().unwrap()).collect::<Vec<_>>(), [0.0, 1.0, 2.0]);
        assert_eq!(points.iter().map(|point| point["y"].as_f64().unwrap()).collect::<Vec<_>>(), expected);
        let definition = definitions.iter().find(|card| card["data"]["plot"]["metadata"]["identifier"] == id).unwrap();
        let expression = &definition["data"]["plot"]["y_expressions"][0];
        assert_eq!(expression["label"], label);
        assert_eq!(expression["source"]["field"], field);
    }
    let histogram = &plots["histogram.plot"]["analysis"]["series"][0];
    assert_eq!(histogram["count"], 3);
    assert_eq!(histogram["statistics"]["mean"], 11.0);
    let bins = histogram["bins"].as_array().unwrap();
    assert_eq!(
        bins.iter()
            .map(|bin| (bin["lower"].as_f64().unwrap(), bin["upper"].as_f64().unwrap(), bin["count"].as_u64().unwrap()))
            .collect::<Vec<_>>(),
        [(10.0, 11.0, 1), (11.0, 12.0, 2)]
    );
    let exact = &plots["exact.plot"]["summaries"][0];
    assert_eq!(exact["count"], 2);
    // Rounding Each Input First Would Yield 9_007_199_254_740_992 Instead.
    assert_eq!(exact["mean"], 9_007_199_254_740_994.0);
    assert_eq!(exact["median"], 9_007_199_254_740_994.0);
    let missing_x = &plots["missing-x.plot"];
    let points = missing_x["series"][0]["points"].as_array().unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points.iter().find(|point| point["x"].is_null()).unwrap()["y"], 9_007_199_254_740_992.0);
    assert_eq!(points.iter().find(|point| point["x"] == 7.0).unwrap()["y"], 9_007_199_254_740_994.0);
    assert_eq!(missing_x["summaries"][0]["count"], 2);
    assert_eq!(missing_x["summaries"][0]["window"], serde_json::json!({"from": 7.0, "to": 7.0}));
    for (id, expected) in [
        ("fractional.plot", vec![(0.1, 0.2, 1), (0.2, 0.3, 2)]),
        ("constant.plot", vec![(7.0, 8.0, 1), (8.0, 9.0, 0)]),
        ("empty.plot", vec![(0.0, 1.0, 0), (1.0, 2.0, 0)]),
    ] {
        let histogram = &plots[id]["analysis"]["series"][0];
        assert_eq!(
            histogram["bins"]
                .as_array()
                .unwrap()
                .iter()
                .map(|bin| (bin["lower"].as_f64().unwrap(), bin["upper"].as_f64().unwrap(), bin["count"].as_u64().unwrap()))
                .collect::<Vec<_>>(),
            expected
        );
    }
    executor.release();
    wait_for_cycle_outcome(&runtime, &cycle, ResultStatus::Passed).await;
    let mut sealed = runtime.open_execution_report(&execution_id).await.unwrap();
    assert_eq!(sealed.origin, dokime_runtime::ReportOrigin::Sealed);
    let saved = crate::service_support::reports::parse(
        std::str::from_utf8(&crate::service_support::reports::opened_bytes(&mut sealed).await).unwrap(),
    );
    let final_plot = &saved["case_runs"][0]["plots"]["captured.plot"];
    assert_eq!(final_plot["series"], current["series"]);
    assert_eq!(final_plot["summaries"], current["summaries"]);
    for key in [
        "wide.plot",
        "sparse.plot",
        "distance.plot",
        "position.plot",
        "histogram.plot",
        "exact.plot",
        "missing-x.plot",
        "fractional.plot",
        "constant.plot",
        "empty.plot",
    ] {
        let final_plot = &saved["case_runs"][0]["plots"][key];
        assert_eq!(final_plot["series"], plots[key]["series"]);
        assert_eq!(final_plot["summaries"], plots[key]["summaries"]);
        assert_eq!(final_plot["analysis"], plots[key]["analysis"]);
    }
    let error =
        runtime.update_plot_settings(&cycle, &execution_id, &document_id, &plot_id, Some(50), DownsampleTechnique::Auto).await.unwrap_err();
    assert_eq!(error.code(), Some("document_closed"));
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// The 5,000-row guard of the saved-sample plot path; `system/telemetry_plot_history.rs` reads 50,000.
#[tokio::test]
async fn large_sample_plot() {
    crate::service_support::plots::sample_plot(5_000, PLOT_POINTS).await;
}

fn assert_summary(summaries: &[PlotSeriesSummary], count: usize, statistics: [f64; 6], window: [f64; 2]) {
    let [summary] = summaries else {
        panic!("one summary per series: {summaries:?}");
    };
    assert_eq!(summary.series_id, "network_count");
    assert_eq!(summary.count, count);
    let actual = [summary.min, summary.max, summary.mean, summary.median, summary.std, summary.rms];
    for (actual, expected) in actual.into_iter().zip(statistics) {
        let actual = actual.expect("a non-empty window has every statistic");
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected} in {summary:?}");
    }
    assert_eq!(summary.window, Some(PlotWindow { from: window[0], to: window[1] }));
}

fn band_window(band: &str, occupied_channels: usize) -> WifiBandWindow {
    WifiBandWindow { band: band.to_owned(), occupied_channels }
}

fn band_scan(band: &str, occupied: usize, total: usize, peak: usize) -> WifiBandScan {
    WifiBandScan {
        band: band.to_owned(),
        occupied_channels: occupied,
        total_networks: total,
        peak_count: peak,
    }
}

// distribution query bins its x column and does not limit evidence to the display budget.

#[tokio::test]
async fn window_context_uses_latest_source_time_then_persisted_position() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    runtime
        .set_suites(
            "plot-query",
            dokime_runtime::definition::SuiteSet::new([TestSuiteDefinition {
                suite_id: "plot.query.suite".to_owned(),
                tags: Vec::new(),
                workflow: None,
                suite: plot_query_suite(),
                record_contracts: Vec::new(),
                telemetry_contracts: vec![wifi_occupancy_contract(), crate::service_support::contracts::minimal_tracking_contract()],
            }]),
        )
        .await
        .unwrap();

    let mut primary = samples("host.wifi.channel_occupancy", 21, None);
    primary.samples.retain(|sample| matches!(sample.values.get("sample_index"), Some(Value::Integer(10 | 20))));
    runtime.publish_telemetry_batch(primary).await.unwrap();
    // Equal-Time Updates Pick The Later Position; An Older Timestamp Arriving Last Does Not Replace Them.
    let context = [(8, 8.0), (8, 88.0), (3, 3.0)]
        .into_iter()
        .map(|(second, value)| {
            let mut sample = TelemetrySample::new(
                dokime_runtime::model::identity::new_id(),
                Identifier::new("tracking").unwrap(),
                0,
                Timestamp::new(format!("2026-01-01T00:00:{second:02}Z")),
                "host.fixture",
            );
            sample.values.insert("elapsed_s".into(), Value::Number(value));
            sample.values.insert("target_locked".into(), Value::Bool(true));
            crate::service_support::contracts::typed_minimal_tracking_sample(sample)
        })
        .collect();
    let batch = TelemetryBatch::new(dokime_runtime::model::identity::new_id(), Identifier::new("tracking").unwrap(), context);
    runtime.publish_telemetry_batch(batch).await.unwrap();
    runtime.storage().flush().await.unwrap();

    let mut request = wifi_request(None, Some(0));
    request.plot_type = Some(dokime_runtime::document::PlotType::TimeSeries);
    request.x_key = Some("timestamp".into());
    request.window_s = Some(10.0);
    request.z_stream = Some("tracking".into());
    request.z_field = Some("elapsed_s".into());
    let result = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let points = &result[0].payload.series[0].points;
    assert_eq!(points.len(), 2);
    assert_eq!(points.iter().map(|point| point.z).collect::<Vec<_>>(), [Some(88.0), Some(88.0)]);
    drop(result);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn distribution_query_budget() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 1000, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();

    // The histogram bins the column it declares on x; y is the count it computes.
    let mut request = wifi_request(None, Some(5));
    request.plot_type = Some(dokime_runtime::document::PlotType::Histogram);
    request.x_key = Some("network_count".to_owned());
    request.y_keys = Vec::new();
    request.histogram.bins = 3;
    request.window_s = Some(2000.0);
    let mut value_on_y = request.clone();
    value_on_y.x_key = None;
    value_on_y.y_keys = vec!["network_count".to_owned()];
    let envelopes = runtime.query_plot_payloads(vec![request, value_on_y], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let payload = serde_json::to_value(&envelopes[0].payload).unwrap();

    let series = &payload["analysis"]["series"][0];
    assert_eq!(series["key"], "network_count");
    assert_eq!(series["count"], 1000);
    let bins = series["bins"].as_array().unwrap();
    let edges = bins.iter().map(|bin| (bin["lower"].as_f64().unwrap(), bin["upper"].as_f64().unwrap())).collect::<Vec<_>>();
    let counts = bins.iter().map(|bin| bin["count"].as_u64().unwrap()).collect::<Vec<_>>();
    assert_eq!(edges, [(0.0, 333.0), (333.0, 666.0), (666.0, 999.0)]);
    assert_eq!(counts, [333, 333, 334]);

    let refused = serde_json::to_value(&envelopes[1].payload).unwrap();
    assert_eq!(refused["analysis"]["kind"], "error");
    assert!(refused["analysis"]["message"].as_str().unwrap().contains("histogram plots bin one x field or x expression"));
}

// record only query filters on execution dimensions without a telemetry stream.

#[tokio::test]
async fn record_query_absent() {
    use dokime_runtime::document::{DocumentFilter, FilterOperator};
    use dokime_runtime::expressions::{DataExpression, DataSource, DataTransform, dimension};

    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    register_custom_suite(&runtime, custom_suite()).await;
    let cycle_id = start_cycle(&runtime, custom_suite()).await;
    runtime.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    let cycle = wait_for_cycle_outcome(&runtime, &cycle_id, ResultStatus::Passed).await;
    let scope = cycle.test_executions[0].scope.clone();
    // Completion Keeps Writing After The Outcome; Settle Until The Saved Cut Stops Moving.
    loop {
        let accepted = runtime.storage().cursor_accepted();
        runtime.storage().flush().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        if runtime.storage().cursor_accepted() == accepted {
            break;
        }
    }

    for (suite, matches) in [("custom.suite", true), ("other.suite", false)] {
        let request = PlotPayloadRequest {
            stream: "default".into(),
            scope: Some(scope.clone()),
            y_expressions: vec![DataExpression {
                label: None,
                source: DataSource::Record {
                    record_type: "test.execution.completed".into(),
                },
                transforms: vec![DataTransform::TimeBetween {
                    previous_record_type: "test.execution.prepared".into(),
                }],
            }],
            filters: vec![DocumentFilter {
                expression: dimension("execution.suite"),
                operator: FilterOperator::Eq,
                value: Some(Value::from(suite)),
            }],
            ..Default::default()
        };
        let result = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
        let count = result[0].payload.series.iter().map(|series| series.points.len()).sum::<usize>();
        assert_eq!(count > 0, matches);
        assert_eq!(result[0].revision, runtime.storage().cursor_saved().sequence);
    }
}

#[tokio::test]
async fn selected_analysis_provenance() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let definitions = runtime.analysis_definitions().await.unwrap();
    assert!(definitions.iter().any(|definition| definition.id == "dokime.analysis.mean"));
    assert_eq!(definitions.iter().any(|definition| definition.id == "dokime.analysis.fft"), cfg!(feature = "fft"));
    let mut request = wifi_request(None, Some(1));
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.mean".into(),
        version: 1,
        parameters: Default::default(),
    });
    let envelopes = runtime.query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let result = envelopes[0].payload.computed.as_ref().unwrap();
    assert_eq!(result.series[0].input_samples, 100);
    assert_eq!(result.series[0].output.outputs[0].values, [49.5]);
    assert_eq!(result.series[0].output.outputs[0].unit, "count");
    assert_eq!(result.source.position, runtime.storage().cursor_saved());
    assert_eq!(result.source.fields, ["network_count"]);
    assert_eq!(result.selection, request.analysis.clone().unwrap());
    assert!(envelopes[0].payload.series.is_empty());

    let mut expressions = request.clone();
    expressions.y_keys.clear();
    expressions.x_expression = Some(dokime_runtime::expressions::telemetry("host.wifi.channel_occupancy", "sample_index"));
    expressions.y_expressions = vec![dokime_runtime::expressions::telemetry("host.wifi.channel_occupancy", "network_count")];
    let equivalent = runtime.query_plot_payloads(vec![expressions.clone()], RuntimePlotQueryMode::Snapshot).await.unwrap();
    assert_eq!(equivalent[0].payload.computed.as_ref().unwrap(), result);
    expressions.y_expressions[0] = expressions.y_expressions[0].clone().mean();
    assert!(runtime.query_plot_payloads(vec![expressions], RuntimePlotQueryMode::Snapshot).await.is_err());

    let batches = runtime.query_plot_stream_batches(&[request]).await.unwrap();
    assert_eq!(batches[0].mode, RuntimePlotQueryMode::Replace);
    assert_eq!(batches[0].envelope.payload.computed.as_ref().unwrap().series[0].input_samples, 100);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn selected_original_coordinates() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime.publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut request = wifi_request(None, Some(1));
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.lttb".into(),
        version: 1,
        parameters: [("points".into(), Value::Integer(7))].into(),
    });
    let envelopes = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let result = &envelopes[0].payload.computed.as_ref().unwrap().series[0];
    assert_eq!(result.input_samples, 100);
    let output = &result.output.outputs[0];
    assert_eq!(result.display_indices[&output.key], [0]);
    assert_eq!(output.axis.len(), 7);
    assert_eq!(output.axis, output.values);
    assert_eq!(output.axis[0], 0.0);
    assert_eq!(output.axis[6], 99.0);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[cfg(feature = "fft")]
#[tokio::test]
async fn selected_fft_manager() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    let mut batch = samples("host.wifi.channel_occupancy", 64, None);
    for (index, sample) in batch.samples.iter_mut().enumerate() {
        sample.values.insert("network_count".into(), Value::Integer(if index % 2 == 0 { 1 } else { -1 }));
        *sample = typed_fixture_sample(sample.clone());
    }
    runtime.publish_telemetry_batch(batch).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut request = wifi_request(None, Some(1));
    request.x_key = Some("timestamp".into());
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.fft".into(),
        version: 1,
        parameters: [("hann".into(), Value::Bool(false))].into(),
    });
    let envelopes = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let result = &envelopes[0].payload.computed.as_ref().unwrap().series[0];
    assert_eq!(result.origin.as_deref(), Some("2026-01-01T00:00:00Z"));
    let output = &result.output.outputs[0];
    assert_eq!(output.axis_unit, "Hz");
    assert_eq!(output.axis.last(), Some(&0.5));
    assert!((output.values.last().unwrap() - 1.0).abs() < 1e-12);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn selected_analysis_read() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    for (id, version, parameters, expected) in [
        ("dokime.analysis.missing", 1, std::collections::BTreeMap::new(), "unavailable"),
        ("dokime.analysis.mean", 999, std::collections::BTreeMap::new(), "version"),
        ("dokime.analysis.mean", 1, [("unknown".into(), Value::Integer(1))].into(), "undeclared"),
    ] {
        let mut request = wifi_request(None, Some(1));
        request.analysis = Some(dokime_runtime::analysis::Selection { id: id.into(), version, parameters });
        let error = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn selected_analysis_axes() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    let mut batch = samples("host.wifi.channel_occupancy", 100, None);
    // Saved Rows Come From The Typed Payload, So Each Edited Sample Is Encoded Again.
    for sample in &mut batch.samples {
        if let Some(Value::Integer(value)) = sample.values.get_mut("sample_index") {
            *value *= 10;
        }
        *sample = crate::service_support::contracts::typed_fixture_sample(sample.clone());
    }
    runtime.publish_telemetry_batch(batch).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut request = wifi_request(None, Some(1));
    request.window_s = Some(9.0);
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.mean".into(),
        version: 1,
        parameters: Default::default(),
    });
    let envelopes = runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
    let payload = &envelopes[0].payload;
    let series = &payload.computed.as_ref().unwrap().series[0];
    assert_eq!(series.input_samples, 10);
    assert_eq!(series.first_axis, 900.0);
    assert_eq!(series.last_axis, 990.0);
    assert_eq!(series.output.outputs[0].values, [94.5]);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// The one-day guard of the saved-window path; `system/telemetry_plot_history.rs` saves seven days.
#[tokio::test]
async fn saved_windows_keep_payload_and_snapshot_bounded() {
    crate::service_support::plots::saved_windows(1).await;
}
