/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::Bench;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::*;

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

// What `ReadChamber` answers.
#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct ChamberReading {
    #[row(description = "What the chamber is at, in °C.")]
    pub temperature_c: f64,

    #[row(
        description = "What read it, for the traceability a template states.\nA template never names this plugin, so anything it wants to say about the bench it reads off a reply: what an operation answers is the whole of what a run knows about the process serving it."
    )]
    pub sensor: String,
}

#[dokime::operation(
    id = "{{crate}}.get_chamber",
    response = ChamberReading,
    description = "Read the chamber through its plugin-owned sensor.",
    policy = POLICY, resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct ReadChamber {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ReadChamber {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<ChamberReading>, Refusal> {
        let bench = ctx.resource::<Bench>();
        let temperature_c = dokime::plugin::refusal(bench.sensor.read_celsius())?;
        Ok(Response::new(ChamberReading { temperature_c, sensor: bench.sensor.describe() }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
