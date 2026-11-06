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
pub struct TargetSet {}

#[dokime::operation(
    id = "{{crate}}.set_target",
    response = TargetSet,
    description = "Set the plugin-owned heater target.",
    policy = POLICY, resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct SetTarget {
    pub celsius: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for SetTarget {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<TargetSet>, Refusal> {
        let bench = ctx.resource::<Bench>();
        let current = bench.heater.lock().await;
        let heater = current.as_ref().ok_or_else(|| Refusal::new(-32_000, "the heater is not open"))?;
        dokime::plugin::refusal(heater.set_target(self.celsius).await)?;
        Ok(Response::new(TargetSet {}))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
