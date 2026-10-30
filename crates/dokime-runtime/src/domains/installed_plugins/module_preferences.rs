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

    pub(crate) fn bind_installs(&self, installs: BTreeMap<String, Uuid>) {
        *self.installs.write().unwrap_or_else(PoisonError::into_inner) = installs;
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

    async fn update(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{SecretSetting, TextArea};

    #[tokio::test]
    async fn direct_secret_startup() {
        let root = tempfile::tempdir().unwrap();
        let config =
            crate::RuntimeConfig::new(root.path(), crate::Metadata::new(Identifier::new("secret.fixture").unwrap(), "Secrets", None))
                .with_credential_store(keyring_core::mock::Store::new().unwrap());
        let mut runtime = crate::Runtime::start(config).await.unwrap();
        let handle = runtime.handle();
        let plugin = crate::InstalledPlugin {
            source: "fixture".into(),
            settings: Settings {
                contributions: vec![
                    SettingsEntry::TextArea(TextArea {
                        id: "workspace".into(),
                        title: "Workspace".into(),
                        description: None,
                        default: "default".into(),
                        required: true,
                    }),
                    SettingsEntry::Secret(SecretSetting {
                        id: "optional".into(),
                        title: "Optional".into(),
                        description: None,
                        required: false,
                    }),
                    SettingsEntry::Secret(SecretSetting {
                        id: "api_token".into(),
                        title: "API Token".into(),
                        description: None,
                        required: true,
                    }),
                ],
            },
            ..Default::default()
        };
        let id = Identifier::new(&plugin.source).unwrap();
        handle
            .plugin_preferences()
            .update(&id, |stored| {
                stored.values = SettingValues::from([
                    ("workspace".into(), "saved".into()),
                    (
                        "api_token".into(),
                        crate::Value::from_plain_json(serde_json::json!({
                            "kind": "stored_secret", "secret_id": "fixture.api_token"
                        }))
                        .unwrap(),
                    ),
                    (
                        "removed".into(),
                        crate::Value::from_plain_json(serde_json::json!({
                            "kind": "environment", "variable": "PATH"
                        }))
                        .unwrap(),
                    ),
                ]);
                stored.saved = stored.values.keys().cloned().collect();
            })
            .await
            .unwrap();
        let configured = serde_json::json!({});
        let missing = handle.plugin_initialization(&plugin, &configured).await.unwrap_err();
        assert!(
            matches!(&missing, crate::runtime::Error::Dokime(crate::Error::UnavailableSecret(identifier)) if identifier == "api_token")
        );
        assert_eq!(missing.to_string(), "secret `api_token` is not in Secret Storage; add it and restart");

        let store = handle.secret_store();
        // A Similar Plugin-Prefixed Identifier Never Satisfies The Declaration.
        store
            .apply(
                vec![crate::secrets::Change {
                    metadata: dokime_domain::model::secrets::catalog::Metadata {
                        identifier: "fixture.api_token".into(),
                        label: "Other".into(),
                        description: None,
                    },
                    value: Some(crate::secrets::SecretValue::new("other".into())),
                }],
                &[],
                0,
            )
            .await
            .unwrap();
        assert!(matches!(handle.plugin_initialization(&plugin, &configured).await,
            Err(crate::runtime::Error::Dokime(crate::Error::UnavailableSecret(identifier))) if identifier == "api_token"));

        store
            .apply(
                vec![crate::secrets::Change {
                    metadata: dokime_domain::model::secrets::catalog::Metadata {
                        identifier: "api_token".into(),
                        label: "API Token".into(),
                        description: None,
                    },
                    value: Some(crate::secrets::SecretValue::new("direct-value".into())),
                }],
                &[],
                1,
            )
            .await
            .unwrap();
        // Retry Reuses The Initialization Path And Reads The Updated Store.
        let retry = handle.plugin_initialization(&plugin, &configured).await.unwrap();
        assert_eq!(retry.secret("api_token").unwrap().expose(), "direct-value");
        assert_eq!(retry.secrets["optional"], None);
        assert_eq!(retry.settings, SettingValues::from([("workspace".into(), "saved".into())]));
        assert_eq!(retry.saved, BTreeSet::from(["workspace".into()]));
        assert!(!format!("{retry:?}").contains("direct-value"));
        runtime.shutdown().await.unwrap();
    }
}
