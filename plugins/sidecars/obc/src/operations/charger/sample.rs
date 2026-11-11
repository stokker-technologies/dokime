/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Sample

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::Chargers;
use crate::Error;
use crate::domain::model::charger::{Reading, Sample};
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;

/* === Constants === */

const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(2),
    attempt_timeout: Duration::from_secs(2),
    cancellation_timeout: Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "obc.charger.sample", response = Sample, policy = POLICY, resources = [Chargers])]
#[derive(Clone, PartialEq)]
pub struct SampleCharger {
    pub device_id: u64,
    pub elapsed_s: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for SampleCharger {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Sample>, Refusal> {
        let value = dokime::plugin::refusal(ctx.resource::<Chargers>().sample(self.device_id, self.elapsed_s))?;
        Ok(Response::new(value))
    }
}

impl Chargers {
    pub fn sample(&self, device_id: u64, elapsed_s: f64) -> Result<Sample, Error> {
        self.with(device_id, |device| {
            let reading = device.sample(elapsed_s);
            let reading = Reading {
                elapsed_s: reading.elapsed_s,
                voltage_v: reading.voltage_v,
                current_a: reading.current_a,
                voltage_rms_v: reading.voltage_rms_v,
                current_rms_a: reading.current_rms_a,
                frequency_hz: reading.frequency_hz,
                phase_deg: reading.phase_deg,
                input_power_w: reading.input_power_w,
                dc_voltage_v: reading.dc_voltage_v,
                dc_current_a: reading.dc_current_a,
            };
            Ok(Sample {
                reading,
                charger: super::snapshot(device_id, device),
            })
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
