/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Start-Up: Energize the Supply, Inspect the Charger, Observe Settling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::fs;
use std::sync::{Mutex, PoisonError};

// Third Party Imports

// Local Imports
use crate::device::Bench;
use crate::prompts::inspection::{CHECK, inspect};
use crate::records::charger::{self, Completed, Initialized, Started};
use crate::settings;
use crate::streams::measurement;
use crate::suites::{self, attach, parameters, read_supply, sample_count, supply, timing};
use dokime::plugin::{Criterion, Record, Step, Template};
use dokime::templates::Context;
use dokime::{Document, Error, Parameter, Result, Severity};

/* === Constants === */

pub const TEMPLATE: &str = "obc.transient.startup";
pub const CHECKPOINT: &str = "obc.transient.startup.checkpoint";
const SAMPLES: &str = "obc.transient.startup.measurement";

// The Largest Reference Image the Evidence Step Attaches.
const MOST_IMAGE_BYTES: u64 = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Point at Which the Operator Inspected the Trace.
#[derive(dokime::Record)]
#[dokime(package = "obc")]
pub struct Checkpoint {
    pub sample_index: u64,
    pub prompt_type: String,
}

/// The Operator's Inspection, Carried From the Prompt to the Evidence Step.
#[derive(Default)]
struct Decision {
    accepted: bool,
    action: String,
    note: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn checkpoint() -> Record<Checkpoint> {
    Record::new("checkpoint", "Inspection Checkpoint")
}

pub fn template() -> Template {
    let template = Template::of::<()>()
        .identifier("startup")
        .label("Start-Up Sequence")
        .description("Measure the AC start-up inrush, inspect the charger, then observe settling.")
        .tag("device")
        .tag("startup");
    let template = charger::catalog(template).prompt(crate::prompts::catalog().into_iter().next().expect("Inspection Prompt"));
    let template = parameters(template, supply());
    let template = parameters(template, timing("Observation Duration", 0.02));
    let template = parameters(
        template,
        [
            Parameter::flag("emit_artifact", "Attach Measurement Summary").default(true).build(),
            Parameter::flag("attach_reference", "Attach reference.png From the Image Directory").default(false).build(),
            Parameter::flag("realtime", "Pace Samples in Real Time").default(false).build(),
            Parameter::flag("continuous", "Observe Until Stopped").default(false).build(),
        ],
    );
    measurement::catalog(
        template
            .initialize(initialize)
            .record(checkpoint())
            .step(Step::new("Initialize and Start Charger", setup).identifier("setup"))
            .step(Step::new("Measure Start-Up Inrush", inrush).identifier("inrush"))
            .step(Step::new("Inspect Powered Charger", inspection).identifier("inspection"))
            .step(Step::new("Observe Settling and Attach Evidence", settle).identifier("settle"))
            .criterion(
                Criterion::new("Start-Up Trace and Operator Inspection", accepted)
                    .identifier("acceptance")
                    .description("Passed once the trace is unfaulted and, for a bounded run, the operator accepted it."),
            )
            .cleanup(cleanup)
            .document(document()),
    )
}

pub fn document() -> Document {
    measurement::document(SAMPLES, TEMPLATE, "Start-Up Sequence")
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    // Open the Bench With the Model Advancing Per Sample or With the Host Clock
    let realtime: bool = ctx.parameter("realtime")?;
    let step_s = if realtime { None } else { Some(ctx.parameter("sample_interval_s")?) };
    suites::open(ctx, read_supply(ctx)?, step_s).await?;
    ctx.set(Mutex::new(Decision::default()));
    ctx.log(Severity::Info, "Charger Initialized").await
}

async fn setup(ctx: &Context) -> Result<()> {
    let initialized = charger::initialized();
    let started = charger::started();
    ctx.record(&initialized, Initialized {}).await?;
    // Energize the Supply at the Current Model Time
    let bench = ctx.get::<Bench>()?;
    let supply = bench.start()?;
    let duration_s: f64 = ctx.parameter("duration_s")?;
    let note = format!("Energized at {} V RMS Into {} Ω for {duration_s} s.", supply.voltage_rms_v, supply.load_ohm);
    ctx.log(Severity::Info, note.clone()).await?;
    ctx.record(&started, Started { note }).await?;
    Ok(())
}

async fn inrush(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    let total = bench.samples_for(ctx.parameter("duration_s")?)?;
    let checkpoint = (total / 2).max(1);
    bench.sequence(checkpoint, total);
    if ctx.parameter::<bool>("continuous")? {
        // Observe Until a Host Stop or Cancellation Ends the Step
        loop {
            suites::wait(ctx, &bench, total).await?;
            let produced = bench.index();
            ctx.progress(produced, produced, "Observing Continuously").await?;
        }
    }
    let produced = bench.index();
    suites::wait(ctx, &bench, checkpoint.saturating_sub(produced)).await?;
    ctx.progress(checkpoint, total, "Measuring Start-Up Inrush").await
}

async fn inspection(ctx: &Context) -> Result<()> {
    let checkpoint = checkpoint();
    let produced = ctx.get::<Bench>()?.index();
    let record = Checkpoint {
        sample_index: produced,
        prompt_type: CHECK.into(),
    };
    ctx.record(&checkpoint, record).await?;
    // Ask for Operator Inspection
    let answer = inspect(ctx, CHECK, "Charger Status Indicator Steady").await?;
    ctx.log(Severity::Info, format!("Inspection Answered: {}", answer.action)).await?;
    let decision = ctx.get::<Mutex<Decision>>()?;
    let mut decision = decision.lock().unwrap_or_else(PoisonError::into_inner);
    *decision = Decision {
        accepted: answer.accepted,
        action: answer.action,
        note: answer.note,
    };
    Ok(())
}

async fn settle(ctx: &Context) -> Result<()> {
    let completed = charger::completed();
    let bench = ctx.get::<Bench>()?;
    let total = bench.samples_for(ctx.parameter("duration_s")?)?;
    let (accepted, action, note) = {
        let decision = ctx.get::<Mutex<Decision>>()?;
        let decision = decision.lock().unwrap_or_else(PoisonError::into_inner);
        (decision.accepted, decision.action.clone(), decision.note.clone())
    };
    if accepted {
        let produced = bench.index();
        suites::wait(ctx, &bench, total.saturating_sub(produced)).await?;
        ctx.progress(total, total, "Observing Settling").await?;
    }
    let produced = bench.index();
    let supply = bench.charger().supply;
    let unfaulted = bench.fault().is_none();
    let step_s = bench.step_s();
    let record = Completed {
        sample_count: produced,
        prompt_action: action.clone(),
        operator_note: note.clone(),
        accepted: accepted && unfaulted,
    };
    ctx.record(&completed, record).await?;
    if ctx.parameter("emit_artifact")? {
        // Attach the Measurement Summary
        let summary = serde_json::json!({
            "supply": {
                "voltage_rms_v": supply.voltage_rms_v,
                "load_ohm": supply.load_ohm,
                "frequency_hz": supply.frequency_hz,
                "reactance_ohm": supply.reactance_ohm,
                "current_limit_a": supply.current_limit_a,
            },
            "sample_count": produced,
            "duration_s": produced as f64 * step_s,
            "prompt_action": action,
            "operator_note": note,
            "accepted": accepted,
        });
        attach(ctx, "startup-summary.json", "application/json", &serde_json::to_vec_pretty(&summary).unwrap_or_default()).await?;
    }
    if ctx.parameter("attach_reference")? {
        // Attach the Installed Reference Image
        let path = settings::images_directory(ctx).await?.join("reference.png");
        let size = fs::metadata(&path)
            .map_err(|error| Error::Invalid(format!("Attach Reference Image: Place reference.png in {}: {error}", path.display())))?
            .len();
        if size > MOST_IMAGE_BYTES {
            return Err(Error::Invalid("Attach Reference Image: Reduce reference.png to at Most 4 MiB".into()));
        }
        let bytes = fs::read(&path).map_err(|error| Error::Invalid(format!("Attach Reference Image: {error}")))?;
        attach(ctx, "reference.png", "image/png", &bytes).await?;
    }
    Ok(())
}

async fn accepted(ctx: &Context) -> Result<bool> {
    let bench = ctx.get::<Bench>()?;
    let unfaulted = bench.fault().is_none();
    if ctx.parameter("continuous")? {
        return Ok(unfaulted && sample_count(ctx, SAMPLES, "").await? > 0);
    }
    let completed = charger::completed();
    let Some(completed) = charger::last(ctx, &completed).await? else {
        return Ok(false);
    };
    // Check / Number of Saved Rows and Operator Acceptance
    let total = bench.samples_for(ctx.parameter("duration_s")?)?;
    Ok(sample_count(ctx, SAMPLES, "").await? >= total && completed.accepted)
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.get::<Bench>()?.stop();
    ctx.log(Severity::Info, "Charger Resources Released").await
}

/* === Tests === */
