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

#[dokime::operation(
    id = "{{crate}}.detect_port",
    response = Discovered,
    description = "Find the sensor port offered by settings.",
    policy = POLICY, resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct DetectPort {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for DetectPort {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Discovered>, Refusal> {
        Ok(Response::new(Discovered::of(ctx.resource::<Bench>().sensor.port().to_owned())))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
