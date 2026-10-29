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

use crate::model::identity::Identifier;
use crate::model::values::Value;

pub type TelemetryConditionEvaluation = crate::model::validation::ConditionEvaluation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryCondition {
    pub stream: Identifier,
    pub field: String,
    pub operator: TelemetryConditionOperator,
    pub target: Option<Value>,
    pub tolerance: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryConditionOperator {
    Exists,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Within,
}

impl harmos::MemorySize for TelemetryCondition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.stream), std::mem::size_of_val(&self.stream)),
            (harmos::MemorySize::memory_size(&self.field), std::mem::size_of_val(&self.field)),
            (harmos::MemorySize::memory_size(&self.target), std::mem::size_of_val(&self.target)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}
