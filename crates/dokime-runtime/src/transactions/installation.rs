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
use crate::state::RuntimeState;
use dokime_contract::domains::InstalledPlugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.artifact.install", version = 2, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallPlugin {
    pub installation: InstalledPlugin,
}

#[harmos::transaction(id = "dokime.artifact.remove", version = 2, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemovePlugin {
    pub source: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for InstallPlugin {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        state.clone().install_plugin(self.installation.clone())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state
            .install_plugin(self.installation.clone())
            .expect("Harmos checked the artifact installation before applying it");
    }
}

impl Transaction<RuntimeState> for RemovePlugin {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        state.clone().remove_plugin(&self.source)
    }

    fn apply(&self, state: &mut RuntimeState) {
        state
            .remove_plugin(&self.source)
            .expect("Harmos checked the artifact removal before applying it");
        state.plugin_owners.remove(&self.source);
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down {
        Err("runtime is shutting down".to_owned())
    } else {
        Ok(())
    }
}

/* === Tests === */
