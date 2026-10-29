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
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::execution::RecordScope;
use crate::model::identity::Metadata;
use crate::model::time::Timestamp;
use crate::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceRecord {
    pub metadata: Metadata,
    pub resource_id: String,
    pub resource_kind: String,
    pub source: String,
    pub lifecycle: ResourceLifecycle,
    pub scope: Option<RecordScope>,
    pub details: BTreeMap<String, Value>,
    pub updated_at: Timestamp,
}

/* === Enums === */

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLifecycle {
    #[default]
    Registered,

    Opened,
    Closed,
    Failed,
    Removed,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
