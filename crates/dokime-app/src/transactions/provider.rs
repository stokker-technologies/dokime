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
use crate::execution::{
    CriterionResult, CriterionStatus, CycleStatus, CycleView, ExecutionStatus, ResultStatus,
    ResultSummary, StepRecord, StepStatus, TestExecution,
};
use crate::records::TestRecordSet;
use crate::{Error, state::RuntimeState};
use dokime::core::{
    CaseId, CommandContext, CommandId, Identifier, Metadata, RecordScope, StepId, TestCycleId,
    TestExecutionId, Timestamp, Value,
};
use dokime::domains::ExecutionProviderTarget;

/* === Constants === */
pub(crate) const CYCLE_NOT_READY: &str = "cycle is not ready to run test cases";
pub(crate) const EXECUTION_NOT_PLANNED: &str =
    "case has no unstarted execution; schedule a rerun first";
pub(crate) const PROVIDER_UNAVAILABLE: &str =
    "every selected case requires a provider execution binding";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.execution.provider.run", version = 4, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunProvider {
    pub mode: crate::execution::ExecutionMode,
    pub test_cycle_id: TestCycleId,
    pub test_execution_id: Option<TestExecutionId>,
    pub context: CommandContext,
    pub command_id: CommandId,
    pub accepted_at: Timestamp,
    #[serde(default)]
    pub plugin_identity: Option<crate::execution::ExecutionPluginIdentity>,
}

pub(super) struct RunPlanGroup {
    pub(super) target: ExecutionProviderTarget,
    pub(super) execution_ids: Vec<TestExecutionId>,
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
        prepare_run_provider(state, self)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        request::queue_records(state, self).collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        apply_run_provider(state, self)
            .expect("Harmos checked provider admission before applying it");
    }
}

/* === Public Functions === */
pub(crate) fn ensure_case_runnable(
    state: &RuntimeState,
    cycle: &CycleView,
    test_case_id: &CaseId,
) -> Result<(), String> {
    if !cycle_admits_run(cycle.lifecycle) {
        return Err(CYCLE_NOT_READY.to_owned());
    }
    let execution = planned_attempt(cycle, test_case_id).ok_or(EXECUTION_NOT_PLANNED)?;
    if state
        .suites
        .provider_target_for_case(execution.snapshot.case())
        .is_none()
    {
        return Err(PROVIDER_UNAVAILABLE.to_owned());
    }
    Ok(())
}

pub(crate) fn planned_attempt<'a>(
    cycle: &'a CycleView,
    test_case_id: &CaseId,
) -> Option<&'a TestExecution> {
    cycle.test_executions.iter().find(|execution| {
        &execution.test_case_id == test_case_id
            && execution.status == ExecutionStatus::Skipped
            && execution.started_at.is_none()
            && execution.finished_at.is_none()
    })
}

pub(crate) fn cycle_admits_run(lifecycle: CycleStatus) -> bool {
    lifecycle == CycleStatus::Ready
}

pub fn apply_run_provider(
    state: &mut RuntimeState,
    run: &RunProvider,
) -> Result<ExecutionProviderTarget, Error> {
    request::queue(state, run)
}

pub(super) fn materialize_execution(
    state: &RuntimeState,
    cycle: &CycleView,
    case: &TestCase,
    execution_id: TestExecutionId,
    plugin_identity: Option<crate::execution::ExecutionPluginIdentity>,
    settings: BTreeMap<String, Value>,
) -> Result<TestExecution, Error> {
    // The authored definition is the registry's suite at the case's revision.
    let suite = state.suites.suite_for_case(case).map(|entry| &entry.suite);
    let template = suite.and_then(|suite| {
        suite
            .test_case_templates
            .iter()
            .find(|template| template.test_case_template_id == case.test_case_template_id)
    });
    if case.suite.is_some() && suite.is_none() {
        return Err(Error::validation(
            "missing_suite_definition",
            "case suite definition is unavailable",
            vec![],
        ));
    }
    if suite.is_some() && template.is_none() {
        return Err(Error::validation(
            "missing_template_definition",
            "case template does not belong to the supplied suite",
            vec![],
        ));
    }
    let snapshot = crate::execution::ExecutionSnapshot::materialize(
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
        let value = execution
            .snapshot
            .parameters()
            .get(name)
            .map(|parameter| parameter.value.clone())
            .unwrap_or(Value::Null);
        crate::definition::materialized_parameter(
            definition,
            crate::definition::ParameterSource::Definition,
            value,
        )
        .map_err(|error| Error::validation("invalid_parameter", error.to_string(), vec![]))?;
    }
    Ok(execution)
}

