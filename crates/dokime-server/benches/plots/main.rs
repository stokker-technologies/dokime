/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Windowed Plot Requests And Interface Frame Application At The Target Scale.

#![recursion_limit = "256"]

/* === Modules === */

mod fixture;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    hint::black_box,
    io::{BufRead, BufReader, Write},
    process::Stdio,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{Criterion, criterion_group, criterion_main};
use tracing::Instrument;

// Local Imports
use dokime::harmos::{
    domain::Portable,
    grpc::{Transcoded, Wire},
};
use dokime::{
    Runtime,
    client::{Client, ResponseStream, Target, contracts},
    domain::model::plots::wire::{PlotRequest, PlotStreamFrame, PlotStreamKind, PlotType, PlotWindow},
};
use dokime_server::grpc::{Config, Service};
/* === Constants === */

const PANEL_WIDTH: u32 = 960;

/* === Macros === */

criterion_group!(benches, benchmark);
criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

fn benchmark(criterion: &mut Criterion) {
    // Prepare The Fixed History Outside The Timed Loop.
    let executor = tokio::runtime::Runtime::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let snapshot_path = root.path().join("snapshots");
    let delta_path = root.path().join("deltas");
    // Vite Loads The Existing Interface Decoder And Retained-Frame Owner.
    let directory = std::env::current_dir().unwrap();
    let generated = directory
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/build/bench")
        .join(format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
        .join("generated/typescript");
    let mut interface = dokime::domain::process::command("node")
        .arg(directory.join("benches/plots/interface.mjs"))
        .arg(&snapshot_path)
        .arg(&delta_path)
        .arg(generated)
        .current_dir(directory.join("../dokime-gui"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(interface.stdout.take().unwrap());
    let input = interface.stdin.as_mut().unwrap();
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    assert_eq!(line.trim(), "ready");
    let prepared = Service::prepare(Config::local()).unwrap();
    let hold = std::sync::Arc::new(fixture::Hold { entered: tokio::sync::Notify::new() });
    let mut owner = executor
        .block_on(Runtime::with_execution_fixture(prepared.configuration(fixture::config(root.path())), "bench_pipeline", hold.clone()))
        .unwrap();
    let handle = owner.handle();
    executor.block_on(async {
        fixture::register_execution(&handle, &hold).await;
        for first in (0..fixture::ROWS).step_by(1024) {
            let count = (fixture::ROWS - first).min(1024);
            let receipt = handle.publish_telemetry_batch(fixture::batch(first, count, false)).await.unwrap();
            assert_eq!(receipt.accepted, count as u64, "{receipt:?}");
            assert_eq!(receipt.dropped, 0, "{receipt:?}");
        }
        handle.storage().flush().await.unwrap();
    });
    let server = executor.block_on(async { prepared.ready().await.unwrap().listen().await.unwrap() });
    let client = executor.block_on(Client::connect(Target::local(server.local_endpoint()).unwrap(), None, None)).unwrap();

    // Measure Thirty Independent Requests Without Changing The Transport Budget.
    let mut group = criterion.benchmark_group("plot_pipeline_three_days_thirty_plots");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
    let end = fixture::ORIGIN_S + (fixture::ROWS - 1) as f64;
    for (name, from, to) in [("live_snapshot_21600s", end - 21_600.0, end), ("zoom_snapshot_30s", end - 150.0, end - 120.0)] {
        let request = requests(from, to);
        let (replies, _) = executor.block_on(query(&client, &request));
        assert_eq!(replies.iter().map(|reply| reply.updates.len()).sum::<usize>(), fixture::PLOTS);
        assert!(replies.iter().flat_map(|reply| &reply.updates).all(|update| {
            let payload = update.envelope.as_ref().unwrap().payload.as_ref().unwrap();
            payload.bucket_width_s.is_some() && payload.frame.as_ref().is_some_and(|frame| frame.point_count > 0)
        }));
        let bytes = replies.iter().map(|reply| encode(reply).len()).sum::<usize>();
        println!("{name}: {bytes} protobuf response bytes across {} plot requests (excludes HTTP/2 and gRPC framing)", fixture::PLOTS);
        group.bench_function(name, |bencher| {
            bencher.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let (replies, duration) = query(&client, &request).await;
                        black_box(replies);
                        elapsed += duration;
                    }
                    elapsed
                })
            });
        });
    }

    // Profile One Request Per Window With The Product Trace Exporter.
    let profiling = std::env::var_os("DOKIME_PLOT_PROFILE").is_some();
    let _trace = profiling.then(dokime_tracing::process).flatten();
    if profiling {
        let end = fixture::ORIGIN_S + (fixture::ROWS - 1) as f64;
        for (name, from, to) in [("profile.live", end - 21_600.0, end), ("profile.zoom", end - 150.0, end - 120.0)] {
            let selected = requests(from, to).remove(0);
            let started = Instant::now();
            let watch = executor.block_on(
                async {
                    let watch = client.stream(contracts::plots::Watch { requests: vec![selected] }).await.unwrap();
                    let reply = data_frame(&watch, PlotStreamKind::Snapshot).await;
                    println!("{name}: {:.3} ms, {} bytes", started.elapsed().as_secs_f64() * 1_000.0, encode(&reply).len());
                    watch
                }
                .instrument(tracing::debug_span!("plot.bench.request", window = name)),
            );
            executor.block_on(watch.close());
        }
    }

    // Exercise The Desktop's Grouped Live Watch At The Default Message Limit.
    executor.block_on(async {
        let watch = client.stream(contracts::plots::Watch { requests: requests(end - 21_600.0, end) }).await.unwrap();
        let grouped = data_frame(&watch, PlotStreamKind::Snapshot).await;
        assert_eq!(grouped.updates.len(), fixture::PLOTS);
        let bytes = encode(&grouped).len();
        let points = grouped
            .updates
            .iter()
            .map(|update| update.envelope.as_ref().unwrap().payload.as_ref().unwrap().frame.as_ref().unwrap().point_count)
            .sum::<u64>();
        let warnings = grouped.updates.iter().flat_map(|update| &update.envelope.as_ref().unwrap().warnings).count();
        println!("desktop_grouped_watch: {bytes} protobuf bytes, {points} points, {warnings} warnings across {} plots", fixture::PLOTS);
        watch.close().await;
    });

    // Capture Each Plot's Real Watch Without Exceeding A Grouped Revision's Budget.
    let frames = executor.block_on(async {
        let mut frames = Vec::new();
        for (index, request) in requests(end - 21_600.0, end).into_iter().enumerate() {
            let watch = client.stream(contracts::plots::Watch { requests: vec![request] }).await.unwrap();
            let snapshot = data_frame(&watch, PlotStreamKind::Snapshot).await;
            let receipt = handle.publish_telemetry_batch(fixture::batch(fixture::ROWS - 60 + index, 1, true)).await.unwrap();
            assert_eq!(receipt.accepted, 1, "{receipt:?}");
            handle.storage().flush().await.unwrap();
            let delta = data_frame(&watch, PlotStreamKind::Delta).await;
            frames.push((snapshot, delta));
            watch.close().await;
        }
        frames
    });
    std::fs::create_dir(&snapshot_path).unwrap();
    std::fs::create_dir(&delta_path).unwrap();
    let mut delta_bytes = 0;
    for (index, (snapshot, delta)) in frames.iter().enumerate() {
        std::fs::write(snapshot_path.join(format!("{index}.bin")), encode(snapshot)).unwrap();
        let bytes = encode(delta);
        delta_bytes += bytes.len();
        std::fs::write(delta_path.join(format!("{index}.bin")), bytes).unwrap();
    }
    println!("late_bucket_delta: {delta_bytes} protobuf bytes across {} plot watches", fixture::PLOTS);

    // Decode The Captured Messages Only After All Fixture Files Exist.
    writeln!(input, "prepare").unwrap();
    input.flush().unwrap();
    line.clear();
    output.read_line(&mut line).unwrap();
    assert_eq!(line.trim(), "prepared");
    for operation in ["decode", "apply_delta"] {
        group.bench_function(operation, |bencher| {
            bencher.iter_custom(|iterations| {
                writeln!(input, "{operation} {iterations}").unwrap();
                input.flush().unwrap();
                line.clear();
                output.read_line(&mut line).unwrap();
                Duration::from_nanos(line.trim().parse().unwrap())
            });
        });
    }
    group.finish();

    // Close Only The Processes And Runtime Owned By This Benchmark.
    drop(interface.stdin.take());
    assert!(interface.wait().unwrap().success());
    drop(client);
    executor.block_on(server.shutdown()).unwrap();
    drop(handle);
    executor.block_on(owner.shutdown()).unwrap();
}

