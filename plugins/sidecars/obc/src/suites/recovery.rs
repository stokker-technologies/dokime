/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Recovery: Remove Power, Clear the Latch and Verify a Controlled Restart

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{parameters, read_supply, supply, timing};
use crate::catalog::numbers;
use crate::device::{Fault, Power};
use crate::measurement::{self, Measurement};
use crate::records::{self, Cleared, Faulted, Initialized, Started, Stopped};
use crate::settings;
use crate::testbench::{self, Testbench};
use dokime::plugin::{Context, Criterion, Observer, Record, Step, Stream, Suite, Template};
use dokime::{Document, Error, Result, Severity};

/* === Constants === */

pub const SUITE: &str = "obc.charger.recovery";
pub const TEMPLATE: &str = "obc.charger.recovery.power";
pub const SAMPLES: &str = "obc.charger.recovery.samples";
pub const INITIALIZED: &str = "obc.charger.recovery.initialized";
pub const STARTED: &str = "obc.charger.recovery.started";
pub const FAULTED: &str = "obc.charger.recovery.faulted";
pub const CLEARED: &str = "obc.charger.recovery.cleared";
pub const STOPPED: &str = "obc.charger.recovery.stopped";

/* === Macros === */

/* === Type Aliases === */

/// The Record Contracts This Procedure Publishes, in Procedure Order.
pub type Contracts = (Record<Initialized>, Record<Started>, Record<Faulted>, Record<Cleared>, Record<Stopped>);

/* === Structs === */

