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
use super::TEMPLATE;
use crate::domain::Error;
use crate::domain::document::{self as system, Subscriptions};
use crate::domain::model::bench::{COARSEST_STEP_S, FINEST_STEP_S, SHORTEST_S, bench};
use crate::domain::model::charger::{Charger, Fault, Power};
use crate::domain::model::measurement;
use crate::domain::model::measurement::Row;
use crate::operations::charger::*;
use dokime::definition::ParameterDefault;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

bench!(Parameters);

/* === Type Aliases === */

/* === Structs === */

#[dokime::stream(id = "obc.charger.protection.samples", window = 4096, row = Row, label = "AC Device Measurements", description = "The stream this suite's readings travel.\nA type of its own rather than a home on [`measurement::Row`]: four suites measure the same physical thing and each owns the stream its evidence travels, so the shape declares no home and every stream declares itself.")]
pub struct Samples;

pub struct Template {
    device: Charger,
    fault_observed: bool,
    initially_powered: bool,
    restart_blocked: bool,
    tripped_at_step: u64,
    published: u64,
}

#[dokime::record(
    id = "obc.charger.protection.initialized",
    label = "Charger Initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime::record(id = "obc.charger.protection.started", label = "Charger Energized", description = "The supply was energized.")]
#[derive(Clone, Debug)]
pub struct Started {}

#[dokime::record(
    id = "obc.charger.protection.faulted",
    label = "Overcurrent Latch Opened the Charger Supply",
    description = "The overcurrent latch opened the supply."
)]
#[derive(Clone, Debug)]
pub struct Faulted {}

#[dokime::record(
    id = "obc.charger.protection.stopped",
    label = "Charger Supply De-Energized; Any Fault Latch Remains Set",
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

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "AC RMS Voltage", unit = "V", default = 230.0)]
    pub ac_voltage_v: f64,

    #[parameter(label = "Load Impedance Magnitude", unit = "ohm", default = 100.0)]
    pub load_ohm: f64,

    #[parameter(
        label = "Load Steps",
        unit = "ohm",
        expansion = "cartesian",
        min = 0.1,
        min_items = 1,
        max_items = 16,
        default = ParameterDefault::literal(dokime::domain::model::values::Value::Array(::std::vec![
            dokime::domain::model::values::Value::Number(100.0),
            dokime::domain::model::values::Value::Number(50.0),
            dokime::domain::model::values::Value::Number(10.0),
        ])), description = "The impedances the procedure applies in order, starting from the one the Charger was energized at. Declared in ohms like the load it replaces.")]
    pub load_steps_ohm: Vec<f64>,

    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,

    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,

    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,

    #[parameter(label = "Observation Duration", unit = "s", default = 0.2, min = SHORTEST_S)]
    pub duration_s: f64,

    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,
}

/* === Enums === */

#[dokime::catalog]
pub enum Required {
    AskReadiness(crate::operations::AskReadiness),
    InitializeCharger(InitializeCharger),
    ProbeChargerRestart(ProbeChargerRestart),
    ReleaseCharger(ReleaseCharger),
    SampleCharger(SampleCharger),
    SetChargerLoad(SetChargerLoad),
    StartCharger(StartCharger),
    StopCharger(StopCharger),
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
    subscribes = Subscriptions,
    records = Records,
    streams = Streams,
    parameters = Parameters, description = "Walk the declared ladder of loads and verify that the RMS protection latch disconnects the Charger.")]
impl Template {
    async fn initialize(supply: &Parameters, ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        // Check Bench Readiness
        let operation = crate::operations::AskReadiness {};
        let readiness = ctx.operation(operation).await?;
        if readiness.stopped {
            return Err(Error::Observation("the bench is stopped".into()));
        }

        // Initialize Charger
        let operation = InitializeCharger { supply: supply.supply(), logging: false };
        let device = ctx.operation(operation).await?.into_owned();
        Ok(Self {
            device,
            fault_observed: false,
            initially_powered: false,
            restart_blocked: false,
            tripped_at_step: 0,
            published: 0,
        })
    }

