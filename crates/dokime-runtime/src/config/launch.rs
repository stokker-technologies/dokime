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
use crate::RuntimeCredentialConfig;
use crate::config::RuntimeConfig;
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
    pub title: String,
    pub label: Option<String>,
    pub output_root: PathBuf,
    pub compaction_interval: std::time::Duration,
    pub local_retention: Option<std::time::Duration>,
    pub application_root: Option<PathBuf>,
    pub seed_plugins: bool,
    pub plugin_lookup: Option<PathBuf>,
    pub explicit_plugins: Vec<PathBuf>,
    pub brand: Option<ResolvedBrand>,
    pub credential_backend: RuntimeCredentialConfig,
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
            title: "Dokime".to_owned(),
            label: None,
            output_root: output_root.into(),
            compaction_interval: std::time::Duration::from_secs(24 * 60 * 60),
            local_retention: None,
            application_root: None,
            seed_plugins: true,
            plugin_lookup: None,
            explicit_plugins: Vec::new(),
            brand: None,
            credential_backend: RuntimeCredentialConfig::default(),
            registrations: Registrations::default(),
        }
    }

    pub fn register(mut self, provider: Arc<dyn crate::runtime::EmbeddedProvider>) -> Self {
        self.registrations.0.push(provider);
        self
    }

    pub(crate) fn into_runtime_config(self) -> (RuntimeConfig, ShellStartup) {
        let startup = ShellStartup { brand: self.brand, native_host_telemetry: self.seed_plugins };
        let metadata = Metadata::new(Identifier::new(HUB_IDENTIFIER).expect("valid application identifier"), self.title, self.label);
        let mut config = RuntimeConfig::new(&self.output_root, metadata).with_credential_backend(self.credential_backend);
        config.compaction_interval = self.compaction_interval;
        config.local_retention = self.local_retention;
        if let Some(root) = self.application_root {
            config = config.with_application_root(root);
        }
        config.plugin_lookup = self.plugin_lookup;
        let mut config = config
            .with_installed_plugins(startup.native_host_telemetry.then(builtin_plugins).unwrap_or_default())
            .with_plugins(PluginBundle::all(&self.explicit_plugins));

        for provider in self.registrations.0 {
            config = config.register(provider);
        }

        (config, startup)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
