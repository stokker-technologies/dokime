/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Desktop Query-To-Draw Benchmark And Its Owned Fixture Host.

#![recursion_limit = "256"]

/* === Modules === */

mod fixture;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

use std::{
    io::{BufRead, BufReader, Write},
    process::Stdio,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::Criterion;
// Local Imports
use dokime::Runtime;
use dokime_server::grpc::{Config, Service};
use tracing_subscriber::{Layer, filter::filter_fn, layer::SubscriberExt, util::SubscriberInitExt};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
struct QueryTimes(Arc<Mutex<std::collections::BTreeMap<&'static str, Vec<f64>>>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> Layer<S> for QueryTimes {
    fn on_new_span(
        &self, attributes: &tracing::span::Attributes<'_>, id: &tracing::Id, context: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if matches!(attributes.metadata().name(), "plot.query" | "plot.watch.frames" | "plot.watch.send") {
            context.span(id).unwrap().extensions_mut().insert(Instant::now());
        }
    }

    fn on_close(&self, id: tracing::Id, context: tracing_subscriber::layer::Context<'_, S>) {
        if let Some(started) = context.span(&id).unwrap().extensions().get::<Instant>() {
            self.0.lock().unwrap().entry(context.span(&id).unwrap().name()).or_default().push(started.elapsed().as_secs_f64() * 1_000.0);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn main() {
    if std::env::args().any(|argument| argument == "--fixture-host") {
        host();
        return;
    }

    // Start The Real Native Desktop Driver Once Outside Criterion's Timers.
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut child = dokime::domain::process::command("node")
        .arg(directory.join("benches/system/driver.mjs"))
        .env("DOKIME_DESKTOP_BENCH_HOST", std::env::current_exe().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let input = child.stdin.as_mut().unwrap();
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    assert_eq!(line.trim(), "ready", "native desktop initialization");

    // Measure Product Work; Timing And Resource Values Are Never Assertions.
    let mut criterion = Criterion::default().configure_from_args();
    let mut group = criterion.benchmark_group("desktop_round_trip");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(1));
    for count in [30, 100] {
        for condition in ["cold", "warm", "live", "late", "zoom", "resize", "navigation", "stalled"] {
            group.bench_function(format!("{count}/{condition}"), |bencher| {
                bencher.iter_custom(|iterations| {
                    writeln!(input, "{count} {condition} {iterations}").unwrap();
                    input.flush().unwrap();
                    line.clear();
                    output.read_line(&mut line).unwrap();
                    Duration::from_secs_f64(line.trim().parse::<f64>().expect("desktop measurement in seconds"))
                });
            });
        }
    }
    group.finish();
    criterion.final_summary();
    drop(child.stdin.take());
    assert!(child.wait().unwrap().success(), "desktop driver completed and retained measurements");
}

fn host() {
    let executor = tokio::runtime::Runtime::new().unwrap();
    let directory = std::env::var_os("DOKIME_DESKTOP_BENCH_FIXTURES").expect("fixture directory under target");
    std::fs::create_dir_all(&directory).unwrap();
    let restored = std::env::var_os("DOKIME_DESKTOP_BENCH_RESTORE").filter(|path| !path.is_empty());
    let restoring = restored.is_some();
    let root = restored.map_or_else(|| tempfile::tempdir_in(&directory).unwrap().keep(), std::path::PathBuf::from);
    assert!(root.canonicalize().unwrap().starts_with(std::fs::canonicalize(&directory).unwrap()), "owned fixture root");
    let queries = QueryTimes::default();
    let probe = queries
        .clone()
        .with_filter(filter_fn(|metadata| matches!(metadata.name(), "plot.query" | "plot.watch.frames" | "plot.watch.send")));
    tracing_subscriber::registry().with(probe).init();
    let prepared = Service::prepare(Config::local()).unwrap();
    let hold = Arc::new(fixture::Hold { entered: tokio::sync::Notify::new() });
    let mut owner = executor
        .block_on(Runtime::with_execution_fixture(prepared.configuration(fixture::config(&root)), "bench_pipeline", hold.clone()))
        .unwrap();
    let handle = owner.handle();
    let manifest = root.join("bench-fixture.json");
    let cycle: dokime::TestCycleId = if restoring {
        let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
        serde_json::from_value(saved["cycle"].clone()).unwrap()
    } else {
        let cycle = executor.block_on(async {
            let cycle = fixture::register_execution(&handle, &hold).await;
            for first in (0..fixture::ROWS).step_by(4096) {
                let count = (fixture::ROWS - first).min(4096);
                let receipt = handle.publish_telemetry_batch(fixture::batch(first, count, false, first, &cycle)).await.unwrap();
                assert_eq!(receipt.accepted, count as u64, "{receipt:?}");
                assert_eq!(receipt.dropped, 0, "{receipt:?}");
            }
            handle.storage().flush().await.unwrap();
            cycle
        });
        std::fs::write(&manifest, serde_json::to_vec(&serde_json::json!({"cycle": cycle.clone()})).unwrap()).unwrap();
        cycle
    };
    let server = executor.block_on(async { prepared.ready().await.unwrap().listen().await.unwrap() });
    println!(
        "{}",
        serde_json::json!({"endpoint":server.local_endpoint(), "execution":fixture::execution_id().to_string(),
        "cycle": cycle.to_string(), "root": root, "rows":fixture::ROWS, "origin_s":fixture::ORIGIN_S, "end_s":fixture::ORIGIN_S + (fixture::ROWS - 1) as f64 / 8.0})
    );
    std::io::stdout().flush().unwrap();
    let mut sequence = fixture::ROWS;
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if line == "live" || line == "late" {
            executor.block_on(async {
                let first = if line == "late" { fixture::ROWS - 480 + (sequence - fixture::ROWS) % 400 } else { sequence };
                let publication = sequence;
                sequence += 8;
                let receipt = handle.publish_telemetry_batch(fixture::batch(first, 8, line == "late", publication, &cycle)).await.unwrap();
                assert_eq!(receipt.accepted, 8, "{receipt:?}");
                handle.storage().flush().await.unwrap();
            });
        }
        let times = std::mem::take(&mut *queries.0.lock().unwrap());
        println!("{}", serde_json::json!({"stages": times}));
        std::io::stdout().flush().unwrap();
    }
    executor.block_on(server.shutdown()).unwrap();
    drop(handle);
    executor.block_on(owner.shutdown()).unwrap();
}

/* === Tests === */
