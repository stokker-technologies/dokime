/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::execution::RecordScope;
use crate::model::identity::Metadata;
use crate::model::time::Timestamp;
use crate::model::values::Value;

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
