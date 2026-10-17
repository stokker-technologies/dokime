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
use harmos::MemorySize;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::execution::{CycleView, ExecutionStatus, ResultSummary, TestExecution};
use crate::core::{CaseId, Metadata, TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */
pub const DOCUMENT_SCHEMA: &str = "dokime.execution.document";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionMetadata {
    pub execution_id: TestExecutionId,
    pub cycle_id: TestCycleId,
    pub case_id: CaseId,
    pub title: String,
    pub status: ExecutionStatus,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub result: Option<ResultSummary>,
    pub template_id: Option<String>,
    pub suite_label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionDocument {
    pub cycle: ExecutionCycle,
    pub execution: TestExecution,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCycle {
    pub cycle_id: TestCycleId,
    pub metadata: Metadata,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for ExecutionMetadata {
    fn memory_size(&self) -> usize {
        let fields = [
            self.execution_id.memory_size(),
            self.cycle_id.memory_size(),
            self.case_id.memory_size(),
            self.title.memory_size(),
            self.started_at.memory_size(),
            self.finished_at.memory_size(),
            self.template_id.memory_size(),
            self.suite_label.memory_size(),
        ];
        if fields.contains(&usize::MAX) {
            return usize::MAX;
        }
        let inline = std::mem::size_of::<TestExecutionId>()
            .saturating_add(std::mem::size_of::<TestCycleId>())
            .saturating_add(std::mem::size_of::<CaseId>())
            .saturating_add(std::mem::size_of::<String>())
            .saturating_add(2 * std::mem::size_of::<Option<Timestamp>>())
            .saturating_add(2 * std::mem::size_of::<Option<String>>());
        let total = fields.into_iter().fold(0usize, usize::saturating_add);
        if total == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>().saturating_add(total.saturating_sub(inline))
    }
}

impl MemorySize for ExecutionCycle {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.cycle_id.memory_size(),
                std::mem::size_of_val(&self.cycle_id),
            ),
            (
                self.metadata.memory_size(),
                std::mem::size_of_val(&self.metadata),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl MemorySize for ExecutionDocument {
    fn memory_size(&self) -> usize {
        let fields = [
            (self.cycle.memory_size(), std::mem::size_of_val(&self.cycle)),
            (
                self.execution.memory_size(),
                std::mem::size_of_val(&self.execution),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl ExecutionMetadata {
    pub fn construction_memory(execution: &TestExecution) -> usize {
        let case = execution.snapshot.case();
        let fields = [
            execution.test_execution_id.as_str().len(),
            execution.test_cycle_id.as_str().len(),
            execution.test_case_id.as_str().len(),
            case.metadata.label.len(),
            execution
                .started_at
                .as_ref()
                .map_or(0, |at| at.as_str().len()),
            execution
                .finished_at
                .as_ref()
                .map_or(0, |at| at.as_str().len()),
            case.test_case_template_id.as_str().len(),
            execution
                .snapshot
                .suite()
                .map_or(0, |suite| suite.label.len()),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add)
    }

    pub fn new(execution: &TestExecution) -> Self {
        Self {
            execution_id: execution.test_execution_id.clone(),
            cycle_id: execution.test_cycle_id.clone(),
            case_id: execution.test_case_id.clone(),
            title: execution.snapshot.case().metadata.label.clone(),
            status: execution.status.clone(),
            started_at: execution
                .started_at
                .as_ref()
                .map(|at| Timestamp::new(at.as_str().to_owned())),
            finished_at: execution
                .finished_at
                .as_ref()
                .map(|at| Timestamp::new(at.as_str().to_owned())),
            result: execution.result.clone(),
            template_id: Some(
                execution
                    .snapshot
                    .case()
                    .test_case_template_id
                    .as_str()
                    .to_owned(),
            ),
            suite_label: execution.snapshot.suite().map(|suite| suite.label.clone()),
        }
    }
}

impl ExecutionDocument {
    pub fn new(cycle: &CycleView, execution: &TestExecution) -> crate::runtime::Result<Self> {
        if cycle.test_cycle_id != execution.test_cycle_id {
            return Err(crate::runtime::Error::Operation(
                "execution document belongs to another cycle".into(),
            ));
        }
        Ok(Self {
            cycle: ExecutionCycle {
                cycle_id: cycle.test_cycle_id.clone(),
                metadata: cycle.metadata.clone(),
            },
            execution: execution.clone(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
