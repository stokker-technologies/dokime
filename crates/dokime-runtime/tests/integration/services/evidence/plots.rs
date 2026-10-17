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
use super::*;
use dokime_runtime::runtime::query::{
    DownsampleTechnique, PlotAnalysis, PlotDomain, PlotPayloadCursor, PlotPayloadRequest,
    PlotSeriesSummary, PlotWindow, RuntimePlotQueryMode, WifiBandScan, WifiBandWindow,
    WifiChannelCount,
};

/* === Constants === */
/// Samples a long fast capture holds: far more than one plot shows.
const LARGE_PLOT_SAMPLES: usize = 50_000;

/// Samples published per batch.
const SAMPLE_BATCH: usize = 1_000;

/// Points the plot asks to show.
const PLOT_POINTS: usize = 400;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// snapshot query with all window uses all samples before downsampling.

#[tokio::test]
async fn snapshot_query_samples() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 30, None))
        .await
        .unwrap();

    let envelopes = runtime
        .query_plot_payloads(
            vec![wifi_request(None, Some(5), None)],
            RuntimePlotQueryMode::Snapshot,
        )
        .await
        .unwrap();

    assert_eq!(envelopes.len(), 1);
    assert_eq!(envelopes[0].payload.series[0].raw_points, 30);
    assert_eq!(envelopes[0].payload.series[0].points.len(), 5);
    assert!(envelopes[0].payload.series[0].downsampled);

    for (budget, expected) in [
        (10, DownsampleTechnique::Lttb),
        (5, DownsampleTechnique::MinMax),
    ] {
        let mut automatic = wifi_request(None, Some(budget), None);
        automatic.technique = DownsampleTechnique::Auto;
        let mut explicit = automatic.clone();
        explicit.technique = expected;
        let automatic = runtime
            .query_plot_payloads(vec![automatic], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap();
        let explicit = runtime
            .query_plot_payloads(vec![explicit], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap();
        assert_eq!(automatic[0].payload.series, explicit[0].payload.series);
    }
    let raw = runtime
        .query_plot_payloads(
            vec![wifi_request(None, Some(0), None)],
            RuntimePlotQueryMode::Snapshot,
        )
        .await
        .unwrap();
    assert_eq!(raw[0].payload.series[0].points.len(), 30);
    assert!(!raw[0].payload.series[0].downsampled);
    let default = runtime
        .query_plot_payloads(
            vec![wifi_request(None, None, None)],
            RuntimePlotQueryMode::Snapshot,
        )
        .await
        .unwrap();
    assert_eq!(
        default[0].payload.max_points,
        dokime_runtime::document::DEFAULT_PLOT_MAX_POINTS
    );
}

#[tokio::test]
async fn batch_request_cursors() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 4, None))
        .await
        .unwrap();
    let stream = "dokime.tel|-|-|host.wifi.channel_occupancy".to_owned();
    let mut first = wifi_request(
        None,
        Some(10),
        Some(PlotPayloadCursor {
            telemetry_streams: [(stream.clone(), 2)].into(),
            record_sequence: None,
        }),
    );
    first.plot_id = Some("first".to_owned());
    let mut second = wifi_request(
        None,
        Some(10),
        Some(PlotPayloadCursor {
            telemetry_streams: [(stream, 3)].into(),
            record_sequence: None,
        }),
    );
    second.plot_id = Some("second".to_owned());

    let envelopes = runtime
        .query_plot_payloads(vec![first, second], RuntimePlotQueryMode::Append)
        .await
        .unwrap();

    assert_eq!(
        envelopes
            .iter()
            .map(|envelope| envelope.plot_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    assert_eq!(envelopes[0].payload.series[0].raw_points, 2);
    // The second request is the one-cursor case: only the samples after it, and
    // the cursor it hands back is the stream's new high-water.
    assert_eq!(envelopes[1].payload.series[0].raw_points, 1);
    assert_eq!(
        envelopes[1]
            .cursor
            .telemetry_streams
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec![4]
    );
    assert_eq!(envelopes[1].revision, 4);
}

// a time series plot summarizes each series over the exact rows of its window,
// whatever the display budget, for the whole window, a trailing window, a
// zoomed range, and an append frame that carries only the newest rows.
#[tokio::test]
async fn series_window_summaries() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // `network_count` is the sample index, one sample per second from midnight.
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 10, None))
        .await
        .unwrap();
    let midnight = 1_767_225_600.0;
    let mut whole = wifi_request(None, Some(3), None);
    whole.plot_type = Some(dokime_runtime::document::PlotType::TimeSeries);
    whole.x_key = None;
    whole.technique = DownsampleTechnique::MinMax;
    let mut trailing = whole.clone();
    trailing.window_s = Some(4.0);
    let mut zoomed = whole.clone();
    zoomed.summary_window = Some(PlotWindow {
        from: midnight + 2.0,
        to: midnight + 4.0,
    });

    let envelopes = runtime
        .query_plot_payloads(
            vec![whole.clone(), trailing, zoomed],
            RuntimePlotQueryMode::Snapshot,
        )
        .await
        .unwrap();

    // The whole window: 0..=9, though the plot draws three points.
    assert!(envelopes[0].payload.series[0].points.len() <= 3);
    assert_summary(
        &envelopes[0].payload.summaries,
        10,
        [0.0, 9.0, 4.5, 4.5, 8.25_f64.sqrt(), 28.5_f64.sqrt()],
        [midnight, midnight + 9.0],
    );
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
        [
            2.0,
            4.0,
            3.0,
            3.0,
            (2.0_f64 / 3.0).sqrt(),
            (29.0_f64 / 3.0).sqrt(),
        ],
        [midnight + 2.0, midnight + 4.0],
    );

    // An append frame draws only the rows after its cursor, yet summarizes the
    // whole window.
    let mut append = whole;
    append.cursor = Some(PlotPayloadCursor {
        telemetry_streams: [("dokime.tel|-|-|host.wifi.channel_occupancy".to_owned(), 7)].into(),
        record_sequence: None,
    });
    let appended = runtime
        .query_plot_payloads(vec![append], RuntimePlotQueryMode::Append)
        .await
        .unwrap();
    assert_eq!(appended[0].payload.series[0].raw_points, 3);
    assert_eq!(
        appended[0].payload.summaries,
        envelopes[0].payload.summaries
    );
}

