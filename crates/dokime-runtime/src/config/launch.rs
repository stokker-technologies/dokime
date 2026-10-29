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
use std::{fmt, path::PathBuf, sync::Arc};

// Third Party Imports

// Local Imports
use crate::config::{RuntimeConfig, StorageSettings};
use crate::domains::installed_plugins::{ResolvedBrand, builtin_plugins};
use crate::plugins::PluginBundle;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

/* === Constants === */

const HUB_IDENTIFIER: &str = "dokime.test_hub";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct Config {
    pub schema: harmos::config::SchemaSettings,
    pub title: String,
    pub label: Option<String>,
    pub output_root: PathBuf,
    pub storage: StorageSettings,
    pub managed_storage: harmos::config::ManagedStorageSettings,
    pub storage_state: harmos::config::StorageStateSettings,

    pub application_root: Option<PathBuf>,
    pub seed_plugins: bool,
    pub plugin_lookup: Option<PathBuf>,
    pub explicit_plugins: Vec<PathBuf>,
    pub brand: Option<ResolvedBrand>,
    registrations: Registrations,
}

pub(crate) struct ShellStartup {
    pub(crate) brand: Option<ResolvedBrand>,
    pub(crate) native_host_telemetry: bool,
}

#[derive(Clone, Default)]
struct Registrations(Vec<Arc<dyn crate::runtime::EmbeddedProvider>>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl fmt::Debug for Registrations {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Registrations").field(&self.0.len()).finish()
    }
}

impl Config {
    pub fn local(output_root: impl Into<PathBuf>) -> Self {
        Self {
            schema: harmos::config::SchemaSettings::default(),
            title: "Dokime".to_owned(),
            label: None,
            output_root: output_root.into(),
            storage: super::storage_defaults(),
            managed_storage: super::managed_storage_defaults(),
            storage_state: super::storage_state_defaults(),
            application_root: None,
            seed_plugins: true,
            plugin_lookup: None,
            explicit_plugins: Vec::new(),
            brand: None,
            registrations: Registrations::default(),
        }
    }

    pub fn register(mut self, provider: Arc<dyn crate::runtime::EmbeddedProvider>) -> Self {
        self.registrations.0.push(provider);
        self
    }

    pub(crate) fn into_runtime_config(self) -> (RuntimeConfig, ShellStartup) {
        let startup = ShellStartup {
            brand: self.brand,
            native_host_telemetry: self.seed_plugins,
        };
        let metadata = Metadata::new(Identifier::new(HUB_IDENTIFIER).expect("valid application identifier"), self.title, self.label);
        let mut config = RuntimeConfig::new(&self.output_root, metadata);
        config.schema = self.schema;
        config.storage = self.storage;
        config.managed_storage = self.managed_storage;
        config.storage_state = self.storage_state;
        if let Some(root) = self.application_root {
            config = config.with_application_root(root);
        }
        config.plugin_lookup = self.plugin_lookup;
        let installed_plugins = if startup.native_host_telemetry { builtin_plugins() } else { Default::default() };
        let mut config = config.with_installed_plugins(installed_plugins).with_plugins(PluginBundle::all(&self.explicit_plugins));

        for provider in self.registrations.0 {
            config = config.register(provider);
        }

        (config, startup)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