/// Whether Every Outage Was Contained and Recovered, and How Many Ran.
struct Outcome {
    recovered: std::sync::atomic::AtomicBool,
    cycles: std::sync::atomic::AtomicU64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> Contracts {
    (
        Record::new(INITIALIZED, "Charger Initialized").description("The charger was initialized for this attempt."),
        Record::new(STARTED, "Charger Energized").description("The supply was energized, initially or after an outage."),
        Record::new(FAULTED, "Charger Power Was Removed; Plugin and Host Remain Alive").description("One outage began."),
        Record::new(CLEARED, "Power Restored and Fault Latch Cleared").description("The outage ended and the latch was cleared."),
        Record::new(STOPPED, "Recovered Charger De-Energized").description("The decision, with the number of outage cycles."),
    )
}

pub fn suite() -> Suite {
    let samples = Stream::<Measurement>::new(SAMPLES, "Recovery AC Measurements").description("Charger input across the outage schedule.");
    let observer =
        Observer::new(&samples, async |ctx: &Context| testbench::measure(ctx, SAMPLES).await).interval_setting(settings::SAMPLE_INTERVAL);
    let template = Template::new(TEMPLATE, "Power Recovery")
        .description("Remove the charger's power, clear the latched fault, and verify a controlled restart.")
        .tag("device")
        .tag("recovery");
    let template = parameters(template, supply(None, None));
    let template = parameters(
        template,
        [
            numbers("outage_durations_s", "Outage Durations", "s", &[0.05, 0.1, 0.2]),
            numbers("recovery_limits_s", "Recovery Limits", "s", &[0.02, 0.05, 0.1]),
        ],
    );
    let template = parameters(template, timing("Observation Duration", 0.02))
        .initialize(initialize)
        .step(Step::new("Initialize and Start Charger", setup).identifier("setup"))
        .step(Step::new("Replay the Outage Schedule", outages).identifier("outages").description(
            "Each cycle removes power for its duration, restores it, clears the latch and requires a restart within its paired limit.",
        ))
        .step(Step::new("Leave Charger De-Energized", stop).identifier("stop"))
        .criterion(
            Criterion::new("Power Loss Is Contained and Restart Succeeds", recovered)
                .identifier("restart")
                .description("Passed once every loss was contained and every controlled restart recovered."),
        )
        .cleanup(cleanup)
        .stream(samples)
        .observer(observer)
        .document(document());
    Suite::new(SUITE, "Charger Recovery")
        .description("Recover the charger from a loss of power: clear the latched fault and verify a controlled restart.")
        .tag("device")
        .tag("recovery")
        .template(template)
}

pub fn document() -> Document {
    measurement::document(SAMPLES, TEMPLATE, "Power Recovery")
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    Testbench::open(ctx, read_supply(ctx)?, Some(ctx.parameter("sample_interval_s")?)).await?;
    ctx.set(Outcome { recovered: true.into(), cycles: 0.into() });
    Ok(())
}

async fn setup(ctx: &Context) -> Result<()> {
    let (initialized, started, _, _, _) = contracts();
    let bench = ctx.get::<Testbench>()?;
    ctx.record(&initialized, Initialized {}).await?;
    {
        let mut state = bench.state();
        let now_s = state.now_s();
        state.charger.start(now_s)?;
        state.label = "running";
    }
    ctx.record(&started, Started { note: "Charger energized.".into() }).await?;
    bench.wait(ctx, bench.samples_for(ctx.parameter("duration_s")?)?).await
}

async fn outages(ctx: &Context) -> Result<()> {
    let (_, started, faulted, cleared, _) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let outcome = ctx.get::<Outcome>()?;
    let outages: Vec<f64> = ctx.parameter("outage_durations_s")?;
    let limits: Vec<f64> = ctx.parameter("recovery_limits_s")?;
    if outages.len() != limits.len() {
        return Err(Error::Invalid(format!(
            "Replay Outages: Declare the Same Number of Outage Durations ({}) and Recovery Limits ({})",
            outages.len(),
            limits.len()
        )));
    }
    for (outage_s, limit_s) in outages.into_iter().zip(limits) {
        // Remove Charger Power
        {
            let mut state = bench.state();
            state.charger.inject(Fault::PowerLoss);
            state.label = "power_loss";
        }
        ctx.record(&faulted, Faulted { fault: Fault::PowerLoss.name().into() }).await?;
        bench.wait(ctx, bench.samples_for(outage_s)?).await?;
        let contained = {
            let state = bench.state();
            let mut probe = state.charger.clone();
            state.charger.reading(state.now_s()).voltage_rms_v == 0.0 && probe.start(state.now_s()).is_err()
        };
        // Restore Power, Clear the Latch and Restart
        {
            let mut state = bench.state();
            let now_s = state.now_s();
            state.charger.clear();
            state.charger.start(now_s)?;
            state.label = "recovered";
        }
        ctx.record(&cleared, Cleared {}).await?;
        let note = format!("Charger restarted under control within {limit_s} s.");
        ctx.record(&started, Started { note }).await?;
        bench.wait(ctx, bench.samples_for(limit_s)?).await?;
        let restored = {
            let state = bench.state();
            state.charger.power == Power::Running && state.charger.reading(state.now_s()).current_rms_a > 0.0
        };
        if !(contained && restored) {
            outcome.recovered.store(false, std::sync::atomic::Ordering::Release);
        }
        outcome.cycles.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
    Ok(())
}

async fn stop(ctx: &Context) -> Result<()> {
    let (_, _, _, _, stopped) = contracts();
    let bench = ctx.get::<Testbench>()?;
    let outcome = ctx.get::<Outcome>()?;
    let unfaulted = {
        let mut state = bench.state();
        state.charger.stop();
        state.label = "stopped";
        state.charger.fault.is_none()
    };
    let cycles = outcome.cycles.load(std::sync::atomic::Ordering::Acquire);
    let record = Stopped {
        accepted: outcome.recovered.load(std::sync::atomic::Ordering::Acquire) && cycles > 0 && unfaulted,
        tripped_at_step: 0,
        cycles,
    };
    ctx.record(&stopped, record).await?;
    Ok(())
}

async fn recovered(ctx: &Context) -> Result<bool> {
    let (_, _, _, _, stopped) = contracts();
    Ok(records::last(ctx, &stopped, "").await?.is_some_and(|stopped| stopped.accepted))
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(Severity::Info, "Charger Resources Released").await
}

/* === Tests === */
