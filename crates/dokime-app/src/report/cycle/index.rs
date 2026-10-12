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
use crate::{Error, Result, report::CycleReportInput};
use dokime::{
    core::{
        CaseId, Identifier, Metadata, RecordScope, TemplateId, TestExecutionId, Timestamp, Value,
    },
    execution::{ExecutionStatus, ResultSummary, StepRecord, TestExecution},
};
use harmos::MemorySize;
use std::collections::{BTreeMap, BTreeSet};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub(crate) struct Case {
    pub count: usize,
    pub first_template: bool,
    pub id: CaseId,
    pub template: TemplateId,
    pub metadata: Metadata,
    pub parameters: BTreeMap<String, Value>,
    pub latest_id: TestExecutionId,
    pub status: ExecutionStatus,
}
#[derive(Clone)]
pub(crate) struct Member {
    pub id: TestExecutionId,
    pub case: CaseId,
    pub template: TemplateId,
    pub status: ExecutionStatus,
    pub result: Option<ResultSummary>,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub parameters: BTreeMap<String, Value>,
    pub scope: RecordScope,
    pub steps: Vec<StepRecord>,
}
#[derive(Clone)]
pub(crate) struct Sample {
    pub member: usize,
    pub sample: usize,
    pub stream: Identifier,
    pub timestamp: Timestamp,
    received_at: Timestamp,
    id: String,
}
#[derive(Clone)]
pub(crate) struct Index {
    pub members: usize,
    pub executions: Vec<Member>,
    pub counts: Vec<[usize; 3]>,
    pub cases: Vec<Case>,
    pub samples: Vec<Sample>,
    seen: BTreeMap<CaseId, usize>,
    templates: BTreeSet<TemplateId>,
}
/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for Member {
    fn memory_size(&self) -> usize {
        let fields = [
            (self.id.memory_size(), std::mem::size_of_val(&self.id)),
            (self.case.memory_size(), std::mem::size_of_val(&self.case)),
            (
                self.template.memory_size(),
                std::mem::size_of_val(&self.template),
            ),
            (
                self.started_at.memory_size(),
                std::mem::size_of_val(&self.started_at),
            ),
            (
                self.finished_at.memory_size(),
                std::mem::size_of_val(&self.finished_at),
            ),
            (
                self.parameters.memory_size(),
                std::mem::size_of_val(&self.parameters),
            ),
            (self.scope.memory_size(), std::mem::size_of_val(&self.scope)),
            (self.steps.memory_size(), std::mem::size_of_val(&self.steps)),
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
impl Member {
    fn construction_memory(execution: &TestExecution) -> usize {
        let input = execution.snapshot.parameters();
        let output = &execution.output_parameters;
        let count = input.len().saturating_add(output.len());
        let values = input
            .iter()
            .filter(|(key, _)| !output.contains_key(*key))
            .chain(output.iter())
            .fold(0usize, |bytes, (key, value)| {
                bytes
                    .saturating_add(key.len())
                    .saturating_add(value.value.memory_size())
            });
        std::mem::size_of::<Self>()
            .saturating_add(execution.test_execution_id.memory_size())
            .saturating_add(execution.test_case_id.memory_size())
            .saturating_add(
                execution
                    .snapshot
                    .case()
                    .test_case_template_id
                    .memory_size(),
            )
            .saturating_add(execution.started_at.memory_size())
            .saturating_add(execution.finished_at.memory_size())
            .saturating_add(execution.scope.memory_size())
            .saturating_add(execution.steps.memory_size())
            .saturating_add(harmos::btree_map_memory::<String, Value>(count))
            .saturating_add(values)
    }
    fn new(execution: &TestExecution) -> Self {
        let input = execution.snapshot.parameters();
        let output = &execution.output_parameters;
        let parameters = input
            .iter()
            .filter(|(key, _)| !output.contains_key(*key))
            .chain(output.iter())
            .map(|(key, value)| (key.clone(), value.value.clone()))
            .collect();
        Self {
            id: execution.test_execution_id.clone(),
            case: execution.test_case_id.clone(),
            template: execution.snapshot.case().test_case_template_id.clone(),
            status: execution.status.clone(),
            result: execution.result.clone(),
            started_at: execution.started_at.clone(),
            finished_at: execution.finished_at.clone(),
            parameters,
            scope: execution.scope.clone(),
            steps: execution.steps.clone(),
        }
    }
}
impl MemorySize for Sample {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.stream
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Identifier>()),
            )
            .saturating_add(
                self.timestamp
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Timestamp>()),
            )
            .saturating_add(
                self.received_at
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Timestamp>()),
            )
            .saturating_add(self.id.capacity())
    }
}
impl MemorySize for Index {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.executions
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Vec<Member>>()),
            )
            .saturating_add(
                self.counts
                    .capacity()
                    .saturating_mul(std::mem::size_of::<[usize; 3]>()),
            )
            .saturating_add(
                self.cases
                    .iter()
                    .map(|case| {
                        case.id
                            .memory_size()
                            .saturating_sub(std::mem::size_of::<CaseId>())
                            .saturating_add(
                                case.template
                                    .memory_size()
                                    .saturating_sub(std::mem::size_of::<TemplateId>()),
                            )
                            .saturating_add(
                                case.metadata
                                    .memory_size()
                                    .saturating_sub(std::mem::size_of::<Metadata>()),
                            )
                            .saturating_add(
                                case.parameters
                                    .memory_size()
                                    .saturating_sub(std::mem::size_of::<BTreeMap<String, Value>>()),
                            )
                            .saturating_add(
                                case.latest_id
                                    .memory_size()
                                    .saturating_sub(std::mem::size_of::<TestExecutionId>()),
                            )
                    })
                    .fold(0usize, usize::saturating_add),
            )
            .saturating_add(
                self.cases
                    .capacity()
                    .saturating_mul(std::mem::size_of::<Case>()),
            )
            .saturating_add(
                self.samples
                    .capacity()
                    .saturating_mul(std::mem::size_of::<Sample>()),
            )
            .saturating_add(
                self.samples
                    .iter()
                    .map(|sample| {
                        sample
                            .memory_size()
                            .saturating_sub(std::mem::size_of::<Sample>())
                    })
                    .fold(0usize, usize::saturating_add),
            )
            .saturating_add(
                harmos::btree_map_memory::<CaseId, usize>(self.seen.len())
                    .saturating_sub(std::mem::size_of_val(&self.seen))
                    .saturating_add(
                        self.seen
                            .keys()
                            .map(|id| {
                                id.memory_size()
                                    .saturating_sub(std::mem::size_of::<CaseId>())
                            })
                            .fold(0usize, usize::saturating_add),
                    ),
            )
            .saturating_add(
                harmos::btree_map_memory::<TemplateId, ()>(self.templates.len())
                    .saturating_sub(std::mem::size_of::<BTreeMap<TemplateId, ()>>())
                    .saturating_add(
                        self.templates
                            .iter()
                            .map(|id| {
                                id.memory_size()
                                    .saturating_sub(std::mem::size_of::<TemplateId>())
                            })
                            .fold(0usize, usize::saturating_add),
                    ),
            )
    }
}
impl Index {
    pub(crate) fn new() -> Self {
        Self {
            members: 0,
            executions: Vec::new(),
            counts: Vec::new(),
            cases: Vec::new(),
            samples: Vec::new(),
            seen: BTreeMap::new(),
            templates: BTreeSet::new(),
        }
    }
    pub(crate) fn capture_growth(&self, input: &CycleReportInput) -> usize {
        let execution = &input.cycle.test_executions[0];
        let case = execution.snapshot.case();
        let keys = harmos::btree_map_memory::<CaseId, usize>(1)
            .saturating_add(case.test_case_id.memory_size())
            .saturating_add(harmos::btree_map_memory::<TemplateId, ()>(1))
            .saturating_add(case.test_case_template_id.memory_size())
            .saturating_add(std::mem::size_of::<Case>())
            .saturating_add(case.test_case_id.memory_size())
            .saturating_add(case.test_case_template_id.memory_size())
            .saturating_add(case.metadata.memory_size())
            .saturating_add(case.parameters.memory_size())
            .saturating_add(execution.test_execution_id.memory_size());
        let samples = input.telemetry.iter().fold(0usize, |bytes, sample| {
            bytes
                .saturating_add(std::mem::size_of::<Sample>())
                .saturating_add(sample.stream.memory_size())
                .saturating_add(sample.timestamp.memory_size())
                .saturating_add(sample.received_at.memory_size())
                .saturating_add(sample.sample_id.as_str().len())
        });
        self.memory_size()
            .saturating_add(Member::construction_memory(execution))
            .saturating_add(std::mem::size_of::<[usize; 3]>())
            .saturating_add(keys)
            .saturating_add(samples)
    }
    pub(crate) fn push_capture(&mut self, input: &CycleReportInput) -> Result<()> {
        let execution = &input.cycle.test_executions[0];
        let case = execution.snapshot.case();
        if let Some(index) = self.seen.get(&case.test_case_id).copied() {
            self.cases[index].count += 1;
            self.cases[index].latest_id = execution.test_execution_id.clone();
            self.cases[index].status = execution.status.clone();
        } else {
            self.cases
                .try_reserve_exact(1)
                .map_err(|source| Error::Operation(source.to_string()))?;
            self.seen
                .insert(case.test_case_id.clone(), self.cases.len());
            self.cases.push(Case {
                count: 1,
                first_template: self.templates.insert(case.test_case_template_id.clone()),
                id: case.test_case_id.clone(),
                template: case.test_case_template_id.clone(),
                metadata: case.metadata.clone(),
                parameters: case.parameters.clone(),
                latest_id: execution.test_execution_id.clone(),
                status: execution.status.clone(),
            });
        }
        self.samples
            .try_reserve_exact(input.telemetry.len())
            .map_err(|source| Error::Operation(source.to_string()))?;
        for (sample, value) in input.telemetry.iter().enumerate() {
            self.samples.push(Sample {
                member: self.members,
                sample,
                stream: value.stream.clone(),
                timestamp: value.timestamp.clone(),
                received_at: value.received_at.clone(),
                id: value.sample_id.to_string(),
            });
        }
        self.executions
            .try_reserve_exact(1)
            .map_err(|source| Error::Operation(source.to_string()))?;
        self.executions.push(Member::new(execution));
        self.counts
            .try_reserve_exact(1)
            .map_err(|source| Error::Operation(source.to_string()))?;
        self.counts.push([
            input.artifact_links.len(),
            input.records.len(),
            crate::report::document::document_warnings(input).count(),
        ]);
        self.members += 1;
        Ok(())
    }
    pub(crate) fn sort(&mut self) {
        self.samples.sort_unstable_by(|left, right| {
            left.stream
                .cmp(&right.stream)
                .then_with(|| left.timestamp.cmp(&right.timestamp))
                .then_with(|| left.received_at.cmp(&right.received_at))
                .then_with(|| left.id.cmp(&right.id))
                .then_with(|| left.member.cmp(&right.member))
                .then_with(|| left.sample.cmp(&right.sample))
        });
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
