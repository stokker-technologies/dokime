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
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use super::SettingsError;
use crate::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecretSetting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub default: Option<SecretReference>,
    #[serde(default)]
    pub required: bool,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecretReference {
    Environment { variable: String },
    StoredCredential { credential_id: String },
}

/* === Traits === */

/* === Implementations === */
impl SecretReference {
    pub fn validate(&self) -> crate::Result<()> {
        let valid = match self {
            Self::Environment { variable } => {
                !variable.is_empty()
                    && variable.bytes().enumerate().all(|(i, c)| c == b'_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
            }
            Self::StoredCredential { credential_id } => !credential_id.trim().is_empty() && !credential_id.contains('\0'),
        };
        if valid { Ok(()) } else { Err(SettingsError::invalid("secret_reference", "invalid secret reference").into()) }
    }
}

impl TryFrom<&Value> for SecretReference {
    type Error = crate::Error;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let reference: Self =
            serde_json::from_value(value.to_plain_json().map_err(|_| SettingsError::invalid("secret_reference", "invalid reference"))?)
                .map_err(|_| SettingsError::invalid("secret_reference", "expected a secret reference, never secret material"))?;
        reference.validate()?;
        Ok(reference)
    }
}

impl From<SecretReference> for Value {
    fn from(reference: SecretReference) -> Self {
        match reference {
            SecretReference::Environment { variable } => Self::Object(std::collections::BTreeMap::from([
                ("kind".into(), "environment".into()),
                ("variable".into(), variable.into()),
            ])),
            SecretReference::StoredCredential { credential_id } => Self::Object(std::collections::BTreeMap::from([
                ("kind".into(), "stored_credential".into()),
                ("credential_id".into(), credential_id.into()),
            ])),
        }
    }
}

impl SecretSetting {
    pub fn validate(&self) -> crate::Result<()> {
        if matches!(self.default, Some(SecretReference::StoredCredential { .. })) {
            return Err(SettingsError::invalid(&self.id, "stored credentials require an explicit operator binding").into());
        }
        if let Some(reference) = &self.default {
            reference.validate()?;
        }
        Ok(())
    }
    pub fn default_value(&self) -> Option<Value> {
        self.default.clone().map(Into::into)
    }
    pub fn validate_value(&self, value: &Value) -> crate::Result<()> {
        SecretReference::try_from(value).map(|_| ()).map_err(|error| SettingsError::invalid(&self.id, error.to_string()).into())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
