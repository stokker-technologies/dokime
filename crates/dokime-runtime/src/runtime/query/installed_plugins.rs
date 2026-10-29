/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use crate::state::PluginOwner;
use crate::{InstalledPlugin, RuntimeState};
use dokime_domain::model::identity::Identifier;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeInstalledPluginFrame {
    pub total_plugins: usize,
    pub returned_plugins: usize,
    pub installed_plugins: Vec<RuntimeInstalledPluginRecord>,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeInstalledPluginRecord {
    pub plugin_id: Identifier,
    pub owner: Option<PluginOwner>,
    pub label: String,
    pub version: String,
    pub enabled: bool,
    pub runtime: String,
    pub status: String,
    pub error: Option<String>,
}

impl RuntimeInstalledPluginRecord {
    fn from_plugin(plugin: &InstalledPlugin, owner: Option<PluginOwner>) -> Self {
        Self {
            plugin_id: Identifier::new(&plugin.source).expect("installed source is valid"),
            owner,
            label: plugin.source.clone(),
            version: plugin.version.clone(),
            enabled: plugin.enabled,
            runtime: "code".to_owned(),
            status: if plugin.enabled { "registered" } else { "disabled" }.to_owned(),
            error: None,
        }
    }
}

pub(crate) fn installed_plugin_frame(state: &RuntimeState) -> RuntimeInstalledPluginFrame {
    let plugins = state
        .installed_plugins
        .values()
        .map(|plugin| RuntimeInstalledPluginRecord::from_plugin(plugin, state.plugin_owners.get(&plugin.source).cloned()))
        .collect::<Vec<_>>();
    RuntimeInstalledPluginFrame {
        total_plugins: plugins.len(),
        returned_plugins: plugins.len(),
        installed_plugins: plugins,
        truncated: false,
    }
}
