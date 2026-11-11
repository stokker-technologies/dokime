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
use crate::domain::model::readiness::Readiness;
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

#[dokime::operation(
    id = "obc.bench.check_readiness",
    response = Readiness,
    description = "Reports whether the testbench will still take work. A template calls it with `AskReadiness {}`.",
    policy = POLICY,
    resources = [crate::Resources]
)]
#[derive(Clone, PartialEq)]
pub struct AskReadiness {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for AskReadiness {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Readiness>, Refusal> {
        let stopped = dokime::plugin::refusal(ctx.resource::<Resources>().app().stopped().await)?;
        Ok(Response::new(Readiness { stopped }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
