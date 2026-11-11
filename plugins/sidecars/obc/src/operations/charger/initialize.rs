/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Initialize

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::Chargers;
use crate::Error;
use crate::domain::model::charger::{Charger, Supply};
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

#[dokime::operation(id = "obc.charger.initialize", response = Charger, policy = POLICY, resources = [Chargers])]
#[derive(Clone, PartialEq)]
pub struct InitializeCharger {
    pub supply: Supply,
    pub logging: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for InitializeCharger {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Charger>, Refusal> {
        let value = dokime::plugin::refusal(ctx.resource::<Chargers>().initialize(self.supply, self.logging))?;
        Ok(Response::new(value))
    }
}

impl Chargers {
    pub fn initialize(&self, supply: Supply, logging: bool) -> Result<Charger, Error> {
        let supply = obc_runtime::device::Supply {
            voltage_rms_v: supply.voltage_rms_v,
            load_ohm: supply.load_ohm,
            frequency_hz: supply.frequency_hz,
            phase_deg: supply.phase_deg,
            current_limit_a: supply.current_limit_a,
        };
        let mut device = obc_runtime::device::Device::initialize(supply)?;
        let device_id = self.next_id.fetch_add(1, super::Ordering::Relaxed);
        if logging {
            let started = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
            let path = std::env::temp_dir().join(format!("obc-charger-{}-{started}-{device_id}.log", std::process::id()));
            std::fs::File::create_new(&path).map_err(|error| Error::Observation(format!("charger log: {error}")))?;
            device.logging(path);
        }
        let state = super::snapshot(device_id, &device);
        self.devices.lock().map_err(|_| Error::DeviceResourcePoisoned)?.insert(device_id, device);
        Ok(state)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
