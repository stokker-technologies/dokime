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
use crate::model::definition::SuiteSet;
use crate::model::plugins::AppCustomizationContribution;
use crate::model::plugins::AppThemeContribution;
use crate::model::plugins::OptionProviderContribution;
use crate::model::plugins::Permission;
use crate::model::plugins::PluginEntrypoint;
use crate::model::prompts::PromptDefinition;
use crate::model::records::RecordContract;
use crate::model::settings::Settings;
use crate::model::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub source: String,
    pub version: String,
    pub description: String,

    #[serde(default = "enabled")]
    pub enabled: bool,

    #[serde(default)]
    pub suites: SuiteSet,

    #[serde(default)]
    pub settings: Settings,

    #[serde(default)]
    pub permissions: Vec<Permission>,

    #[serde(default)]
    pub record_contracts: Vec<RecordContract>,

    #[serde(default)]
    pub listeners: Vec<PluginListener>,

    #[serde(default)]
    pub prompts: Vec<PromptDefinition>,

    #[serde(default)]
    pub telemetry_contracts: Vec<TelemetryContract>,

    #[serde(default)]
    pub settings_status: Option<PluginEntrypoint>,

    #[serde(default)]
    pub settings_discoveries: Vec<SettingDiscovery>,

    #[serde(default)]
    pub app_customizations: Vec<AppCustomizationContribution>,

    #[serde(default)]
    pub app_themes: Vec<AppThemeContribution>,

    #[serde(default)]
    pub option_providers: Vec<OptionProviderContribution>,

    #[serde(default)]
    pub analyses: Vec<crate::model::analysis::Definition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginListener {
    pub operation: String,
    pub record: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingDiscovery {
    pub field: String,
    pub operation: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl InstalledPlugin {
    pub fn listened_records(&self) -> impl Iterator<Item = &str> {
        self.listeners.iter().map(|listener| listener.record.as_str())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn enabled() -> bool {
    true
}

/* === Tests === */
