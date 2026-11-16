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
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{Criterion, SamplingMode, Throughput};
use tokio::runtime::Runtime;

// Local Imports
use super::publication::{Fixture, settings};
use dokime::SettingDeclaration;
use dokime::plugin::HostOperation;
use dokime::prelude::*;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_runtime::{HostSettings, PluginProcess, SchedulerClock, services};
use prost::Message as _;

/* === Constants === */

/// The Plugin Process Reads Its Stream Count and Period From This Variable.
const LAYOUT: &str = "DOKIME_BENCH_WINDOWS";

/// Streams, Sample Period and Samples per Stream: 10.05 s at 1 kHz, 20 ms at 1 MHz.
///
/// The 1 kHz capture ends inside a host window, so the step's sleep cuts its last window early; the 1 MHz
/// capture spans two whole 10 ms setting windows.
const CASES: [(u64, u64, u64); 6] = [
    (1, 1_000_000, 10_050),
    (16, 1_000_000, 10_050),
    (128, 1_000_000, 10_050),
    (1, 1_000, 20_000),
    (16, 1_000, 20_000),
    (128, 1_000, 20_000),
];

/// The Window 1 MHz Streams Take From Their Plugin Setting Instead of the Host's `interval_window`.
const MHZ_WINDOW: Duration = Duration::from_millis(10);

/// Iterations a `--once` Run Measures per Case for Its Reported Median.
const ONCE_ITERATIONS: u64 = 5;

/// Samples the Plugin's Window Observers Answered for Executions, Across Every Stream.
static ANSWERED: AtomicU64 = AtomicU64::new(0);
/// Samples Answered for the Unscoped Connection Demand, Which Storage Does Not Keep.
static UNSCOPED: AtomicU64 = AtomicU64::new(0);

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
    unscoped: u64,
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
            if context.execution_id().is_empty() {
                UNSCOPED.fetch_add(window.count, Ordering::SeqCst);
            } else {
                ANSWERED.fetch_add(window.count, Ordering::SeqCst);
            }
            Ok(LevelColumns {
                value: (0..window.count).map(|offset| ((window.sequence + offset) % 1024) as f64).collect(),
            })
        })
        .every(period);
        // Each 1 MHz Stream Takes Its Window From the Plugin Setting, as the Library's 1 MHz Bench Does.
        let observer = if period == Duration::from_micros(1) { observer.window_setting("sample_window") } else { observer };
        template = template.stream(stream).observer(observer);
    }
    let window = prost_types::Duration::try_from(MHZ_WINDOW).map_err(|error| Error::Invalid(format!("Serve Window Benchmark: {error}")))?;
    Plugin::new("storage.windows", "1.0.0")
        .setting(SettingDeclaration {
            name: "sample_window".into(),
            title: "Sample Window".into(),
            description: "Sampling time one window of a 1 MHz stream covers".into(),
            default: Some(dokime::SettingValue {
                value: Some(dokime::setting_value::Value::Duration(window)),
            }),
        })
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
    let once = std::env::var_os("DOKIME_BENCH_ONCE").is_some();
    let clocks: [(&str, fn() -> Arc<dyn SchedulerClock>); 2] =
        [("controlled", || settings().clock), ("real", || Arc::new(services::WallClock))];
    for (clock, source) in clocks {
        for (streams, period_ns, rows) in CASES {
            let total = streams * rows;
            let (rate, window) = if period_ns == 1_000 { ("1 MHz", MHZ_WINDOW) } else { ("1 kHz", dokime_runtime::INTERVAL_WINDOW) };
            super::boundary(
                &format!("Interval Windows Stored, {streams} Streams at {rate}, {total} Samples, {clock} Clock, {window:?} Windows"),
                "warm plugin process, fresh execution; release ready step, host window grants, window observer columns, transport, \
                 Harmos batch admission, Dokime columnar Arrow admission, produced-count record (durable evidence barrier) and \
                 captured prompt; execution end and shutdown excluded; 1 MHz streams take their window from a plugin setting",
            );
            group.throughput(Throughput::Elements(total));
            let environment = [(LAYOUT.to_owned(), format!("{streams}:{period_ns}"))];
            let mut plugin = executor
                .block_on(PluginProcess::start_with_environment(
                    &std::env::current_exe().unwrap(),
                    true,
                    &settings().placement,
                    &environment,
                ))
                .unwrap();
            let mut rates = Vec::new();
            group.bench_function(format!("interval_windows_stored/{clock}/{streams}x{period_ns}ns/{total}"), |b| {
                b.iter_custom(|iterations| {
                    executor.block_on(async {
                        let mut elapsed = Duration::ZERO;
                        let measured = if once { iterations.max(ONCE_ITERATIONS) } else { iterations };
                        for _ in 0..measured {
                            let host = HostSettings { clock: source(), ..settings() };
                            let fixture = Fixture::new(host, plugin.endpoint(), "windows", rows as usize).await;
                            fixture.prompt("ready").await;
                            let started = Instant::now();
                            fixture.release("ready").await;
                            fixture.prompt("captured").await;
                            let taken = started.elapsed();
                            elapsed += taken;
                            fixture.release("captured").await;
                            let completed = fixture.cycle.wait().await.unwrap();
                            let execution = &completed.executions[0];
                            // The Real Clock Also Samples While the Ready Prompt Waits, So Its Rate Counts Every Stored Sample.
                            let kept = execution.samples.len() as u64;
                            rates.push(kept as f64 / taken.as_secs_f64());
                            if once {
                                println!("Measured | {:.0} samples/s | {kept} samples", kept as f64 / taken.as_secs_f64());
                            }
                            let retained = kept == total;
                            if clock == "controlled" && !retained || clock == "real" && kept < total {
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
                                    .map(|produced| (produced.rows, produced.unscoped))
                                    .fold((0, 0), |sum, (rows, unscoped)| (sum.0 + rows, sum.1 + unscoped));
                                let mut counts = BTreeMap::<u64, usize>::new();
                                for count in stored.values() {
                                    *counts.entry(*count).or_default() += 1;
                                }
                                println!(
                                    "Retained | {} of {total} samples | {} of {streams} streams stored | streams by count {counts:?} | answered by the sleep's end {} | unscoped answered before then {} | outcome {:?} | states {:?}",
                                    execution.samples.len(),
                                    stored.len(),
                                    answered.0,
                                    answered.1,
                                    execution.execution.outcome,
                                    completed.states,
                                );
                            }
                            // Real-Time Overload Is a Measured Outcome; the Controlled Clock Must Retain Every Sample.
                            assert!(clock == "real" || retained, "Store Interval Windows: Retain Every Sample");
                            fixture.host.shutdown().await.unwrap();
                            services::remove_credentials(fixture.root.as_path()).unwrap();
                        }
                        super::report(elapsed, measured)
                    })
                });
            });
            // Report the Median Rate Over Every Measured Iteration; Figures Are Never Assertions.
            rates.sort_by(f64::total_cmp);
            if let Some(median) = rates.get(rates.len() / 2) {
                println!(
                    "Median | {median:.0} samples/s | {} iterations | {streams} streams at {rate}, {clock} clock, {window:?} windows",
                    rates.len()
                );
            }
            executor.block_on(plugin.close()).unwrap();
        }
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
    let unscoped = UNSCOPED.swap(0, Ordering::SeqCst);
    context.record(&Record::<Produced>::new("produced", "Produced"), Produced { rows, unscoped }).await?;
    context.host().prompts.create("captured".into(), "continue".into(), Default::default()).await?;
    Ok(())
}

/* === Tests === */