    #[dokime::step(
        id = "initialize",
        label = "Initialize and Start Charger",
        description = "Bring the simulated Charger to its nominal operating point.\nEnergises the supply at the configured voltage and load, then waits for the RMS readings to settle before any fault is provoked."
    )]
    async fn setup(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Publish Initialized
        let record = Initialized {};
        ctx.publish(record).await;
        // Start Charger
        let operation = StartCharger { device_id: self.device.device_id };
        self.device = ctx.operation(operation).await?.into_owned();
        let record = Started {};
        ctx.publish(record).await;
        // Sample Charger
        let operation = SampleCharger {
            device_id: self.device.device_id,
            elapsed_s: 0.1,
        };
        let sample = ctx.operation(operation).await?.into_owned();
        self.device = sample.charger;
        let reading = sample.reading;
        self.initially_powered = self.device.power == Power::Running && self.device.fault.is_none() && reading.current_rms_a > 0.0;
        let phase = if self.initially_powered { "running" } else { "initial_fault" };
        // Publish Measurement
        let row = measurement::row(self.published, reading, phase);
        ctx.stream::<Row>().into::<Samples>().push(row).await;
        self.published += 1;
        Ok(())
    }

    #[dokime::step(
        label = "Walk the Load Ladder",
        description = "Walk the declared ladder of load impedances until the latch opens.\n`load_steps_ohm` is one ordered run of impedances belonging to this case, not three cases: the procedure applies each in turn, samples the Charger after each, and stops at the rung where the RMS protection opens the supply."
    )]
    async fn overload(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let interval_s = ctx.parameters().sample_interval_s;
        let ladder = ctx.parameters().load_steps_ohm.clone();
        let mut elapsed_s = 0.1;

        for (rung, load_ohm) in ladder.into_iter().enumerate() {
            // Apply Load
            let operation = SetChargerLoad {
                device_id: self.device.device_id,
                load_ohm,
            };
            self.device = ctx.operation(operation).await?.into_owned();
            elapsed_s += interval_s;
            // Sample Charger
            let operation = SampleCharger {
                device_id: self.device.device_id,
                elapsed_s,
            };
            let sample = ctx.operation(operation).await?.into_owned();
            self.device = sample.charger;
            let reading = sample.reading;
            self.fault_observed = self.initially_powered && self.device.fault == Some(Fault::Overcurrent) && reading.current_rms_a == 0.0;
            let phase = if self.fault_observed { "faulted" } else { "loaded" };
            // Publish Measurement
            let row = measurement::row(self.published, reading, phase);
            ctx.stream::<Row>().into::<Samples>().push(row).await;
            self.published += 1;
            if self.fault_observed {
                self.tripped_at_step = rung as u64 + 1;
                let record = Faulted {};
                ctx.publish(record).await;
                break;
            }
        }

        // Probe Restart While Latched
        let operation = ProbeChargerRestart { device_id: self.device.device_id };
        self.restart_blocked = ctx.operation(operation).await?.blocked;
        Ok(())
    }

    #[dokime::step(
        label = "Leave Charger Safely Disconnected",
        description = "Confirm the supply stays open and leave the Charger de-energised."
    )]
    async fn stop(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Sample Charger
        let operation = SampleCharger {
            device_id: self.device.device_id,
            elapsed_s: 0.1 + ctx.parameters().duration_s,
        };
        let sample = ctx.operation(operation).await?.into_owned();
        self.device = sample.charger;
        let reading = sample.reading;
        // Publish Measurement
        let row = measurement::row(self.published, reading, "disconnected");
        ctx.stream::<Row>().into::<Samples>().push(row).await;
        self.published += 1;
        // Stop Charger
        let operation = StopCharger { device_id: self.device.device_id };
        self.device = ctx.operation(operation).await?.into_owned();
        let record = Stopped {
            accepted: self.fault_observed && self.restart_blocked && self.device.power == Power::Faulted,
            tripped_at_step: self.tripped_at_step,
        };
        ctx.publish(record).await;
        Ok(())
    }

    #[dokime::criterion(
        id = "obc.charger.protection.latch",
        label = "Overcurrent Disconnects and Prevents Restart",
        description = "Passed once the overload latched a fault and blocked the restart."
    )]
    fn protected(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let records = ctx.records::<Stopped>()?;
        let stopped = records.last().ok_or_else(|| Error::MissingEvidence(Stopped::RECORD_TYPE.into()))?;

        // Check / Accepted Outcome
        let accepted_ok = stopped.accepted;
        Ok(accepted_ok)
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        // Release Charger
        let operation = ReleaseCharger { device_id: self.device.device_id };
        let result = ctx.operation(operation).await;
        if let Err(error) = &result {
            ctx.warn(format!("charger cleanup: {error}"));
        }
        result?;
        ctx.info("Charger resources released");
        Ok(())
    }

    fn document() -> Document {
        let mut document = measurement::document(<Samples as dokime::StreamSpec>::ID, "Overcurrent Protection");
        let section = system::section(TEMPLATE);
        document.section(section);
        document
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
