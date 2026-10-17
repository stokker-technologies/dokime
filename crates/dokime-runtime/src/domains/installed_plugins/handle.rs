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
    installed_plugins::{PluginCredentialSecretUpdate, PluginSettingsUpdate, RuntimePluginManagementError, RuntimePluginSettings},
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
    pub(crate) fn action_dispatch_context(&self, plugin: &InstalledPlugin) -> Result<DispatchContext, RuntimePluginManagementError> {
        let resolved = self.plugin_preferences().read(
            &Identifier::new(&plugin.source).map_err(|error| RuntimePluginManagementError::InvalidInput(error.to_string()))?,
            &plugin.settings,
            &self.credential_store(),
        )?;
        let credentials_configured = resolved.redacted_credentials.missing_required.is_empty();

        Ok(DispatchContext { credentials_configured })
    }

    pub async fn plugin_settings(&self, plugin_id: &Identifier) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        let plugin = self
            .installed_plugins()
            .get(plugin_id.as_str())
            .cloned()
            .ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))?;
        let resolved = self.plugin_preferences().read(plugin_id, &plugin.settings, &self.credential_store())?;
        Ok(RuntimePluginSettings {
            settings: plugin.settings,
            values: resolved.values,
            discovered: resolved.discovered,
            credential_bindings: resolved.redacted_credentials,
        })
    }

    pub(crate) async fn discover_plugin_settings(&self, plugin_id: &Identifier, dispatcher: &crate::plugins::dispatch::Dispatcher) {
        let Some(plugin) = self.installed_plugins().get(plugin_id.as_str()).cloned() else {
            return;
        };
        let saved = match self.plugin_preferences().read(plugin_id, &plugin.settings, &self.credential_store()) {
            Ok(resolved) => resolved.saved,
            Err(error) => {
                tracing::warn!(%plugin_id, %error, "saved settings did not resolve; nothing was discovered");
                return;
            }
        };
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
        self.plugin_preferences().record_discovered(plugin_id, discovered)
    }

    pub async fn save_plugin_settings(
        &self, plugin_id: &Identifier, update: PluginSettingsUpdate,
    ) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        let plugin = self
            .installed_plugins()
            .get(plugin_id.as_str())
            .cloned()
            .ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))?;
        self.plugin_preferences().write_values(plugin_id, &plugin.settings, update.values)?;
        self.plugin_settings(plugin_id).await
    }

    pub async fn save_plugin_credential_secret(
        &self, plugin_id: &Identifier, credential_id: &str, update: PluginCredentialSecretUpdate,
    ) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        if update.secret.is_empty() {
            return Err(RuntimePluginManagementError::InvalidInput("credential secret must not be empty".to_owned()));
        }
        let plugin = self
            .installed_plugins()
            .get(plugin_id.as_str())
            .cloned()
            .ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))?;
        if !plugin.settings.secrets().any(|secret| secret.id == credential_id) {
            return Err(RuntimePluginManagementError::InvalidInput(format!(
                "credential {credential_id} is not declared by plugin plugin {plugin_id}"
            )));
        }
        let stored_id = format!("{}.{}", plugin_id.as_str(), credential_id);
        self.credential_store()
            .set_secret(&stored_id, &update.secret)
            .map_err(|error| RuntimePluginManagementError::Store(error.to_string()))?;
        self.plugin_preferences().bind_credential(plugin_id, credential_id, stored_id)?;
        self.plugin_settings(plugin_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
