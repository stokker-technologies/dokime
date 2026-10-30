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

// Local Imports
use crate::InstalledPlugin;
use crate::domains::DispatchContext;
use crate::{
    RuntimeHandle,
    installed_plugins::{PluginSettingsUpdate, RuntimePluginManagementError, RuntimePluginSettings},
};
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub(crate) async fn action_dispatch_context(&self, plugin: &InstalledPlugin) -> Result<DispatchContext, RuntimePluginManagementError> {
        let mut secrets_configured = true;
        for secret in plugin.settings.secrets().filter(|secret| secret.required) {
            secrets_configured &=
                self.secret_store().contains(&secret.id).await.map_err(|error| RuntimePluginManagementError::Store(error.to_string()))?;
        }

        Ok(DispatchContext { secrets_configured })
    }

    pub async fn plugin_settings(&self, plugin_id: &Identifier) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        let settings = self.plugin_settings_schema(plugin_id).await?;
        let resolved = self.plugin_preferences().read(plugin_id, &settings).await?;
        let mut secrets = Vec::new();
        for declaration in settings.secrets() {
            let available = self
                .secret_store()
                .contains(&declaration.id)
                .await
                .map_err(|error| RuntimePluginManagementError::Store(error.to_string()))?;
            secrets.push(dokime_domain::model::secrets::SecretAvailability {
                identifier: declaration.id.clone(),
                available,
            });
        }
        Ok(RuntimePluginSettings {
            settings,
            values: resolved.values,
            discovered: resolved.discovered,
            secrets,
        })
    }

    pub(crate) async fn discover_plugin_settings(&self, plugin_id: &Identifier, dispatcher: &crate::plugins::dispatch::Dispatcher) {
        let state = self.state_read();
        let Some(plugin) = state.installed_plugins.get(plugin_id.as_str()) else {
            return;
        };
        let saved = match self.plugin_preferences().read(plugin_id, &plugin.settings).await {
            Ok(resolved) => resolved.saved,
            Err(error) => {
                tracing::warn!(%plugin_id, %error, "saved settings did not resolve; nothing was discovered");
                return;
            }
        };
        drop(state);
        let discovered = dispatcher.discover(&saved).await;
        if discovered.is_empty() {
            return;
        }
        if let Err(error) = self.record_discovered_settings(plugin_id, discovered).await {
            tracing::warn!(%plugin_id, %error, "discovered settings were not stored");
        }
    }

    pub async fn record_discovered_settings(
        &self, plugin_id: &Identifier, discovered: crate::settings::SettingValues,
    ) -> Result<(), RuntimePluginManagementError> {
        self.plugin_preferences().record_discovered(plugin_id, discovered).await
    }

    pub async fn save_plugin_settings(
        &self, plugin_id: &Identifier, update: PluginSettingsUpdate,
    ) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        let settings = self.plugin_settings_schema(plugin_id).await?;
        self.plugin_preferences().write_values(plugin_id, &settings, update.values).await?;
        self.plugin_settings(plugin_id).await
    }

    async fn plugin_settings_schema(&self, plugin_id: &Identifier) -> Result<crate::settings::Settings, RuntimePluginManagementError> {
        let admitted = self.state_read().installed_plugins.get(plugin_id.as_str()).map(|plugin| plugin.settings.clone());
        if let Some(settings) = admitted {
            return Ok(settings);
        }
        let (install, settings) = self
            .plugin_store()
            .program_settings(plugin_id.as_str())
            .await
            .map_err(|error| RuntimePluginManagementError::Store(error.to_string()))?
            .ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))?;
        self.plugin_preferences().bind_install(plugin_id, install);
        Ok(settings)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
