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

#[path = "../../../dokime-runtime-next/benches/storage/sample_carrier.rs"]
mod sample_carrier;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Write},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    process::Stdio,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::Criterion;
// Local Imports
use tracing::{Id, Subscriber, span::Attributes};
use tracing_subscriber::{
    Layer,
    filter::filter_fn,
    layer::{Context as LayerContext, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
};

use criterion::SamplingMode;
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
struct QueryTimes(Arc<Mutex<BTreeMap<&'static str, Vec<f64>>>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for QueryTimes {
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: LayerContext<'_, S>) {
        if attributes.metadata().name().starts_with("plot.") || attributes.metadata().name().starts_with("document.") {
            context.span(id).unwrap().extensions_mut().insert(Instant::now());
        }
    }

    fn on_close(&self, id: Id, context: LayerContext<'_, S>) {
        if let Some(started) = context.span(&id).unwrap().extensions().get::<Instant>() {
            self.0.lock().unwrap().entry(context.span(&id).unwrap().name()).or_default().push(started.elapsed().as_secs_f64() * 1_000.0);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn main() {
    if std::env::args().any(|argument| argument == "--remote") {
        if std::env::var_os("DOKIME_BENCH_SDK").is_some() {
            fixture::serve().unwrap();
        } else {
            Runtime::new().unwrap().block_on(sample_carrier::serve(fixture::samples)).unwrap();
        }
        return;
    }
    if std::env::args().any(|argument| argument == "--fixture-host") {
        host();
        return;
    }

    // Start The Real Native Desktop Driver Once Outside Criterion's Timers.
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut child = dokime_runtime_next::process::command("node")
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
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(1));
    for count in [30, 100] {
        for condition in ["cold", "warm", "live", "late", "zoom", "resize", "navigation", "stalled"] {
            println!(
                "Boundary | Desktop {count}/{condition} | native query through visible draw or explicit refusal; {condition}; durable history; producer prompt handoff/actions included when named, without a publication durability barrier; cold reopens host and desktop before timer"
            );
            group.bench_function(format!("{count}/{condition}"), |bencher| {
                bencher.iter_custom(|iterations| {
                    writeln!(input, "{} {count} {condition} {iterations}", fixture::ROWS).unwrap();
                    input.flush().unwrap();
                    line.clear();
                    output.read_line(&mut line).unwrap();
                    let seconds = line
                        .trim()
                        .parse::<f64>()
                        .unwrap_or_else(|error| panic!("Measure desktop_round_trip/{count}/{condition}: raw seconds {line:?}: {error}"));
                    Duration::try_from_secs_f64(seconds)
                        .unwrap_or_else(|error| panic!("Measure desktop_round_trip/{count}/{condition}: raw seconds {line:?}: {error}"))
                });
            });
        }
    }
    group.finish();
    let mut history = criterion.benchmark_group("desktop_history_round_trip");
    history.sample_size(10);
    history.sampling_mode(SamplingMode::Flat);
    history.warm_up_time(Duration::from_secs(1));
    history.measurement_time(Duration::from_secs(1));
    for rows in [307_200, 1_536_000, 3_072_000] {
        for count in [30, 100] {
            for condition in ["cold", "warm", "live"] {
                println!(
                    "Boundary | Desktop {rows}/{count}/{condition} | document request, bounded document pages, visible plot queries, native delivery and visible draw; three-day history; host reopen excluded; live includes producer prompt handoff and saved visibility"
                );
                history.bench_function(format!("{rows}/{count}/{condition}"), |bencher| {
                    bencher.iter_custom(|iterations| {
                        writeln!(input, "{rows} {count} {condition} {iterations}").unwrap();
                        input.flush().unwrap();
                        line.clear();
                        output.read_line(&mut line).unwrap();
                        let seconds = line.trim().parse::<f64>().unwrap_or_else(|error| {
                            panic!("Measure desktop_history_round_trip/{rows}/{count}/{condition}: raw seconds {line:?}: {error}")
                        });
                        Duration::try_from_secs_f64(seconds).unwrap_or_else(|error| {
                            panic!("Measure desktop_history_round_trip/{rows}/{count}/{condition}: raw seconds {line:?}: {error}")
                        })
                    })
                });
            }
        }
    }
    history.finish();
    criterion.final_summary();
    drop(child.stdin.take());
    assert!(child.wait().unwrap().success(), "desktop driver completed and retained measurements");
}

fn host() {
    let executor = Runtime::new().unwrap();
    let directory = std::env::var_os("DOKIME_DESKTOP_BENCH_FIXTURES").expect("fixture directory under target");
    std::fs::create_dir_all(&directory).unwrap();
    let restored = std::env::var_os("DOKIME_DESKTOP_BENCH_RESTORE").filter(|path| !path.is_empty());
    let restoring = restored.is_some();
    let root = restored.map_or_else(|| tempfile::tempdir_in(&directory).unwrap().keep(), PathBuf::from);
    assert!(root.canonicalize().unwrap().starts_with(std::fs::canonicalize(&directory).unwrap()), "owned fixture root");
    let queries = QueryTimes::default();
    let probe =
        queries.clone().with_filter(filter_fn(|metadata| metadata.name().starts_with("plot.") || metadata.name().starts_with("document.")));
    tracing_subscriber::registry().with(probe).init();
    let fixture = executor.block_on(fixture::Fixture::start(&root, restoring));
    println!(
        "{}",
        serde_json::json!({"endpoint":fixture.session.endpoint(), "execution":fixture.execution,
        "cycle": fixture.cycle, "root": root, "rows":fixture::rows(), "sample_rate_hz": 1e9 / fixture::sample_ns() as f64, "origin_s":fixture::ORIGIN_S,
        "shutdown_ms": Duration::from_secs(30).as_millis(),
        "end_s":fixture::ORIGIN_S + (fixture::rows() - 1) as f64 * fixture::sample_ns() as f64 / 1e9})
    );
    std::io::stdout().flush().unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut history_ready = restoring;
        for line in std::io::stdin().lock().lines() {
            let line = line.unwrap();
            // Prove Native Launch Before Admitting The Multi-Million-Row History.
            if line == "prepare" && !history_ready {
                executor.block_on(fixture.prepared());
                history_ready = true;
            }
            if line == "live" || line == "late" {
                executor.block_on(fixture.publish(&line));
            }
            let times = std::mem::take(&mut *queries.0.lock().unwrap());
            println!("{}", serde_json::json!({"stages": times}));
            std::io::stdout().flush().unwrap();
        }
    }));
    executor.block_on(fixture.close());
    eprintln!("Desktop fixture shutdown: complete");
    if let Err(error) = result {
        resume_unwind(error);
    }
}

/* === Tests === */
