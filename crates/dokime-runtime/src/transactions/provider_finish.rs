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
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::definition::TestCase;
use crate::domains::{ExecutionProviderTarget, InstalledPluginErrorRecord, InstalledPluginPhase};
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::records::ValidationMode as RecordValidationMode;
use crate::records::validate_record;
use crate::state::RuntimeState;
use crate::telemetry::{TelemetrySample, ValidationMode as TelemetryValidationMode, validate_telemetry_sample};
use crate::transactions::prompts::{PromptSettlement, SettledPrompts, commit_settled_prompts, settle_prompts};
use crate::transactions::records::draft::BuiltinDraft;
use crate::transactions::records::draft::Drafts;
use crate::transport::{ProviderCycleOutcome, ProviderExecutionFailure, ProviderExecutionOutcome};
use dokime_domain::model::diagnostics::DebugLevel;
use dokime_domain::model::diagnostics::ErrorCause;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthSeverity;
use dokime_domain::model::execution::result::CriterionResult;
use dokime_domain::model::execution::result::CriterionStatus;
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::ResultStatus;
use dokime_domain::model::execution::result::ResultSummary;
use dokime_domain::model::execution::result::StepRecord;
use dokime_domain::model::execution::result::StepStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::CriterionId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::StepId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::identity::TraceId;
use dokime_domain::model::records::evidence::Record;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.execution.provider.finish", version = 6, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinishProvider {
    pub test_cycle_id: TestCycleId,
    pub target: ExecutionProviderTarget,
    pub started_execution_ids: Vec<TestExecutionId>,
    pub outcome: Result<ProviderCycleOutcome, ProviderExecutionFailure>,
    pub finished_at: Timestamp,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ProviderFinishEffects {
    pub telemetry: Vec<TelemetrySample>,
    pub terminal_execution_ids: Vec<TestExecutionId>,
}

struct FinishPlan {
    evidence: Drafts,
    terminal: Drafts,
    cycle: Option<CycleView>,
    rejected_evidence: Vec<String>,
    health: Option<(HealthSeverity, String, Option<String>)>,
    effects: ProviderFinishEffects,
    prompts: Drafts,
    settled: SettledPrompts,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for FinishProvider {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.run_requests.values().any(|request| {
            request.summary.cycle_id == self.test_cycle_id && !request.summary.phase.terminal() && request.completion_at.is_some()
        }) {
            return Err("provider work cannot change after completion is staged".into());
        }
        Ok(())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        prepare_finish_provider(state, self).collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        apply_finish_provider(state, self);
    }
}

impl FinishPlan {
    fn new(state: &RuntimeState) -> Self {
        Self {
            evidence: Drafts::new(state),
            terminal: Drafts::new(state),
            cycle: None,
            rejected_evidence: Vec::new(),
            health: None,
            effects: ProviderFinishEffects::default(),
            prompts: Drafts::new(state),
            settled: SettledPrompts::default(),
        }
    }

    fn collect(&self, out: &mut harmos::state::Records) {
        self.evidence.collect(out);
        self.terminal.collect(out);
        self.prompts.collect(out);
    }

    fn apply(self, state: &mut RuntimeState, finish: &FinishProvider) -> ProviderFinishEffects {
        record_provider_plugin_errors(state, finish);
        self.evidence.apply(state);
        let scope = RecordScope::default().with_cycle(finish.test_cycle_id.clone());
        for error in self.rejected_evidence {
            state.record_health(
                scope.clone(),
                HealthSeverity::Error,
                "Provider execution evidence rejected.",
                Some(error),
                finish.finished_at.clone(),
            );
        }
        if let Some(mut cycle) = self.cycle {
            let active_ids = state
                .cycles
                .get(&cycle.test_cycle_id)
                .into_iter()
                .flat_map(|previous| &previous.execution_ids)
                .filter_map(|id| state.executions.get(id))
                .filter(|execution| matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running))
                .map(|execution| &execution.test_execution_id)
                .collect::<BTreeSet<_>>();
            let terminal_scopes = cycle
                .test_executions
                .iter()
                .filter(|execution| {
                    active_ids.contains(&execution.test_execution_id)
                        && !matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running | ExecutionStatus::Invalidated)
                })
                .map(|execution| execution.scope.clone())
                .collect::<Vec<_>>();
            if let Some(request) = state
                .run_requests
                .values_mut()
                .find(|request| request.summary.cycle_id == cycle.test_cycle_id && !request.summary.phase.terminal())
            {
                request.conclusion = Some(request.test_execution_id.as_ref().map_or(cycle.lifecycle, |id| {
                    lifecycle_for(Some(&summarize(cycle.test_executions.iter().filter(|execution| &execution.test_execution_id == id))))
                }));
                // An earlier target group's cause outlives a later group's, so
                // the run reports why it first went wrong.
                match &finish.outcome {
                    Err(error) => {
                        request.failure.get_or_insert_with(|| format!("{}: {}", error.stage, error.message));
                    }
                    Ok(outcome) => {
                        if let Some(error) = outcome.failures.first() {
                            request.failure.get_or_insert_with(|| format!("{}: {}", error.stage, error.message));
                        }
                    }
                }
                for execution_id in &finish.started_execution_ids {
                    if !request.settled_execution_ids.contains(execution_id) {
                        request.settled_execution_ids.push(execution_id.clone());
                    }
                }
                request.summary.phase = super::provider::request::RunPhase::Finishing;
                request.summary.finished += terminal_scopes.len() as u64;
                for execution in &cycle.test_executions {
                    if terminal_scopes.iter().any(|scope| scope.test_execution_id.as_ref() == Some(&execution.test_execution_id)) {
                        request.summary.unstarted += u64::from(execution.started_at.is_none());
                        request.summary.failed += u64::from(execution.status == ExecutionStatus::Failed);
                    }
                }
                cycle.lifecycle = state.cycles[&cycle.test_cycle_id].lifecycle;
            }
            state.replace_cycle(cycle).expect("cycle projection has valid ownership");
        }
        self.terminal.apply(state);
        self.prompts.apply(state);
        commit_settled_prompts(state, self.settled);
        if let Some((severity, message, detail)) = self.health {
            state.record_health(scope, severity, message, detail, finish.finished_at.clone());
        }
        self.effects
    }
}

