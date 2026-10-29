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
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Local Imports
use super::PluginMetadata;
use crate::settings::SettingValues;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginEntry {
    #[serde(flatten)]
    pub installation: Installation,

    pub metadata: Option<PluginMetadata>,
    pub error: Option<String>,

    #[serde(default)]
    pub active: bool,

    #[serde(default)]
    pub restart_required: bool,

    #[serde(default)]
    pub boot_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installation {
    pub install_id: Uuid,
    pub sha256: String,
    pub installed_at: Timestamp,
    pub state: PluginState,
    pub last_boot: Option<PluginLastBoot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginSettings {
    pub state: PluginState,
    pub digest: String,
    pub installed_at: Timestamp,

    #[serde(default)]
    pub last_boot: Option<PluginLastBoot>,

    #[serde(default)]
    pub programs: BTreeMap<String, ProgramValues>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProgramValues {
    pub values: SettingValues,
    pub saved: BTreeSet<String>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginState {
    Installed,
    Disabled,
    RemovalPending,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum PluginLastBoot {
    Loaded { at: Timestamp },
    Failed { error: String, at: Timestamp },
}

/* === Traits === */

/* === Implementations === */

impl PluginSettings {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.digest.len() != 64 || !self.digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
            return Err("digest must be lowercase hexadecimal SHA-256".into());
        }
        Ok(())
    }

    pub(crate) fn installation(&self, install_id: Uuid) -> Installation {
        Installation {
            install_id,
            sha256: self.digest.clone(),
            installed_at: self.installed_at.clone(),
            state: self.state,
            last_boot: self.last_boot.clone(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
