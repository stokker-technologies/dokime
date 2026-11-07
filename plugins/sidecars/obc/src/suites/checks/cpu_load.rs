/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::Error;
use dokime::prelude::*;
use plugin_host::host::cpu::load::{LoadControl, LoadHold, LoadTarget};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Template;

#[dokime::parameters]
pub struct Parameters {
    #[parameter(label = "Target", unit = "%", default = 50.0, min = 0.01, max = 100.0)]
    pub target_percent: f64,

    #[parameter(label = "Hold Duration", unit = "s", default = 1.0, min = 0.001)]
    pub duration_s: f64,
}

/* === Enums === */

#[dokime::catalog]
pub enum Required {
    LoadHold(LoadHold),
    Wait(crate::operations::wait::Wait),
}

#[dokime::catalog]
pub enum Subscriptions {
    LoadControl(LoadControl),
}

/* === Traits === */

/* === Implementations === */

#[dokime::template(id = "obc.checks.cpu_load", label = "Host CPU Load", parameters = Parameters,
    requires = Required, subscribes = Subscriptions, tags = ["example", "host"],
    description = "Start a closed-loop host CPU workload, hold it for the configured duration and cancel its handle.")]
impl Template {
    async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }

    #[dokime::step(label = "Hold Host CPU Load")]
    async fn hold(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Start Host CPU Load
        let operation = LoadHold {
            target: LoadTarget::Percent(ctx.parameters().target_percent),
            threads: None,
        };
        let load = ctx.operation(operation).start().await?;

        // Hold the Workload
        let operation = crate::operations::wait::Wait { duration_s: ctx.parameters().duration_s };
        ctx.operation(operation).await?;

        // Stop and Join the Workload
        let summary = load.cancel().await?;
        ctx.info(format!("CPU mean: {} %, RMS error: {} %, duration: {} s", summary.mean, summary.error_rms, summary.duration_s));
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