/* === Public Functions === */
pub(super) fn apply_finish_provider(state: &mut RuntimeState, finish: &FinishProvider) -> ProviderFinishEffects {
    prepare_finish_provider(state, finish).apply(state, finish)
}

pub(crate) fn prepare_finish_effects(state: &RuntimeState, finish: &FinishProvider) -> ProviderFinishEffects {
    prepare_finish_provider(state, finish).effects
}

pub(super) fn summarize<'a>(executions: impl IntoIterator<Item = &'a TestExecution>) -> ResultSummary {
    let mut summary = ResultSummary { status: ResultStatus::Pending, passed: 0, failed: 0, blocked: 0, skipped: 0 };
    for execution in executions {
        if execution.status == ExecutionStatus::Interrupted {
            summary.blocked += 1;
            continue;
        }
        let Some(result) = &execution.result else {
            continue;
        };
        summary.passed += result.passed;
        summary.failed += result.failed;
        summary.blocked += result.blocked;
        summary.skipped += result.skipped;
        if result.status == ResultStatus::Cancelled && execution.status != ExecutionStatus::Skipped {
            summary.status = ResultStatus::Cancelled;
        }
    }
    if summary.status != ResultStatus::Cancelled {
        summary.status = if summary.failed > 0 {
            ResultStatus::Failed
        } else if summary.blocked > 0 {
            ResultStatus::Blocked
        } else {
            ResultStatus::Passed
        };
    }
    summary
}

/* === Private Functions === */
fn prepare_finish_provider(state: &RuntimeState, finish: &FinishProvider) -> FinishPlan {
    let mut plan = match &finish.outcome {
        Ok(outcome) => prepare_provider_outcome(state, finish, outcome),
        Err(error) => prepare_provider_error(state, finish, error),
    };
    plan.effects.terminal_execution_ids = provider_terminal_executions(plan.cycle.as_ref(), &finish.started_execution_ids);
    if state.run_requests.values().any(|request| request.summary.cycle_id == finish.test_cycle_id && !request.summary.phase.terminal()) {
        plan.terminal.retain(|record| record.scope.as_ref().is_some_and(|scope| scope.test_execution_id.is_some()));
    }
    // A prompt never outlives its execution: whatever this finish ends settles
    // its open prompts in the same transaction.
    let ended = plan.cycle.as_ref().map_or_else(BTreeSet::new, |cycle| {
        cycle
            .test_executions
            .iter()
            .filter(|execution| !matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running))
            .map(|execution| execution.test_execution_id.clone())
            .collect()
    });
    plan.prompts = plan.terminal.following();
    plan.settled = settle_prompts(state, &mut plan.prompts, PromptSettlement::Ended, &finish.finished_at, |scope| {
        scope.test_execution_id.as_ref().is_some_and(|id| ended.contains(id))
    });
    plan
}

