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
use dokime_domain::model::plugins::InstalledPlugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallPlugin {
    pub installation: InstalledPlugin,
}

#[derive(harmos_sdk_derive::Described, Clone, Debug, Serialize, Deserialize)]
pub struct RemovePlugin {
    pub source: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl InstallPlugin {
    pub const ID: &'static str = "dokime.artifact.install";

    pub fn definition() -> (harmos::DefinitionId, u32) {
        (harmos::DefinitionId::new(Self::ID), 1)
    }
}

impl harmos::Identified for RemovePlugin {
    const ID: &'static str = "dokime.artifact.remove";

    fn definition() -> (harmos::DefinitionId, u32) {
        static VERSION: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        let version = *VERSION.get_or_init(|| {
            let schema = harmos::schema::graph::Schema::from_declaration(
                <Self as harmos::schema::Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .expect("valid stored data schema");
            harmos::schema::identity::revision(&schema).expect("valid stored data revision")
        });
        (harmos::DefinitionId::new(Self::ID), version)
    }
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

/* === Public Functions === */

/* === Private Functions === */

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".to_owned()) } else { Ok(()) }
}

/* === Tests === */
