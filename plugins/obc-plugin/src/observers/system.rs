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
use sysinfo::{Pid, System as Machine};
// Local Imports
// Third-Party Imports
use crate::{ObcPlugin, Resources};
use dokime_runtime::harmos::sidecar::{ObserverContext, PublishError, Publisher};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::stream(
    id = "obc-plugin.system",
    window = 4096,
    label = "Testbench Health",
    description = "Process and system CPU and memory observed by the OBC plugin."
)]
pub struct SystemRow {
    process_cpu_percent: f64,
    process_memory_bytes: i64,
    system_cpu_percent: f64,
    system_memory_used_bytes: i64,
}

pub(crate) struct System {
    machine: Machine,
}

impl Default for System {
    fn default() -> Self {
        Self {
            machine: Machine::new(),
        }
    }
}

/* === Enums === */
#[dokime_runtime::catalog]
pub enum Streams {
    SystemRow(SystemRow),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::observer(interval_ms = 1_000, streams(SystemRow), activation = "demand")]
impl System {
    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Resources, Publisher<ObcPlugin>>,
    ) -> Result<(), PublishError> {
        self.machine.refresh_all();
        let system_cpu_percent = f64::from(self.machine.global_cpu_usage());
        let system_memory_used_bytes = integer(self.machine.used_memory());
        let process = self.machine.process(Pid::from_u32(std::process::id()));

        let row = SystemRow {
            process_cpu_percent: process
                .map(|process| f64::from(process.cpu_usage()))
                .unwrap_or_default(),
            process_memory_bytes: process
                .map(|process| integer(process.memory()))
                .unwrap_or_default(),
            system_cpu_percent,
            system_memory_used_bytes,
        };
        ctx.publish(row).await?;

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn integer(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/* === Tests === */
