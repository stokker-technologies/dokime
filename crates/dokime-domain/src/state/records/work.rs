/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Lifecycle Projections Captured With Their State Transaction

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::{
    execution::result,
    identity::{TestCycleId, TestExecutionId},
    time::Timestamp,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::record(id = "dokime.history.work")]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Work {
    pub executions: Vec<Execution>,
    pub cycles: Vec<Cycle>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Execution {
    pub id: TestExecutionId,
    pub status: result::ExecutionStatus,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub result: Option<result::ResultSummary>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Cycle {
    pub id: TestCycleId,
    pub status: result::CycleStatus,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<&result::TestExecution> for Execution {
    fn from(execution: &result::TestExecution) -> Self {
        Self {
            id: execution.test_execution_id.clone(),
            status: execution.status.clone(),
            started_at: execution.started_at.clone(),
            finished_at: execution.finished_at.clone(),
            result: execution.result.clone(),
        }
    }
}

impl From<&result::TestCycle> for Cycle {
    fn from(cycle: &result::TestCycle) -> Self {
        Self {
            id: cycle.test_cycle_id.clone(),
            status: cycle.lifecycle,
        }
    }
}

impl harmos::MemorySize for Work {
    fn memory_size(&self) -> usize {
        let executions =
            self.executions.iter().fold(self.executions.capacity().saturating_mul(std::mem::size_of::<Execution>()), |bytes, execution| {
                bytes
                    .saturating_add(execution.id.as_str().len())
                    .saturating_add(execution.started_at.as_ref().map_or(0, |time| time.as_str().len()))
                    .saturating_add(execution.finished_at.as_ref().map_or(0, |time| time.as_str().len()))
            });
        let bytes = std::mem::size_of::<Self>()
            .saturating_add(executions)
            .saturating_add(self.cycles.capacity().saturating_mul(std::mem::size_of::<Cycle>()));
        self.cycles.iter().fold(bytes, |bytes, cycle| bytes.saturating_add(cycle.id.as_str().len()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
