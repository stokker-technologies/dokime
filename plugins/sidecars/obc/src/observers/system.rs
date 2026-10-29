/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use sysinfo::{Pid, System as Machine};

use crate::ObcPlugin;
use dokime::harmos::sidecar::{ObserverContext, PublishError, Publisher};

#[dokime::stream(
    id = "obc.system",
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

#[dokime::catalog]
pub enum Streams {
    SystemRow(SystemRow),
}

impl Default for System {
    fn default() -> Self {
        Self { machine: Machine::new() }
    }
}

#[dokime::observer(interval_ms = 1_000, streams(SystemRow), activation = "demand")]
impl System {
    async fn observe(&mut self, ctx: &ObserverContext<'_, Publisher<ObcPlugin>>) -> Result<(), PublishError> {
        self.machine.refresh_all();
        let system_cpu_percent = f64::from(self.machine.global_cpu_usage());
        let system_memory_used_bytes = integer(self.machine.used_memory());
        let process = self.machine.process(Pid::from_u32(std::process::id()));

        let row = SystemRow {
            process_cpu_percent: process.map(|process| f64::from(process.cpu_usage())).unwrap_or_default(),
            process_memory_bytes: process.map(|process| integer(process.memory())).unwrap_or_default(),
            system_cpu_percent,
            system_memory_used_bytes,
        };
        ctx.publish(row).await?;

        Ok(())
    }
}

fn integer(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
