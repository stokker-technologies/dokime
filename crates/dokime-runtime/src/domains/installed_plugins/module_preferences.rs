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
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Mutex, PoisonError, RwLock},
};

// Third Party Imports
use uuid::Uuid;

// Local Imports
use crate::settings::{SettingValues, Settings, SettingsEntry};
use crate::{config::Settings as ApplicationSettings, installed_plugins::RuntimePluginManagementError, plugins::ProgramValues};
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug)]
pub(crate) struct PluginPreferences {
    settings: ApplicationSettings,
    installs: RwLock<BTreeMap<String, Uuid>>,
    transient: Mutex<BTreeMap<String, ProgramValues>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedPluginPreferences {
    pub values: SettingValues,
    pub saved: BTreeSet<String>,
    pub discovered: SettingValues,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PluginPreferences {
    pub(crate) fn new(settings: ApplicationSettings) -> Self {
        Self {
            settings,
            installs: RwLock::default(),
            transient: Mutex::default(),
        }
    }

    pub(crate) fn bind_install(&self, plugin_id: &Identifier, install: Uuid) {
        self.installs.write().unwrap_or_else(PoisonError::into_inner).insert(plugin_id.as_str().to_owned(), install);
    }

    pub(crate) async fn read(
        &self, plugin_id: &Identifier, settings: &Settings,
    ) -> Result<ResolvedPluginPreferences, RuntimePluginManagementError> {
        let mut stored = self.stored(plugin_id).await?;
        // Removed Secret References Are Never Admitted As Setting Values.
        stored.values.retain(|id, value| {
            let removed = matches!(value, crate::values::Value::Object(fields)
                if matches!(fields.get("kind"), Some(crate::values::Value::Text(kind))
                    if kind == "stored_secret" || kind == "environment"));
            !removed && !matches!(settings.get(id), Some(SettingsEntry::Secret(_)))
        });
        stored.saved.retain(|id| stored.values.contains_key(id));
        let saved_values =
            stored.values.iter().filter(|(id, _)| stored.saved.contains(*id)).map(|(id, value)| (id.clone(), value.clone())).collect();
        let values = settings.resolve(&saved_values).map_err(invalid_input)?;
        // Resolved against the catalog alone: a discovered value this plugin no
        // longer declares is dropped rather than shown.
        let discovered = stored.values.into_iter().filter(|(id, _)| settings.get(id).is_some() && !stored.saved.contains(id)).collect();

        Ok(ResolvedPluginPreferences { saved: stored.saved, values, discovered })
    }

    pub(crate) async fn record_discovered(
        &self, plugin_id: &Identifier, discovered: SettingValues,
    ) -> Result<(), RuntimePluginManagementError> {
        self.update(plugin_id, move |stored| {
            for (id, value) in discovered {
                if !stored.saved.contains(&id) {
                    stored.values.insert(id, value);
                }
            }
        })
        .await
    }

    pub(crate) async fn write_values(
        &self, plugin_id: &Identifier, schema: &Settings, values: SettingValues,
    ) -> Result<(), RuntimePluginManagementError> {
        schema.resolve(&values).map_err(invalid_input)?;
        // Saving a discovered value as it stands is what makes it an edited
        // one; discovered values for other fields stay offered.
        self.update(plugin_id, move |stored| {
            let saved = std::mem::take(&mut stored.saved);
            stored.values.retain(|id, _| !saved.contains(id));
            stored.saved = values.keys().cloned().collect();
            stored.values.extend(values);
        })
        .await
    }

    fn install(&self, plugin_id: &Identifier) -> Option<Uuid> {
        self.installs.read().unwrap_or_else(PoisonError::into_inner).get(plugin_id.as_str()).copied()
    }

    async fn stored(&self, plugin_id: &Identifier) -> Result<ProgramValues, RuntimePluginManagementError> {
        let Some(install) = self.install(plugin_id) else {
            return Ok(self.transient.lock().unwrap_or_else(PoisonError::into_inner).get(plugin_id.as_str()).cloned().unwrap_or_default());
        };
        let plugin_id = plugin_id.clone();
        self.settings
            .read(move |document| {
                document.plugins.get(&install).and_then(|plugin| plugin.programs.get(plugin_id.as_str())).cloned().unwrap_or_default()
            })
            .await
            .map_err(|error| RuntimePluginManagementError::Store(error.to_string()))
    }

    pub(crate) async fn update(
        &self, plugin_id: &Identifier, change: impl FnOnce(&mut ProgramValues) + Send + 'static,
    ) -> Result<(), RuntimePluginManagementError> {
        let Some(install) = self.install(plugin_id) else {
            let mut transient = self.transient.lock().unwrap_or_else(PoisonError::into_inner);
            change(transient.entry(plugin_id.as_str().to_owned()).or_default());
            return Ok(());
        };
        let plugin_id = plugin_id.clone();
        self.settings
            .edit(move |document| {
                let plugin = document
                    .plugins
                    .get_mut(&install)
                    .ok_or_else(|| RuntimePluginManagementError::Store(format!("plugin install `{install}` is no longer authorized")))?;
                change(plugin.programs.entry(plugin_id.as_str().to_owned()).or_default());
                Ok(())
            })
            .await
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn invalid_input(error: impl std::fmt::Display) -> RuntimePluginManagementError {
    RuntimePluginManagementError::InvalidInput(error.to_string())
}

/* === Tests === */
