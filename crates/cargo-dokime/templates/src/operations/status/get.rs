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
    id = "{{crate}}.get_status",
    response = StatusResponse,
    description = "Report whether the bench is reachable.",
    policy = POLICY, resources = [crate::Bench]
)]
#[derive(Clone, PartialEq)]
pub struct GetStatus {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for GetStatus {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<StatusResponse>, Refusal> {
        let bench = ctx.resource::<Bench>();
        Ok(Response::new(
            StatusResponse::ready()
                .field("sensor", bench.sensor.describe())
                .field("units_connected", bench.connected().len() as i64)
                .field("upload_configured", bench.upload_configured),
        ))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
