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

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct ChamberReleased {}

#[dokime::operation(
    id = "{{crate}}.release_chamber",
    response = ChamberReleased,
    description = "Turn off and release the plugin-owned heater.",
    policy = POLICY, resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct ReleaseChamber {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ReleaseChamber {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<ChamberReleased>, Refusal> {
        let bench = ctx.resource::<Bench>();
        let mut current = bench.heater.lock().await;
        if let Some(heater) = current.as_ref() {
            dokime::plugin::refusal(heater.off().await)?;
        }
        current.take();
        Ok(Response::new(ChamberReleased {}))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
