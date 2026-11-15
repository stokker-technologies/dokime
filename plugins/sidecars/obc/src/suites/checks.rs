/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// OBC Checks: A Deterministic Pulse Before a Charger Is Connected

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::atomic::{AtomicU64, Ordering};

// Third Party Imports
use tokio::sync::Notify;

// Local Imports
use super::parameters;
use crate::catalog::{count, field, flag, number, plot};
use crate::prompts::{PULSE_CONFIRM, confirm};
use crate::records::{RunCompleted, Started};
use crate::settings;
use crate::testbench;
use dokime::plugin::{Context, Criterion, Observer, Record, Step, Stream, Suite, Template};
use dokime::{Document, Error, PlotKind, PlotStyle, Result, Section, value};

/* === Constants === */

pub const SUITE: &str = "obc.checks";
pub const TEMPLATE: &str = "obc.checks.pulse";
pub const PULSE: &str = "obc.pulse";
pub const STARTED: &str = "obc.run.started";
pub const COMPLETED: &str = "obc.run.completed";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Deterministic Pulse Sample Produced Inside the Plugin.
#[derive(dokime::Row)]
#[harmos(package = "obc")]
pub struct Pulse {
    pub sample_index: i64,
    pub value: f64,
}

/// The Pulse Counter and the Host Time One Sample Takes.
struct Pulses {
    produced: AtomicU64,
    published: Notify,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> (Record<Started>, Record<RunCompleted>) {
    (
        Record::new(STARTED, "Run Started").description("The run has begun."),
        Record::new(COMPLETED, "Run Completed").description("The run finished, and this is what it produced."),
    )
}

pub fn suite() -> Suite {
    let pulse = Stream::<Pulse>::new(PULSE, "OBC Pulse").description("Deterministic samples produced inside the OBC plugin.");
    let observer = Observer::new(&pulse, sample).interval_setting(settings::SAMPLE_INTERVAL);
    let template =
        Template::new(TEMPLATE, "Pulse").description("Streams a short deterministic pulse from inside the plugin.").tag("example");
    let template = parameters(
        template,
        [
            count("sample_count", "Sample Count", 8, 1.0, 64.0),
            number("tolerance_au", "Value Tolerance", "a.u.", 1e-9, Some(1e-9), None),
            flag("require_confirmation", "Require Confirmation", false),
        ],
    )
    .initialize(initialize)
    .step(Step::new("Start", start).identifier("start").description("Say the run has begun, after an optional confirmation."))
    .step(Step::new("Stream Samples", stream).identifier("stream").description("Wait for the samples this case asked for."))
    .step(Step::new("Complete", complete).identifier("complete").description("Say what the run produced."))
    .criterion(
        Criterion::new("Pulse Completed", completed)
            .identifier("acceptance")
            .description("Passed once the saved pulse has the requested count and its extremes match the model."),
    )
    .stream(pulse)
    .observer(observer)
    .document(document());
    Suite::new(SUITE, "OBC Checks").description("Bench checks run inside the OBC plugin before a charger is connected.").template(template)
}

pub fn document() -> Document {
    let mut chart = plot("obc.checks.pulse.signals", "Pulse", PlotKind::TimeSeries, "a.u.");
    chart.style = PlotStyle::LinePoints as i32;
    chart.y = vec![field(PULSE, "value")];
    chart.marker_records = vec![STARTED.into(), COMPLETED.into()];
    Document::new("Pulse").id("obc.checks.pulse.report").section(Section::new("Pulse").id("obc.checks.pulse.section").plot(chart))
}

/// The Deterministic Pulse Value at One Sample Index.
pub fn pulse_value(index: u64) -> f64 {
    (index as f64 * 0.5).sin() + index as f64 * 0.05
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    ctx.set(Pulses {
        produced: AtomicU64::new(0),
        published: Notify::new(),
    });
    Ok(())
}

async fn start(ctx: &Context) -> Result<()> {
    let (started, _) = contracts();
    if ctx.parameter("require_confirmation")? && !confirm(ctx, PULSE_CONFIRM).await? {
        return Err(Error::Invalid("Start Pulse: The Operator Declined the Simulated Capture".into()));
    }
    ctx.record(&started, Started { note: "Pulse capture started.".into() }).await?;
    Ok(())
}

async fn stream(ctx: &Context) -> Result<()> {
    let wanted: u64 = ctx.parameter("sample_count")?;
    let pulses = ctx.get::<Pulses>()?;
    let produced = pulses.produced.load(Ordering::Acquire);
    let pending = u32::try_from(wanted.saturating_sub(produced)).map_err(|_| Error::Invalid("Stream Pulse: Too Many Samples".into()))?;
    ctx.sleep(settings::sample_interval(ctx).await? * pending).await?;
    // The Last Tick Fires With the Sleep; Await Its Sample Without Advancing Time
    loop {
        let published = pulses.published.notified();
        if pulses.produced.load(Ordering::Acquire) >= wanted {
            return Ok(());
        }
        published.await;
    }
}

async fn complete(ctx: &Context) -> Result<()> {
    let (_, completed) = contracts();
    let samples = ctx.get::<Pulses>()?.produced.load(Ordering::Acquire);
    ctx.record(&completed, RunCompleted { samples }).await?;
    Ok(())
}

async fn completed(ctx: &Context) -> Result<bool> {
    let wanted: u64 = ctx.parameter("sample_count")?;
    let tolerance: f64 = ctx.parameter("tolerance_au")?;
    // Check / Number of Rows the Run Produced
    let (_, completed) = contracts();
    if crate::records::last(ctx, &completed, "").await?.is_none_or(|record| record.samples < wanted) {
        return Ok(false);
    }
    let Some(statistics) = ctx.statistics(PULSE, "value", "").await?.statistics else {
        return Ok(false);
    };
    // Check / Saved Extremes Match the Model Over Every Saved Index
    let values = (0..statistics.count).map(pulse_value);
    let expected_min = values.clone().fold(f64::INFINITY, f64::min);
    let expected_max = values.fold(f64::NEG_INFINITY, f64::max);
    let number = |value: Option<dokime::Value>| match value.and_then(|value| value.kind) {
        Some(value::Kind::Number(number)) => Some(number),
        _ => None,
    };
    let within = |saved: Option<f64>, expected: f64| saved.is_some_and(|saved| (saved - expected).abs() <= tolerance);
    Ok(within(number(statistics.min), expected_min) && within(number(statistics.max), expected_max))
}

async fn sample(ctx: &Context) -> Result<Pulse> {
    if !testbench::captures(ctx, PULSE) {
        return Ok(Pulse::default());
    }
    let pulses = ctx.get::<Pulses>()?;
    let index = pulses.produced.fetch_add(1, Ordering::AcqRel);
    pulses.published.notify_waiters();
    Ok(Pulse {
        sample_index: index as i64,
        value: pulse_value(index),
    })
}

/* === Tests === */
