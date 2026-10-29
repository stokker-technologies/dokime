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

use crate::state::RuntimeState;
use dokime_domain::model::plugins::InstalledPlugin;

#[harmos::transaction(id = "dokime.artifact.install")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallPlugin {
    pub installation: InstalledPlugin,
}

#[harmos::transaction(id = "dokime.artifact.remove")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemovePlugin {
    pub source: String,
}

impl Transaction<RuntimeState> for InstallPlugin {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        state.catalog_candidate().install_plugin(self.installation.clone())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.install_plugin(self.installation.clone()).expect("Harmos checked the artifact installation before applying it");
    }
}

impl Transaction<RuntimeState> for RemovePlugin {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        state.catalog_candidate().remove_plugin(&self.source)
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.remove_plugin(&self.source).expect("Harmos checked the artifact removal before applying it");
        state.plugin_owners.remove(&self.source);
    }
}

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".to_owned()) } else { Ok(()) }
}
