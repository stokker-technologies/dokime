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

#[dokime::stream(id = "obc.charger.recovery.samples", window = 4096, row = Row, label = "AC Device Measurements", description = "The stream this suite's readings travel.\nA type of its own rather than a home on [`measurement::Row`]: four suites measure the same physical thing and each owns the stream its evidence travels, so the shape declares no home and every stream declares itself.")]
pub struct Samples;

pub struct Template {
    device: Charger,
    recovered: bool,
    cycles: u64,
    published: u64,
}

// Records

#[dokime::record(
    id = "obc.charger.recovery.initialized",
    label = "Charger Initialized",
    description = "The Charger was initialized for this attempt."
)]
#[derive(Clone, Debug)]
pub struct Initialized {}

#[dokime::record(
    id = "obc.charger.recovery.started",
    label = "Charger Energized",
    description = "The supply was energized, once before the loss and once after it."
)]
#[derive(Clone, Debug)]
pub struct Started {
    #[record(message, description = "What energizing meant this time: the first start, or the restart.")]
    pub note: String,
}

#[dokime::record(
    id = "obc.charger.recovery.faulted",
    label = "Charger Power Was Removed; Plugin and Host Remain Alive",
    description = "Power was removed from the Charger."
)]
#[derive(Clone, Debug)]
pub struct Faulted {}

#[dokime::record(
    id = "obc.charger.recovery.cleared",
    label = "Power Restored and Fault Latch Cleared",
    description = "Power was restored and the latch cleared."
)]
#[derive(Clone, Debug)]
pub struct Cleared {}

#[dokime::record(
    id = "obc.charger.recovery.stopped",
    label = "Recovered Charger De-Energized",
    description = "The recovered Charger was de-energized, and what the procedure decided."
)]
#[derive(Clone, Debug)]
pub struct Stopped {
    #[record(description = "Whether the recovery behaved the way this procedure required.")]
    pub accepted: bool,

    #[record(description = "How many outage/limit pairs this case replayed.")]
    pub cycles: u64,
}

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "AC RMS Voltage", unit = "V", default = 230.0)]
    pub ac_voltage_v: f64,

    #[parameter(label = "Load Impedance Magnitude", unit = "ohm", default = 100.0)]
    pub load_ohm: f64,

    #[parameter(
        label = "Outage Durations",
        unit = "s",
        expansion = zip("outage"),
        min = 0.0,
        min_items = 1,
        max_items = 16,
        default = ParameterDefault::literal(dokime::domain::model::values::Value::Array(::std::vec![
            dokime::domain::model::values::Value::Number(0.05),
            dokime::domain::model::values::Value::Number(0.1),
            dokime::domain::model::values::Value::Number(0.2),
        ])), description = "How long the Charger stays without power on each cycle, in model seconds.")]
    pub outage_durations_s: Vec<f64>,

    #[parameter(
        label = "Recovery Limits",
        unit = "s",
        expansion = zip("outage"),
        min = SHORTEST_S,
        min_items = 1,
        max_items = 16,
        default = ParameterDefault::literal(dokime::domain::model::values::Value::Array(::std::vec![
            dokime::domain::model::values::Value::Number(0.02),
            dokime::domain::model::values::Value::Number(0.05),
            dokime::domain::model::values::Value::Number(0.1),
        ])), description = "How long the restart after each outage is allowed to take, in model seconds, read in step with [`Parameters::outage_durations_s`].")]
    pub recovery_limits_s: Vec<f64>,

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
    ClearChargerFault(ClearChargerFault),
    InitializeCharger(InitializeCharger),
    InjectChargerFault(InjectChargerFault),
    ProbeChargerRestart(ProbeChargerRestart),
    ReleaseCharger(ReleaseCharger),
    SampleCharger(SampleCharger),
    StartCharger(StartCharger),
    StopCharger(StopCharger),
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

