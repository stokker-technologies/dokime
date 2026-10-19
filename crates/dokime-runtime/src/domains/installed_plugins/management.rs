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
use serde::{Deserialize, Serialize};
use thiserror::Error;
// Local Imports
use crate::settings::{SettingValues, Settings};
use dokime_domain::model::credentials::RedactedCredentialBinding;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimePluginSettings {
    pub settings: Settings,
    pub values: SettingValues,
    #[serde(default)]
    pub discovered: SettingValues,
    pub credential_bindings: PluginRuntimeCredentialBindings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginSettingsUpdate {
    #[serde(default)]
    pub values: SettingValues,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginCredentialSecretUpdate {
    pub secret: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PluginRuntimeCredentialBindings {
    #[serde(default)]
    pub bindings: std::collections::BTreeMap<String, RedactedCredentialBinding>,
    #[serde(default)]
    pub missing_required: Vec<String>,
}

/* === Enums === */
#[derive(Clone, Debug, Error, PartialEq)]
pub enum RuntimePluginManagementError {
    #[error("installed plugin {0} was not found")]
    NotFound(Identifier),

    #[error("installed plugin input is invalid: {0}")]
    InvalidInput(String),

    #[error("installed plugin store operation failed: {0}")]
    Store(String),
}

/* === Traits === */

/* === Implementations === */
impl From<std::io::Error> for RuntimePluginManagementError {
    fn from(error: std::io::Error) -> Self {
        Self::Store(error.to_string())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
