/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use crate::error::Error;
use crate::suites::common::system::{self, SystemInputs};
use dokime::document::{Document, Plot, PlotStyle, Section};
use dokime::execution::{CriterionContext, ObserverContext, StepContext};

pub const ACCEPTANCE: &str = "obc.checks.pulse.acceptance";
const LABEL: &str = "Pulse";
pub const DEFAULT_SAMPLE_COUNT: i64 = 8;
pub const SAMPLE_INTERVAL_MS: u64 = 100;
pub const DEFAULT_TOLERANCE: f64 = 1e-9;
const MAX_SAMPLE_COUNT: u64 = 64;
const PROGRESS_POLL: Duration = Duration::from_millis(5);

// Records

#[dokime::record(
    id = "obc.run.started",
    label = "Run started",
    description = "The run has begun.\nDeclared here, beside the step that publishes it, and named once on the template's `records = Records`. The struct is the contract: there is no id to keep in agreement with a separate declaration."
)]
#[derive(Clone, Debug)]
pub struct RunStarted {}

#[dokime::record(id = "obc.run.completed", label = "Run completed", description = "The run finished, and this is what it produced.")]
#[derive(Clone, Debug)]
pub struct RunCompleted {
    #[record(description = "How many rows the declared stream produced before the run completed.")]
    pub samples: u64,
}

pub struct Pulse;

#[derive(Default)]
pub struct PulseResources {
    armed: AtomicBool,
    produced: AtomicU64,
}

// The attribute reads these fields once and generates every half: the columns
// the stream's contract declares, the values a sample carries, and the id
// every other reference to this stream names it by. A field's column type is
// resolved from its Rust type, so `value: f64` is a number column because the
// compiler said so.
#[dokime::stream(
    id = "obc.pulse",
    window = 4096,
    label = "OBC Pulse",
    description = "Deterministic samples produced inside the OBC plugin."
)]
pub struct PulseRow {
    #[row(role = counter)]
    pub sample_index: i64,

    pub value: f64,
}

#[dokime::parameters]
pub struct PulseParameters {
    #[parameter(
        label = "Sample Count",
        default = DEFAULT_SAMPLE_COUNT,
        min = 1,
        max = MAX_SAMPLE_COUNT,
    )]
    pub sample_count: u64,

    #[parameter(
        label = "Value Tolerance",
        unit = "a.u.",
        default = DEFAULT_TOLERANCE,
        min = DEFAULT_TOLERANCE,
    )]
    pub tolerance: f64,

    #[parameter(
        label = "Capture System During Pulse",
        default = false,
        description = "Demonstrates a step-scoped hold on the sidecar's global system stream."
    )]
    pub capture_system: bool,

    #[parameter(
        label = "Require Confirmation",
        default = false,
        description = "Ask one confirmation before simulated pulse capture begins."
    )]
    pub require_confirmation: bool,
}

#[dokime::prompt(
    id = "obc.checks.pulse.confirm",
    label = "Start Simulated Pulse",
    description = "Confirm that you want to start simulated pulse capture for this execution."
)]
pub struct ConfirmPulse;

#[dokime::catalog]
pub enum Records {
    RunStarted(RunStarted),
    RunCompleted(RunCompleted),
}

#[dokime::catalog]
pub enum Prompts {
    ConfirmPulse(ConfirmPulse),
}

#[dokime::catalog]
pub enum ObserverStreams {
    PulseRow(PulseRow),
}

#[dokime::template(
    id = "obc.checks.pulse",
    label = LABEL,
    tags = ["example", "sidecar"],
    attributes(path = "OBC/Pulse"),
    // What this template's steps publish, as the types that declare them.
    records = Records,
    prompts = Prompts,
    // The host around the bench, drawn by the closing system section.
    subscribes = SystemInputs,
    // The inputs this template is given, as the type that declares them.
    parameters = PulseParameters, description = "Streams a short deterministic pulse from inside the sidecar.")]
impl Pulse {
    async fn initialize(
        _: &PulseParameters, _ctx: &mut dokime::execution::InitializationContext<'_>,
    ) -> Result<(Self, PulseResources), Error> {
        let template = Self;
        let shared = PulseResources::default();
        Ok((template, shared))
    }