fn prepare_provider_outcome(state: &RuntimeState, finish: &FinishProvider, outcome: &ProviderCycleOutcome) -> FinishPlan {
    let mut plan = FinishPlan::new(state);
    let Some(mut cycle) = state.cycle_view(&finish.test_cycle_id) else {
        return plan;
    };
    let stopped = matches!(cycle.lifecycle, CycleStatus::Stopping | CycleStatus::Stopped);
    let bounded =
        state.run_requests.values().any(|request| request.summary.cycle_id == finish.test_cycle_id && !request.summary.phase.terminal());
    let outcome_ids = outcome.executions.iter().map(|execution| execution.test_execution_id.as_str()).collect::<BTreeSet<_>>();
    let execution_ids = cycle
        .test_executions
        .iter()
        .filter(|execution| {
            execution.status != ExecutionStatus::Invalidated
                && (!bounded || finish.started_execution_ids.contains(&execution.test_execution_id))
                && (matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
                    || outcome_ids.contains(execution.test_execution_id.as_str()))
        })
        .map(|execution| execution.test_execution_id.clone())
        .collect::<Vec<_>>();
    let mut failed = false;
    let mut stop_after = None;
    let mut terminal_records = Vec::new();

    for (index, execution_id) in execution_ids.iter().enumerate() {
        let Some(existing) = cycle.test_executions.iter().find(|execution| &execution.test_execution_id == execution_id).cloned() else {
            continue;
        };
        let case = existing.snapshot.case();
        let projection = outcome
            .executions
            .iter()
            .find(|candidate| candidate.test_execution_id == existing.test_execution_id.as_str())
            .or_else(|| outcome.executions.iter().find(|candidate| candidate.case_id == existing.test_case_id.as_str()));
        let mut execution = match projection {
            Some(projection) => execution_from_outcome(&existing, case, projection, &finish.finished_at),
            None => missing_execution(&existing, &finish.finished_at),
        };
        if let Some(projection) = projection
            && let Err(error) =
                append_provider_evidence(state, &execution, projection, finish, &mut plan.evidence, &mut plan.effects.telemetry)
        {
            mark_evidence_failure(&mut execution, &error, &finish.finished_at);
            plan.rejected_evidence.push(error.message);
            failed = true;
        }
        if execution.status != ExecutionStatus::Passed {
            failed = true;
        }
        if let Some(record) = terminal_execution_record(&execution, &case.metadata.label) {
            terminal_records.push(record);
        }
        let passed = execution.status == ExecutionStatus::Passed;
        upsert_execution(&mut cycle.test_executions, execution);
        if !bounded && !passed && !stopped && !cycle.continue_on_failure {
            stop_after = Some(index);
            break;
        }
    }
    if let Some(index) = stop_after {
        let unattempted = &execution_ids[index + 1..];
        if !bounded {
            cycle.test_executions.retain(|execution| !unattempted.contains(&execution.test_execution_id));
        }
    }
    if stopped {
        for execution in &mut cycle.test_executions {
            if bounded && !finish.started_execution_ids.contains(&execution.test_execution_id) {
                continue;
            }
            if matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
                || execution.steps.iter().any(|step| matches!(step.status, StepStatus::Pending | StepStatus::Running))
            {
                finish_execution(execution, ExecutionStatus::Cancelled, ResultStatus::Cancelled, finish.finished_at.clone());
            }
        }
        cycle.lifecycle = CycleStatus::Stopped;
    }
    cycle.result = Some(summarize(&cycle.test_executions));
    if !stopped {
        cycle.lifecycle = lifecycle_for(cycle.result.as_ref());
    }
    cycle.touch_at(finish.finished_at.clone());
    plan.terminal = plan.evidence.following();
    for record in terminal_records {
        plan.terminal.draft(record, finish.finished_at.clone());
    }
    let succeeded = !stopped && outcome.status == "passed" && !failed;
    let scope = request_scope(state, &cycle.test_cycle_id);
    if !bounded {
        plan.terminal.draft(
            BuiltinDraft {
                record_type: if stopped {
                    BuiltinRecordType::TestCycleStopped
                } else if succeeded {
                    BuiltinRecordType::TestCycleCompleted
                } else {
                    BuiltinRecordType::TestCycleFailed
                },
                source: "runtime".to_owned(),
                severity: if succeeded || stopped { RecordSeverity::Info } else { RecordSeverity::Error },
                message: Some(format!("Cycle {} finished with {}.", cycle.metadata.label, outcome.status)),
                details: BTreeMap::from([("status".to_owned(), Value::from(outcome.status.clone()))]),
                scope: Some(scope.clone()),
            },
            finish.finished_at.clone(),
        );
    }
    plan.terminal.draft(
        BuiltinDraft {
            record_type: if succeeded {
                BuiltinRecordType::ProviderExecutionCompleted
            } else if stopped {
                BuiltinRecordType::TestExecutionStopped
            } else {
                BuiltinRecordType::ProviderExecutionFailed
            },
            source: "provider".to_owned(),
            severity: if succeeded || stopped { RecordSeverity::Info } else { RecordSeverity::Error },
            message: Some(if stopped {
                "Test execution stopped by operator.".to_owned()
            } else {
                format!("Provider execution finished with {}.", outcome.status)
            }),
            details: detailed(BTreeMap::new(), first_failure(outcome).and_then(|failure| failure.detail.as_deref())),
            scope: Some(scope.clone()),
        },
        finish.finished_at.clone(),
    );
    plan.health = Some((
        if succeeded { HealthSeverity::Info } else { HealthSeverity::Error },
        format!("Provider execution {}.", outcome.status),
        outcome.failures.first().map(|failure| failure.message.clone()),
    ));
    plan.cycle = Some(cycle);
    plan
}

