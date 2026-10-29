/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::PluginEntrypoint;
use crate::model::identity::Metadata;
use crate::model::plugins::Permission;
use crate::model::values::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptionProviderContribution {
    pub metadata: Metadata,

    #[serde(default)]
    pub input_schema: Option<Value>,

    #[serde(default)]
    pub permissions: Vec<Permission>,

    pub source: OptionProviderSource,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OptionProviderOption {
    pub value: String,
    pub label: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub disabled: bool,

    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OptionProviderSource {
    Static {
        #[serde(default)]
        options: Vec<OptionProviderOption>,
    },
    Runtime {
        entrypoint: PluginEntrypoint,
    },
}
