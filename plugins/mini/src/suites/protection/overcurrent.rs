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
use super::super::parameters::Protection;
use super::super::{measurement, support};
use super::TEMPLATE;
use crate::error::Error;
use dokime::prelude::*;
use mini_runtime::device::{Device, Fault, Power, Reading};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::stream(id = "mini.charger.protection.samples", window = 4096, row = measurement::InputMeasurement, label = "AC Device Measurements", description = "The stream this suite's readings travel.\nA type of its own rather than a home on [`measurement::InputMeasurement`]: four suites measure the same physical thing and each owns the stream its evidence travels, so the shape declares no home and every stream declares itself.")]
pub struct Samples;

pub struct Overcurrent {
    device: Device,
    fault_observed: bool,
    initially_powered: bool,
    restart_blocked: bool,
    tripped_at_step: u64,
    published: u64,
}

/* === Records === */

#[dokime::record(
    id = "mini.charger.protection.initialized",
    label = "Charger initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime::record(
    id = "mini.charger.protection.started",
    label = "Charger energized",
    description = "The supply was energized."
)]
#[derive(Clone, Debug)]
pub struct Started {}

#[dokime::record(
    id = "mini.charger.protection.faulted",
    label = "Overcurrent latch opened the Charger supply",
    description = "The overcurrent latch opened the supply."
)]
#[derive(Clone, Debug)]
pub struct Faulted {}

#[dokime::record(
    id = "mini.charger.protection.stopped",
    label = "Charger supply de-energized; any fault latch remains set",
    description = "The procedure left the Charger safely disconnected, and what it decided."
)]
#[derive(Clone, Debug)]
pub struct Stopped {
    #[record(description = "Whether the protection behaved the way this procedure required.")]
    pub accepted: bool,
    #[record(
        description = "Which rung of the load ladder opened the latch, counted from one, or zero when no declared impedance reached the limit."
    )]
    #[record(
        description = "Which rung of the load ladder opened the latch, counted from one, or zero when no declared impedance reached the limit."
    )]
    pub tripped_at_step: u64,
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
    Stopped(Stopped),
}

#[dokime::catalog]
pub enum Streams {
    Samples(Samples),
}

/* === Traits === */

/* === Implementations === */
#[dokime::template(id = TEMPLATE, label = "Overcurrent Protection", tags = ["device", "protection"],
    requires = Required,
    records = Records,
    streams = Streams,
    parameters = Protection, description = "Walk the declared ladder of loads and verify that the RMS protection latch disconnects the Charger.")]
impl Overcurrent {
    async fn initialize(
        supply: &Protection,
        ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, Error> {
        super::super::support::ready(ctx).await?;
        Ok(Self {
            device: Device::initialize(supply.supply()).map_err(Error::Device)?,
            fault_observed: false,
            initially_powered: false,
            restart_blocked: false,
            tripped_at_step: 0,
            published: 0,
        })
    }
    fn document() -> Document {
        measurement::document(
            <Samples as dokime::StreamSpec>::ID,
            "Overcurrent Protection",
        )
    }
    #[dokime::step(
        id = "initialize",
        label = "Initialize and Start Charger",
        description = "Bring the simulated Charger to its nominal operating point.\nEnergises the supply at the configured voltage and load, then waits for the RMS readings to settle before any fault is provoked."
    )]
    async fn setup(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.publish(Initialized {}).await;
        self.device.start().map_err(Error::Device)?;
        ctx.publish(Started {}).await;
        let reading = self.device.sample(0.1);
        self.initially_powered = self.device.power == Power::Running
            && self.device.fault.is_none()
            && reading.current_rms_a > 0.0;
        let phase = if self.initially_powered {
            "running"
        } else {
            "initial_fault"
        };
        publish(ctx, &mut self.published, reading, phase).await;
        Ok(())
    }
    #[dokime::step(
        label = "Walk the Load Ladder",
        description = "Walk the declared ladder of load impedances until the latch opens.\n`load_steps` is one ordered run of impedances belonging to this case, not three cases: the procedure applies each in turn, samples the Charger after each, and stops at the rung where the RMS protection opens the supply."
    )]
    async fn overload(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let interval_s = ctx.parameters().sample_interval_s;
        let ladder = ctx.parameters().load_steps.clone();
        let mut elapsed_s = 0.1;

        for (rung, load_ohm) in ladder.into_iter().enumerate() {
            self.device.supply.load_ohm = load_ohm;
            elapsed_s += interval_s;
            let reading = self.device.sample(elapsed_s);
            self.fault_observed = self.initially_powered
                && self.device.fault == Some(Fault::Overcurrent)
                && reading.current_rms_a == 0.0;
            let phase = if self.fault_observed {
                "faulted"
            } else {
                "loaded"
            };
            publish(ctx, &mut self.published, reading, phase).await;
            if self.fault_observed {
                self.tripped_at_step = rung as u64 + 1;
                ctx.publish(Faulted {}).await;
                break;
            }
        }

        self.restart_blocked = self.device.start().is_err();
        Ok(())
    }

    #[dokime::step(
        label = "Leave Charger Safely Disconnected",
        description = "Confirm the supply stays open and leave the Charger de-energised."
    )]
    async fn stop(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let reading = self.device.sample(0.1 + ctx.parameters().duration_s);
        publish(ctx, &mut self.published, reading, "disconnected").await;
        self.device.stop();
        ctx.publish(Stopped {
            accepted: self.fault_observed
                && self.restart_blocked
                && self.device.power == Power::Faulted,
            tripped_at_step: self.tripped_at_step,
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
        id = "mini.charger.protection.latch",
        label = "Overcurrent disconnects and prevents restart",
        description = "Passed once the overload latched a fault and blocked the restart."
    )]
    fn protected(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
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