fn request_scope(state: &RuntimeState, cycle_id: &TestCycleId) -> RecordScope {
    state
        .run_requests
        .values()
        .find(|request| request.summary.cycle_id == *cycle_id && !request.summary.phase.terminal())
        .and_then(|request| request.test_execution_id.as_ref())
        .and_then(|id| state.executions.get(id))
        .map(|execution| execution.scope.clone())
        .unwrap_or_else(|| RecordScope::default().with_cycle(cycle_id.clone()))
}

fn prepare_provider_error(state: &RuntimeState, finish: &FinishProvider, error: &ProviderExecutionFailure) -> FinishPlan {
    let mut plan = FinishPlan::new(state);
    let scope = request_scope(state, &finish.test_cycle_id);
    let bounded =
        state.run_requests.values().any(|request| request.summary.cycle_id == finish.test_cycle_id && !request.summary.phase.terminal());
    let mut stopped = false;
    let mut terminal_records = Vec::new();
    plan.cycle = state.cycle_view(&finish.test_cycle_id);
    if let Some(cycle) = plan.cycle.as_mut() {
        stopped = matches!(cycle.lifecycle, CycleStatus::Stopping | CycleStatus::Stopped);
        cycle.lifecycle = if stopped { CycleStatus::Stopped } else { CycleStatus::Failed };
        for execution in &mut cycle.test_executions {
            if !finish.started_execution_ids.contains(&execution.test_execution_id) {
                continue;
            }
            let active = matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running);
            if active {
                finish_execution(
                    execution,
                    if stopped { ExecutionStatus::Cancelled } else { ExecutionStatus::Failed },
                    if stopped { ResultStatus::Cancelled } else { ResultStatus::Failed },
                    finish.finished_at.clone(),
                );
            }
            if active && let Some(record) = terminal_execution_record(execution, "Test execution") {
                terminal_records.push(record);
            }
        }
        cycle.result = Some(summarize(&cycle.test_executions));
        cycle.touch_at(finish.finished_at.clone());
    }
    for record in terminal_records {
        plan.terminal.draft(record, finish.finished_at.clone());
    }
    if !bounded {
        plan.terminal.draft(
            BuiltinDraft {
                record_type: if stopped { BuiltinRecordType::TestCycleStopped } else { BuiltinRecordType::TestCycleFailed },
                source: "runtime".to_owned(),
                severity: if stopped { RecordSeverity::Info } else { RecordSeverity::Error },
                message: Some(if stopped { "Cycle stopped by operator.".to_owned() } else { format!("Cycle failed: {}", error.message) }),
                details: detailed(BTreeMap::from([("stage".to_owned(), Value::from(error.stage.clone()))]), error.detail.as_deref()),
                scope: Some(scope.clone()),
            },
            finish.finished_at.clone(),
        );
    }
    plan.terminal.draft(
        BuiltinDraft {
            record_type: if stopped { BuiltinRecordType::TestExecutionStopped } else { BuiltinRecordType::ProviderExecutionFailed },
            source: "provider".to_owned(),
            severity: if stopped { RecordSeverity::Info } else { RecordSeverity::Error },
            message: Some(if stopped {
                "Provider execution cancelled by operator.".to_owned()
            } else {
                format!("Provider execution failed: {}", error.message)
            }),
            details: detailed(BTreeMap::from([("stage".to_owned(), Value::from(error.stage.clone()))]), error.detail.as_deref()),
            scope: Some(scope.clone()),
        },
        finish.finished_at.clone(),
    );
    if !stopped {
        plan.health = Some((HealthSeverity::Error, "Provider execution failed.".to_owned(), Some(error.message.clone())));
    }
    plan
}

