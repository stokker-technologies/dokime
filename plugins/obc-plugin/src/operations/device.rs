/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Manual device operation handlers

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::{
    Resources,
    domain::{
        model::device::DeviceState,
        operations::{ReadDevice, SetPower},
    },
};
use dokime_runtime::prelude::{Refusal, Supervisor};
use obc_runtime::device::{Device, Fault, Power};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct DeviceReader {
    resources: Arc<Resources>,
}

pub struct PowerControl {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::operation]
impl DeviceReader {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, _: &mut ReadDevice) -> Result<DeviceState, Refusal> {
        let device = dokime_runtime::plugin::refusal(self.resources.app().device().await)?;
        Ok(observed(&device))
    }
}

#[dokime_runtime::operation]
impl PowerControl {
    pub async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(&mut self, _: &Supervisor, request: &mut SetPower) -> Result<DeviceState, Refusal> {
        dokime_runtime::plugin::refusal(self.resources.app().set_power(request.enabled).await)?;
        let device = dokime_runtime::plugin::refusal(self.resources.app().device().await)?;
        Ok(observed(&device))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn observed(device: &Device) -> DeviceState {
    let enabled = device.power == Power::Running;
    let fault = device.fault.map(|fault| match fault {
        Fault::Overcurrent => "overcurrent".to_owned(),
        Fault::PowerLoss => "power_loss".to_owned(),
    });
    DeviceState { enabled, voltage_rms_v: if enabled { device.supply.voltage_rms_v } else { 0.0 }, fault }
}

/* === Tests === */
