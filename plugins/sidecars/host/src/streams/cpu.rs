/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CPU Load Operations and the Load Control Stream Their Holds Publish

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use tokio::sync::broadcast::error::RecvError;

// Local Imports
use crate::Host;
use crate::collectors::cpu::Load;
use crate::collectors::cpu::hold::{self, LoadControl, LoadHold, LoadSummary};
use crate::collectors::cpu::sampling::Sampler;
use crate::settings::{self, CpuLoad};
use dokime::operations::{Context, Operation};
use dokime::plugin::{Plugin, Publisher, Stream};
use dokime::{Result, Visibility};

/* === Constants === */

pub const LOAD: &str = "host.cpu.load";
pub const LOAD_HOLD: &str = "host.cpu.load.hold";

/// A Load Read May Calibrate First, Which Takes at Most the One-Second Calibration Window.
const LOAD_BUDGET: Duration = Duration::from_secs(2);

/// A Hold Runs Until Cancelled.
const HOLD_BUDGET: Duration = Duration::from_secs(1 << 32);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Read the Calibrated CPU Capacity and Current Utilization.
#[derive(dokime::Message)]
#[dokime(package = "host")]
pub struct LoadRead {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Load Control Stream and the Public CPU Load Operations.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let stream = Stream::<LoadControl>::new(LoadControl::STREAM, LoadControl::LABEL)
        .description("Measured Closed-Loop CPU Load and Generator Throughput per Controller Period.")
        .publish_on(publish);
    let load = Operation::new(LOAD, Visibility::Public, read)
        .policy(settings::policy(LOAD_BUDGET, Duration::ZERO))
        .description("Report the Calibrated CPU Capacity, Current Utilization and Their Product.");
    let load_hold = Operation::new(LOAD_HOLD, Visibility::Public, load_hold)
        .policy(settings::policy(HOLD_BUDGET, settings::CPU_LOAD_CANCELLATION_GRACE))
        .description("Hold a CPU Utilization or Throughput Target Until Cancelled, Publishing Each Controller Period.");
    Ok(plugin.stream(stream).operation(load).operation(load_hold))
}

/* === Private Functions === */

async fn read(ctx: &Context, _: LoadRead) -> Result<Load> {
    ctx.plugin::<Host>()?.cpu.load(ctx).await
}

async fn load_hold(ctx: &Context, request: LoadHold) -> Result<LoadSummary> {
    let cpu = &ctx.plugin::<Host>()?.cpu;
    let settings = CpuLoad::read(ctx).await?;
    let capacity_gflops = cpu.capacity_gflops(ctx).await?;
    let period = settings.sample_period;
    let sampler = tokio::task::spawn_blocking(move || Sampler::start(capacity_gflops, period))
        .await
        .map_err(|error| dokime::Error::Invalid(format!("Hold CPU Load: {error}; Restart the Plugin")))??;
    let source = sampler.source();
    let cancel = async {
        ctx.cancelled().await;
    };
    hold::hold(request, settings, source, cancel, |row| {
        cpu.publish(row);
        std::future::ready(Ok(()))
    })
    .await
}

/// Forward Every Hold's Controller Rows While the Stream Has Demand.
async fn publish(ctx: &dokime::templates::Context, publisher: Publisher<LoadControl>) -> Result<()> {
    let mut rows = ctx.plugin::<Host>()?.cpu.subscribe();
    loop {
        match rows.recv().await {
            Ok(row) => publisher.publish(row).await?,
            Err(RecvError::Lagged(_)) => continue,
            Err(RecvError::Closed) => return Ok(()),
        }
    }
}


/* === Tests === */