fn execution_from_outcome(existing: &TestExecution, case: &TestCase, outcome: &ProviderExecutionOutcome, at: &Timestamp) -> TestExecution {
    let status = result_status(&outcome.status);
    let mut execution = TestExecution {
        saved: None,
        mode: existing.mode,
        step_attempts: existing.step_attempts.clone(),
        active_step_attempt: existing.active_step_attempt.clone(),
        capture_phase: existing.capture_phase,
        // Authored traceability is stated once, before the first step, and the
        // finish never restates it: what the template read off the device is
        // what the attempt is filed under.
        traceability: existing.traceability.clone(),
        test_cycle_id: existing.test_cycle_id.clone(),
        test_execution_id: existing.test_execution_id.clone(),
        test_case_id: case.test_case_id.clone(),
        suite: existing.suite.clone().or_else(|| case.suite.clone()),
        status: if outcome.status == "skipped" { ExecutionStatus::Skipped } else { execution_status(&status) },
        scope: existing.scope.clone(),
        snapshot: existing.snapshot.clone(),
        invalidation: existing.invalidation.clone(),
        output_parameters: existing.output_parameters.clone(),
        documents: existing.documents.clone(),
        records: existing.records.clone(),
        started_at: if outcome.status == "skipped" {
            existing.started_at.clone()
        } else {
            existing.started_at.clone().or_else(|| Some(at.clone()))
        },
        finished_at: Some(at.clone()),
        steps: admitted_outcome_steps(existing, outcome, &status, at),
        result: Some(if outcome.status == "skipped" { ResultSummary { skipped: 1, ..summary(status) } } else { summary(status) }),
    };
    if execution.active_step_attempt.is_some() {
        execution.interrupt_step_attempt(super::lifecycle::interrupted_step_detail());
        execution.status = ExecutionStatus::Interrupted;
        execution.result = None;
    }
    execution.refresh_step_records();
    execution
}

fn missing_execution(existing: &TestExecution, at: &Timestamp) -> TestExecution {
    let mut execution = existing.clone();
    finish_execution(&mut execution, ExecutionStatus::Blocked, ResultStatus::Blocked, at.clone());
    execution
}

fn admitted_outcome_steps(
    existing: &TestExecution, outcome: &ProviderExecutionOutcome, result: &ResultStatus, at: &Timestamp,
) -> Vec<StepRecord> {
    let Some(template) = existing.snapshot.template() else {
        return outcome_steps(outcome, &existing.test_execution_id, result, at);
    };
    if template.phases.is_empty() {
        return outcome_steps(outcome, &existing.test_execution_id, result, at);
    }
    let projected = outcome_steps(outcome, &existing.test_execution_id, result, at);
    let mut criteria = template
        .criteria
        .iter()
        .map(|definition| CriterionResult {
            criterion_id: definition.criterion_id.clone(),
            metadata: definition.metadata.clone(),
            status: CriterionStatus::Indeterminate,
            observed: None,
            message: Some("Provider did not report a verdict.".into()),
        })
        .collect::<Vec<_>>();
    for returned in projected.last().into_iter().flat_map(|step| &step.criteria) {
        if let Some(admitted) = criteria.iter_mut().find(|criterion| criterion.metadata.identifier == returned.metadata.identifier) {
            admitted.status = returned.status.clone();
            admitted.observed.clone_from(&returned.observed);
            admitted.message.clone_from(&returned.message);
        } else {
            criteria.push(returned.clone());
        }
    }
    let mut steps = existing.steps.clone();
    for step in &mut steps {
        // Scheduled placeholders belong to the execution, not the skipped step
        // where admission initially displayed them. Relocate one terminal set.
        step.criteria.clear();
        if let Some(definition) = template.phases.iter().find(|phase| phase.metadata.identifier == step.metadata.identifier) {
            step.step_id = definition.step_id.clone();
            step.metadata = definition.metadata.clone();
        }
        if let Some(phase) = outcome.phases.iter().find(|phase| phase.identifier == step.metadata.identifier.as_str()) {
            step.status = step_status(&phase.status);
            step.started_at.get_or_insert_with(|| at.clone());
            step.finished_at.get_or_insert_with(|| at.clone());
        } else if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
            step.status = StepStatus::Skipped;
            step.finished_at = Some(at.clone());
        }
    }
    let criterion_step = outcome
        .phases
        .last()
        .and_then(|last| steps.iter().position(|step| step.metadata.identifier.as_str() == last.identifier))
        .or_else(|| steps.iter().rposition(|step| step.started_at.is_some()))
        .unwrap_or(0);
    if let Some(step) = steps.get_mut(criterion_step) {
        step.criteria = criteria;
    }
    detail_on_the_step_that_failed(&mut steps, outcome);
    steps
}

