/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generated Telemetry and Two Operator Checkpoints

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::domain::model::charger::{Charger, Supply};
use crate::domain::model::measurement::{self, Row};
use crate::domain::{Error, model::prompts::checkpoint::SteadyCheck};
use crate::operations::charger::*;
use dokime::prelude::*;

/* === Constants === */

const STREAM: &str = "obc.benchmarks.samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Template {
    device: Option<Charger>,
}

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "Rows", default = 100_000, min = 1, max = 200_000)]
    pub rows: u64,

    #[parameter(label = "Member", default = 0, expansion = "cartesian")]
    pub member: u64,
}

#[dokime::stream(id = "obc.benchmarks.samples", row = Row, window = 4096, label = "Benchmark Samples")]
pub struct Samples;

/* === Enums === */

#[dokime::catalog]
pub enum Required {
    InitializeCharger(InitializeCharger),
    StartCharger(StartCharger),
    SampleChargerBatch(SampleChargerBatch),
    StopCharger(StopCharger),
    ReleaseCharger(ReleaseCharger),
}
#[dokime::catalog]
pub enum Streams {
    Samples(Samples),
}

#[dokime::catalog]
pub enum Prompts {
    SteadyCheck(SteadyCheck),
}

/* === Traits === */

/* === Implementations === */

#[dokime::template(
    id = "obc.benchmarks.capture", label = "High-Volume Capture", parameters = Parameters,
    streams = Streams, prompts = Prompts, requires = Required,
    description = "Capture generated measurements, then answer two checkpoints while the execution remains active."
)]
impl Template {
    async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self { device: None })
    }

    #[dokime::step(label = "Capture Generated Measurements")]
    async fn capture(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let supply = Supply {
            voltage_rms_v: 230.0 + ctx.parameters().member as f64,
            load_ohm: 100.0,
            frequency_hz: 50.0,
            phase_deg: 15.0,
            current_limit_a: 5.0,
        };
        // Initialize Charger
        let operation = InitializeCharger { supply, logging: false };
        let device = ctx.operation(operation).await?.into_owned();
        let device_id = device.device_id;
        self.device = Some(device);

        // Start Charger
        let operation = StartCharger { device_id };
        ctx.operation(operation).await?;
        let rows = ctx.parameters().rows;
        for first_index in (0..rows).step_by(SampleChargerBatch::MAX_ROWS as usize) {
            // Acquire a Batch of Measurements
            let operation = SampleChargerBatch {
                device_id,
                first_index,
                count: (rows - first_index).min(SampleChargerBatch::MAX_ROWS),
                interval_s: 0.0001,
            };
            let batch = ctx.operation(operation).await?;

            // Publish Each Captured Measurement
            for (offset, sample) in batch.rows.iter().enumerate() {
                let index = first_index + offset as u64;
                let row = measurement::row(index, sample.reading, "capture");
                ctx.stream::<Row>().into::<Samples>().push(row).await;
            }
        }
        // Stop Charger
        let operation = StopCharger { device_id };
        ctx.operation(operation).await?;
        Ok(())
    }

    #[dokime::step(label = "First Operator Checkpoint")]
    async fn first(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Ask for Operator Acceptance
        let prompt = SteadyCheck::default();
        let response = ctx.ask(prompt).await?;
        if !response.accepted {
            return Err(Error::ConfirmationDeclined);
        }
        Ok(())
    }

    #[dokime::step(label = "Second Operator Checkpoint")]
    async fn second(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Ask for Operator Acceptance
        let prompt = SteadyCheck::default();
        let response = ctx.ask(prompt).await?;
        if !response.accepted {
            return Err(Error::ConfirmationDeclined);
        }
        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        let Some(device) = &self.device else {
            return Ok(());
        };
        // Release Charger
        let operation = ReleaseCharger { device_id: device.device_id };
        let result = ctx.operation(operation).await;
        if let Err(error) = &result {
            ctx.warn(format!("charger cleanup: {error}"));
        }
        result?;
        Ok(())
    }

    fn document() -> Document {
        measurement::document(STREAM, "Benchmark Capture")
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
