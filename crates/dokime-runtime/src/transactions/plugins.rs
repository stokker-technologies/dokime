/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

use crate::state::ProviderCatalog;
use crate::{InstalledPlugin, RuntimeState};

#[harmos::transaction(id = "dokime.artifact.host.install", irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallHostPlugin {
    pub installation: InstalledPlugin,
}

#[harmos::transaction(id = "dokime.providers.reconcile", irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconcileProviders {
    pub providers: Vec<ProviderCatalog>,
}

#[harmos::transaction(id = "dokime.providers.reload", irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReloadProviderCatalog {
    pub providers: Vec<ProviderCatalog>,
    pub provider: String,
}

impl Transaction<RuntimeState> for InstallHostPlugin {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        active(state)?;
        state.catalog_candidate().install_host_plugin(self.installation.clone())
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

fn active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".into()) } else { Ok(()) }
}