fn outcome_steps(
    outcome: &ProviderExecutionOutcome, execution_id: &TestExecutionId, result: &ResultStatus, at: &Timestamp,
) -> Vec<StepRecord> {
    let mut steps = outcome
        .phases
        .iter()
        .enumerate()
        .map(|(index, phase)| StepRecord {
            step_id: StepId::from_seed(format!("provider-outcome:{execution_id}:phase:{index}:{}", phase.identifier)),
            metadata: Metadata::new(
                identifier(&format!("provider.{}", phase.identifier)),
                phase.label.clone(),
                Some(format!("Provider module {}.", outcome.module)),
            ),
            status: step_status(&phase.status),
            started_at: Some(at.clone()),
            finished_at: Some(at.clone()),
            measurements: Vec::new(),
            criteria: Vec::new(),
            detail: None,
        })
        .collect::<Vec<_>>();
    if steps.is_empty() {
        steps.push(StepRecord {
            step_id: StepId::from_seed(format!("provider-outcome:{execution_id}:execution")),
            metadata: Metadata::new(identifier("provider.execution"), "Provider Execution", Some(outcome.module.clone())),
            status: step_status_for_result(result),
            started_at: Some(at.clone()),
            finished_at: Some(at.clone()),
            measurements: Vec::new(),
            criteria: Vec::new(),
            detail: None,
        });
    }
    if let Some(step) = steps.last_mut() {
        step.criteria = outcome
            .criteria
            .iter()
            .enumerate()
            .map(|(index, criterion)| CriterionResult {
                criterion_id: CriterionId::from_seed(format!("provider-outcome:{execution_id}:criterion:{index}:{}", criterion.identifier)),
                metadata: Metadata::new(identifier(&criterion.identifier), criterion.label.clone(), None),
                status: criterion_status(&criterion.status),
                observed: Some(Value::from(criterion.status.clone())),
                message: criterion.message.clone(),
            })
            .collect();
    }
    detail_on_the_step_that_failed(&mut steps, outcome);
    steps
}

fn detail_on_the_step_that_failed(steps: &mut [StepRecord], outcome: &ProviderExecutionOutcome) {
    let Some(detail) = outcome.failures.iter().find_map(|failure| failure.detail.as_ref()) else {
        return;
    };
    let failed = steps
        .iter()
        .rposition(|step| matches!(step.status, StepStatus::Failed))
        .or_else(|| steps.iter().rposition(|step| step.started_at.is_some()));
    if let Some(step) = failed.and_then(|failed| steps.get_mut(failed)) {
        step.detail = Some(detail.as_ref().clone());
    }
}

fn append_provider_evidence(
    state: &RuntimeState, execution: &TestExecution, outcome: &ProviderExecutionOutcome, finish: &FinishProvider, records: &mut Drafts,
    telemetry: &mut Vec<TelemetrySample>,
) -> Result<(), ProviderExecutionFailure> {
    let at = &finish.finished_at;
    let owner = &finish.target.source;
    let record_stage = "evidence_record";
    for (index, draft) in outcome.records.iter().enumerate() {
        let record_type =
            Identifier::new(draft.record_type.clone()).map_err(|error| evidence_failure(outcome, record_stage, error.to_string()))?;
        let native = crate::transactions::records::admission::provider(state, owner, &record_type)
            .map_err(|error| evidence_failure(outcome, record_stage, error))?;
        let contract = state
            .record_definitions
            .get(&record_type)
            .ok_or_else(|| evidence_failure(outcome, record_stage, format!("Provider record type {record_type} is not registered.")))?;
        let mut record = Record::new(
            dokime_domain::model::identity::RecordId::from_seed(format!(
                "provider-event:{}:{index}:{}:{}",
                execution.test_execution_id, draft.record_type, draft.timestamp
            )),
            draft.timestamp.clone(),
            record_type,
            contract.schema_version,
            "provider",
            draft.severity,
        );

        record.message = draft.message.clone();
        record.error = draft.error.clone();
        record.details = draft.details.clone();
        record.scope = Some(execution.scope.clone());
        validate_record(Some(contract), &record, RecordValidationMode::Strict)
            .map_err(|error| evidence_failure(outcome, record_stage, error.to_string()))?;
        if native {
            records.push_sidecar(record, owner);
        } else {
            records.push(record);
        }
    }
    for group in &outcome.telemetry {
        if group.samples.is_empty() {
            continue;
        }
        let stream =
            Identifier::new(group.stream.clone()).map_err(|error| evidence_failure(outcome, "evidence_telemetry", error.to_string()))?;
        let contract = state.telemetry_definitions.get(&stream).ok_or_else(|| {
            evidence_failure(outcome, "evidence_telemetry", format!("Provider telemetry stream {stream} is not registered."))
        })?;
        for row in &group.samples {
            let mut sample = TelemetrySample::new(crate::model::identity::new_id(), stream.clone(), 0, at.clone(), "provider");
            sample.schema_version = contract.schema_version;
            sample.received_at = at.clone();
            sample.scope = Some(execution.scope.clone());
            sample.values = row.values.clone();
            sample.storage_payload = Some(row.payload.clone());
            // A reported sample that already carries its own "timestamp"
            // keeps it; one that does not is stamped with the provider's
            // finish instant, the only production time this batch carries.
            sample
                .values
                .entry("timestamp".to_owned())
                .or_insert_with(|| Value::Datetime(at.to_datetime().expect("runtime timestamp is valid")));
            sample.summary.insert("template".to_owned(), Value::from(outcome.template_identifier.clone()));
            sample.summary.insert("label".to_owned(), Value::from(group.label.clone()));
            validate_telemetry_sample(Some(contract), &sample, TelemetryValidationMode::Strict)
                .map_err(|error| evidence_failure(outcome, "evidence_telemetry", error.to_string()))?;
            telemetry.push(sample);
        }
    }
    Ok(())
}