pub(crate) fn scheduled_execution_record(
    cycle_id: &TestCycleId,
    execution_id: TestExecutionId,
    snapshot: crate::execution::ExecutionSnapshot,
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
fn prepare_run_provider(
    state: &RuntimeState,
    run: &RunProvider,
) -> Result<Vec<RunPlanGroup>, Error> {
    if run.mode == crate::execution::ExecutionMode::Interactive && run.test_execution_id.is_none() {
        return Err(Error::conflict(
            "interactive_selection_required",
            "interactive runs require one explicit execution",
        ));
    }
    let cycle_view = state
        .cycles
        .get(&run.test_cycle_id)
        .ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
    if !cycle_admits_run(cycle_view.lifecycle) {
        return Err(Error::conflict("conflict", CYCLE_NOT_READY));
    }
    if cycle_view.execution_ids.is_empty() {
        return Err(Error::conflict(
            "empty_cycle",
            "cycle has no test cases; append cases before running it",
        ));
    }
    let members = cycle_view
        .execution_ids
        .iter()
        .map(|id| {
            state.executions.get(id).ok_or_else(|| {
                Error::conflict(
                    "execution_not_planned",
                    "cycle references a missing execution owner",
                )
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let selected = members
        .into_iter()
        .filter(|execution| {
            execution.status == ExecutionStatus::Skipped
                && execution.started_at.is_none()
                && execution.finished_at.is_none()
                && run
                    .test_execution_id
                    .as_ref()
                    .is_none_or(|id| &execution.test_execution_id == id)
        })
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(Error::conflict(
            "execution_not_planned",
            EXECUTION_NOT_PLANNED,
        ));
    }
    if selected
        .iter()
        .any(|execution| execution.test_cycle_id != run.test_cycle_id)
    {
        return Err(Error::conflict(
            "execution_owner_mismatch",
            "planned execution belongs to another cycle",
        ));
    }
    let grouped = provider_target_groups(selected.iter().map(|execution| {
        state
            .suites
            .provider_target_for_case(execution.snapshot.case())
    }))?;
    if run
        .plugin_identity
        .as_ref()
        .is_some_and(|identity| identity.source != grouped[0].0.source)
    {
        return Err(Error::conflict(
            "provider_identity_mismatch",
            "snapshot identity belongs to another provider",
        ));
    }
    let mut groups = Vec::new();
    for (target, members) in grouped {
        let mut execution_ids = Vec::new();
        for original in members.into_iter().map(|index| selected[index]) {
            if original.snapshot.plugin() != run.plugin_identity.as_ref() {
                return Err(Error::conflict(
                    "provider_identity_changed",
                    "the materialized execution belongs to another plugin revision",
                ));
            }
            execution_ids.push(original.test_execution_id.clone());
        }
        groups.push(RunPlanGroup {
            target,
            execution_ids,
        });
    }
    Ok(groups)
}

fn provider_target_groups(
    targets: impl IntoIterator<Item = Option<ExecutionProviderTarget>>,
) -> Result<Vec<(ExecutionProviderTarget, Vec<usize>)>, Error> {
    let mut groups: Vec<(ExecutionProviderTarget, Vec<usize>)> = Vec::new();
    for (index, target) in targets.into_iter().enumerate() {
        let target = target
            .ok_or_else(|| Error::unavailable("runtime_unavailable", PROVIDER_UNAVAILABLE))?;
        if let Some((first, _)) = groups.first()
            && first.source != target.source
        {
            return Err(Error::conflict(
                "conflict",
                format!(
                    "selected cases resolve to different provider sources: {} and {}",
                    first.source, target.source,
                ),
            ));
        }
        match groups.iter_mut().find(|(existing, _)| existing == &target) {
            Some((_, members)) => members.push(index),
            None => groups.push((target, vec![index])),
        }
    }

    if groups.is_empty() {
        return Err(Error::unavailable(
            "runtime_unavailable",
            "cycle execution requires a provider execution binding",
        ));
    }
    Ok(groups)
}

fn scheduled_steps(
    template: Option<&TestCaseTemplate>,
    execution_id: &TestExecutionId,
) -> Vec<StepRecord> {
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
    case: &TestCase,
    template: Option<&TestCaseTemplate>,
) -> BTreeMap<dokime::core::DocumentId, DocumentDefinition> {
    let id =
        dokime::core::DocumentId::from_identifier("default").expect("default document id is valid");
    let mut document = DocumentDefinition::new(
        "default",
        template
            .map(|template| template.metadata.label.clone())
            .unwrap_or_else(|| case.metadata.label.clone()),
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
    Identifier::new(value).unwrap_or_else(|_| {
        Identifier::new("provider.invalid_identifier").expect("fallback identifier is valid")
    })
}

/* === Tests === */
