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
use crate::{
    config::Settings as ApplicationSettings,
    credentials::{CredentialStore, CredentialStoreError},
    installed_plugins::{PluginRuntimeCredentialBindings, RuntimePluginManagementError},
    plugins::ProgramValues,
};
use crate::core::Identifier;
use crate::settings::{SecretReference, SettingValues, Settings};

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
    pub redacted_credentials: PluginRuntimeCredentialBindings,
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

    pub(crate) fn bind_installs(&self, installs: BTreeMap<String, Uuid>) {
        *self
            .installs
            .write()
            .unwrap_or_else(PoisonError::into_inner) = installs;
    }

    pub(crate) fn read(
        &self,
        plugin_id: &Identifier,
        settings: &Settings,
        credentials: &CredentialStore,
    ) -> Result<ResolvedPluginPreferences, RuntimePluginManagementError> {
        let stored = self.stored(plugin_id);
        let saved_values = stored
            .values
            .iter()
            .filter(|(id, _)| stored.saved.contains(*id))
            .map(|(id, value)| (id.clone(), value.clone()))
            .collect();
        let values = settings.resolve(&saved_values).map_err(invalid_input)?;
        let redacted_credentials = redacted_bindings(settings, &values, credentials)?;
        // Resolved against the catalog alone: a discovered value this plugin no
        // longer declares is dropped rather than shown.
        let discovered = stored
            .values
            .into_iter()
            .filter(|(id, _)| settings.get(id).is_some() && !stored.saved.contains(id))
            .collect();

        Ok(ResolvedPluginPreferences {
            saved: stored.saved,
            values,
            discovered,
            redacted_credentials,
        })
    }

    pub(crate) fn record_discovered(
        &self,
        plugin_id: &Identifier,
        discovered: SettingValues,
    ) -> Result<(), RuntimePluginManagementError> {
        self.update(plugin_id, |stored| {
            for (id, value) in discovered {
                if !stored.saved.contains(&id) {
                    stored.values.insert(id, value);
                }
            }
        })
    }

    pub(crate) fn write_values(
        &self,
        plugin_id: &Identifier,
        schema: &Settings,
        values: SettingValues,
    ) -> Result<(), RuntimePluginManagementError> {
        schema.resolve(&values).map_err(invalid_input)?;
        // Saving a discovered value as it stands is what makes it an edited
        // one; discovered values for other fields stay offered.
        self.update(plugin_id, |stored| {
            let saved = std::mem::take(&mut stored.saved);
            stored.values.retain(|id, _| !saved.contains(id));
            stored.saved = values.keys().cloned().collect();
            stored.values.extend(values);
        })
    }

    pub(crate) fn bind_credential(
        &self,
        plugin_id: &Identifier,
        handle: &str,
        credential_id: String,
    ) -> Result<(), RuntimePluginManagementError> {
        self.update(plugin_id, |stored| {
            stored.values.insert(
                handle.to_owned(),
                SecretReference::StoredCredential { credential_id }.into(),
            );
            stored.saved.insert(handle.to_owned());
        })
    }

    fn install(&self, plugin_id: &Identifier) -> Option<Uuid> {
        self.installs
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(plugin_id.as_str())
            .copied()
    }

    fn stored(&self, plugin_id: &Identifier) -> ProgramValues {
        let Some(install) = self.install(plugin_id) else {
            return self
                .transient
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .get(plugin_id.as_str())
                .cloned()
                .unwrap_or_default();
        };
        self.settings.read(|document| {
            document
                .plugins
                .get(&install)
                .and_then(|plugin| plugin.programs.get(plugin_id.as_str()))
                .cloned()
                .unwrap_or_default()
        })
    }

    fn update(
        &self,
        plugin_id: &Identifier,
        change: impl FnOnce(&mut ProgramValues),
    ) -> Result<(), RuntimePluginManagementError> {
        let Some(install) = self.install(plugin_id) else {
            let mut transient = self
                .transient
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            change(transient.entry(plugin_id.as_str().to_owned()).or_default());
            return Ok(());
        };
        self.settings.edit(|document| {
            let plugin = document.plugins.get_mut(&install).ok_or_else(|| {
                RuntimePluginManagementError::Store(format!(
                    "plugin install `{install}` is no longer authorized"
                ))
            })?;
            change(
                plugin
                    .programs
                    .entry(plugin_id.as_str().to_owned())
                    .or_default(),
            );
            Ok(())
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn redacted_bindings(
    settings: &Settings,
    values: &SettingValues,
    credentials: &CredentialStore,
) -> Result<PluginRuntimeCredentialBindings, RuntimePluginManagementError> {
    let mut redacted = BTreeMap::new();
    let mut missing_required = Vec::new();
    for declaration in settings.secrets() {
        let configured = match values
            .get(&declaration.id)
            .map(SecretReference::try_from)
            .transpose()
            .map_err(invalid_input)?
        {
            Some(SecretReference::StoredCredential { credential_id }) => {
                let binding = credentials
                    .redacted_binding(credential_id)
                    .map_err(credential_error)?;
                let configured = binding.configured;
                redacted.insert(declaration.id.clone(), binding);
                configured
            }
            Some(SecretReference::Environment { variable }) => {
                std::env::var(variable).is_ok_and(|value| !value.is_empty())
            }
            None => false,
        };
        if declaration.required && !configured {
            missing_required.push(declaration.id.clone());
        }
    }
    Ok(PluginRuntimeCredentialBindings {
        missing_required,
        bindings: redacted,
    })
}

fn credential_error(error: CredentialStoreError) -> RuntimePluginManagementError {
    RuntimePluginManagementError::Store(error.to_string())
}

fn invalid_input(error: impl std::fmt::Display) -> RuntimePluginManagementError {
    RuntimePluginManagementError::InvalidInput(error.to_string())
}

/* === Tests === */
