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

/* === Macros === */

/* === Type Aliases === */
pub type InstalledSidecar = harmos::plugin::sidecar::Supervised<
    Dokime,
    harmos::plugin::sidecar::Executable,
    SidecarNoticeHandler,
>;

pub type SidecarNoticeHandler =
    Box<dyn Fn(&harmos::services::Service<Dokime>, harmos::plugin::sidecar::Notice) + Send + Sync>;

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
    pub page_rows: std::num::NonZeroUsize,
    pub page_bytes: std::num::NonZeroUsize,
    pub page_rows_ceiling: std::num::NonZeroUsize,
    pub page_bytes_ceiling: std::num::NonZeroUsize,
    pub decoded_bytes: std::num::NonZeroUsize,
    pub decoded_bytes_ceiling: std::num::NonZeroUsize,
    pub execution: harmos::storage::QueryLimits,
    pub execution_ceiling: harmos::storage::QueryLimits,
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
        registrar: harmos::plugin::sidecar::Registrar,
    ) -> crate::Result<Self> {
        let plugin = crate::capabilities::installed_plugins::contributions::decode_registrations(
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
        // Keep Each Read Bounded And Charge It To The Shared Budget.
        let execution_memory = std::num::NonZeroUsize::new(32 * 1024 * 1024).unwrap();
        Self {
            page_rows: settings.query_page.rows,
            page_bytes: settings.query_page.bytes,
            page_rows_ceiling: settings.query_page_ceiling.rows,
            page_bytes_ceiling: execution_memory,
            decoded_bytes: execution_memory,
            decoded_bytes_ceiling: execution_memory,
            execution: harmos::storage::QueryLimits {
                memory_bytes: execution_memory,
                ..settings.query_execution
            },
            execution_ceiling: harmos::storage::QueryLimits {
                memory_bytes: execution_memory,
                ..settings.query_execution_ceiling
            },
        }
    }
}

impl StorageReadSettings {
    pub(crate) fn validate(&self) -> crate::Result<()> {
        self.execution
            .check_ceiling(self.execution_ceiling)
            .map_err(|error| crate::Error::Operation(error.to_string()))?;
        if self.page_rows > self.page_rows_ceiling
            || self.page_bytes > self.page_bytes_ceiling
            || self.decoded_bytes > self.decoded_bytes_ceiling
        {
            return Err(crate::Error::Operation(
                "storage read defaults exceed configured ceilings".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn apply(&self, settings: &mut harmos::storage::StorageSettings) {
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
        settings.query_execution = self.execution;
        settings.query_execution_ceiling = self.execution_ceiling;
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

fn default_compaction_interval() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/* === Tests === */
