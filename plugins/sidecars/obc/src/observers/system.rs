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
// Third-Party Imports
use crate::domain::model::settings::SYSTEM_INTERVAL_MS;
use crate::{Error, ObcPlugin};
use dokime::harmos::ExecutionContext;
use dokime::harmos::sidecar::{ObserverContext, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::stream(
    id = "obc.system",
    window = 4096,
    label = "Testbench Health",
    description = "Process and system CPU and memory observed by the OBC plugin."
)]
pub struct SystemRow {
    #[row(unit = "%")]
    process_cpu_percent: f64,

    #[row(unit = "B")]
    process_memory_bytes: i64,

    #[row(unit = "%")]
    system_cpu_percent: f64,

    #[row(unit = "B")]
    system_memory_used_bytes: i64,
}

pub(crate) struct System;

/* === Enums === */

#[dokime::catalog]
pub enum Streams {
    SystemRow(SystemRow),
}

/* === Traits === */

/* === Implementations === */

#[dokime::observer(interval_ms = SYSTEM_INTERVAL_MS, streams(SystemRow), activation = "demand")]
impl System {
    async fn initialize(_: &ExecutionContext) -> Result<Self, Error> {
        Ok(Self)
    }

    async fn observe(&mut self, ctx: &ObserverContext<'_, Publisher<ObcPlugin>>) -> Result<(), Error> {
        // Collect System Measurements
        let operation = crate::operations::system::CollectSystem {};
        let reading = ctx.operation(operation).await?;

        // Publish System Measurements
        let row = SystemRow {
            process_cpu_percent: reading.process_cpu_percent,
            process_memory_bytes: reading.process_memory_bytes,
            system_cpu_percent: reading.system_cpu_percent,
            system_memory_used_bytes: reading.system_memory_used_bytes,
        };
        ctx.publish(row).await?;

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
