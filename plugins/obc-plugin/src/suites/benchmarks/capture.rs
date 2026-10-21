/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Generated Telemetry And Two Operator Checkpoints

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::suites::common::measurement::{self, InputMeasurement};
use crate::{error::Error, prompts::checkpoint::SteadyCheck};
use dokime::prelude::*;
use obc_runtime::device::{Device, Supply};

/* === Constants === */
const STREAM: &str = "obc.benchmarks.samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Capture;

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "Rows", default = 100_000, min = 1, max = 200_000)]
    pub rows: u64,

    #[parameter(label = "Member", default = 0, expansion = "cartesian")]
    pub member: u64,
}

#[dokime::stream(id = "obc.benchmarks.samples", row = InputMeasurement, window = 4096, label = "Benchmark Samples")]
pub struct Samples;

/* === Enums === */
#[dokime::catalog]
pub enum Streams {
    Samples(Samples),
}

#[dokime::catalog]
pub enum Prompts {
    Check(SteadyCheck),
}

/* === Traits === */

/* === Implementations === */
#[dokime::template(
    id = "obc.benchmarks.capture", label = "High-Volume Capture", parameters = Parameters,
    streams = Streams, prompts = Prompts,
    description = "Capture generated measurements, then answer two checkpoints while the execution remains active."
)]
impl Capture {
    async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    fn document() -> Document {
        measurement::document(STREAM, "Benchmark Capture")
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
        let mut device = Device::initialize(supply)?;
        device.start()?;
        let rows = ctx.parameters().rows;
        for index in 0..rows {
            let reading = device.sample(index as f64 * 0.0001);
            measurement::publish::<Samples, _, _, _>(ctx, index, reading, "capture").await;
        }
        device.stop();
        Ok(())
    }

    #[dokime::step(label = "First Operator Checkpoint")]
    async fn first(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        accept(ctx).await
    }

    #[dokime::step(label = "Second Operator Checkpoint")]
    async fn second(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        accept(ctx).await
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn accept(ctx: &mut StepContext<'_, Capture, (), Parameters>) -> Result<(), Error> {
    let response = ctx.ask(SteadyCheck::default()).await?;
    if !response.accepted {
        return Err(Error::ConfirmationDeclined);
    }
    Ok(())
}

/* === Tests === */
