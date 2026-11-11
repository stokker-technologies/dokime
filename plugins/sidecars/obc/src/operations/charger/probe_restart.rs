/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Probe Restart

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::Chargers;
use crate::Error;
use crate::domain::model::charger::Restart;
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

#[dokime::operation(id = "obc.charger.probe_restart", response = Restart, policy = POLICY, resources = [Chargers])]
#[derive(Clone, PartialEq)]
pub struct ProbeChargerRestart {
    pub device_id: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ProbeChargerRestart {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Restart>, Refusal> {
        let value = dokime::plugin::refusal(ctx.resource::<Chargers>().probe_restart(self.device_id))?;
        Ok(Response::new(value))
    }
}

impl Chargers {
    pub fn probe_restart(&self, device_id: u64) -> Result<Restart, Error> {
        self.with(device_id, |device| match device.start() {
            Ok(()) => Ok(Restart { blocked: false }),
            Err(obc_runtime::Error::FaultLatched) => Ok(Restart { blocked: true }),
            Err(error) => Err(error.into()),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
