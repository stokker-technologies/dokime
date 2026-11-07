/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Release

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::Chargers;
use crate::Error;
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

#[dokime::operation(id = "obc.charger.release", response = (), policy = POLICY, resources = [Chargers])]
#[derive(Clone, PartialEq)]
pub struct ReleaseCharger {
    pub device_id: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ReleaseCharger {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<()>, Refusal> {
        let value = dokime::plugin::refusal(ctx.resource::<Chargers>().release(self.device_id))?;
        Ok(Response::new(value))
    }
}

impl Chargers {
    pub fn release(&self, device_id: u64) -> Result<(), Error> {
        let Some(mut device) = self.devices.lock().map_err(|_| Error::DeviceResourcePoisoned)?.remove(&device_id) else {
            return Ok(());
        };
        device.stop();
        if let Some(path) = device.log {
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(Error::Observation(format!("charger log cleanup: {error}"))),
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