// a wifi occupancy plot carries its per-scan and per-band counts: readings
// sharing a whole second form one scan, a channel keeping its largest reading.
#[tokio::test]
async fn wifi_occupancy_scans() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // (second, channel, networks); 1005 encodes 6 GHz channel 5.
    let readings = [
        (0, 1, 2),
        (0, 6, 3),
        (0, 36, 0),
        (1, 6, 1),
        (1, 6, 5),
        (1, 36, 4),
        (1, 1005, 2),
        (2, 1, 0),
        (2, 6, 0),
    ];
    let stream = identifier("host.wifi.channel_occupancy");
    let samples = readings
        .iter()
        .enumerate()
        .map(|(index, (second, channel, networks))| {
            let mut sample = TelemetrySample::new(dokime_runtime::model::identity::new_id(), stream.clone(),
                0,
                Timestamp::new(format!("2026-01-01T00:00:0{second}.{index}Z")),
                "host.fixture",
            );
            sample
                .values
                .insert("sample_index".to_owned(), Value::Integer(index as i64));
            sample
                .values
                .insert("wifi_channel".to_owned(), Value::Integer(*channel));
            sample
                .values
                .insert("network_count".to_owned(), Value::Integer(*networks));
            crate::service_support::contracts::typed_fixture_sample(sample)
        })
        .collect();
    runtime
        .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream, samples))
        .await
        .unwrap();
    let mut request = wifi_request(None, Some(0), None);
    request.plot_type = Some(dokime_runtime::document::PlotType::WifiChannelOccupancy);
    request.x_key = Some("timestamp".to_owned());
    request.series_by = Some("wifi_channel".to_owned());

    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();

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
    assert_eq!(
        bands,
        &[
            band_window("2.4 GHz", 2),
            band_window("5 GHz", 1),
            band_window("6 GHz", 1),
        ]
    );
    let totals = scans
        .iter()
        .map(|scan| (scan.occupied_channels, scan.total_networks, scan.peak_count))
        .collect::<Vec<_>>();
    assert_eq!(totals, [(2, 5, 3), (3, 11, 5), (0, 0, 0)]);
    assert_eq!(
        scans[1].channels,
        [(6, 5), (36, 4), (1005, 2)].map(|(channel, count)| WifiChannelCount { channel, count })
    );
    assert_eq!(
        scans[1].bands,
        [
            band_scan("2.4 GHz", 1, 5, 5),
            band_scan("5 GHz", 1, 4, 4),
            band_scan("6 GHz", 1, 2, 2),
        ]
    );
    assert_eq!(
        scans[0].bands,
        [band_scan("2.4 GHz", 2, 5, 3), band_scan("5 GHz", 0, 0, 0)]
    );
    assert!(scans[0].time.starts_with("2026-01-01T00:00:00"));
}

