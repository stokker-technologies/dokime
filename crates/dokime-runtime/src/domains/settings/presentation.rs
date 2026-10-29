/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use dokime_domain::model::appearance::HostCardBody;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

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
