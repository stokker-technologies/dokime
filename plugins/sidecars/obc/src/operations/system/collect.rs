/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// System Collect

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Third Party Imports
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

// Local Imports
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

#[derive(Default)]
pub struct Collector {
    machine: Mutex<Option<System>>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct SystemReading {
    pub process_cpu_percent: f64,
    pub process_memory_bytes: i64,
    pub system_cpu_percent: f64,
    pub system_memory_used_bytes: i64,
}

#[dokime::operation(id = "obc.system.collect", response = SystemReading, policy = POLICY, resources = [Collector])]
#[derive(Clone, PartialEq)]
pub struct CollectSystem {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for CollectSystem {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<SystemReading>, Refusal> {
        let collector = ctx.resource::<Collector>();
        let row = collector.refresh().await?;
        Ok(Response::new(row))
    }
}

impl Collector {
    pub(crate) async fn refresh(self: &Arc<Self>) -> Result<SystemReading, Refusal> {
        let collector = Arc::clone(self);
        let task = tokio::task::spawn_blocking(move || collector.collect());
        dokime::plugin::refusal(task.await)?
    }

    fn collect(&self) -> Result<SystemReading, Refusal> {
        let mut state = dokime::plugin::refusal(self.machine.lock().map_err(|_| crate::Error::DeviceResourcePoisoned))?;
        if state.is_none() {
            *state = Some(dokime::plugin::refusal(System::new())?);
        }
        let machine = state.as_mut().expect("the collector initialized its machine");
        // Collect Only The Counters This Operation Returns.
        machine.refresh_cpu_usage();
        machine.refresh_memory();
        let pid = Pid::from_u32(std::process::id());
        machine.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, ProcessRefreshKind::nothing().with_cpu().with_memory());
        let process = machine.process(pid);
        let row = SystemReading {
            process_cpu_percent: process.map(|process| f64::from(process.cpu_usage())).unwrap_or_default(),
            process_memory_bytes: process.map(|process| integer(process.memory())).unwrap_or_default(),
            system_cpu_percent: f64::from(machine.global_cpu_usage()),
            system_memory_used_bytes: integer(machine.used_memory()),
        };
        Ok(row)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn integer(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/* === Tests === */