#[dokime::template(id = TEMPLATE, label = "Power Recovery", tags = ["device", "recovery"],
    requires = Required,
    subscribes = Subscriptions,
    records = Records,
    streams = Streams,
    parameters = Parameters, description = "Remove the Charger's power, clear the latched fault, and verify a controlled restart.")]
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
            recovered: false,
            cycles: 0,
            published: 0,
        })
    }

    #[dokime::step(id = "initialize", label = "Initialize and Start Charger")]
    async fn setup(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Publish Initialized
        let record = Initialized {};
        ctx.publish(record).await;
        // Start Charger
        let operation = StartCharger { device_id: self.device.device_id };
        self.device = ctx.operation(operation).await?.into_owned();
        let record = Started { note: "Charger energized.".into() };
        ctx.publish(record).await;
        // Sample Charger
        let operation = SampleCharger {
            device_id: self.device.device_id,
            elapsed_s: 0.1,
        };
        let sample = ctx.operation(operation).await?.into_owned();
        self.device = sample.charger;
        let reading = sample.reading;
        // Publish Measurement
        let row = measurement::row(self.published, reading, "running");
        ctx.stream::<Row>().into::<Samples>().push(row).await;
        self.published += 1;
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
            // Remove Charger Power
            let operation = InjectChargerFault {
                device_id: self.device.device_id,
                fault: Fault::PowerLoss,
            };
            self.device = ctx.operation(operation).await?.into_owned();
            elapsed_s += outage_s;
            // Sample Charger
            let operation = SampleCharger {
                device_id: self.device.device_id,
                elapsed_s: elapsed_s,
            };
            let sample = ctx.operation(operation).await?.into_owned();
            self.device = sample.charger;
            let dark = sample.reading;
            // Probe Restart While Power Is Lost
            let operation = ProbeChargerRestart { device_id: self.device.device_id };
            let restart = ctx.operation(operation).await?;
            let contained = dark.voltage_rms_v == 0.0 && restart.blocked;
            let record = Faulted {};
            ctx.publish(record).await;
            // Publish Measurement
            let row = measurement::row(self.published, dark, "power_loss");
            ctx.stream::<Row>().into::<Samples>().push(row).await;
            self.published += 1;

            // Clear Fault Latch
            let operation = ClearChargerFault { device_id: self.device.device_id };
            self.device = ctx.operation(operation).await?.into_owned();
            let record = Cleared {};
            ctx.publish(record).await;
            // Start Charger
            let operation = StartCharger { device_id: self.device.device_id };
            self.device = ctx.operation(operation).await?.into_owned();
            let record = Started {
                note: format!("Charger restarted under control within {limit_s} s."),
            };
            ctx.publish(record).await;
            elapsed_s += limit_s;
            // Sample Charger
            let operation = SampleCharger {
                device_id: self.device.device_id,
                elapsed_s: elapsed_s,
            };
            let sample = ctx.operation(operation).await?.into_owned();
            self.device = sample.charger;
            let restored = sample.reading;
            let recovered = restored.current_rms_a > 0.0 && self.device.power == Power::Running;
            // Publish Measurement
            let row = measurement::row(self.published, restored, "recovered");
            ctx.stream::<Row>().into::<Samples>().push(row).await;
            self.published += 1;

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
        // Stop Charger
        let operation = StopCharger { device_id: self.device.device_id };
        self.device = ctx.operation(operation).await?.into_owned();
        // Publish Stopped
        let record = Stopped {
            accepted: self.recovered && self.cycles > 0 && self.device.fault.is_none(),
            cycles: self.cycles,
        };
        ctx.publish(record).await;
        Ok(())
    }

    #[dokime::criterion(
        id = "obc.charger.recovery.restart",
        label = "Power Loss Is Contained and Restart Succeeds",
        description = "Passed once the loss was contained and the controlled restart recovered."
    )]
    fn recovered(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
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
        let mut document = measurement::document(<Samples as dokime::StreamSpec>::ID, "Power Recovery");
        let section = system::section(TEMPLATE);
        document.section(section);
        document
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
