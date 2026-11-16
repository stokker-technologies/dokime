/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Interval Window Ingestion Benchmark: Samples per Second Stored End to End

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{Criterion, SamplingMode, Throughput};
use tokio::runtime::Runtime;

// Local Imports
use super::publication::{Fixture, settings};
use dokime::plugin::HostOperation;
use dokime::prelude::*;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_runtime::{PluginProcess, services};
use prost::Message as _;

/* === Constants === */

/// The Plugin Process Reads Its Stream Count and Period From This Variable.
const LAYOUT: &str = "DOKIME_BENCH_WINDOWS";

/// Streams, Sample Period and Samples per Stream: 10.05 s at 1 kHz, 20 ms at 1 MHz.
///
/// Both captures end inside a window, so the step's sleep cuts its last window early.
const CASES: [(u64, u64, u64); 6] = [
    (1, 1_000_000, 10_050),
    (16, 1_000_000, 10_050),
    (128, 1_000_000, 10_050),
    (1, 1_000, 20_000),
    (16, 1_000, 20_000),
    (128, 1_000, 20_000),
];

/// Samples the Plugin's Window Observers Answered for Executions, Across Every Stream.
static ANSWERED: AtomicU64 = AtomicU64::new(0);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[harmos(package = "storage.windows.v1")]
struct Level {
    value: f64,
}

/// One Window of a Level Stream.
#[derive(dokime::Message)]
#[harmos(package = "storage.windows.v1")]
struct LevelColumns {
    value: Vec<f64>,
}

#[derive(dokime::Record)]
#[harmos(package = "storage.windows.v1")]
struct Produced {
    rows: u64,
}

