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
use std::sync::{Arc, Mutex, PoisonError};

// Third Party Imports

// Local Imports
use crate::suites::{self, parameters, read_supply, supply, timing};

use crate::device::Bench;
use crate::device::{Fault, Power};
use crate::records::charger::{self, Faulted, Initialized, Started, Stopped};
use crate::streams::measurement;
use dokime::plugin::{Criterion, Step, Template};
use dokime::templates::Context;
use dokime::{Document, Parameter, Result, Severity};

/* === Constants === */

pub const TEMPLATE: &str = "obc.transient.overcurrent";
const SAMPLES: &str = "obc.transient.overcurrent.measurement";

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

pub fn template() -> Template {
    let template = Template::of::<()>()
        .identifier("overcurrent")
        .label("Overcurrent Protection")
        .description("Step the load until the overcurrent latch opens the supply, then probe a restart.")
        .tag("device")
        .tag("protection");
    let template = charger::catalog(template);
    let template = parameters(template, supply());
    let template =
        parameters(template, [Parameter::numbers("load_steps_ohm", "Load Steps", [100.0, 50.0, 10.0]).unit("ohm").items(1..=16).build()]);
    measurement::catalog(
        parameters(template, timing("Observation Duration per Load Step", 0.02))
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
            .document(document()),
    )
}

pub fn document() -> Document {
    measurement::document(SAMPLES, TEMPLATE, "Overcurrent Protection")
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    suites::open(ctx, read_supply(ctx)?, Some(ctx.parameter("sample_interval_s")?)).await?;
    ctx.set(Mutex::new(Outcome::default()));
    Ok(())
}

async fn setup(ctx: &Context) -> Result<()> {
    let initialized = charger::initialized();
    let started = charger::started();
    let bench = ctx.get::<Bench>()?;
    let hold = bench.samples_for(ctx.parameter("duration_s")?)?;
    ctx.record(&initialized, Initialized {}).await?;
    bench.start()?;
    ctx.record(
        &started,
        Started {
            note: "Charger energized at the nominal load.".into(),
        },
    )
    .await?;
    // Settle at the Nominal Load
    suites::wait(ctx, &bench, hold).await?;
    let initially_powered = bench.charger().power == Power::Running && bench.charger().reading(bench.now_s()).current_rms_a > 0.0;
    outcome(ctx)?.lock().unwrap_or_else(PoisonError::into_inner).initially_powered = initially_powered;
    Ok(())
}

async fn overload(ctx: &Context) -> Result<()> {
    let faulted = charger::faulted();
    let bench = ctx.get::<Bench>()?;
    let hold = bench.samples_for(ctx.parameter("duration_s")?)?;
    let ladder: Vec<f64> = ctx.parameter("load_steps_ohm")?;
    let initially_powered = outcome(ctx)?.lock().unwrap_or_else(PoisonError::into_inner).initially_powered;
    for (rung, load_ohm) in ladder.into_iter().enumerate() {
        // Apply the Load; Protection Evaluates It at Once
        bench.set_load(load_ohm)?;
        let tripped = bench.fault() == Some(Fault::Overcurrent) && bench.charger().reading(bench.now_s()).current_rms_a == 0.0;
        bench.label(if tripped { "faulted" } else { "loaded" });
        suites::wait(ctx, &bench, hold).await?;
        if initially_powered && tripped {
            ctx.record(&faulted, Faulted { fault: Fault::Overcurrent.name().into() }).await?;
            let shared = outcome(ctx)?;
            let mut shared = shared.lock().unwrap_or_else(PoisonError::into_inner);
            shared.fault_observed = true;
            shared.tripped_at_step = rung as u64 + 1;
            break;
        }
    }
    // Probe a Restart While Latched
    let restart_blocked = bench.charger().start(bench.now_s()).is_err();
    bench.stop();
    ctx.log(Severity::Info, format!("Restart Blocked While Latched: {restart_blocked}")).await?;
    outcome(ctx)?.lock().unwrap_or_else(PoisonError::into_inner).restart_blocked = restart_blocked;
    Ok(())
}

async fn disconnect(ctx: &Context) -> Result<()> {
    let stopped = charger::stopped();
    let bench = ctx.get::<Bench>()?;
    let hold = bench.samples_for(ctx.parameter("duration_s")?)?;
    bench.label("disconnected");
    suites::wait(ctx, &bench, hold).await?;
    bench.stop();
    let latched = bench.charger().power == Power::Faulted;
    let record = {
        let outcome = outcome(ctx)?;
        let outcome = outcome.lock().unwrap_or_else(PoisonError::into_inner);
        Stopped {
            accepted: outcome.fault_observed && outcome.restart_blocked && latched,
            tripped_at_step: outcome.tripped_at_step,
        }
    };
    ctx.record(&stopped, record).await?;
    Ok(())
}

async fn protected(ctx: &Context) -> Result<bool> {
    let stopped = charger::stopped();
    Ok(charger::last(ctx, &stopped).await?.is_some_and(|stopped| stopped.accepted))
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.get::<Bench>()?.stop();
    ctx.log(Severity::Info, "Charger Resources Released").await
}

fn outcome(ctx: &Context) -> Result<Arc<Mutex<Outcome>>> {
    ctx.get::<Mutex<Outcome>>()
}

/* === Tests === */
