/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Start

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::Chargers;
use crate::Error;
use crate::domain::model::charger::Charger;
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

#[dokime::operation(id = "obc.charger.start", response = Charger, policy = POLICY, resources = [Chargers])]
#[derive(Clone, PartialEq)]
pub struct StartCharger {
    pub device_id: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for StartCharger {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Charger>, Refusal> {
        let value = dokime::plugin::refusal(ctx.resource::<Chargers>().start(self.device_id))?;
        Ok(Response::new(value))
    }
}

impl Chargers {
    pub fn start(&self, device_id: u64) -> Result<Charger, Error> {
        self.with(device_id, |device| {
            device.start()?;
            Ok(super::snapshot(device_id, device))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