// an xy interval plot's analysis counts the samples its bins summarize.
#[tokio::test]
async fn interval_raw_points() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // `sample_index` and `network_count` both run 0..=9.
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 10, None))
        .await
        .unwrap();
    let mut request = wifi_request(None, Some(0), None);
    request.plot_type = Some(dokime_runtime::document::PlotType::XyInterval);
    request.interval.bin_width = 4.0;
    request.interval.sigma = 1.0;

    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();

    let Some(PlotAnalysis::SigmaInterval { series }) = &envelopes[0].payload.analysis else {
        panic!("no interval: {:?}", envelopes[0].payload.analysis);
    };
    assert_eq!(
        series[0]
            .bins
            .iter()
            .map(|bin| bin.count)
            .collect::<Vec<_>>(),
        [4, 4, 2]
    );
    assert_eq!(series[0].raw_points, 10);
}

// an append frame draws only the rows after its cursor, yet its cartesian
// domains and point count describe the whole window it extends.
#[tokio::test]
async fn append_frame_domains() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    // `network_count` is the sample index, one sample per second from midnight.
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 10, None))
        .await
        .unwrap();
    let midnight = 1_767_225_600.0;
    let mut whole = wifi_request(None, Some(0), None);
    whole.plot_type = Some(dokime_runtime::document::PlotType::TimeSeries);
    whole.x_key = None;
    let mut append = whole.clone();
    append.cursor = Some(PlotPayloadCursor {
        telemetry_streams: [("dokime.tel|-|-|host.wifi.channel_occupancy".to_owned(), 7)].into(),
        record_sequence: None,
    });

    let snapshot = runtime
        .query_plot_payloads(vec![whole], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let appended = runtime
        .query_plot_payloads(vec![append], RuntimePlotQueryMode::Append)
        .await
        .unwrap();

    let snapshot = snapshot[0].payload.cartesian.as_ref().unwrap();
    let frame = appended[0].payload.cartesian.as_ref().unwrap();
    assert_eq!(frame.series[0].x.len(), 3);
    assert_eq!(
        frame.x_domain,
        Some(PlotDomain {
            min: midnight,
            max: midnight + 9.0
        })
    );
    assert_eq!(frame.y_domain, Some(PlotDomain { min: 0.0, max: 9.0 }));
    assert_eq!(frame.point_count, 10);
    assert_eq!(
        (&frame.x_domain, &frame.y_domain, frame.point_count),
        (&snapshot.x_domain, &snapshot.y_domain, snapshot.point_count)
    );
}

