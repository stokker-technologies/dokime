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
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use crate::{report::saved::CapturedExecutionReportInput, runtime::Error, runtime::Result};
use dokime_domain::model::definition::parameters::ParameterDefinition;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::ResultSummary;
use dokime_domain::model::execution::result::StepRecord;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::TemplateId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

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
    pub definitions: Vec<ParameterDefinition>,
    pub latest_id: TestExecutionId,
    pub status: ExecutionStatus,
    pub planned: bool,
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
    pub generated_at: Timestamp,
    pub planned: bool,
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
pub(crate) struct Annotation {
    pub kind: AnnotationKind,
    pub member: usize,
    pub position: usize,
    id: String,
    at: Timestamp,
}

#[derive(Clone)]
pub(crate) struct Index {
    pub members: usize,
    pub executions: Vec<Member>,
    pub counts: Vec<[usize; 3]>,
    pub cases: Vec<Case>,
    pub samples: Vec<Sample>,
    pub annotations: Vec<Annotation>,
    seen: BTreeMap<CaseId, usize>,
    templates: BTreeSet<TemplateId>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AnnotationKind {
    Note,
    Health,
}

/* === Traits === */

/* === Implementations === */

impl Annotation {}

impl Member {
    fn new(execution: &TestExecution, generated_at: &Timestamp, planned: bool) -> Self {
        let input = execution.snapshot.parameters();
        let output = &execution.output_parameters;
        let mut parameters = BTreeMap::new();
        for (key, value) in input.iter().filter(|(key, _)| !output.contains_key(*key)).chain(output.iter()) {
            parameters.insert(key.clone(), value.value.clone());
        }
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
            generated_at: generated_at.clone(),
            planned,
        }
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
            annotations: Vec::new(),
            seen: BTreeMap::new(),
            templates: BTreeSet::new(),
        }
    }
    pub(crate) fn push_capture(&mut self, captured: &CapturedExecutionReportInput) -> Result<()> {
        let input = &captured.input;
        let execution = &input.cycle.test_executions[0];
        self.push_case(execution, false)?;
        self.samples.try_reserve_exact(input.telemetry.len()).map_err(|source| Error::Operation(source.to_string()))?;
        for (sample, value) in input.telemetry.iter().enumerate() {
            self.samples.push(Sample {
                member: self.members,
                sample,
                stream: value.stream.clone(),
                timestamp: value.timestamp.clone(),
                received_at: value.received_at.clone(),
                id: value.sample_id.as_str().to_owned(),
            });
        }
        let context = &captured.context;
        self.annotations
            .try_reserve_exact(context.notes.len().saturating_add(context.health.len()))
            .map_err(|source| Error::Operation(source.to_string()))?;
        for (position, note) in context.notes.iter().enumerate() {
            self.annotations.push(Annotation {
                kind: AnnotationKind::Note,
                member: self.members,
                position,
                id: note.note_id.as_str().to_owned(),
                at: note.created_at.clone(),
            });
        }
        for (position, item) in context.health.iter().enumerate() {
            self.annotations.push(Annotation {
                kind: AnnotationKind::Health,
                member: self.members,
                position,
                id: item.health_item_id.as_str().to_owned(),
                at: item.observed_at.clone(),
            });
        }
        self.push_execution(execution, &input.generated_at, false)?;
        self.counts.try_reserve_exact(1).map_err(|source| Error::Operation(source.to_string()))?;
        self.counts.push([input.artifact_links.len(), input.records.len(), crate::report::document::document_warnings(input).count()]);
        self.members += 1;
        Ok(())
    }
    // A Planned Member Is A Cycle Row With Its Parameters And No Captured Evidence.
    pub(crate) fn push_planned(&mut self, execution: &TestExecution, generated_at: &Timestamp) -> Result<()> {
        self.push_case(execution, true)?;
        self.push_execution(execution, generated_at, true)
    }
    fn push_case(&mut self, execution: &TestExecution, planned: bool) -> Result<()> {
        let case = execution.snapshot.case();
        if let Some(index) = self.seen.get(&case.test_case_id).copied() {
            self.cases[index].count += 1;
            self.cases[index].latest_id = execution.test_execution_id.clone();
            self.cases[index].status = execution.status.clone();
            self.cases[index].planned = planned;
            return Ok(());
        }
        self.cases.try_reserve_exact(1).map_err(|source| Error::Operation(source.to_string()))?;
        self.seen.insert(case.test_case_id.clone(), self.cases.len());
        self.cases.push(Case {
            count: 1,
            first_template: self.templates.insert(case.test_case_template_id.clone()),
            id: case.test_case_id.clone(),
            template: case.test_case_template_id.clone(),
            metadata: case.metadata.clone(),
            parameters: case.parameters.clone(),
            definitions: execution.snapshot.parameter_definitions().to_vec(),
            latest_id: execution.test_execution_id.clone(),
            status: execution.status.clone(),
            planned,
        });
        Ok(())
    }
    fn push_execution(&mut self, execution: &TestExecution, generated_at: &Timestamp, planned: bool) -> Result<()> {
        self.executions.try_reserve_exact(1).map_err(|source| Error::Operation(source.to_string()))?;
        self.executions.push(Member::new(execution, generated_at, planned));
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
        // Members Repeat Cycle Annotations; Keep Each Identity Once At Its Latest
        // Observation, Later Members Winning Ties, Then Restore Member Order.
        self.annotations.sort_unstable_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.id.cmp(&right.id))
                .then_with(|| right.at.cmp(&left.at))
                .then_with(|| right.member.cmp(&left.member))
                .then_with(|| right.position.cmp(&left.position))
        });
        self.annotations.dedup_by(|later, kept| later.kind == kept.kind && later.id == kept.id);
        self.annotations.sort_unstable_by(|left, right| {
            left.kind.cmp(&right.kind).then_with(|| left.member.cmp(&right.member)).then_with(|| left.position.cmp(&right.position))
        });
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
