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
use std::{collections::BTreeMap, fmt, path::PathBuf, sync::Arc, time::Duration};
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::Dokime;
use crate::InstalledPlugin;
use dokime::core::{CredentialBackendKind, Metadata, SessionId};
use dokime::domains::ActionDispatcher;

/* === Constants === */
/// The computation budget one execution or cycle report finalization reads under.
/// Sealing reads a whole capture (every record, every telemetry page, every
/// document) and renders it, which on a slow debug build can take far longer than
/// any interactive query may. Finalization is unattended and already admitted,
/// so it gets minutes; exhausting even this is retried before it is final.
pub const FINALIZATION_BUDGET: Duration = Duration::from_secs(5 * 60);

/// One gibibyte: the smallest default read pool, and the memory the default
/// pool always leaves to the rest of the machine.
const GIB: u64 = 1024 * 1024 * 1024;

/// The physical memory assumed when the machine's cannot be detected.
const FALLBACK_PHYSICAL_MEMORY: u64 = 4 * GIB;

/// The smallest read pool, kept even on a machine too small for the 1 GiB floor.
const MINIMUM_READ_MEMORY: u64 = 256 * 1024 * 1024;

/// Live views and followers the read pool admits at once. Every one is
/// charged to the pool, so the pool, not the count, bounds their memory.
const DEFAULT_MAX_VIEWS: usize = 256;
const DEFAULT_MAX_FOLLOWERS: usize = 256;

/* === Macros === */

/* === Type Aliases === */
pub type InstalledSidecar =
    harmos::sidecar::Supervised<Dokime, harmos::sidecar::Executable, SidecarNoticeHandler>;

pub type SidecarNoticeHandler =
    Box<dyn Fn(&harmos::services::Service<Dokime>, harmos::sidecar::Notice) + Send + Sync>;

