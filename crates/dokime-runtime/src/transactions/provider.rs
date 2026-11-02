/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod request;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::definition::{TestCase, TestCaseTemplate};
use crate::document::DocumentDefinition;
use crate::domains::ExecutionProviderTarget;
use crate::{runtime::Error, state::RuntimeState};
use dokime_domain::model::execution::CommandContext;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CriterionResult;
use dokime_domain::model::execution::result::CriterionStatus;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::ResultStatus;
use dokime_domain::model::execution::result::ResultSummary;
use dokime_domain::model::execution::result::StepRecord;
use dokime_domain::model::execution::result::StepStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::StepId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;
use dokime_domain::state::records::set::TestRecordSet;

/* === Constants === */

pub(crate) const EXECUTION_NOT_PLANNED: &str = "case has no unstarted execution; schedule a rerun first";
pub(crate) const PROVIDER_UNAVAILABLE: &str = "every selected case requires a provider execution binding";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.provider.run")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunProvider {
    pub mode: dokime_domain::model::execution::ExecutionMode,
    pub test_cycle_id: TestCycleId,
    pub test_execution_id: Option<TestExecutionId>,
    pub context: CommandContext,
    pub command_id: CommandId,
    pub accepted_at: Timestamp,
}

pub(super) struct RunPlan {
    pub(super) target: ExecutionProviderTarget,
    pub(super) execution_id: TestExecutionId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for RunProvider {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if let Some(request) = state.run_requests.get(&self.command_id) {
            return if request.summary.cycle_id == self.test_cycle_id
                && request.test_execution_id == self.test_execution_id
                && request.summary.mode == self.mode
            {
                Ok(())
            } else {
                Err("request identity belongs to another selection".into())
            };
        }
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        prepare_run_provider(state, self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        crate::transactions::history::collect(state, self, Some(&self.test_cycle_id), out);
        request::queue_records(state, self).collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        apply_run_provider(state, self).expect("Harmos checked provider admission before applying it");
    }
}

/* === Public Functions === */

pub(crate) fn execution_available(state: &RuntimeState, execution: &TestExecution) -> bool {
    execution.status == ExecutionStatus::Skipped
        && execution.started_at.is_none()
        && execution.finished_at.is_none()
        && !state
            .run_requests
            .values()
            .any(|request| !request.summary.phase.terminal() && request.selected_execution_ids.contains(&execution.test_execution_id))
}

pub fn apply_run_provider(state: &mut RuntimeState, run: &RunProvider) -> Result<ExecutionProviderTarget, Error> {
    request::queue(state, run)
}

pub(super) fn materialize_execution(
    state: &RuntimeState, cycle: &CycleView, case: &TestCase, execution_id: TestExecutionId,
    plugin_identity: Option<dokime_domain::model::execution::snapshot::ExecutionPluginIdentity>, settings: BTreeMap<String, Value>,
) -> Result<TestExecution, Error> {
    // The authored definition is the registry's suite at the case's revision.
    let suite = state.suites.suite_for_case(case).map(|entry| &entry.suite);
    let template = suite
        .and_then(|suite| suite.test_case_templates.iter().find(|template| template.test_case_template_id == case.test_case_template_id));
    if case.suite.is_some() && suite.is_none() {
        return Err(Error::validation("missing_suite_definition", "case suite definition is unavailable", vec![]));
    }
    if suite.is_some() && template.is_none() {
        return Err(Error::validation("missing_template_definition", "case template does not belong to the supplied suite", vec![]));
    }
    let snapshot = dokime_domain::model::execution::snapshot::ExecutionSnapshot::materialize(
        case,
        suite,
        execution_documents(case, template),
        plugin_identity,
        settings,
    )
    .map_err(|error| Error::validation("invalid_parameter", error.to_string(), vec![]))?
    .with_host_version(Some(env!("CARGO_PKG_VERSION").to_owned()));

    let execution = scheduled_execution_record(&cycle.test_cycle_id, execution_id, snapshot);

    for definition in execution.snapshot.parameter_definitions() {
        let name = definition.metadata.identifier.as_str();
        let value = execution.snapshot.parameters().get(name).map(|parameter| parameter.value.clone()).unwrap_or(Value::Null);
        crate::definition::materialized_parameter(definition, crate::definition::ParameterSource::Definition, value)
            .map_err(|error| Error::validation("invalid_parameter", error.to_string(), vec![]))?;
    }
    Ok(execution)
}

pub(crate) fn scheduled_execution_record(
    cycle_id: &TestCycleId, execution_id: TestExecutionId, snapshot: dokime_domain::model::execution::snapshot::ExecutionSnapshot,
) -> TestExecution {
    let case = snapshot.case();
    let template = snapshot.template();
    let scope = RecordScope::default()
        .with_cycle(cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(execution_id.clone())
        .with_template(case.test_case_template_id.clone());
    TestExecution {
        saved: None,
        mode: Default::default(),
        active_step_attempt: None,
        capture_phase: Default::default(),
        step_attempts: Vec::new(),
        test_cycle_id: cycle_id.clone(),
        test_execution_id: execution_id.clone(),
        test_case_id: case.test_case_id.clone(),
        suite: case.suite.clone(),
        status: ExecutionStatus::Pending,
        scope,
        invalidation: None,
        output_parameters: BTreeMap::new(),
        documents: execution_documents(case, template),
        records: TestRecordSet::default(),
        started_at: None,
        finished_at: None,
        steps: scheduled_steps(template, &execution_id),
        traceability: Vec::new(),
        result: Some(summary(ResultStatus::Pending)),
        snapshot,
    }
}

/* === Private Functions === */

fn prepare_run_provider(state: &RuntimeState, run: &RunProvider) -> Result<RunPlan, Error> {
    let id = run
        .test_execution_id
        .as_ref()
        .ok_or_else(|| Error::conflict("execution_selection_required", "start requires one explicit execution"))?;
    let cycle = state.cycles.get(&run.test_cycle_id).ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
    cycle.ensure_active().map_err(|reason| Error::conflict("cycle_closed", reason))?;
    if cycle.execution_ids.is_empty() {
        return Err(Error::conflict("empty_cycle", "cycle has no test cases; append executions before starting work"));
    }
    if !cycle.execution_ids.contains(id) {
        return Err(Error::conflict("execution_not_planned", "execution is not a member of its cycle"));
    }
    let execution =
        state.executions.get(id).ok_or_else(|| Error::conflict("execution_not_planned", "cycle references a missing execution owner"))?;
    if execution.test_cycle_id != run.test_cycle_id {
        return Err(Error::conflict("execution_owner_mismatch", "planned execution belongs to another cycle"));
    }
    if !execution_available(state, execution) {
        return Err(Error::conflict("execution_not_planned", EXECUTION_NOT_PLANNED));
    }
    let target = state
        .suites
        .provider_target_for_case(execution.snapshot.case())
        .ok_or_else(|| Error::unavailable("runtime_unavailable", PROVIDER_UNAVAILABLE))?;
    let identity = state.execution_identity(&target.source);
    if execution.snapshot.plugin() != identity.as_ref() {
        return Err(Error::conflict("provider_identity_changed", "the materialized execution belongs to another plugin revision"));
    }
    Ok(RunPlan { target, execution_id: id.clone() })
}

fn scheduled_steps(template: Option<&TestCaseTemplate>, execution_id: &TestExecutionId) -> Vec<StepRecord> {
    let mut steps = template
        .map(|template| {
            template
                .phases
                .iter()
                .map(|phase| StepRecord {
                    step_id: phase.step_id.clone(),
                    metadata: phase.metadata.clone(),
                    status: StepStatus::Pending,
                    started_at: None,
                    finished_at: None,
                    measurements: Vec::new(),
                    criteria: Vec::new(),
                    detail: None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if steps.is_empty() {
        steps.push(StepRecord {
            step_id: StepId::from_seed(format!("provider-scheduled:{execution_id}:execution")),
            metadata: Metadata::new(
                identifier("provider.execution"),
                "Provider Execution",
                template.map(|template| template.metadata.label.clone()),
            ),
            status: StepStatus::Pending,
            started_at: None,
            finished_at: None,
            measurements: Vec::new(),
            criteria: Vec::new(),
            detail: None,
        });
    }
    if let (Some(template), Some(step)) = (template, steps.last_mut()) {
        step.criteria = template
            .criteria
            .iter()
            .map(|criterion| CriterionResult {
                criterion_id: criterion.criterion_id.clone(),
                metadata: criterion.metadata.clone(),
                status: CriterionStatus::Pending,
                observed: None,
                message: None,
            })
            .collect();
    }
    steps
}

fn execution_documents(
    case: &TestCase, template: Option<&TestCaseTemplate>,
) -> BTreeMap<dokime_domain::model::identity::DocumentId, DocumentDefinition> {
    let id = dokime_domain::model::identity::DocumentId::from_identifier("default").expect("default document id is valid");
    let mut document = DocumentDefinition::new(
        "default",
        template.map(|template| template.metadata.label.clone()).unwrap_or_else(|| case.metadata.label.clone()),
    );
    if let Some(authored) = template.and_then(|template| template.document.as_ref()) {
        document.merge_content_from(authored);
    }
    BTreeMap::from([(id, document)])
}

fn summary(status: ResultStatus) -> ResultSummary {
    ResultSummary {
        passed: u32::from(status == ResultStatus::Passed),
        failed: u32::from(status == ResultStatus::Failed),
        blocked: u32::from(status == ResultStatus::Blocked),
        skipped: 0,
        status,
    }
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).unwrap_or_else(|_| Identifier::new("provider.invalid_identifier").expect("fallback identifier is valid"))
}

/* === Tests === */
