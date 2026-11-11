/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Operations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::Resources;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::{Refusal, StatusResponse};

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
    id = "obc.bench.get_status",
    response = StatusResponse,
    description = "Reports bench readiness without advancing the Charger. The settings section selects this operation as its readiness probe.",
    policy = POLICY,
    resources = [crate::Resources]
)]
#[derive(Clone, PartialEq)]
pub struct GetStatus {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for GetStatus {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<StatusResponse>, Refusal> {
        let stopped = dokime::plugin::refusal(ctx.resource::<Resources>().app().stopped().await)?;
        Ok(Response::new(StatusResponse::new(if stopped { "disconnected" } else { "connected" })))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
