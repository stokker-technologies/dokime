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

// Third Party Imports

// Local Imports
use super::super::parameters::Recovery;
use super::super::{measurement, support};
use super::TEMPLATE;
use crate::error::Error;
use dokime::prelude::*;
use mini_runtime::device::{Device, Fault, Power, Reading};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::stream(id = "mini.charger.recovery.samples", window = 4096, row = measurement::InputMeasurement, label = "AC Device Measurements", description = "The stream this suite's readings travel.\nA type of its own rather than a home on [`measurement::InputMeasurement`]: four suites measure the same physical thing and each owns the stream its evidence travels, so the shape declares no home and every stream declares itself.")]
pub struct Samples;

pub struct PowerLoss {
    device: Device,
    recovered: bool,
    cycles: u64,
    published: u64,
}

/* === Records === */

#[dokime::record(
    id = "mini.charger.recovery.initialized",
    label = "Charger initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime::record(
    id = "mini.charger.recovery.started",
    label = "Charger energized",
    description = "The supply was energized, once before the loss and once after it."
)]
#[derive(Clone, Debug)]
pub struct Started {
    #[record(
        message,
        description = "What energizing meant this time: the first start, or the restart."
    )]
    pub note: String,
}

#[dokime::record(
    id = "mini.charger.recovery.faulted",
    label = "Charger power was removed; plugin and host remain alive",
    description = "Power was removed from the Charger."
)]
#[derive(Clone, Debug)]
pub struct Faulted {}

#[dokime::record(
    id = "mini.charger.recovery.cleared",
    label = "Power restored and fault latch cleared",
    description = "Power was restored and the latch cleared."
)]
#[derive(Clone, Debug)]
pub struct Cleared {}

#[dokime::record(
    id = "mini.charger.recovery.stopped",
    label = "Recovered Charger de-energized",
    description = "The recovered Charger was de-energized, and what the procedure decided."
)]
#[derive(Clone, Debug)]
pub struct Stopped {
    #[record(description = "Whether the recovery behaved the way this procedure required.")]
    pub accepted: bool,
    #[record(description = "How many outage/limit pairs this case replayed.")]
    pub cycles: u64,
}

/* === Enums === */
#[dokime::catalog]
pub enum Required {
    AskReadiness(crate::operations::readiness::AskReadiness),
}

#[dokime::catalog]
pub enum Records {
    Initialized(Initialized),
    Started(Started),
    Faulted(Faulted),
    Cleared(Cleared),
    Stopped(Stopped),
}

#[dokime::catalog]
pub enum Streams {
    Samples(Samples),
}

/* === Traits === */

/* === Implementations === */
#[dokime::template(id = TEMPLATE, label = "Power-Loss Recovery", tags = ["device", "recovery"],
    requires = Required,
    records = Records,
    streams = Streams,
    parameters = Recovery, description = "Lose Charger power, clear the latched fault, and verify a controlled restart.")]
impl PowerLoss {
    async fn initialize(
        supply: &Recovery,
        ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, Error> {
        super::super::support::ready(ctx).await?;
        Ok(Self {
            device: Device::initialize(supply.supply()).map_err(Error::Device)?,
            recovered: false,
            cycles: 0,
            published: 0,
        })
    }
    fn document() -> Document {
        measurement::document(<Samples as dokime::StreamSpec>::ID, "Power-Loss Recovery")
    }
    #[dokime::step(id = "initialize", label = "Initialize and Start Charger")]
    async fn setup(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.publish(Initialized {}).await;
        self.device.start().map_err(Error::Device)?;
        ctx.publish(Started {
            note: "Charger energized.".into(),
        })
        .await;
        let reading = self.device.sample(0.1);
        publish(ctx, &mut self.published, reading, "running").await;
        Ok(())
    }
    #[dokime::step(
        label = "Replay the Outage Schedule",
        description = "Replay the declared outage schedule, one pair at a time.\nEach cycle removes power for its own duration, restores it, clears the latch, and requires the restart to be observable within the paired budget. A case whose two lists are different lengths has half a cycle in it and is refused by name rather than silently truncated."
    )]
    async fn outages(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let outages = ctx.parameters().outage_durations_s.clone();
        let limits = ctx.parameters().recovery_limits_s.clone();
        if outages.len() != limits.len() {
            return Err(Error::Observation(format!(
                "{} outage durations and {} recovery limits are read in step with each other, \
                 so a case declares the same number of each",
                outages.len(),
                limits.len(),
            )));
        }

        let mut elapsed_s = 0.1;
        let mut every_cycle_recovered = true;
        for (outage_s, limit_s) in outages.into_iter().zip(limits) {
            self.device.inject(Fault::PowerLoss);
            elapsed_s += outage_s;
            let dark = self.device.sample(elapsed_s);
            let contained = dark.voltage_rms_v == 0.0 && self.device.start().is_err();
            ctx.publish(Faulted {}).await;
            publish(ctx, &mut self.published, dark, "power_loss").await;

            self.device.clear();
            ctx.publish(Cleared {}).await;
            self.device.start().map_err(Error::Device)?;
            ctx.publish(Started {
                note: format!("Charger restarted under control within {limit_s} s."),
            })
            .await;
            elapsed_s += limit_s;
            let restored = self.device.sample(elapsed_s);
            let recovered = restored.current_rms_a > 0.0 && self.device.power == Power::Running;
            publish(ctx, &mut self.published, restored, "recovered").await;

            every_cycle_recovered &= contained && recovered;
            self.cycles += 1;
        }
        self.recovered = every_cycle_recovered;
        Ok(())
    }

    #[dokime::step(
        label = "Leave Charger De-Energized",
        description = "Leave the recovered Charger de-energized and record what was decided."
    )]
    async fn stop(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        self.device.stop();
        ctx.publish(Stopped {
            accepted: self.recovered && self.cycles > 0 && self.device.fault.is_none(),
            cycles: self.cycles,
        })
        .await;
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        self.device.stop();
        ctx.info("Charger resources released");
        Ok(())
    }

    #[dokime::criterion(
        id = "mini.charger.recovery.restart",
        label = "Power loss is contained and restart succeeds",
        description = "Passed once the loss was contained and the controlled restart recovered."
    )]
    fn recovered(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        support::decided(ctx.records::<Stopped>(), |stopped| stopped.accepted)
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn publish<T, O, I>(
    ctx: &mut StepContext<'_, T, O, I>,
    published: &mut u64,
    reading: Reading,
    phase: &str,
) {
    measurement::publish::<Samples, _, _, _>(ctx, *published, reading, phase).await;
    *published += 1;
}

/* === Tests === */