#[tokio::test]
async fn execution_plots_samples() {
    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = booted_with_fixture(root.path(), executor.clone()).await;
    register_plot_suite(&runtime).await;
    let mut suite = fixture_suite(&["passed"]);
    let mut document = dokime_runtime::document::DocumentDefinition::new("default", "Plot Settings");
    let mut section = dokime_runtime::document::DocumentSection::new("Settings");
    let mut plot = dokime_runtime::document::Plot::time_series("Captured Plot");
    plot.id("captured.plot");
    plot.stream = Some("host.wifi.channel_occupancy".into());
    plot.y = vec!["network_count".into()];
    section.plot(plot);
    document.section(section);
    suite.test_case_templates[0].document = Some(document);
    register_python_suite(&runtime, suite.clone()).await;
    let cycle = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &cycle, CycleStatus::Running).await;
    let execution = running.test_executions[0].scope.clone();
    let execution_id = execution.test_execution_id.as_ref().unwrap().clone();
    let document_id = dokime_runtime::core::DocumentId::from_identifier("default").unwrap();
    let plot_id = Identifier::new("captured.plot").unwrap();
    runtime
        .update_plot_settings(
            &cycle,
            &execution_id,
            &document_id,
            &plot_id,
            Some(25),
            DownsampleTechnique::MinMax,
        )
        .await
        .unwrap();
    let snapshot = runtime.watch_state().latest();
    let definition = &snapshot.state().executions[&execution_id].documents[&document_id];
    let saved_plot = definition
        .cards
        .iter()
        .map(|card| &card.body)
        .chain(
            definition
                .sections
                .iter()
                .flat_map(|section| section.cards.iter().map(|card| &card.body)),
        )
        .find_map(|body| match body {
            dokime_runtime::document::DocumentCardBody::Plot { plot }
                if plot.metadata.identifier == plot_id =>
            {
                Some(plot)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(saved_plot.display_points, Some(25));
    assert_eq!(saved_plot.downsample, DownsampleTechnique::MinMax);
    drop(snapshot);
    for (scope, count) in [
        (None, 7),
        (Some(RecordScope::default().with_cycle(cycle.clone())), 5),
        (Some(execution.clone()), 2),
    ] {
        let receipt = runtime
            .publish_telemetry_batch(samples("host.wifi.channel_occupancy", count, scope))
            .await
            .unwrap();
        assert_eq!(receipt.accepted, count as u64);
        assert_eq!(receipt.dropped, 0);
    }

    let envelopes = runtime
        .query_plot_payloads(
            vec![wifi_request(Some(execution), Some(100), None)],
            RuntimePlotQueryMode::Snapshot,
        )
        .await
        .unwrap();

    assert_eq!(envelopes[0].payload.series[0].raw_points, 2);
    assert_eq!(envelopes[0].payload.series[0].points.len(), 2);
    executor.release();
    wait_for_cycle_lifecycle(&runtime, &cycle, CycleStatus::Completed).await;
    let error = runtime
        .update_plot_settings(
            &cycle,
            &execution_id,
            &document_id,
            &plot_id,
            Some(50),
            DownsampleTechnique::Auto,
        )
        .await
        .unwrap_err();
    assert_eq!(error.code(), Some("document_closed"));
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// A plot over a large sample set reads within its display budget.
#[tokio::test]
async fn large_sample_plot() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    for _ in 0..LARGE_PLOT_SAMPLES / SAMPLE_BATCH {
        runtime
            .publish_telemetry_batch(samples("host.wifi.channel_occupancy", SAMPLE_BATCH, None))
            .await
            .unwrap();
    }
    let mut request = wifi_request(None, Some(PLOT_POINTS), None);
    // Markers read records from storage; samples come from the live window.
    request.marker_record_types = vec!["test.cycle.started".to_owned()];
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .expect("a large sample set plots within its display budget");
    let series = &envelopes[0].payload.series[0];
    assert!(series.raw_points > PLOT_POINTS);
    assert!(series.downsampled);
    assert!(series.points.len() <= PLOT_POINTS);
}

/// One summary of the `network_count` series: count, then min, max, mean,
/// median, std and rms, over the inclusive x range `window`.
fn assert_summary(
    summaries: &[PlotSeriesSummary],
    count: usize,
    statistics: [f64; 6],
    window: [f64; 2],
) {
    let [summary] = summaries else {
        panic!("one summary per series: {summaries:?}");
    };
    assert_eq!(summary.series_id, "network_count");
    assert_eq!(summary.count, count);
    let actual = [
        summary.min,
        summary.max,
        summary.mean,
        summary.median,
        summary.std,
        summary.rms,
    ];
    for (actual, expected) in actual.into_iter().zip(statistics) {
        let actual = actual.expect("a non-empty window has every statistic");
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual} != {expected} in {summary:?}"
        );
    }
    assert_eq!(
        summary.window,
        Some(PlotWindow {
            from: window[0],
            to: window[1],
        })
    );
}