fn mark_evidence_failure(execution: &mut TestExecution, failure: &ProviderExecutionFailure, at: &Timestamp) {
    execution.status = ExecutionStatus::Failed;
    execution.result = Some(summary(ResultStatus::Failed));
    let criterion = CriterionResult {
        criterion_id: CriterionId::from_seed(format!("provider-evidence:{}:{}", execution.test_execution_id, failure.stage)),
        metadata: Metadata::new(identifier("provider.evidence_validation"), "Provider Evidence Validation", None),
        status: CriterionStatus::Failed,
        observed: Some(Value::from(failure.stage.clone())),
        message: Some(failure.message.clone()),
    };
    if let Some(step) = execution.steps.last_mut() {
        step.status = StepStatus::Failed;
        step.criteria.push(criterion);
    } else {
        execution.steps.push(StepRecord {
            step_id: StepId::from_seed(format!("provider-evidence:{}", execution.test_execution_id)),
            metadata: Metadata::new(identifier("provider.evidence_validation"), "Provider Evidence Validation", None),
            status: StepStatus::Failed,
            started_at: Some(at.clone()),
            finished_at: Some(at.clone()),
            measurements: Vec::new(),
            criteria: vec![criterion],
            // Host-side evidence rejection: there is no plugin failure boundary
            // behind it, so there is nothing to detail.
            detail: None,
        });
    }
}

fn evidence_failure(outcome: &ProviderExecutionOutcome, stage: &str, message: String) -> ProviderExecutionFailure {
    ProviderExecutionFailure {
        stage: stage.to_owned(),
        template_identifier: Some(outcome.template_identifier.clone()),
        module: Some(outcome.module.clone()),
        message,
        // The host refused evidence the provider already finished producing.
        // There is no plugin-side failure boundary behind it to detail.
        detail: None,
    }
}

fn first_failure(outcome: &ProviderCycleOutcome) -> Option<&ProviderExecutionFailure> {
    outcome.failures.iter().chain(outcome.executions.iter().flat_map(|execution| &execution.failures)).next()
}

fn detailed(mut details: BTreeMap<String, Value>, detail: Option<&crate::plugin::Diagnostic>) -> BTreeMap<String, Value> {
    let Some(detail) = detail else {
        return details;
    };
    if !detail.location.is_empty() {
        details.insert("location".to_owned(), Value::from(detail.location.clone()));
    }
    if !detail.chain.is_empty() {
        details.insert("chain".to_owned(), Value::Array(detail.chain.iter().cloned().map(Value::from).collect()));
    }

    details
}

fn record_provider_plugin_errors(state: &mut RuntimeState, finish: &FinishProvider) {
    let session_id = state.session_id.clone();
    let failures = match &finish.outcome {
        Ok(outcome) => {
            outcome.failures.iter().chain(outcome.executions.iter().flat_map(|execution| &execution.failures)).collect::<Vec<_>>()
        }
        Err(error) => vec![error],
    };
    for failure in failures {
        state.record_installed_plugin_error(provider_failure_record(
            &finish.test_cycle_id,
            &finish.target,
            &session_id,
            failure,
            &finish.finished_at,
        ));
    }
}

fn provider_failure_record(
    cycle_id: &TestCycleId, target: &ExecutionProviderTarget, session_id: &SessionId, failure: &ProviderExecutionFailure, at: &Timestamp,
) -> InstalledPluginErrorRecord {
    let mut record = InstalledPluginErrorRecord::new(
        Identifier::new(target.source.clone()).unwrap_or_else(|_| identifier("provider.unknown")),
        None,
        InstalledPluginPhase::Provider,
        DebugLevel::Error,
        failure.message.clone(),
        at.clone(),
    )
    .with_code(format!("provider_{}", failure.stage))
    .with_source("runtime.provider_execution")
    .with_scope(session_id.clone(), Some(RecordScope::default().with_cycle(cycle_id.clone())))
    .with_trace_id(TraceId::from_seed(format!("provider-failure:{cycle_id}:{}:{}:{}", target.source, failure.stage, failure.message)));
    if let Some(detail) = failure.detail.as_ref() {
        // The record's own shape already holds the two halves of a diagnostic
        // worth reading here: why, one cause at a time, and where in the
        // source. The backtrace stays off it — a stored error row is read in a
        // list, and frames belong to the console the failure was logged to.
        record.cause_chain = detail.chain.iter().cloned().map(ErrorCause::new).collect();
        if !detail.location.is_empty() {
            // The most precise origin there is, which is what `source` asks
            // for. The subsystem it came from is still in `code` and `phase`.
            record.source = Some(detail.location.clone());
        }
    }

    record
}

