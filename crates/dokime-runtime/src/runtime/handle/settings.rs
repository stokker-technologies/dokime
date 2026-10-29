/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{collections::BTreeMap, time::Duration};

use crate::InstalledPlugin;
use crate::plugin::Configuration;
use crate::settings::SecretReference;
use crate::settings::SettingValues;
use crate::settings::Settings;
use crate::{RuntimeHandle, runtime::Error, runtime::Result, secrets::SecretStore};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;

impl RuntimeHandle {
    pub(crate) async fn settings_snapshot(&self, plugin: &InstalledPlugin) -> Result<SettingValues> {
        self.resolved_settings(plugin).await.map(|resolved| resolved.0)
    }

    async fn resolved_settings(&self, plugin: &InstalledPlugin) -> Result<(SettingValues, std::collections::BTreeSet<String>)> {
        self.plugin_preferences()
            .read(&Identifier::new(&plugin.source).map_err(operation)?, &plugin.settings, &self.secret_store())
            .await
            .map(|resolved| (resolved.values, resolved.saved))
            .map_err(operation)
    }

    pub(crate) async fn plugin_initialization(&self, plugin: &InstalledPlugin, configured: &serde_json::Value) -> Result<Configuration> {
        let (mut settings, mut saved) = self.resolved_settings(plugin).await?;

        apply_configured_settings(plugin, configured, &mut settings, &mut saved)?;

        let secrets = self.resolve_secrets(&plugin.settings, &settings).await?;

        Ok(Configuration { settings, secrets, saved })
    }

    pub(crate) async fn resolve_secrets(&self, catalog: &Settings, values: &SettingValues) -> Result<BTreeMap<String, Option<String>>> {
        resolved_secrets(catalog, values, &self.secret_store(), Duration::from_secs(5)).await
    }
}

async fn resolved_secrets(
    catalog: &Settings, values: &SettingValues, store: &SecretStore, timeout: Duration,
) -> Result<BTreeMap<String, Option<String>>> {
    let mut secrets = BTreeMap::new();
    for definition in catalog.secrets() {
        let reference = values.get(&definition.id).map(SecretReference::try_from).transpose().map_err(operation)?;
        let secret = match reference {
            Some(SecretReference::Environment { variable }) => match std::env::var(&variable) {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(Error::Operation(format!("secret setting {:?} references a non-text environment value", definition.id)));
                }
            },
            Some(SecretReference::StoredSecret { secret_id }) => read_stored_secret(store, &definition.id, &secret_id, timeout).await?,
            None => None,
        };
        secrets.insert(definition.id.clone(), secret);
    }
    Ok(secrets)
}

fn apply_configured_settings(
    plugin: &InstalledPlugin, configured: &serde_json::Value, settings: &mut SettingValues, saved: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    for (id, value) in configured.as_object().into_iter().flatten() {
        let entry = plugin
            .settings
            .get(id)
            .ok_or_else(|| operation(format!("configured plugin {:?} sets unknown setting {id:?}", plugin.source)))?;
        let value = Value::from_plain_json(value.clone()).map_err(operation)?;
        entry.validate_value(&value).map_err(operation)?;
        settings.insert(id.clone(), value);
        // A configured bundle seeds a value the way an operator would, so it
        // is a filled-in setting rather than a default a discovery route may
        // fill. It is still never written back over what an operator saved.
        saved.insert(id.clone());
    }
    Ok(())
}

#[tracing::instrument(
    level = "info",
    skip(secrets),
    fields(setting_id = %setting_id, secret_id = %secret_id)
)]
async fn read_stored_secret(secrets: &SecretStore, setting_id: &str, secret_id: &str, timeout: Duration) -> Result<Option<String>> {
    let secrets = secrets.clone();
    let secret_id = secret_id.to_owned();
    let read = secrets.get_secret(&secret_id);
    match tokio::time::timeout(timeout, read).await {
        Ok(Ok(secret)) => Ok(secret.map(|secret| secret.expose().to_owned())),
        Ok(Err(_backend_error)) => {
            Err(Error::Operation(format!("secret setting {setting_id:?} could not be resolved from secret storage")))
        }
        Err(_elapsed) => Err(Error::Operation(format!(
            "secret setting {setting_id:?} could not be resolved from secret storage: \
             timed out after {timeout:?}"
        ))),
    }
}

fn operation(error: impl std::fmt::Display) -> Error {
    Error::Operation(error.to_string())
}
