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
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::secrets::SecretAvailability;

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

    pub secrets: Vec<SecretAvailability>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PluginSettingsUpdate {
    #[serde(default)]
    pub values: SettingValues,
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

impl From<crate::storage::Error> for RuntimePluginManagementError {
    fn from(error: crate::storage::Error) -> Self {
        Self::Store(error.to_string())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
