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
use crate::domain::model::connections::Connections;
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
    id = "obc.bench.list_connections",
    response = Connections,
    description = "Reports the units this process has been told about. The connection listener accumulates them across executions; a template calls the operation without holding the plugin.",
    policy = POLICY,
    resources = [crate::Resources]
)]
#[derive(Clone, PartialEq)]
pub struct ListConnections {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ListConnections {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Connections>, Refusal> {
        Ok(Response::new(Connections {
            resources: ctx.resource::<Resources>().connected(),
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
