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
use obc_runtime::device::{Device, Fault, Power};

// Local Imports
use crate::{Resources, domain::model::device::DeviceState};
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
    id = "obc.device.read",
    visibility = Public,
    response = DeviceState,
    description = "Reads the manual simulated device without changing its power or advancing a charger execution.",
    policy = POLICY,
    resources = [crate::Resources],
    grpc(package = "obc.device.v1", service = "Device", method = "Read")
)]
#[derive(Clone, PartialEq)]
pub struct ReadDevice {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for ReadDevice {
    type Error = Refusal;

    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<DeviceState>, Refusal> {
        let device = dokime::plugin::refusal(ctx.resource::<Resources>().app().device().await)?;
        Ok(Response::new(observed(&device)))
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
    DeviceState {
        enabled,
        voltage_rms_v: if enabled { device.supply.voltage_rms_v } else { 0.0 },
        fault,
    }
}

/* === Tests === */
