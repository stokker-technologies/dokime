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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HostCardBody {
    #[serde(default)]
    pub blocks: Vec<HostCardBlock>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostCardKeyValue {
    pub label: String,
    pub value: String,

    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostCardMetric {
    pub label: String,
    pub value: String,

    #[serde(default)]
    pub unit: Option<String>,

    #[serde(default)]
    pub trend: Option<String>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HostCardBlock {
    Text {
        text: String,
    },
    Status {
        label: String,
        tone: HostCardTone,

        #[serde(default)]
        detail: Option<String>,
    },
    KeyValueList {
        #[serde(default)]
        items: Vec<HostCardKeyValue>,
    },
    Metrics {
        #[serde(default)]
        items: Vec<HostCardMetric>,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostCardTone {
    #[default]
    Neutral,

    Info,
    Success,
    Warning,
    Danger,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
