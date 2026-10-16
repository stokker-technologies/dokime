/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Dokime Storage Settings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    num::{NonZeroU64, NonZeroUsize},
    time::Duration,
};

// Third Party Imports
use harmos::storage::{StorageBudget, StorageRequestSettings, StorageResponseSettings};
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageSettings {
    pub budget: StorageBudget,
    pub request: StorageRequestSettings,
    pub response: StorageResponseSettings,
    pub page_rows: std::num::NonZeroUsize,
    pub page_bytes: std::num::NonZeroUsize,
    pub page_rows_ceiling: std::num::NonZeroUsize,
    pub page_bytes_ceiling: std::num::NonZeroUsize,
    pub decoded_bytes: std::num::NonZeroUsize,
    /// The per-view budget of interactive reads: request receipts, cycle and
    /// execution lists, record pages, plot reads and every live follow. It is
    /// the Harmos default for any read that names no limits and is elastic: a
    /// view holds a floor and charges what it produces, so many fit the pool.
    pub interactive: harmos::storage::QueryLimits,
    /// Test hook: how long each native-output read finalization performs
    /// stalls within its read budget. Zero outside tests.
    #[doc(hidden)]
    #[serde(skip)]
    pub finalization_read_stall: Duration,
    /// Test hook: how many page reads of each sample replay fail with the
    /// storage backend deadline before reading. Zero outside tests.
    #[doc(hidden)]
    #[serde(skip)]
    pub replay_read_deadlines: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for StorageSettings {
    fn default() -> Self {
        let memory = effective_memory();
        let read = nonzero(memory / 4);
        let writes = nonzero(memory / 20);
        let batch = nonzero((8 * 1024 * 1024).min(read.get() / 4).max(1));
        let active = std::thread::available_parallelism()
            .map_or(4, NonZeroUsize::get)
            .clamp(4, 16);
        // Query conversion coexists with the native page and encoded response.
        let scan = nonzero(batch.get().min(read.get() / active / 32).max(1));
        let execution = harmos::storage::QueryLimits {
            memory_bytes: read,
            admission: harmos::storage::Admission::Elastic,
            spill_bytes: 0,
            timeout: Duration::from_secs(30),
        };
        Self {
            budget: StorageBudget {
                read_memory_bytes: read,
            },
            request: StorageRequestSettings {
                max_pending_queries: nonzero(128),
                max_active_queries: nonzero(active),
                max_pending_write_batches: nonzero(256),
                max_pending_write_bytes: writes,
                reserved_control_batches: 256 / 20,
                reserved_control_bytes: writes.get() / 20,
            },
            response: StorageResponseSettings {
                batch_rows: nonzero(65_536),
                batch_bytes: batch,
            },
            page_rows: nonzero(4096),
            page_bytes: scan,
            page_rows_ceiling: nonzero(65_536),
            page_bytes_ceiling: read,
            decoded_bytes: batch,
            interactive: execution,
            finalization_read_stall: Duration::ZERO,
            replay_read_deadlines: 0,
        }
    }
}

impl StorageSettings {
    /// All Native Reads Share The Same Memory Policy, Including Reports.
    pub fn finalization(&self) -> harmos::storage::QueryLimits {
        self.interactive
    }

    pub fn materialization(&self) -> NonZeroUsize {
        self.budget.read_memory_bytes
    }

    pub(crate) fn validate(&self) -> crate::Result<()> {
        if self.page_rows > self.page_rows_ceiling
            || self.page_bytes > self.page_bytes_ceiling
            || self.decoded_bytes > self.budget.read_memory_bytes
            || self.interactive.memory_bytes > self.budget.read_memory_bytes
        {
            return Err(crate::Error::Operation(
                "storage read limits exceed shared capacity".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn apply(&self, settings: &mut harmos::storage::StorageSettings) {
        settings.budget = self.budget;
        settings.request = self.request;
        settings.response = self.response;
        settings.query_page = harmos::storage::ReadLimits {
            rows: self.page_rows,
            bytes: self.page_bytes,
        };
        settings.query_page_ceiling = harmos::storage::ReadLimits {
            rows: self.page_rows_ceiling,
            bytes: self.page_bytes_ceiling,
        };
        settings.query_decoded_bytes = self.decoded_bytes;
        settings.query_execution = self.interactive;
        settings.query_execution_ceiling = harmos::storage::QueryLimits {
            memory_bytes: self.budget.read_memory_bytes,
            ..self.interactive
        };
    }

    /// Explicit Product Policy For Every Harmos Storage Setting.
    pub fn harmos(
        &self,
        flush_interval: Duration,
        maximum_batch: NonZeroUsize,
    ) -> harmos::storage::StorageSettings {
        let maximum_batch = maximum_batch.min(self.request.max_pending_write_bytes);
        let mut settings = harmos::storage::StorageSettings {
            budget: self.budget,
            request: self.request,
            response: self.response,
            target_batch_bytes: nonzero(128 * 1024).min(maximum_batch),
            max_batch_bytes: maximum_batch,
            query_page: harmos::storage::ReadLimits {
                rows: self.page_rows,
                bytes: self.page_bytes,
            },
            query_page_ceiling: harmos::storage::ReadLimits {
                rows: self.page_rows_ceiling,
                bytes: self.page_bytes_ceiling,
            },
            query_decoded_bytes: self.decoded_bytes,
            query_execution: self.interactive,
            query_execution_ceiling: harmos::storage::QueryLimits {
                memory_bytes: self.budget.read_memory_bytes,
                ..self.interactive
            },
            replication_limits: harmos::storage::ReadLimits {
                rows: nonzero(64),
                bytes: nonzero(64 * 1024),
            },
            replica_lag_positions: NonZeroU64::new(4096).unwrap(),
            replication_shutdown: Duration::ZERO,
            max_files: nonzero(256),
            max_mutations: nonzero(1024),
            mutation_memory_bytes: nonzero(1024 * 1024),
            mutation_retention: Duration::from_secs(300),
            file_session_timeout: Duration::from_secs(60),
            view_idle_timeout: Duration::from_secs(300),
            snapshot_bytes: nonzero(1024 * 1024),
            write_batch_entries: nonzero(4096),
            object_prune_entries: nonzero(256),
            write_batch_bytes: maximum_batch,
            flush_interval,
            io_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_base: Duration::from_millis(10),
            retry_cap: Duration::from_secs(1),
        };
        self.apply(&mut settings);
        settings
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value.max(1)).unwrap()
}

fn effective_memory() -> usize {
    let system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
    );
    let physical = match system.total_memory() {
        0 => 4 * 1024 * 1024 * 1024,
        bytes => bytes,
    };
    let effective = system
        .cgroup_limits()
        .map(|limits| limits.total_memory)
        .filter(|bytes| *bytes > 0)
        .map_or(physical, |bytes| physical.min(bytes));
    usize::try_from(effective).unwrap_or(usize::MAX)
}

/* === Tests === */
