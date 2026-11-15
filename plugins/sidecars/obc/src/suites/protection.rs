/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Protection: Step the Load Until the Overcurrent Latch Opens the Supply

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Mutex;

// Third Party Imports

// Local Imports
use super::{parameters, read_supply, supply, timing};
use crate::catalog::numbers;
use crate::device::{Fault, Power};
use crate::measurement::{self, Measurement};
use crate::records::{self, Faulted, Initialized, Started, Stopped};
use crate::settings;
use crate::testbench::{self, Testbench};
use dokime::plugin::{Context, Criterion, Observer, Record, Step, Stream, Suite, Template};
use dokime::{Document, Result, Severity};

/* === Constants === */

pub const SUITE: &str = "obc.charger.protection";
pub const TEMPLATE: &str = "obc.charger.protection.overcurrent";
pub const SAMPLES: &str = "obc.charger.protection.samples";
pub const INITIALIZED: &str = "obc.charger.protection.initialized";
pub const STARTED: &str = "obc.charger.protection.started";
pub const FAULTED: &str = "obc.charger.protection.faulted";
pub const STOPPED: &str = "obc.charger.protection.stopped";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct Outcome {
    initially_powered: bool,
    fault_observed: bool,
    restart_blocked: bool,
    tripped_at_step: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> (Record<Initialized>, Record<Started>, Record<Faulted>, Record<Stopped>) {
    (
        Record::new(INITIALIZED, "Charger Initialized").description("The charger was initialized for this attempt."),
        Record::new(STARTED, "Charger Energized").description("The supply was energized before the load ladder."),
        Record::new(FAULTED, "Overcurrent Latch Opened the Charger Supply").description("Protection tripped on a load step."),
        Record::new(STOPPED, "Charger Supply De-Energized; Any Fault Latch Remains Set")
            .description("The decision, with the load step that tripped protection."),
    )
}

pub fn suite() -> Suite {
    let samples = Stream::<Measurement>::new(SAMPLES, "Protection AC Measurements").description("Charger input while the load steps up.");
    let observer =
        Observer::new(&samples, async |ctx: &Context| testbench::measure(ctx, SAMPLES).await).interval_setting(settings::SAMPLE_INTERVAL);
    let template = Template::new(TEMPLATE, "Overcurrent Protection")
        .description("Step the load until the overcurrent latch opens the supply, then probe a restart.")
        .tag("device")
        .tag("protection");
    let template = parameters(template, supply(None, None));
    let template = parameters(template, [numbers("load_steps_ohm", "Load Steps", "ohm", &[100.0, 50.0, 10.0])]);
    let template = parameters(template, timing("Observation Duration per Load Step", 0.02))
        .initialize(initialize)
        .step(Step::new("Initialize and Start Charger", setup).identifier("setup"))
        .step(Step::new("Walk the Load Ladder", overload).identifier("overload"))
        .step(Step::new("Leave Charger Safely Disconnected", disconnect).identifier("disconnect"))
        .criterion(
            Criterion::new("Overcurrent Disconnects and Prevents Restart", protected)
                .identifier("latch")
                .description("Passed once protection tripped on a load step and the latched charger refused to restart."),
        )
        .cleanup(cleanup)
        .stream(samples)
        .observer(observer)
        .document(document());
    Suite::new(SUITE, "Charger Protection")
        .description("Verify that the charger's protection disconnects it once the load draws too much current.")
        .tag("device")
        .tag("protection")
        .template(template)
}

pub fn document() -> Document {
    measurement::document(SAMPLES, TEMPLATE, "Overcurrent Protection")
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    Testbench::open(ctx, read_supply(ctx)?, Some(ctx.parameter("sample_interval_s")?)).await?;
    ctx.set(Mutex::new(Outcome::default()));
    Ok(())
}

async fn setup(ctx: &Context) -> Result<()> {
    let (initialized, started, _, _) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let hold = bench.samples_for(ctx.parameter("duration_s")?)?;
    ctx.record(&initialized, Initialized {}).await?;
    {
        let mut state = bench.state();
        let now_s = state.now_s();
        state.charger.start(now_s)?;
        state.label = "running";
    }
    ctx.record(
        &started,
        Started {
            note: "Charger energized at the nominal load.".into(),
        },
    )
    .await?;
    // Settle at the Nominal Load
    bench.wait(ctx, hold).await?;
    let initially_powered = {
        let state = bench.state();
        state.charger.power == Power::Running && state.charger.reading(state.now_s()).current_rms_a > 0.0
    };
    outcome(ctx)?.lock().unwrap_or_else(std::sync::PoisonError::into_inner).initially_powered = initially_powered;
    Ok(())
}

async fn overload(ctx: &Context) -> Result<()> {
    let (_, _, faulted, _) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let hold = bench.samples_for(ctx.parameter("duration_s")?)?;
    let ladder: Vec<f64> = ctx.parameter("load_steps_ohm")?;
    let initially_powered = outcome(ctx)?.lock().unwrap_or_else(std::sync::PoisonError::into_inner).initially_powered;
    for (rung, load_ohm) in ladder.into_iter().enumerate() {
        // Apply the Load; Protection Evaluates It at Once
        let tripped = {
            let mut state = bench.state();
            let now_s = state.now_s();
            state.charger.set_load(load_ohm, now_s)?;
            let tripped = state.charger.fault == Some(Fault::Overcurrent) && state.charger.reading(now_s).current_rms_a == 0.0;
            state.label = if tripped { "faulted" } else { "loaded" };
            tripped
        };
        bench.wait(ctx, hold).await?;
        if initially_powered && tripped {
            ctx.record(&faulted, Faulted { fault: Fault::Overcurrent.name().into() }).await?;
            let shared = outcome(ctx)?;
            let mut shared = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            shared.fault_observed = true;
            shared.tripped_at_step = rung as u64 + 1;
            break;
        }
    }
    // Probe a Restart While Latched
    let restart_blocked = {
        let mut state = bench.state();
        let now_s = state.now_s();
        let mut probe = state.charger.clone();
        let blocked = probe.start(now_s).is_err();
        state.charger.stop();
        blocked
    };
    ctx.log(Severity::Info, format!("Restart Blocked While Latched: {restart_blocked}")).await?;
    outcome(ctx)?.lock().unwrap_or_else(std::sync::PoisonError::into_inner).restart_blocked = restart_blocked;
    Ok(())
}

async fn disconnect(ctx: &Context) -> Result<()> {
    let (_, _, _, stopped) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let hold = bench.samples_for(ctx.parameter("duration_s")?)?;
    bench.state().label = "disconnected";
    bench.wait(ctx, hold).await?;
    let latched = {
        let mut state = bench.state();
        state.charger.stop();
        state.charger.power == Power::Faulted
    };
    let record = {
        let outcome = outcome(ctx)?;
        let outcome = outcome.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        Stopped {
            accepted: outcome.fault_observed && outcome.restart_blocked && latched,
            tripped_at_step: outcome.tripped_at_step,
            cycles: 0,
        }
    };
    ctx.record(&stopped, record).await?;
    Ok(())
}

async fn protected(ctx: &Context) -> Result<bool> {
    let (_, _, _, stopped) = contracts();
    Ok(records::last(ctx, &stopped, "").await?.is_some_and(|stopped| stopped.accepted))
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(Severity::Info, "Charger Resources Released").await
}

fn outcome(ctx: &Context) -> Result<std::sync::Arc<Mutex<Outcome>>> {
    ctx.get::<Mutex<Outcome>>()
}

/* === Tests === */
