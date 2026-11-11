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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// Third Party Imports

// Local Imports
// Third-Party Imports
use crate::domain::Error;
use crate::domain::document::{self as system, Subscriptions};
use crate::domain::model::settings::SAMPLE_INTERVAL_MS;
use dokime::document::{Document, Plot, PlotStyle, Section};
use dokime::execution::{CriterionContext, ObserverContext, StepContext};
use dokime::prelude::StreamExt;

/* === Constants === */

pub const ACCEPTANCE: &str = "obc.checks.pulse.acceptance";
const LABEL: &str = "Pulse";
pub const DEFAULT_SAMPLE_COUNT: i64 = 8;
pub const DEFAULT_TOLERANCE: f64 = 1e-9;
const MAX_SAMPLE_COUNT: u64 = 64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Records

#[dokime::record(
    id = "obc.run.started",
    label = "Run Started",
    description = "The run has begun.\nDeclared here, beside the step that publishes it, and named once on the template's `records = Records`. The struct is the contract: there is no id to keep in agreement with a separate declaration."
)]
#[derive(Clone, Debug)]
pub struct RunStarted {}

#[dokime::record(id = "obc.run.completed", label = "Run Completed", description = "The run finished, and this is what it produced.")]
#[derive(Clone, Debug)]
pub struct RunCompleted {
    #[record(description = "How many rows the declared stream produced before the run completed.")]
    pub samples: u64,
}

pub struct Template;

pub struct PulseResources {
    armed: AtomicBool,
    sample_count: u64,
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
pub struct Row {
    #[row(role = counter)]
    pub sample_index: i64,

    pub value: f64,
}

#[dokime::parameters]
pub struct Parameters {
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
    pub tolerance_au: f64,

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

/* === Enums === */

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
pub enum Streams {
    Row(Row),
}

/* === Traits === */

/* === Implementations === */

#[dokime::template(
    id = "obc.checks.pulse",
    label = LABEL,
    tags = ["example", "sidecar"],
    attributes(path = "OBC/Pulse"),
    // What this template's steps publish, as the types that declare them.
    records = Records,
    prompts = Prompts,
    // The host around the bench, drawn by the closing system section.
    subscribes = Subscriptions,
    // The inputs this template is given, as the type that declares them.
    parameters = Parameters, description = "Streams a short deterministic pulse from inside the sidecar.")]
impl Template {
    async fn initialize(
        parameters: &Parameters, _ctx: &mut dokime::execution::InitializationContext<'_>,
    ) -> Result<(Self, PulseResources), Error> {
        let template = Self;
        let shared = PulseResources {
            armed: AtomicBool::new(false),
            sample_count: parameters.sample_count,
            produced: AtomicU64::new(0),
        };
        Ok((template, shared))
    }

    #[dokime::step(
        label = "Start",
        description = "The first phase: say the run has begun.\nThe host advances its checklist on the step's id, which the engine reports before it calls this. What the step itself does is emit the record this template declared."
    )]
    async fn start(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // === Confirm Capture === //

        if ctx.parameters().require_confirmation {
            // Ask for Confirmation
            let prompt = ConfirmPulse;
            let accepted = ctx.ask(prompt).await?;
            if !accepted {
                return Err(Error::ConfirmationDeclined);
            }
        }

        // === Start Signal Capture === //

        let record = RunStarted {};
        ctx.publish(record).await;

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
        let mut samples = ctx.listen::<Row>();
        ctx.resources().armed.store(true, Ordering::Release);
        while let Some(row) = samples.next().await {
            if row.sample_index as u64 + 1 >= wanted {
                break;
            }
        }

        Ok(())
    }

    #[dokime::step(label = "Complete", description = "The last phase: say what the run produced.")]
    async fn complete(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let produced = ctx.resources().produced.load(Ordering::Acquire);
        let record = RunCompleted { samples: produced };
        ctx.publish(record).await;

        Ok(())
    }

    #[dokime::criterion(id = ACCEPTANCE, label = "Pulse Completed", description = "Decides the run from the rows it produced.\nThree answers, not two: `true` passed, `false` failed, and an error is indeterminate — a sample this cannot read has told it nothing, which is not the same as telling it the run was wrong.")]
    fn completed(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        // Check / Captured System Continuity
        let continuity_ok = !ctx.parameters().capture_system || ctx.gaps::<crate::observers::system::SystemRow>().is_empty();
        if !continuity_ok {
            return Ok(false);
        }
        let samples = ctx.samples::<Row>();
        let tolerance_au = ctx.parameters().tolerance_au;
        for (index, sample) in samples.iter().enumerate() {
            if !sample.value.is_finite() {
                return Err(Error::Unmeasured { index, column: "value" });
            }
            let sample_index = u64::try_from(sample.sample_index).map_err(|_| Error::Unmeasured { index, column: "sample_index" })?;
            // Check / Expected Pulse Value
            let value_ok = (sample.value - pulse_value(sample_index)).abs() <= tolerance_au;
            if !value_ok {
                return Ok(false);
            }
        }

        // Check / Number of Rows
        let number_ok = samples.len() as u64 >= ctx.parameters().sample_count;
        Ok(number_ok)
    }

    #[dokime::observer(interval_ms = SAMPLE_INTERVAL_MS, streams = Streams)]
    async fn pulse(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        if !ctx.resources().armed.load(Ordering::Acquire) {
            return Ok(());
        }
        // Count Published Rows
        let index = ctx.resources().produced.load(Ordering::Acquire);
        let row = Row {
            sample_index: index as i64,
            value: pulse_value(index),
        };
        ctx.publish(row).await;
        if index + 1 >= ctx.resources().sample_count {
            ctx.resources().armed.store(false, Ordering::Release);
        }
        ctx.resources().produced.store(index + 1, Ordering::Release);

        Ok(())
    }

    fn document() -> Document {
        let mut pulse = Plot::time_series("Pulse");
        pulse.id("obc.checks.pulse.signals");
        pulse.unit("a.u.");
        pulse.style(PlotStyle::LinePoints);
        pulse.y(Row::column_value());
        pulse.marker_records([RunStarted::marker(), RunCompleted::marker()]);

        let mut signals = Section::new("Pulse");
        signals.plot(pulse);

        let mut document = Document::new(LABEL);
        document.section(signals);
        let section = system::section("obc.checks.pulse");
        document.section(section);

        document
    }
}

/* === Public Functions === */

pub fn pulse_value(index: u64) -> f64 {
    (index as f64 * 0.5).sin() + index as f64 * 0.05
}

/* === Private Functions === */

/* === Tests === */