#[derive(dokime::Parameters)]
#[harmos(package = "storage.windows.v1")]
struct Capture {
    rows: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Whether This Process Was Started as the Window Plugin.
pub fn selected() -> bool {
    std::env::var_os(LAYOUT).is_some()
}

/// Serve `streams` Interval Streams of One `double`, Each Answering Whole Windows at Once.
pub fn serve() -> Result<()> {
    let layout = std::env::var(LAYOUT).map_err(|error| Error::Invalid(format!("Serve Window Benchmark: {error}")))?;
    let (streams, period) = layout.split_once(':').ok_or_else(|| Error::Invalid("Serve Window Benchmark: Use streams:period_ns".into()))?;
    let streams = streams.parse::<u64>().map_err(|error| Error::Invalid(format!("Serve Window Benchmark: {error}")))?;
    let period = Duration::from_nanos(period.parse::<u64>().map_err(|error| Error::Invalid(format!("Serve Window Benchmark: {error}")))?);
    let mut template = Template::new("windows", "Windows")
        .parameters(Capture { rows: 1 })
        .step(Step::new("Capture", async move |context: &Context| capture(context, period).await));
    for index in 0..streams {
        let stream = Stream::<Level>::new(format!("level.stream_{index:03}"), format!("Level {}", index + 1));
        let observer = Observer::window(&stream, async |context: &Context, window: &SampleWindow| {
            if !context.execution_id().is_empty() {
                ANSWERED.fetch_add(window.count, Ordering::SeqCst);
            }
            Ok(LevelColumns {
                value: (0..window.count).map(|offset| ((window.sequence + offset) % 1024) as f64).collect(),
            })
        })
        .every(period);
        template = template.stream(stream).observer(observer);
    }
    Plugin::new("storage.windows", "1.0.0")
        .suite(Suite::new("storage.windows", "Window Benchmark").template(template))
        .prompt(Prompt::new("continue", "Continue", "Continue the Measurement?").action("continue", "Continue"))
        .requires_host(HostOperation::PromptsCreate)
        .requires_host(HostOperation::EvidenceRecordsAppend)
        .requires_host(HostOperation::TimeSleep)
        .record(Record::<Produced>::new("produced", "Produced"))
        .remote()
        .serve()
}

pub fn benchmark(criterion: &mut Criterion, executor: &Runtime) {
    let mut group = criterion.benchmark_group("windows");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    for (streams, period_ns, rows) in CASES {
        let total = streams * rows;
        let rate = if period_ns == 1_000 { "1 MHz" } else { "1 kHz" };
        super::boundary(
            &format!("Interval Windows Stored, {streams} Streams at {rate}, {total} Samples"),
            "warm plugin process, fresh execution on the controlled clock and the default interval window; release ready step, \
             host window grants, window observer columns, transport, Harmos batch admission, Dokime columnar Arrow admission, \
             produced-count record (durable evidence barrier) and captured prompt; execution end and shutdown excluded",
        );
        group.throughput(Throughput::Elements(total));
        let environment = [(LAYOUT.to_owned(), format!("{streams}:{period_ns}"))];
        let mut plugin = executor
            .block_on(PluginProcess::start_with_environment(&std::env::current_exe().unwrap(), true, &settings().placement, &environment))
            .unwrap();
        group.bench_function(format!("interval_windows_stored/{streams}x{period_ns}ns/{total}"), |b| {
            b.iter_custom(|iterations| {
                executor.block_on(async {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        let fixture = Fixture::new(plugin.endpoint(), "windows", rows as usize).await;
                        fixture.prompt("ready").await;
                        let started = Instant::now();
                        fixture.release("ready").await;
                        fixture.prompt("captured").await;
                        let taken = started.elapsed();
                        elapsed += taken;
                        if std::env::var_os("DOKIME_BENCH_ONCE").is_some() {
                            println!("Measured | {:.0} samples/s | {total} samples", total as f64 / taken.as_secs_f64());
                        }
                        fixture.release("captured").await;
                        let completed = fixture.cycle.wait().await.unwrap();
                        let execution = &completed.executions[0];
                        if execution.samples.len() as u64 != total {
                            // Report Where Samples Went Missing Before Refusing the Case.
                            let mut stored = BTreeMap::<&str, u64>::new();
                            for sample in &execution.samples {
                                *stored.entry(sample.stream.as_str()).or_default() += 1;
                            }
                            let answered = execution
                                .evidence
                                .iter()
                                .filter(|entry| entry.kind == "record:produced")
                                .filter_map(|entry| RecordsAppendRequest::decode(entry.payload.as_slice()).ok())
                                .filter_map(|record| Produced::decode(record.payload.as_slice()).ok())
                                .map(|produced| produced.rows)
                                .sum::<u64>();
                            let mut counts = BTreeMap::<u64, usize>::new();
                            for count in stored.values() {
                                *counts.entry(*count).or_default() += 1;
                            }
                            println!(
                                "Retained | {} of {total} samples | {} of {streams} streams stored | streams by count {counts:?} | answered by the sleep's end {answered} | outcome {:?} | states {:?}",
                                execution.samples.len(),
                                stored.len(),
                                execution.execution.outcome,
                                completed.states,
                            );
                        }
                        assert_eq!(execution.samples.len() as u64, total, "Store Interval Windows: Retain Every Sample");
                        fixture.host.shutdown().await.unwrap();
                        services::remove_credentials(fixture.root.as_path()).unwrap();
                    }
                    super::report(elapsed, iterations)
                })
            });
        });
        executor.block_on(plugin.close()).unwrap();
    }
    group.finish();
}

/* === Private Functions === */

async fn capture(context: &Context, period: Duration) -> Result<()> {
    context.host().prompts.create("ready".into(), "continue".into(), Default::default()).await?;
    let rows = context.parameters::<Capture>()?.rows;
    // A Controlled Sleep Settles After Every Window Due by Its End, So It Stores Every Requested Sample.
    let span = period.checked_mul(u32::try_from(rows).map_err(|_| Error::Invalid("Capture: Too Many Samples".into()))?);
    context.sleep(span.ok_or_else(|| Error::Invalid("Capture: Duration Overflow".into()))?).await?;
    // The Record Holds How Many Samples the Observers Answered by the Sleep's End.
    let rows = ANSWERED.swap(0, Ordering::SeqCst);
    context.record(&Record::<Produced>::new("produced", "Produced"), Produced { rows }).await?;
    context.host().prompts.create("captured".into(), "continue".into(), Default::default()).await?;
    Ok(())
}

/* === Tests === */
