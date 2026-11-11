/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operations Wait

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;

/* === Constants === */

const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(1 << 32),
    attempt_timeout: Duration::from_secs(1 << 32),
    cancellation_timeout: Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "obc.wait", response = (), policy = POLICY)]
#[derive(Clone, PartialEq)]
pub struct Wait {
    pub duration_s: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Wait {
    type Error = Refusal;
    async fn run(self, _: Context<'_, Self>) -> Result<Response<()>, Refusal> {
        let duration = dokime::plugin::refusal(Duration::try_from_secs_f64(self.duration_s))?;
        tokio::time::sleep(duration).await;
        Ok(Response::new(()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