/* === Structs === */
#[derive(Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub session_id: SessionId,
    pub storage_root: PathBuf,
    pub application_root: PathBuf,
    #[serde(default = "default_save_interval")]
    pub save_interval: Duration,
    #[serde(default)]
    pub storage_reads: StorageReadSettings,
    #[serde(default = "default_compaction_interval")]
    pub compaction_interval: Duration,
    #[serde(default)]
    pub local_retention: Option<Duration>,
    pub hub_metadata: Metadata,
    pub credential_backend: RuntimeCredentialConfig,
    // A trusted bench is an isolated network where plain FTP sources are accepted.
    #[serde(default)]
    pub trusted_bench: bool,
    #[serde(skip)]
    pub installed_plugins: Vec<InstalledPlugin>,
    #[serde(default)]
    pub plugins: Vec<crate::plugins::PluginBundle>,
    #[serde(default)]
    pub plugin_lookup: Option<PathBuf>,
    #[serde(skip)]
    pub(crate) action_dispatchers: BTreeMap<dokime::core::Identifier, Arc<dyn ActionDispatcher>>,
    #[serde(skip)]
    pub(crate) embedded_providers: Vec<Arc<dyn crate::runtime::EmbeddedProvider>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageReadSettings {
    /// The storage read ledger (Harmos `read_memory_bytes`): by default a
    /// quarter of physical memory, at least 1 GiB, never more than physical
    /// memory less 1 GiB. Live readers, report captures and report production
    /// all reserve from it: captures retain their inputs, point reads and
    /// rendering reserve scratch.
    #[serde(default = "default_read_memory")]
    pub memory_bytes: std::num::NonZeroUsize,
    /// Read views the ledger admits at once (Harmos `max_views`).
    #[serde(default = "default_max_views")]
    pub max_views: std::num::NonZeroUsize,
    /// Live followers the ledger admits at once (Harmos `max_followers`).
    #[serde(default = "default_max_followers")]
    pub max_followers: std::num::NonZeroUsize,
    pub page_rows: std::num::NonZeroUsize,
    pub page_bytes: std::num::NonZeroUsize,
    pub page_rows_ceiling: std::num::NonZeroUsize,
    pub page_bytes_ceiling: std::num::NonZeroUsize,
    pub decoded_bytes: std::num::NonZeroUsize,
    pub decoded_bytes_ceiling: std::num::NonZeroUsize,
    /// The per-view budget of interactive reads: request receipts, cycle and
    /// execution lists, record pages, plot reads and every live follow. It is
    /// the Harmos default for any read that names no limits and is elastic: a
    /// view holds a floor and charges what it produces, so many fit the pool.
    pub interactive: harmos::storage::QueryLimits,
    /// The per-view budget of report reads: opening, capturing and
    /// materializing a report and archiving an execution, whose structured
    /// outputs exceed the interactive profile. Only these reads reserve it, and
    /// they reserve it whole at open so a report never fails halfway.
    pub report: harmos::storage::QueryLimits,
    /// The largest admissible budget of any read; both profiles and
    /// finalization's must fit it.
    pub ceiling: harmos::storage::QueryLimits,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCredentialConfig {
    pub backend: CredentialBackendKind,
    #[serde(default)]
    pub local_file: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeConfig {
    pub fn new(storage_root: impl Into<PathBuf>, hub_metadata: Metadata) -> Self {
        let storage_root = storage_root.into();
        Self {
            session_id: SessionId::new(),
            save_interval: default_save_interval(),
            storage_reads: StorageReadSettings::default(),
            compaction_interval: default_compaction_interval(),
            local_retention: None,
            // Programmatic configurations keep one disposable tree; shipped
            // shells always name a separate application root.
            application_root: storage_root.join("application"),
            hub_metadata,
            credential_backend: RuntimeCredentialConfig::default(),
            trusted_bench: false,
            installed_plugins: Vec::new(),
            plugins: Vec::new(),
            plugin_lookup: None,
            action_dispatchers: BTreeMap::new(),
            embedded_providers: Vec::new(),
            storage_root,
        }
    }

    pub fn register(mut self, provider: Arc<dyn crate::runtime::EmbeddedProvider>) -> Self {
        self.embedded_providers.push(provider);
        self
    }

    pub fn with_application_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.application_root = root.into();
        self
    }

    pub fn with_credential_backend(mut self, backend: RuntimeCredentialConfig) -> Self {
        self.credential_backend = backend;
        self
    }

    pub fn with_trusted_bench(mut self, trusted: bool) -> Self {
        self.trusted_bench = trusted;
        self
    }

    pub fn with_installed_plugins(
        mut self,
        plugins: impl IntoIterator<Item = InstalledPlugin>,
    ) -> Self {
        self.installed_plugins = plugins.into_iter().collect();
        self
    }

    pub fn with_declared_plugin(
        mut self,
        source: &str,
        version: &str,
        description: &str,
        registrar: harmos::sidecar::Registrar,
    ) -> crate::Result<Self> {
        let plugin = crate::domains::installed_plugins::contributions::decode_registrations(
            source,
            version,
            description,
            &registrar.finish(),
        )?;
        self.installed_plugins.push(plugin);
        Ok(self)
    }

    pub fn with_plugins(
        mut self,
        plugins: impl IntoIterator<Item = crate::plugins::PluginBundle>,
    ) -> Self {
        self.plugins = plugins.into_iter().collect();
        self
    }

    pub fn with_plugin_lookup(mut self, directory: impl Into<PathBuf>) -> Self {
        self.plugin_lookup = Some(directory.into());
        self
    }

    pub(crate) fn paths(&self) -> crate::config::ApplicationPaths {
        crate::config::ApplicationPaths::new(&self.application_root)
    }

    pub fn with_action_dispatcher(
        mut self,
        plugin_id: dokime::core::Identifier,
        dispatcher: Arc<dyn ActionDispatcher>,
    ) -> Self {
        self.action_dispatchers.insert(plugin_id, dispatcher);
        self
    }
}

impl Default for StorageReadSettings {
    fn default() -> Self {
        let settings = harmos::storage::StorageSettings::default();
        // Structured Outputs And Their Frozen Reports Exceed The Generic 4 MiB Profile.
        // Only Report Reads Reserve The Larger Profile; Each View Holds Its
        // Reservation For Its Whole Life, So Interactive Reads Stay Small.
        let report_memory = std::num::NonZeroUsize::new(32 * 1024 * 1024).unwrap();
        Self {
            memory_bytes: default_read_memory(),
            max_views: default_max_views(),
            max_followers: default_max_followers(),
            page_rows: settings.query_page.rows,
            page_bytes: settings.query_page.bytes,
            page_rows_ceiling: settings.query_page_ceiling.rows,
            page_bytes_ceiling: report_memory,
            decoded_bytes: report_memory,
            decoded_bytes_ceiling: report_memory,
            // Interactive Reads Hold A Floor And Charge What They Produce;
            // Report Reads Reserve Their Whole Profile So They Never Fail Halfway.
            interactive: settings.query_execution.elastic(),
            report: harmos::storage::QueryLimits {
                memory_bytes: report_memory,
                ..settings.query_execution
            }
            .with_admission(harmos::storage::Admission::Guaranteed),
            // The admissible ceiling widens to the report profile and to
            // finalization's time budget.
            ceiling: harmos::storage::QueryLimits {
                memory_bytes: report_memory,
                timeout: settings
                    .query_execution_ceiling
                    .timeout
                    .max(FINALIZATION_BUDGET),
                ..settings.query_execution_ceiling
            },
            finalization_read_stall: Duration::ZERO,
            replay_read_deadlines: 0,
        }
    }
}

impl StorageReadSettings {
    /// The limits report finalization reads under: the report profile with
    /// the finalization budget instead of the interactive deadline.
    pub fn finalization(&self) -> harmos::storage::QueryLimits {
        harmos::storage::QueryLimits {
            timeout: FINALIZATION_BUDGET,
            ..self.report
        }
    }

    /// The memory one report materialization may fill: the report profile,
    /// reserved whole for the report rather than whatever the ledger has left.
    pub fn materialization(&self) -> std::num::NonZeroUsize {
        self.report.memory_bytes
    }

    pub(crate) fn validate(&self) -> crate::Result<()> {
        for profile in [self.interactive, self.report, self.finalization()] {
            profile
                .check_ceiling(self.ceiling)
                .map_err(|error| crate::Error::Operation(error.to_string()))?;
        }
        if self.page_rows > self.page_rows_ceiling
            || self.page_bytes > self.page_bytes_ceiling
            || self.decoded_bytes > self.decoded_bytes_ceiling
            || self.materialization() > self.decoded_bytes
        {
            return Err(crate::Error::Operation(
                "storage read defaults exceed configured ceilings".into(),
            ));
        }
        // Every Per-Kind Declaration Must Leave The Pool Room For Others.
        let declared = [
            self.interactive.memory_bytes,
            self.report.memory_bytes,
            self.ceiling.memory_bytes,
            self.page_bytes_ceiling,
            self.decoded_bytes_ceiling,
        ];
        if declared.iter().any(|bytes| *bytes >= self.memory_bytes) {
            return Err(crate::Error::Operation(
                "a storage read declaration does not fit below the read pool".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn apply(&self, settings: &mut harmos::storage::StorageSettings) {
        settings.read_memory_bytes = self.memory_bytes;
        settings.max_views = self.max_views;
        settings.max_followers = self.max_followers;
        settings.query_page = harmos::storage::ReadLimits {
            rows: self.page_rows,
            bytes: self.page_bytes,
        };
        settings.query_page_ceiling = harmos::storage::ReadLimits {
            rows: self.page_rows_ceiling,
            bytes: self.page_bytes_ceiling,
        };
        settings.query_decoded_bytes = self.decoded_bytes;
        settings.query_decoded_bytes_ceiling = self.decoded_bytes_ceiling;
        // Reads That Name No Limits Are Interactive.
        settings.query_execution = self.interactive;
        settings.query_execution_ceiling = self.ceiling;
    }
}

impl fmt::Debug for RuntimeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeConfig")
            .field("embedded_providers", &self.embedded_providers.len())
            .field("session_id", &self.session_id)
            .field("storage_root", &self.storage_root)
            .field("storage_reads", &self.storage_reads)
            .field("application_root", &self.application_root)
            .field("hub_metadata", &self.hub_metadata)
            .field("credential_backend", &self.credential_backend)
            .field("installed_plugins", &self.installed_plugins)
            .field("plugins", &self.plugins)
            .field(
                "action_dispatchers",
                &self.action_dispatchers.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Default for RuntimeCredentialConfig {
    fn default() -> Self {
        Self {
            backend: CredentialBackendKind::OsKeyring,
            local_file: None,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn default_save_interval() -> Duration {
    Duration::from_millis(super::policy::DEFAULT_SAVE_INTERVAL_MS)
}

fn default_read_memory() -> std::num::NonZeroUsize {
    read_pool_for(physical_memory())
}

fn default_max_views() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(DEFAULT_MAX_VIEWS).unwrap()
}

fn default_max_followers() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(DEFAULT_MAX_FOLLOWERS).unwrap()
}

/// The machine's physical memory in bytes, or 4 GiB when it cannot be read.
fn physical_memory() -> u64 {
    let system = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
    );
    match system.total_memory() {
        0 => FALLBACK_PHYSICAL_MEMORY,
        bytes => bytes,
    }
}

/// The default read pool for `physical` bytes of memory: a quarter of it, at
/// least 1 GiB, at most all but 1 GiB of it, and never below 256 MiB.
fn read_pool_for(physical: u64) -> std::num::NonZeroUsize {
    let pool = (physical / 4)
        .max(GIB)
        .min(physical.saturating_sub(GIB))
        .max(MINIMUM_READ_MEMORY);
    std::num::NonZeroUsize::new(usize::try_from(pool).unwrap_or(usize::MAX))
        .expect("the read pool floor is nonzero")
}

fn default_compaction_interval() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(mebibytes: usize) -> std::num::NonZeroUsize {
        std::num::NonZeroUsize::new(mebibytes * 1024 * 1024).unwrap()
    }

    #[test]
    fn apply_sets_each_configured_read_limit() {
        let mut reads = StorageReadSettings {
            memory_bytes: bytes(300),
            page_bytes: bytes(3),
            decoded_bytes: bytes(5),
            ..StorageReadSettings::default()
        };
        reads.interactive.memory_bytes = bytes(7);
        reads.report.memory_bytes = bytes(5);
        reads.validate().unwrap();
        let mut settings = harmos::storage::StorageSettings::default();

        reads.apply(&mut settings);

        assert_eq!(settings.read_memory_bytes, bytes(300));
        assert_eq!(settings.query_page.rows, reads.page_rows);
        assert_eq!(settings.query_page.bytes, bytes(3));
        assert_eq!(settings.query_page_ceiling.rows, reads.page_rows_ceiling);
        assert_eq!(settings.query_page_ceiling.bytes, reads.page_bytes_ceiling);
        assert_eq!(settings.query_decoded_bytes, bytes(5));
        assert_eq!(
            settings.query_decoded_bytes_ceiling,
            reads.decoded_bytes_ceiling
        );
        assert_eq!(settings.query_execution.memory_bytes, bytes(7));
        assert_eq!(
            settings.query_execution_ceiling.timeout,
            reads.ceiling.timeout
        );
    }

    #[test]
    fn only_report_reads_reserve_the_report_profile() {
        let reads = StorageReadSettings::default();
        reads.validate().unwrap();

        assert_eq!(reads.interactive.memory_bytes, bytes(4));
        assert_eq!(
            reads.interactive.admission,
            harmos::storage::Admission::Elastic
        );
        assert_eq!(
            reads.report.admission,
            harmos::storage::Admission::Guaranteed
        );
        assert_eq!(
            reads.finalization().admission,
            harmos::storage::Admission::Guaranteed
        );
        assert_eq!(reads.report.memory_bytes, bytes(32));
        assert_eq!(reads.finalization().memory_bytes, bytes(32));
        assert_eq!(reads.finalization().timeout, FINALIZATION_BUDGET);
        assert_eq!(reads.materialization(), bytes(32));
        assert_eq!(reads.interactive.timeout, reads.report.timeout);
    }

    #[test]
    fn read_pool_is_a_quarter_of_memory_within_its_clamps() {
        let gib = |count: u64| count * GIB;
        let pool = |physical: u64| read_pool_for(physical).get() as u64;

        assert_eq!(pool(gib(16)), gib(4));
        assert_eq!(pool(gib(64)), gib(16));
        assert_eq!(pool(gib(4)), gib(1), "the 1 GiB floor");
        assert_eq!(
            pool(gib(3) / 2),
            gib(1) / 2,
            "all but 1 GiB of a small machine"
        );
        assert_eq!(pool(gib(1)), MINIMUM_READ_MEMORY, "the 256 MiB floor");
        assert_eq!(pool(FALLBACK_PHYSICAL_MEMORY), gib(1));
    }

    #[test]
    fn apply_admits_many_views_and_followers() {
        let reads = StorageReadSettings::default();
        let mut settings = harmos::storage::StorageSettings::default();

        reads.apply(&mut settings);

        assert_eq!(settings.max_views.get(), DEFAULT_MAX_VIEWS);
        assert_eq!(settings.max_followers.get(), DEFAULT_MAX_FOLLOWERS);
    }

    #[test]
    fn validate_refuses_a_declaration_that_fills_the_pool() {
        let reads = StorageReadSettings {
            memory_bytes: bytes(32),
            ..StorageReadSettings::default()
        };

        assert!(reads.validate().is_err());
    }

    #[test]
    fn validate_refuses_a_profile_above_the_ceiling() {
        let mut reads = StorageReadSettings::default();
        reads.report.memory_bytes = bytes(64);

        assert!(reads.validate().is_err());
    }
}