fn provider_terminal_executions(cycle: Option<&CycleView>, started: &[TestExecutionId]) -> Vec<TestExecutionId> {
    cycle.map_or_else(Vec::new, |cycle| {
        started
            .iter()
            .filter_map(|id| {
                cycle
                    .test_executions
                    .iter()
                    .find(|execution| &execution.test_execution_id == id)
                    .filter(|execution| !matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running))
                    .map(|execution| execution.test_execution_id.clone())
            })
            .collect()
    })
}

fn finish_execution(execution: &mut TestExecution, status: ExecutionStatus, result: ResultStatus, at: Timestamp) {
    let interrupted = execution.active_step_attempt.is_some();
    execution.interrupt_step_attempt(super::lifecycle::interrupted_step_detail());
    execution.status = if interrupted { ExecutionStatus::Interrupted } else { status };
    execution.finished_at = Some(at.clone());
    execution.result = if interrupted { None } else { Some(summary(result.clone())) };
    for step in &mut execution.steps {
        if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
            step.status = step_status_for_result(&result);
            step.finished_at = Some(at.clone());
        }
    }
    execution.refresh_step_records();
}

fn terminal_execution_record(execution: &TestExecution, label: &str) -> Option<BuiltinDraft> {
    let (record_type, severity, status) = match execution.status {
        ExecutionStatus::Passed => (BuiltinRecordType::TestExecutionCompleted, RecordSeverity::Info, "completed"),
        ExecutionStatus::Failed => (BuiltinRecordType::TestExecutionFailed, RecordSeverity::Error, "failed"),
        ExecutionStatus::Blocked => (BuiltinRecordType::TestExecutionBlocked, RecordSeverity::Warning, "blocked"),
        ExecutionStatus::Cancelled => (BuiltinRecordType::TestExecutionStopped, RecordSeverity::Info, "stopped"),
        ExecutionStatus::Interrupted => (BuiltinRecordType::TestExecutionInterrupted, RecordSeverity::Warning, "interrupted"),
        ExecutionStatus::Skipped | ExecutionStatus::Invalidated | ExecutionStatus::Pending | ExecutionStatus::Running => {
            return None;
        }
    };
    Some(BuiltinDraft {
        record_type,
        source: "runtime".to_owned(),
        severity,
        message: Some(format!("Test execution {label} {status}.")),
        details: BTreeMap::from([
            ("test_execution_id".to_owned(), Value::from(execution.test_execution_id.as_str())),
            ("status".to_owned(), Value::from(status)),
        ]),
        scope: Some(execution.scope.clone()),
    })
}

fn upsert_execution(executions: &mut Vec<TestExecution>, execution: TestExecution) {
    if let Some(existing) = executions.iter_mut().find(|existing| existing.test_execution_id == execution.test_execution_id) {
        *existing = execution;
    } else {
        executions.push(execution);
    }
}

pub(super) fn lifecycle_for(summary: Option<&ResultSummary>) -> CycleStatus {
    match summary.map(|summary| &summary.status) {
        Some(ResultStatus::Passed) => CycleStatus::Completed,
        Some(ResultStatus::Failed) => CycleStatus::Failed,
        Some(ResultStatus::Blocked) => CycleStatus::Blocked,
        Some(ResultStatus::Cancelled) => CycleStatus::Stopped,
        _ => CycleStatus::Ready,
    }
}

fn result_status(status: &str) -> ResultStatus {
    match status {
        "passed" => ResultStatus::Passed,
        "failed" => ResultStatus::Failed,
        "cancelled" | "skipped" => ResultStatus::Cancelled,
        _ => ResultStatus::Blocked,
    }
}

fn execution_status(status: &ResultStatus) -> ExecutionStatus {
    match status {
        ResultStatus::Pending => ExecutionStatus::Pending,
        ResultStatus::Passed => ExecutionStatus::Passed,
        ResultStatus::Failed => ExecutionStatus::Failed,
        ResultStatus::Blocked => ExecutionStatus::Blocked,
        ResultStatus::Cancelled => ExecutionStatus::Cancelled,
    }
}

fn step_status(status: &str) -> StepStatus {
    match status {
        "passed" => StepStatus::Passed,
        "failed" => StepStatus::Failed,
        "cancelled" => StepStatus::Cancelled,
        _ => StepStatus::Skipped,
    }
}

fn step_status_for_result(status: &ResultStatus) -> StepStatus {
    match status {
        ResultStatus::Pending => StepStatus::Pending,
        ResultStatus::Passed => StepStatus::Passed,
        ResultStatus::Failed => StepStatus::Failed,
        ResultStatus::Blocked => StepStatus::Skipped,
        ResultStatus::Cancelled => StepStatus::Cancelled,
    }
}

fn criterion_status(status: &str) -> CriterionStatus {
    match status {
        "passed" => CriterionStatus::Passed,
        "failed" => CriterionStatus::Failed,
        "indeterminate" => CriterionStatus::Indeterminate,
        _ => CriterionStatus::Skipped,
    }
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