async fn register_plot_suite(runtime: &RuntimeHandle) {
    runtime
        .set_suites(
            "plot-query",
            dokime_runtime::definition::SuiteSet::new([TestSuiteDefinition {
                suite_id: "plot.query.suite".to_owned(),
                tags: Vec::new(),
                workflow: None,
                suite: plot_query_suite(),
                record_contracts: Vec::new(),
                telemetry_contracts: vec![wifi_occupancy_contract()],
            }]),
        )
        .await
        .unwrap();
}

fn plot_query_suite() -> TestSuite {
    TestSuite {
        metadata: Metadata::new(
            Identifier::new("plot.query.suite").unwrap(),
            "Plot Query Suite",
            None,
        ),
        tags: Vec::new(),
        parameter_groups: Vec::new(),
        test_case_templates: Vec::new(),
    }
}

fn band_window(band: &str, occupied_channels: usize) -> WifiBandWindow {
    WifiBandWindow {
        band: band.to_owned(),
        occupied_channels,
    }
}

fn band_scan(band: &str, occupied: usize, total: usize, peak: usize) -> WifiBandScan {
    WifiBandScan {
        band: band.to_owned(),
        occupied_channels: occupied,
        total_networks: total,
        peak_count: peak,
    }
}

fn wifi_request(
    scope: Option<RecordScope>,
    max_points: Option<usize>,
    cursor: Option<PlotPayloadCursor>,
) -> PlotPayloadRequest {
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
        cursor,
        technique: DownsampleTechnique::Raw,
        ..Default::default()
    }
}

/* === Tests === */

// distribution query bins its x column and does not limit evidence to the display budget.

