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

pub use crate::domains::PluginListener as ListenerRegistration;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::domains::{AppCustomizationContribution, AppThemeContribution};

/* === Constants === */

pub const SUITES_NAMESPACE: &str = "dokime.suites";
pub const PERMISSIONS_NAMESPACE: &str = "dokime.permissions";
pub const PROMPTS_NAMESPACE: &str = "dokime.prompts";
pub const RECORDS_NAMESPACE: &str = "dokime.records";
pub const LISTENERS_NAMESPACE: &str = "dokime.listeners";
pub const TELEMETRY_NAMESPACE: &str = "dokime.telemetry";
pub const SETTINGS_NAMESPACE: &str = "dokime.settings";
pub const ANALYSES_NAMESPACE: &str = "dokime.analyses";
pub const OPTIONS_NAMESPACE: &str = "dokime.options";
pub const APPEARANCE_NAMESPACE: &str = "dokime.appearance";
pub const DOMAIN_SCHEMA: u32 = 1;
pub const ROUTE_SCHEMA: u32 = 2;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SettingsRegistration {
    #[serde(default)]
    pub status_entrypoint: Option<crate::domains::PluginEntrypoint>,

    #[serde(default)]
    pub catalog: crate::settings::Settings,

    #[serde(default)]
    pub discoveries: Vec<crate::plugin::settings::Discovery>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppearanceRegistration {
    #[serde(default)]
    pub customizations: Vec<AppCustomizationContribution>,

    #[serde(default)]
    pub themes: Vec<AppThemeContribution>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn domain_schema(namespace: &str) -> u32 {
    match namespace {
        SUITES_NAMESPACE => 2,
        LISTENERS_NAMESPACE => 2,
        OPTIONS_NAMESPACE => ROUTE_SCHEMA,
        SETTINGS_NAMESPACE => 3,
        _ => DOMAIN_SCHEMA,
    }
}

/* === Private Functions === */

/* === Tests === */
