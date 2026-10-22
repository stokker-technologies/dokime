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
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::state::ProviderCatalog;
use crate::{InstalledPlugin, RuntimeState};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.artifact.host.install", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallHostPlugin {
    pub installation: InstalledPlugin,
}

#[harmos::transaction(id = "dokime.providers.reconcile", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconcileProviders {
    pub providers: Vec<ProviderCatalog>,
}

#[harmos::transaction(id = "dokime.providers.reload", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReloadProviderCatalog {
    pub providers: Vec<ProviderCatalog>,
    pub provider: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for InstallHostPlugin {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        active(state)?;
        state.clone().install_host_plugin(self.installation.clone())
    }
    fn apply(&self, state: &mut RuntimeState) {
        state.install_host_plugin(self.installation.clone()).expect("host catalog admission was checked");
    }
}

impl Transaction<RuntimeState> for ReconcileProviders {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        active(state)?;
        state.validate_providers(&self.providers)
    }
    fn apply(&self, state: &mut RuntimeState) {
        state.reconcile_providers(&self.providers).expect("provider catalog admission was checked");
    }
}

impl Transaction<RuntimeState> for ReloadProviderCatalog {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        active(state)?;
        state.validate_provider_reload(&self.providers, &self.provider)
    }
    fn apply(&self, state: &mut RuntimeState) {
        state.reload_provider_catalog(&self.providers, &self.provider).expect("development catalog replacement was checked");
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".into()) } else { Ok(()) }
}

/* === Tests === */
