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
use std::{collections::BTreeMap, fmt, path::PathBuf, sync::Arc};

// Third Party Imports
use harmos::storage::StorageSettings;

// Local Imports
use crate::Dokime;
use crate::InstalledPlugin;
use crate::domains::ActionDispatcher;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::SessionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type InstalledSidecar = harmos::sidecar::Supervised<Dokime, harmos::sidecar::Executable, SidecarNoticeHandler>;
pub type SidecarNoticeHandler = Box<dyn Fn(&harmos::ServiceContext<Dokime>, harmos::sidecar::Notice) + Send + Sync>;

/* === Structs === */

#[derive(Clone)]
pub struct RuntimeConfig {
    pub schema: harmos::config::SchemaSettings,
    pub session_id: SessionId,
    pub storage_root: PathBuf,
    pub application_root: PathBuf,

    pub storage: StorageSettings,

    pub local_storage: crate::storage::sqlite::Settings,

    pub managed_storage: harmos::config::ManagedStorageSettings,

    pub storage_state: harmos::config::StorageStateSettings,

    pub hub_metadata: Metadata,

    #[cfg(feature = "host")]
    pub(crate) credential_store: Option<Arc<keyring_core::CredentialStore>>,

    // A trusted bench is an isolated network where plain FTP sources are accepted.
    pub trusted_bench: bool,

    pub installed_plugins: Vec<InstalledPlugin>,

    pub plugins: Vec<crate::plugins::PluginBundle>,

    pub plugin_lookup: Option<PathBuf>,

    pub(crate) action_dispatchers: BTreeMap<dokime_domain::model::identity::Identifier, Arc<dyn ActionDispatcher>>,

    pub(crate) embedded_providers: Vec<Arc<dyn crate::runtime::EmbeddedProvider>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeConfig {
    pub fn new(storage_root: impl Into<PathBuf>, hub_metadata: Metadata) -> Self {
        let storage_root = storage_root.into();
        Self {
            schema: harmos::config::SchemaSettings::default(),
            session_id: crate::model::identity::new_id::<SessionId>(),
            storage: crate::config::storage_defaults(),
            local_storage: crate::config::local_storage_defaults(),
            managed_storage: super::managed_storage_defaults(),
            storage_state: super::storage_state_defaults(),
            // Programmatic configurations keep one disposable tree; shipped
            // shells always name a separate application root.
            application_root: storage_root.join("application"),
            hub_metadata,

            #[cfg(feature = "host")]
            credential_store: None,
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

    /// Supply Credential Storage As A Live Composition Resource, Separate From Settings.
    #[cfg(feature = "host")]
    pub fn with_credential_store(mut self, store: Arc<keyring_core::CredentialStore>) -> Self {
        self.credential_store = Some(store);
        self
    }

    pub fn with_trusted_bench(mut self, trusted: bool) -> Self {
        self.trusted_bench = trusted;
        self
    }

    pub fn with_installed_plugins(mut self, plugins: impl IntoIterator<Item = InstalledPlugin>) -> Self {
        self.installed_plugins = plugins.into_iter().collect();
        self
    }

    pub fn with_declared_plugin(
        mut self, source: &str, version: &str, description: &str, registrar: harmos::sidecar::Registrar,
    ) -> crate::runtime::Result<Self> {
        let plugin =
            crate::domains::installed_plugins::contributions::decode_registrations(source, version, description, &registrar.finish())?;
        self.installed_plugins.push(plugin);
        Ok(self)
    }

    pub fn with_plugins(mut self, plugins: impl IntoIterator<Item = crate::plugins::PluginBundle>) -> Self {
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
        mut self, plugin_id: dokime_domain::model::identity::Identifier, dispatcher: Arc<dyn ActionDispatcher>,
    ) -> Self {
        self.action_dispatchers.insert(plugin_id, dispatcher);
        self
    }
}

impl fmt::Debug for RuntimeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeConfig")
            .field("embedded_providers", &self.embedded_providers.len())
            .field("session_id", &self.session_id)
            .field("storage_root", &self.storage_root)
            .field("storage", &self.storage)
            .field("application_root", &self.application_root)
            .field("hub_metadata", &self.hub_metadata)
            .field("installed_plugins", &self.installed_plugins)
            .field("plugins", &self.plugins)
            .field("action_dispatchers", &self.action_dispatchers.keys().collect::<Vec<_>>())
            .finish()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
