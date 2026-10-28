/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Manual device operation implementations

use crate::{
    Resources,
    domain::{
        model::device::DeviceState,
        operations::{ReadDevice, SetPower},
    },
};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;
use obc_runtime::device::{Device, Fault, Power};

impl Operation for ReadDevice {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<DeviceState>, Refusal> {
        let device = dokime::plugin::refusal(ctx.resource::<Resources>().app().device().await)?;
        Ok(Response::new(observed(&device)))
    }
}

impl Operation for SetPower {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<DeviceState>, Refusal> {
        let resources = ctx.resource::<Resources>();
        dokime::plugin::refusal(resources.app().set_power(self.enabled).await)?;
        let device = dokime::plugin::refusal(resources.app().device().await)?;
        Ok(Response::new(observed(&device)))
    }
}

fn observed(device: &Device) -> DeviceState {
    let enabled = device.power == Power::Running;
    let fault = device.fault.map(|fault| match fault {
        Fault::Overcurrent => "overcurrent".to_owned(),
        Fault::PowerLoss => "power_loss".to_owned(),
    });
    DeviceState {
        enabled,
        voltage_rms_v: if enabled { device.supply.voltage_rms_v } else { 0.0 },
        fault,
    }
}
