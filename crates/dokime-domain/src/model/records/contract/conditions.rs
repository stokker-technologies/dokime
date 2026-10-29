/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::model::identity::Identifier;
use crate::model::values::Value;

pub type RecordConditionEvaluation = crate::model::validation::ConditionEvaluation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordCondition {
    pub record_type: Identifier,

    #[serde(default, rename = "match")]
    pub match_fields: BTreeMap<String, Value>,
}

impl harmos::MemorySize for RecordCondition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.record_type), std::mem::size_of_val(&self.record_type)),
            (harmos::MemorySize::memory_size(&self.match_fields), std::mem::size_of_val(&self.match_fields)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}
