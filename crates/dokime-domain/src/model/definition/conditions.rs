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

use crate::model::records::RecordCondition;
use crate::model::telemetry::TelemetryCondition;

pub type RuntimeConditionEvaluation = crate::model::validation::ConditionEvaluation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeCondition {
    Record {
        #[serde(flatten)]
        condition: RecordCondition,
    },
    Telemetry {
        #[serde(flatten)]
        condition: TelemetryCondition,
    },
    All {
        conditions: Vec<RuntimeCondition>,
    },
    Any {
        conditions: Vec<RuntimeCondition>,
    },
}

impl harmos_domain::MemorySize for RuntimeCondition {
    fn memory_size(&self) -> usize {
        condition_memory(self, 0)
    }
}

fn condition_memory(condition: &RuntimeCondition, depth: usize) -> usize {
    if depth > 64 {
        return usize::MAX;
    }
    let heap = match condition {
        RuntimeCondition::Record { condition } => {
            let owned = harmos_domain::MemorySize::memory_size(condition);
            if owned == usize::MAX {
                return usize::MAX;
            }
            owned.saturating_sub(std::mem::size_of_val(condition))
        }
        RuntimeCondition::Telemetry { condition } => {
            let owned = harmos_domain::MemorySize::memory_size(condition);
            if owned == usize::MAX {
                return usize::MAX;
            }
            owned.saturating_sub(std::mem::size_of_val(condition))
        }
        RuntimeCondition::All { conditions } | RuntimeCondition::Any { conditions } => {
            let mut bytes = conditions.capacity().saturating_mul(std::mem::size_of::<RuntimeCondition>());
            for condition in conditions {
                let owned = condition_memory(condition, depth + 1);
                if owned == usize::MAX {
                    return usize::MAX;
                }
                bytes = bytes.saturating_add(owned.saturating_sub(std::mem::size_of::<RuntimeCondition>()));
            }
            bytes
        }
    };
    std::mem::size_of::<RuntimeCondition>().saturating_add(heap)
}
