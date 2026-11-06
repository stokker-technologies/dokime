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
pub struct HeaterIdentity {
    pub serial: String,
    pub firmware: String,
}

#[dokime::operation(
    id = "{{crate}}.open_heater",
    response = HeaterIdentity,
    description = "Open the heater on the plugin and read its identity.",
    policy = POLICY, resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct OpenHeater {
    pub port: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for OpenHeater {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<HeaterIdentity>, Refusal> {
        let bench = ctx.resource::<Bench>();
        let mut current = bench.heater.lock().await;
        if let Some(heater) = current.as_ref() {
            dokime::plugin::refusal(heater.off().await)?;
        }
        let heater = dokime::plugin::refusal(crate::bench::Heater::open(self.port).await)?;
        let identity = dokime::plugin::refusal(heater.identity().await)?;
        *current = Some(heater);
        Ok(Response::new(HeaterIdentity {
            serial: identity.serial,
            firmware: identity.firmware,
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