#[tokio::test]
async fn distribution_query_budget() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 1000, None))
        .await
        .unwrap();

    // The histogram bins the column it declares on x; y is the count it computes.
    let mut request = wifi_request(None, Some(5), None);
    request.plot_type = Some(dokime_runtime::document::PlotType::Histogram);
    request.x_key = Some("network_count".to_owned());
    request.y_keys = Vec::new();
    request.histogram.bins = 3;
    request.window_s = Some(2000.0);
    let mut value_on_y = request.clone();
    value_on_y.x_key = None;
    value_on_y.y_keys = vec!["network_count".to_owned()];
    let envelopes = runtime
        .query_plot_payloads(vec![request, value_on_y], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let payload = serde_json::to_value(&envelopes[0].payload).unwrap();

    let series = &payload["analysis"]["series"][0];
    assert_eq!(series["key"], "network_count");
    assert_eq!(series["count"], 1000);
    let bins = series["bins"].as_array().unwrap();
    let edges = bins
        .iter()
        .map(|bin| {
            (
                bin["lower"].as_f64().unwrap(),
                bin["upper"].as_f64().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let counts = bins
        .iter()
        .map(|bin| bin["count"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(edges, [(0.0, 333.0), (333.0, 666.0), (666.0, 999.0)]);
    assert_eq!(counts, [333, 333, 334]);

    let refused = serde_json::to_value(&envelopes[1].payload).unwrap();
    assert_eq!(refused["analysis"]["kind"], "error");
    assert!(
        refused["analysis"]["message"]
            .as_str()
            .unwrap()
            .contains("histogram plots bin one x field or x expression")
    );
}

// record only query filters on execution dimensions without a telemetry stream.

#[tokio::test]
async fn record_query_absent() {
    use dokime_runtime::document::{DocumentFilter, FilterOperator};
    use dokime_runtime::expressions::{DataExpression, DataSource, DataTransform, dimension};

    let root = tempdir().unwrap();
    let (_owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
    register_custom_suite(&runtime, custom_suite()).await;
    let cycle_id = start_cycle(&runtime, custom_suite(), false).await;
    runtime
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    let cycle = wait_for_cycle_lifecycle(&runtime, &cycle_id, CycleStatus::Completed).await;
    let scope = cycle.test_executions[0].scope.clone();

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
        let result = runtime
            .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap();
        let count = result[0]
            .payload
            .series
            .iter()
            .map(|series| series.points.len())
            .sum::<usize>();
        assert_eq!(count > 0, matches);
        assert!(result[0].cursor.telemetry_streams.is_empty());
    }
}

#[tokio::test]
async fn selected_analysis_provenance() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None))
        .await
        .unwrap();
    let definitions = runtime.analysis_definitions().await.unwrap();
    assert!(
        definitions
            .iter()
            .any(|definition| definition.id == "dokime.analysis.mean")
    );
    assert_eq!(
        definitions
            .iter()
            .any(|definition| definition.id == "dokime.analysis.fft"),
        cfg!(feature = "fft")
    );
    let mut request = wifi_request(None, Some(1), None);
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.mean".into(),
        version: 1,
        parameters: Default::default(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let result = envelopes[0].payload.computed.as_ref().unwrap();
    assert_eq!(result.series[0].input_samples, 100);
    assert_eq!(result.series[0].output.outputs[0].values, [49.5]);
    assert_eq!(result.series[0].output.outputs[0].unit, "count");
    assert_eq!(result.source.epoch, runtime.process_epoch().to_string());
    assert_eq!(result.source.cursor, envelopes[0].cursor);
    assert_eq!(result.source.fields, ["network_count"]);
    assert_eq!(result.selection, request.analysis.clone().unwrap());
    assert!(envelopes[0].payload.series.is_empty());

    let mut expressions = request.clone();
    expressions.y_keys.clear();
    expressions.x_expression = Some(dokime_runtime::expressions::telemetry(
        "host.wifi.channel_occupancy",
        "sample_index",
    ));
    expressions.y_expressions = vec![dokime_runtime::expressions::telemetry(
        "host.wifi.channel_occupancy",
        "network_count",
    )];
    let equivalent = runtime
        .query_plot_payloads(vec![expressions.clone()], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    assert_eq!(equivalent[0].payload.computed.as_ref().unwrap(), result);
    expressions.y_expressions[0] = expressions.y_expressions[0].clone().mean();
    assert!(
        runtime
            .query_plot_payloads(vec![expressions], RuntimePlotQueryMode::Snapshot)
            .await
            .is_err()
    );

    request.cursor = Some(envelopes[0].cursor.clone());
    let batches = runtime.query_plot_stream_batches(&[request]).await.unwrap();
    assert_eq!(batches[0].mode, RuntimePlotQueryMode::Replace);
    assert_eq!(
        batches[0]
            .envelope
            .payload
            .computed
            .as_ref()
            .unwrap()
            .series[0]
            .input_samples,
        100
    );
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn selected_original_coordinates() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None))
        .await
        .unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.lttb".into(),
        version: 1,
        parameters: [("points".into(), Value::Integer(7))].into(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
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
        sample.values.insert(
            "network_count".into(),
            Value::Integer(if index % 2 == 0 { 1 } else { -1 }),
        );
    }
    runtime.publish_telemetry_batch(batch).await.unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.x_key = Some("timestamp".into());
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.fft".into(),
        version: 1,
        parameters: [("hann".into(), Value::Bool(false))].into(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
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
        (
            "dokime.analysis.missing",
            1,
            std::collections::BTreeMap::new(),
            "unavailable",
        ),
        (
            "dokime.analysis.mean",
            999,
            std::collections::BTreeMap::new(),
            "version",
        ),
        (
            "dokime.analysis.mean",
            1,
            [("unknown".into(), Value::Integer(1))].into(),
            "undeclared",
        ),
    ] {
        let mut request = wifi_request(None, Some(1), None);
        request.analysis = Some(dokime_runtime::analysis::Selection {
            id: id.into(),
            version,
            parameters,
        });
        let error = runtime
            .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap_err();
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
    for sample in &mut batch.samples {
        if let Some(Value::Integer(value)) = sample.values.get_mut("sample_index") {
            *value *= 10;
        }
    }
    runtime.publish_telemetry_batch(batch).await.unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.window_s = Some(9.0);
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "dokime.analysis.mean".into(),
        version: 1,
        parameters: Default::default(),
    });
    let envelopes = runtime
        .query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot)
        .await
        .unwrap();
    let payload = &envelopes[0].payload;
    let series = &payload.computed.as_ref().unwrap().series[0];
    assert_eq!(series.input_samples, 10);
    assert_eq!(series.first_axis, 900.0);
    assert_eq!(series.last_axis, 990.0);
    assert_eq!(series.output.outputs[0].values, [94.5]);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

