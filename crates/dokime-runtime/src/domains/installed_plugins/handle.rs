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
    installed_plugins::{PluginSecretUpdate, PluginSettingsUpdate, RuntimePluginManagementError, RuntimePluginSettings},
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
        let resolved = self
            .plugin_preferences()
            .read(
                &Identifier::new(&plugin.source).map_err(|error| RuntimePluginManagementError::InvalidInput(error.to_string()))?,
                &plugin.settings,
                &self.secret_store(),
            )
            .await?;
        let secrets_configured = resolved.redacted_secrets.missing_required.is_empty();

        Ok(DispatchContext { secrets_configured })
    }

    pub async fn plugin_settings(&self, plugin_id: &Identifier) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        let state = self.state_read();
        let plugin =
            state.installed_plugins.get(plugin_id.as_str()).ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))?;
        let resolved = self.plugin_preferences().read(plugin_id, &plugin.settings, &self.secret_store()).await?;
        Ok(RuntimePluginSettings {
            settings: plugin.settings.clone(),
            values: resolved.values,
            discovered: resolved.discovered,
            secret_bindings: resolved.redacted_secrets,
        })
    }

    pub(crate) async fn discover_plugin_settings(&self, plugin_id: &Identifier, dispatcher: &crate::plugins::dispatch::Dispatcher) {
        let state = self.state_read();
        let Some(plugin) = state.installed_plugins.get(plugin_id.as_str()) else {
            return;
        };
        let saved = match self.plugin_preferences().read(plugin_id, &plugin.settings, &self.secret_store()).await {
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
        let state = self.state_read();
        let plugin =
            state.installed_plugins.get(plugin_id.as_str()).ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))?;
        self.plugin_preferences().write_values(plugin_id, &plugin.settings, update.values).await?;
        drop(state);
        self.plugin_settings(plugin_id).await
    }

    pub async fn save_plugin_secret(
        &self, plugin_id: &Identifier, secret_id: &str, update: PluginSecretUpdate,
    ) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        if update.secret.is_empty() {
            return Err(RuntimePluginManagementError::InvalidInput("secret secret must not be empty".to_owned()));
        }
        let state = self.state_read();
        let plugin =
            state.installed_plugins.get(plugin_id.as_str()).ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))?;
        if !plugin.settings.secrets().any(|secret| secret.id == secret_id) {
            return Err(RuntimePluginManagementError::InvalidInput(format!(
                "secret {secret_id} is not declared by plugin plugin {plugin_id}"
            )));
        }
        let stored_id = format!("{}.{}", plugin_id.as_str(), secret_id);
        self.secret_store()
            .set_secret(&stored_id, &update.secret)
            .await
            .map_err(|error| RuntimePluginManagementError::Store(error.to_string()))?;
        self.plugin_preferences().bind_secret(plugin_id, secret_id, stored_id).await?;
        drop(state);
        self.plugin_settings(plugin_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
