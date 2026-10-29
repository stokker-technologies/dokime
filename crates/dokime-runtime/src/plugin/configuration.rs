/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::plugin::settings::PluginSettings;
use dokime_domain::model::values::Value;

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub settings: BTreeMap<String, Value>,

    #[serde(default)]
    pub secrets: BTreeMap<String, Option<String>>,

    #[serde(default)]
    pub saved: BTreeSet<String>,
}

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl Configuration {
    pub fn declared<S: PluginSettings>() -> Self {
        let catalog = S::catalog();
        Self {
            settings: catalog
                .contributions
                .iter()
                .filter_map(|definition| definition.default_value().map(|value| (definition.id().to_owned(), value)))
                .collect(),
            secrets: catalog.secrets().map(|definition| (definition.id.clone(), None)).collect(),
            saved: BTreeSet::new(),
        }
    }

    pub fn settings<S: PluginSettings>(&self) -> crate::Result<S> {
        let catalog = S::catalog();
        let mut admitted = crate::settings::SettingValues::new();
        for (id, value) in &self.settings {
            if catalog.get(id).is_some() {
                admitted.insert(id.clone(), value.clone());
            } else {
                tracing::debug!(setting = %id, "a saved value names a setting this plugin no longer declares");
            }
        }
        let values = catalog.resolve(&admitted)?;
        S::decode(&values, self)
    }

    pub fn secrets_configured<S: PluginSettings>(&self) -> bool {
        S::catalog().secrets().filter(|declared| declared.required).all(|declared| self.secret(&declared.id).is_ok())
    }

    pub fn is_saved(&self, id: &str) -> bool {
        self.saved.contains(id)
    }

    pub fn setting<T: DeserializeOwned>(&self, id: &str) -> Result<T, crate::Error> {
        if self.secrets.contains_key(id) {
            return Err(crate::Error::SecretSetting(id.to_owned()));
        }
        let value = self.settings.get(id).ok_or_else(|| crate::Error::UnknownSetting(id.to_owned()))?;
        serde_json::from_value(value.to_plain_json().map_err(|_| crate::Error::SettingType(id.to_owned()))?)
            .map_err(|_| crate::Error::SettingType(id.to_owned()))
    }

    pub fn secret(&self, id: &str) -> Result<Secret, crate::Error> {
        self.secrets
            .get(id)
            .ok_or_else(|| crate::Error::UnknownSecret(id.to_owned()))?
            .as_ref()
            .map(|value| Secret(value.clone()))
            .ok_or_else(|| crate::Error::UnavailableSecret(id.to_owned()))
    }
}

impl fmt::Debug for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Configuration")
            .field("settings", &self.settings)
            .field("secrets", &"<redacted>")
            .field("saved", &self.saved)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::Configuration;

    #[test]
    fn initialization_roundtrips_declared_values() {
        let configuration: Configuration =
            serde_json::from_str(r#"{"settings":{},"secrets":{"password":"protected"},"saved":["password"]}"#).unwrap();
        let encoded = serde_json::to_vec(&configuration).unwrap();
        let decoded: Configuration = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.settings, configuration.settings);
        assert_eq!(decoded.secrets, configuration.secrets);
        assert_eq!(decoded.saved, configuration.saved);
    }
}