/* === Private Functions === */

async fn query(client: &Client, requests: &[PlotRequest]) -> (Vec<PlotStreamFrame>, Duration) {
    let mut replies = Vec::with_capacity(requests.len());
    let mut elapsed = Duration::ZERO;
    for request in requests {
        let started = Instant::now();
        let watch = client.stream(contracts::plots::Watch { requests: vec![request.clone()] }).await.unwrap();
        let reply = data_frame(&watch, PlotStreamKind::Snapshot).await;
        elapsed += started.elapsed();
        watch.close().await;
        replies.push(reply);
    }
    (replies, elapsed)
}

async fn data_frame(watch: &ResponseStream<PlotStreamFrame>, kind: PlotStreamKind) -> PlotStreamFrame {
    loop {
        let mut frame = watch.message().await.unwrap().unwrap();
        assert!(frame.updates.iter().all(|update| update.kind != PlotStreamKind::Error), "{frame:?}");
        frame.updates.retain(|update| update.kind == kind);
        if !frame.updates.is_empty() {
            return frame;
        }
    }
}

fn requests(from: f64, to: f64) -> Vec<PlotRequest> {
    (0..fixture::PLOTS)
        .map(|column| PlotRequest {
            plot_id: Some(format!("plot.{column}")),
            plot_type: PlotType::TimeSeries,
            stream: "bench.pipeline".into(),
            y_keys: vec![format!("y{column}")],
            viewport: Some(PlotWindow { from, to }),
            panel_width: Some(PANEL_WIDTH),
            window_s: Some(21_600.0),
            scope: Some(dokime::domain::model::identity::wire::EvidenceScope {
                execution_id: Some(fixture::execution_id().to_string()),
                ..Default::default()
            }),
            style: dokime::domain::model::documents::wire::PlotStyle::Unspecified,
            x_scale: dokime::domain::model::plots::wire::AxisScale::Unspecified,
            y_scale: dokime::domain::model::plots::wire::AxisScale::Unspecified,
            technique: dokime::domain::model::document::DownsampleTechnique::Auto,
            marker_record_types: Vec::new(),
            filters: Vec::new(),
            x_key: None,
            series_by: None,
            x_expression: None,
            y_expressions: Vec::new(),
            series_by_expression: None,
            z_stream: None,
            z_field: None,
            unit: None,
            max_points: None,
            z_expression: None,
            fft: None,
            interval: None,
            histogram: None,
            analysis: None,
            summary_window: None,
        })
        .collect()
}

fn encode<T: Portable>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    <Transcoded as Wire<T>>::encode(value, &mut bytes).unwrap();
    bytes
}

/* === Tests === */
