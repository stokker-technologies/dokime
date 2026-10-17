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
use std::{collections::BTreeMap, time::Duration};
// Third Party Imports

// Local Imports
use crate::InstalledPlugin;
use crate::{runtime::Error, runtime::Result, RuntimeHandle, credentials::CredentialStore};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;
use crate::plugin::Configuration;
use crate::settings::SecretReference;
use crate::settings::SettingValues;
use crate::settings::Settings;

/* === Constants === */
const CREDENTIAL_READ_TIMEOUT: Duration = Duration::from_secs(5);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) fn settings_snapshot(&self, plugin: &InstalledPlugin) -> Result<SettingValues> {
        self.resolved_settings(plugin).map(|resolved| resolved.0)
    }

    fn resolved_settings(
        &self,
        plugin: &InstalledPlugin,
    ) -> Result<(SettingValues, std::collections::BTreeSet<String>)> {
        self.plugin_preferences()
            .read(
                &Identifier::new(&plugin.source).map_err(operation)?,
                &plugin.settings,
                &self.credential_store(),
            )
            .map(|resolved| (resolved.values, resolved.saved))
            .map_err(operation)
    }

    pub(crate) async fn plugin_initialization(
        &self,
        plugin: &InstalledPlugin,
        configured: &serde_json::Value,
    ) -> Result<Configuration> {
        let (mut settings, mut saved) = self.resolved_settings(plugin)?;

        apply_configured_settings(plugin, configured, &mut settings, &mut saved)?;

        let secrets = self.resolve_secrets(&plugin.settings, &settings).await?;

        Ok(Configuration {
            settings,
            secrets,
            saved,
        })
    }

    pub(crate) async fn resolve_secrets(
        &self,
        catalog: &Settings,
        values: &SettingValues,
    ) -> Result<BTreeMap<String, Option<String>>> {
        resolved_secrets(catalog, values, &self.credential_store()).await
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn resolved_secrets(
    catalog: &Settings,
    values: &SettingValues,
    credentials: &CredentialStore,
) -> Result<BTreeMap<String, Option<String>>> {
    let mut secrets = BTreeMap::new();
    for definition in catalog.secrets() {
        let reference = values
            .get(&definition.id)
            .map(SecretReference::try_from)
            .transpose()
            .map_err(operation)?;
        let secret = match reference {
            Some(SecretReference::Environment { variable }) => match std::env::var(&variable) {
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(Error::Operation(format!(
                        "secret setting {:?} references a non-text environment value",
                        definition.id
                    )));
                }
            },
            Some(SecretReference::StoredCredential { credential_id }) => {
                read_stored_credential(credentials, &definition.id, &credential_id).await?
            }
            None => None,
        };
        secrets.insert(definition.id.clone(), secret);
    }
    Ok(secrets)
}

fn apply_configured_settings(
    plugin: &InstalledPlugin,
    configured: &serde_json::Value,
    settings: &mut SettingValues,
    saved: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    for (id, value) in configured.as_object().into_iter().flatten() {
        let entry = plugin.settings.get(id).ok_or_else(|| {
            operation(format!(
                "configured plugin {:?} sets unknown setting {id:?}",
                plugin.source
            ))
        })?;
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
    skip(credentials),
    fields(setting_id = %setting_id, credential_id = %credential_id)
)]
async fn read_stored_credential(
    credentials: &CredentialStore,
    setting_id: &str,
    credential_id: &str,
) -> Result<Option<String>> {
    let credentials = credentials.clone();
    let credential_id = credential_id.to_owned();
    let read = tokio::task::spawn_blocking(move || credentials.get_secret(&credential_id));
    match tokio::time::timeout(CREDENTIAL_READ_TIMEOUT, read).await {
        Ok(Ok(Ok(secret))) => Ok(secret.map(|secret| secret.expose().to_owned())),
        Ok(Ok(Err(_backend_error))) => Err(Error::Operation(format!(
            "secret setting {setting_id:?} could not be resolved from credential storage"
        ))),
        Ok(Err(join_error)) => Err(Error::Operation(format!(
            "secret setting {setting_id:?} could not be resolved from credential storage: \
             the lookup task failed ({join_error})"
        ))),
        Err(_elapsed) => Err(Error::Operation(format!(
            "secret setting {setting_id:?} could not be resolved from credential storage: \
             timed out after {CREDENTIAL_READ_TIMEOUT:?}"
        ))),
    }
}

fn operation(error: impl std::fmt::Display) -> Error {
    Error::Operation(error.to_string())
}

/* === Tests === */
