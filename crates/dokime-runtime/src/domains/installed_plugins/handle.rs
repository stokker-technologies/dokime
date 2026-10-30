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
    fn settings_plugin(&self, plugin_id: &Identifier) -> Result<InstalledPlugin, RuntimePluginManagementError> {
        self.state_read()
            .installed_plugins
            .get(plugin_id.as_str())
            .cloned()
            .or_else(|| {
                self.plugin_settings_catalogs.read().expect("plugin settings catalog lock poisoned").get(plugin_id.as_str()).cloned()
            })
            .ok_or_else(|| RuntimePluginManagementError::NotFound(plugin_id.clone()))
    }

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
        let plugin = self.settings_plugin(plugin_id)?;
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
        let plugin = self.settings_plugin(plugin_id)?;
        self.plugin_preferences().write_values(plugin_id, &plugin.settings, update.values).await?;
        self.plugin_settings(plugin_id).await
    }

    pub async fn save_plugin_secret(
        &self, plugin_id: &Identifier, secret_id: &str, update: PluginSecretUpdate,
    ) -> Result<RuntimePluginSettings, RuntimePluginManagementError> {
        if update.secret.is_empty() {
            return Err(RuntimePluginManagementError::InvalidInput("secret secret must not be empty".to_owned()));
        }
        let plugin = self.settings_plugin(plugin_id)?;
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
        self.plugin_settings(plugin_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn failed_plugin_settings_remain_editable() {
        let root = tempfile::tempdir().unwrap();
        let id = Identifier::new("hcl").unwrap();
        let metadata = dokime_domain::model::identity::Metadata::new(id.clone(), "Fixture", None);
        let config =
            crate::config::RuntimeConfig::new(root.path(), metadata).with_credential_store(keyring_core::mock::Store::new().unwrap());
        let mut runtime = crate::Runtime::start(config).await.unwrap();
        let handle = runtime.handle();
        let plugin = InstalledPlugin {
            source: id.to_string(),
            settings: crate::settings::Settings {
                contributions: vec![crate::settings::SettingsEntry::SecretReference(crate::settings::SecretSetting {
                    id: "api_token".into(),
                    title: "Token".into(),
                    description: None,
                    default: None,
                    required: true,
                })],
            },
            ..Default::default()
        };
        handle.plugin_settings_catalogs.write().unwrap().insert(id.to_string(), plugin);
        assert!(!handle.state_read().installed_plugins.contains_key(id.as_str()));
        let settings = handle.plugin_settings(&id).await.unwrap();
        assert_eq!(settings.secret_bindings.missing_required, ["api_token"]);
        let settings = handle.save_plugin_secret(&id, "api_token", PluginSecretUpdate { secret: "fixture".into() }).await.unwrap();
        assert!(settings.secret_bindings.missing_required.is_empty());
        assert!(settings.secret_bindings.bindings.contains_key("api_token"));
        assert!(!handle.state_read().installed_plugins.contains_key(id.as_str()));
        runtime.shutdown().await.unwrap();
    }
}
