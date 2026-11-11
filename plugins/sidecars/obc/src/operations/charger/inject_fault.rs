/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Inject Fault

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::Chargers;
use crate::Error;
use crate::domain::model::charger::{Charger, Fault};
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

#[dokime::operation(id = "obc.charger.inject_fault", response = Charger, policy = POLICY, resources = [Chargers])]
#[derive(Clone, PartialEq)]
pub struct InjectChargerFault {
    pub device_id: u64,
    pub fault: Fault,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for InjectChargerFault {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Charger>, Refusal> {
        let value = dokime::plugin::refusal(ctx.resource::<Chargers>().inject_fault(self.device_id, self.fault))?;
        Ok(Response::new(value))
    }
}

impl Chargers {
    pub fn inject_fault(&self, device_id: u64, fault: Fault) -> Result<Charger, Error> {
        self.with(device_id, |device| {
            device.inject(match fault {
                Fault::Overcurrent => obc_runtime::device::Fault::Overcurrent,
                Fault::PowerLoss => obc_runtime::device::Fault::PowerLoss,
            });
            Ok(super::snapshot(device_id, device))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
