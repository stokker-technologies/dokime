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
// Local Imports
use dokime_domain::model::appearance::HostCardBody;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SettingsSectionContribution {
    pub metadata: Metadata,
    #[serde(default)]
    pub order: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SettingsCardContribution {
    pub metadata: Metadata,
    pub section_id: Identifier,
    pub body: HostCardBody,
    #[serde(default)]
    pub refresh: SettingsCardRefreshPolicy,
    #[serde(default)]
    pub order: Option<i32>,
}

/* === Enums === */
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SettingsCardRefreshPolicy {
    #[default]
    Manual,
    Cached {
        max_age_ms: u64,
    },
    OnDemand,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
