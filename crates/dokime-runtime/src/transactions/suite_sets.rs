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
use crate::definition::SuiteSet;
use crate::state::RuntimeState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetSuiteSet {
    pub source: String,
    pub catalog: SuiteSet,
}

#[derive(harmos_sdk_derive::Described, Clone, Debug, Serialize, Deserialize)]
pub struct RemoveSuiteSet {
    pub source: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SetSuiteSet {
    pub const ID: &'static str = "dokime.execution.catalog.set";

    pub fn definition() -> (harmos::DefinitionId, u32) {
        (harmos::DefinitionId::new(Self::ID), 1)
    }
}

impl harmos::Identified for RemoveSuiteSet {
    const ID: &'static str = "dokime.execution.catalog.unregister-source";

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

impl Transaction<RuntimeState> for SetSuiteSet {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        let mut projected = state.catalog_candidate();
        set_suites(&mut projected, &self.source, &self.catalog)
    }

    fn apply(&self, state: &mut RuntimeState) {
        set_suites(state, &self.source, &self.catalog).expect("Harmos checked the suite set before applying it");
    }
}

impl Transaction<RuntimeState> for RemoveSuiteSet {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        let mut projected = state.catalog_candidate();
        remove_suites(&mut projected, &self.source)
    }

    fn apply(&self, state: &mut RuntimeState) {
        remove_suites(state, &self.source).expect("Harmos checked suite removal before applying it");
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn ensure_host_source(state: &RuntimeState, source: &str) -> Result<(), String> {
    if matches!(
        state.plugin_owners.get(source),
        Some(crate::state::PluginOwner::Bundle { .. } | crate::state::PluginOwner::Embedded { .. })
    ) {
        Err(format!("source `{source}` belongs to a registered provider"))
    } else {
        Ok(())
    }
}

fn set_suites(state: &mut RuntimeState, source: &str, suites: &SuiteSet) -> Result<(), String> {
    ensure_host_source(state, source)?;
    state.suites.set(source, suites.clone())?;
    state.sync_suite_definitions().map_err(|error| error.to_string())?;
    if let Some(installation) = state.installed_plugins.get_mut(source) {
        installation.suites = suites.clone();
    }
    state.plugin_owners.insert(source.to_owned(), crate::state::PluginOwner::Host);
    Ok(())
}

fn remove_suites(state: &mut RuntimeState, source: &str) -> Result<(), String> {
    ensure_host_source(state, source)?;
    if let Some(mut installation) = state.installed_plugins.get(source).cloned() {
        installation.suites = SuiteSet::default();
        // This source was separately installed: retain its non-suite declarations
        // and existing authority.
        state.install_plugin(installation)
    } else {
        state.suites.unregister_source(source);
        state.sync_suite_definitions().map_err(|error| error.to_string())?;
        state.plugin_owners.remove(source);
        Ok(())
    }
}

/* === Tests === */