    fn document() -> Document {
        let mut pulse = Plot::time_series("Pulse");
        pulse.id("obc.checks.pulse.signals");
        pulse.unit("a.u.");
        pulse.style(PlotStyle::LinePoints);
        pulse.y(PulseRow::column_value());
        pulse.marker_records([RunStarted::marker(), RunCompleted::marker()]);

        let mut signals = Section::new("Pulse");
        signals.plot(pulse);

        let mut document = Document::new("obc.checks.pulse.document", LABEL);
        document.section(signals);
        document.section(system::section("obc.checks.pulse"));

        document
    }

    #[dokime::step(
        label = "Start",
        description = "The first phase: say the run has begun.\nThe host advances its checklist on the step's id, which the engine reports before it calls this. What the step itself does is emit the record this template declared."
    )]
    async fn start(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        if ctx.parameters().require_confirmation && !ctx.ask(ConfirmPulse).await? {
            return Err(Error::ConfirmationDeclined);
        }

        ctx.publish(RunStarted {}).await;
        ctx.resources().armed.store(true, Ordering::Release);

        Ok(())
    }

    #[dokime::step(
        label = "Stream Samples",
        description = "Waits for the declared stream to produce the rows this case asked for.\nIt waits on a count rather than on a clock, so the evidence a run produced is what ended it. A cancellation ends the wait wherever it has got to, and the criterion below then decides over the rows that exist."
    )]
    async fn stream(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // The guard releases this hold on return, error, or cancellation. No
        // template-wide samples(...) declaration is needed for this window.
        let _system = if ctx.parameters().capture_system { Some(ctx.lease::<crate::observers::system::SystemRow>().await?) } else { None };
        let wanted = ctx.parameters().sample_count;
        let started = ctx.resources().produced.load(Ordering::Acquire);
        while ctx.resources().produced.load(Ordering::Acquire).saturating_sub(started) < wanted {
            tokio::time::sleep(PROGRESS_POLL).await;
        }

        Ok(())
    }

    #[dokime::step(label = "Complete", description = "The last phase: say what the run produced.")]
    async fn complete(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let produced = ctx.resources().produced.load(Ordering::Acquire);
        ctx.publish(RunCompleted { samples: produced }).await;

        Ok(())
    }

    #[dokime::criterion(id = ACCEPTANCE, label = "Pulse Completed", description = "Decides the run from the rows it produced.\nThree answers, not two: `true` passed, `false` failed, and an error is indeterminate — a sample this cannot read has told it nothing, which is not the same as telling it the run was wrong.")]
    fn completed(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        if ctx.parameters().capture_system && !ctx.gaps::<crate::observers::system::SystemRow>().is_empty() {
            return Ok(false);
        }
        let samples = ctx.samples::<PulseRow>();
        let tolerance = ctx.parameters().tolerance;
        for (index, sample) in samples.iter().enumerate() {
            if !sample.value.is_finite() {
                return Err(Error::Unmeasured { index, column: "value" });
            }
            let sample_index = u64::try_from(sample.sample_index).map_err(|_| Error::Unmeasured { index, column: "sample_index" })?;
            if (sample.value - pulse_value(sample_index)).abs() > tolerance {
                return Ok(false);
            }
        }

        Ok(samples.len() as u64 >= ctx.parameters().sample_count)
    }

    #[dokime::observer(interval_ms = SAMPLE_INTERVAL_MS, streams = ObserverStreams)]
    async fn pulse(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        if !ctx.resources().armed.load(Ordering::Acquire) {
            return Ok(());
        }
        // Count A Row Only Once Published, So The Stream Step Never Ends Ahead Of It.
        let index = ctx.resources().produced.load(Ordering::Acquire);
        ctx.publish(PulseRow {
            sample_index: index as i64,
            value: pulse_value(index),
        })
        .await;
        ctx.resources().produced.store(index + 1, Ordering::Release);

        Ok(())
    }
}

pub fn pulse_value(index: u64) -> f64 {
    (index as f64 * 0.5).sin() + index as f64 * 0.05
}
