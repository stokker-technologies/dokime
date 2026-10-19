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
use std::collections::BTreeMap;
// Third Party Imports
use harmos::state::Transaction;
// Local Imports
use dokime_domain::model;
use dokime_domain::model::identity::CommandId;
use dokime_runtime::definition::{MaterializedParameter, ParameterSource, SuiteSet, TestCase, TestCaseTemplate, TestSuite};
use dokime_runtime::domains::TestSuiteDefinition;
use dokime_runtime::state::RuntimeState;
use dokime_runtime::transactions::{
    CycleControlAction, InvalidateExecutions, MarkRuntimeStarted, PrepareAttempt, RecoverInterruptedCycles, RunProvider,
    UpsertScopedParameter,
};
use dokime_runtime::{CaseId, Identifier, Metadata, TemplateId, TestCycleId, Timestamp, Value};
use {
    dokime_domain::model::execution::result::CycleStatus, dokime_domain::model::execution::result::ExecutionStatus,
    dokime_domain::model::execution::result::ResultStatus, dokime_domain::model::execution::result::ResultSummary,
    dokime_domain::model::execution::result::TestExecution, dokime_domain::model::execution::snapshot::ExecutionPluginIdentity,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
struct FixtureAppend {
    admission_identifier: CommandId,
    test_cycle_id: TestCycleId,
    cases: Vec<TestCase>,
    occurred_at: Timestamp,
    settings: BTreeMap<String, BTreeMap<String, Value>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl FixtureAppend {
    fn execution_ids(&self) -> Vec<dokime_runtime::TestExecutionId> {
        self.cases
            .iter()
            .enumerate()
            .map(|(ordinal, _)| {
                dokime_runtime::TestExecutionId::from_seed(format!("fixture-append:{}:{ordinal}", self.admission_identifier))
            })
            .collect()
    }

    fn transition(&self, state: &RuntimeState) -> Result<RuntimeState, String> {
        let unique = self.cases.iter().map(|case| (case.test_case_id.clone(), case.clone())).collect::<BTreeMap<_, _>>();
        let cases = unique.into_values().collect::<Vec<_>>();
        let definition_revisions = cases
            .iter()
            .map(|case| state.case_definition_revision(case).map(|revision| (case.test_case_id.clone(), revision)))
            .collect::<Result<_, _>>()?;
        let registration =
            dokime_runtime::transactions::RegisterCases { registration_id: self.admission_identifier.clone(), cases, definition_revisions };
        registration.check(state)?;
        let mut next = state.clone();
        registration.apply(&mut next);
        let executions = self.execution_ids().into_iter().zip(self.cases.iter().map(|case| case.test_case_id.clone())).collect();
        let append = dokime_runtime::transactions::AppendExecutions {
            cycle_id: self.test_cycle_id.clone(),
            executions,
            occurred_at: self.occurred_at.clone(),
            settings: self.settings.clone(),
        };
        append.check(&next)?;
        append.apply(&mut next);
        Ok(next)
    }

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        self.transition(state).map(|_| ())
    }

    fn apply(&self, state: &mut RuntimeState) {
        *state = self.transition(state).expect("fixture registration and explicit append were checked");
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn metadata(id: &str) -> Metadata {
    Metadata::new(Identifier::new(id).unwrap(), id, None)
}

fn identity(digest: &str) -> ExecutionPluginIdentity {
    ExecutionPluginIdentity {
        source: "fixture.plugin".into(),
        version: Some("1.0.0".into()),
        implementation: dokime_domain::model::execution::snapshot::ExecutionImplementation::Bundle { content_digest: digest.into() },
    }
}

fn parameter(value: i64, source: ParameterSource) -> MaterializedParameter {
    MaterializedParameter { metadata: metadata("voltage"), group_id: None, source, value: Value::from(value), unit: None }
}

// Exercise the same case materialization used by authored edits, before dispatch.
fn materialize_cases(
    state: &mut RuntimeState, cycle_id: &TestCycleId, cases: Vec<TestCase>, settings: BTreeMap<String, Value>,
) -> Result<(), String> {
    let mut planned = state.clone();
    planned.run_requests.clear();
    planned.run_queue.clear();
    let mut cycle = planned.cycle_view(cycle_id).unwrap();
    cycle.test_executions.clear();
    cycle.lifecycle = CycleStatus::Ready;
    planned.replace_cycle(cycle).unwrap();
    let append = FixtureAppend {
        test_cycle_id: cycle_id.clone(),
        admission_identifier: dokime_runtime::model::identity::new_id::<CommandId>(),
        cases,
        settings: BTreeMap::from([("fixture.plugin".into(), settings)]),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    append.check(&planned)?;
    append.apply(&mut planned);
    *state = planned;
    Ok(())
}

fn admitted() -> (RuntimeState, TestCycleId) {
    let mut state = RuntimeState::new(dokime_runtime::model::identity::new_id());
    let template: TestCaseTemplate = serde_json::from_value(serde_json::json!({
        "test_case_template_id": TemplateId::from_seed("snapshot.template"),
        "metadata": metadata("snapshot.template"), "parameters": [],
        "criteria": [{"criterion_id": dokime_runtime::CriterionId::from_seed("criterion"),
            "metadata": metadata("criterion"),
            "expected": Value::from(42_i64)}]
    }))
    .unwrap();
    let suite = TestSuite {
        metadata: metadata("snapshot.suite"),
        tags: vec![],
        parameter_groups: vec![],
        test_case_templates: vec![template.clone()],
    };
    let case = TestCase {
        suite_id: "snapshot.suite".to_owned(),
        test_case_id: CaseId::from_seed("snapshot.case"),
        test_case_template_id: template.test_case_template_id,
        metadata: metadata("snapshot.case"),
        suite: Some(suite.metadata.clone()),
        parameters: BTreeMap::from([("voltage".into(), Value::from(12_i64))]),
        generation_index: 0,
        label_number: 0,
    };
    let mut definition = TestSuiteDefinition::new("snapshot.suite", suite).unwrap();
    definition.workflow = Some("fixture.workflow".into());
    state.suites.set("fixture.plugin", SuiteSet::new([definition])).unwrap();
    let now = dokime_runtime::model::time::now_utc();
    let id = TestCycleId::from_seed("snapshot.cycle");
    create_cycle(&mut state, &id, "snapshot.cycle");
    state.plugin_owners.insert(
        "fixture.plugin".into(),
        dokime_runtime::state::PluginOwner::Bundle {
            path: "fixture.dokime".into(),
            package: "fixture".into(),
            installation: None,
            content_digest: "old".into(),
        },
    );
    state.installed_plugins.insert(
        "fixture.plugin".into(),
        dokime_runtime::InstalledPlugin { source: "fixture.plugin".into(), version: "1.0.0".into(), ..Default::default() },
    );
    materialize_cases(&mut state, &id, vec![case], BTreeMap::from([("station".into(), Value::from("A"))])).unwrap();
    let run = RunProvider {
        mode: Default::default(),
        test_cycle_id: id.clone(),
        test_execution_id: Some(state.cycles[&id].execution_ids[0].clone()),
        context: Default::default(),
        command_id: CommandId::from_seed("snapshot.run"),
        accepted_at: now,
    };
    run.check(&state).unwrap();
    run.apply(&mut state);
    dispatch_request(&mut state, &run.command_id);
    (state, id)
}

fn planned() -> (RuntimeState, TestCycleId) {
    let (mut state, existing) = admitted();
    state.run_requests.clear();
    state.run_queue.clear();
    let id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    create_cycle(&mut state, &id, "queued.cycle");
    let append = FixtureAppend {
        admission_identifier: dokime_runtime::model::identity::new_id::<CommandId>(),
        test_cycle_id: id.clone(),
        cases: state.cycle_view(&existing).unwrap().cases().cloned().collect(),
        settings: BTreeMap::new(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    append.check(&state).unwrap();
    append.apply(&mut state);
    (state, id)
}

fn create_cycle(state: &mut RuntimeState, id: &TestCycleId, identifier: &str) {
    let create = dokime_runtime::transactions::CreateCycle {
        definition: dokime_runtime::transactions::CycleDefinition {
            metadata: metadata(identifier),
            continue_automatically: false,
            continue_on_failure: false,
        },
        test_cycle_id: id.clone(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    create.check(state).unwrap();
    create.apply(state);
}

fn queued() -> (RuntimeState, TestCycleId, CommandId) {
    let (mut state, cycle_id) = planned();
    let run = RunProvider {
        mode: Default::default(),
        test_cycle_id: cycle_id.clone(),
        test_execution_id: Some(state.cycles[&cycle_id].execution_ids[0].clone()),
        context: Default::default(),
        command_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        accepted_at: dokime_runtime::model::time::now_utc(),
    };
    run.check(&state).unwrap();
    run.apply(&mut state);
    (state, cycle_id, run.command_id)
}

fn dispatch_request(state: &mut RuntimeState, id: &CommandId) {
    while !state.run_requests[id].pending.is_empty() {
        let transaction = dokime_runtime::transactions::PrepareAttempt {
            request_id: id.clone(),
            ordinal: state.run_requests[id].summary.prepared,
            at: dokime_runtime::model::time::now_utc(),
        };
        transaction.check(state).unwrap();
        transaction.apply(state);
    }
    let transaction = dokime_runtime::transactions::DispatchProvider { request_id: id.clone(), at: dokime_runtime::model::time::now_utc() };
    transaction.check(state).unwrap();
    transaction.apply(state);
}

fn provider_outcome(execution: &TestExecution) -> dokime_runtime::transport::ProviderCycleOutcome {
    serde_json::from_value(serde_json::json!({
        "status": "passed", "dependency_imports": [],
        "provider": { "status": "ready",
            "domain_registration_count": 1,
            "test_suite_count": 1,
            "test_suite_identifiers": [],
            "error": null },
        "executions": [{ "case_id": execution.test_case_id, "test_execution_id": execution.test_execution_id,
            "template_identifier": "snapshot.template",
                "label": "snapshot",
                "module": "fixture",
                "status": "passed",
                "phases": [],
                "records": [],
                "telemetry": [],
                "criteria": [],
                "failures": [] }],
        "failures": []
    }))
    .unwrap()
}

fn settle_request(state: &mut RuntimeState, id: &CommandId) {
    let settle = dokime_runtime::transactions::SettleRun { request_id: id.clone(), at: dokime_runtime::model::time::now_utc() };
    settle.check(state).unwrap();
    settle.apply(state);
    let unsealed = dokime_runtime::transactions::Unsealed {
        cause: "state-only fixture has no report storage".into(),
        execution_ids: state.run_requests[id].selected_execution_ids.clone(),
    };
    let complete = dokime_runtime::transactions::CompleteRun { request_id: id.clone(), at: settle.at, unsealed: Some(unsealed) };
    complete.check(state).unwrap();
    complete.apply(state);
}

fn projected_status(state: &RuntimeState, cycle: &TestCycleId, execution: &TestExecution) -> Value {
    assert!(!execution.output_parameters.contains_key("execution.status"));
    dokime_runtime::traceability::execution_traceability(
        &state.cycle_view(cycle).unwrap(),
        execution,
        dokime_runtime::model::time::now_utc(),
    )
    .into_iter()
    .flat_map(|snapshot| snapshot.entries)
    .find(|entry| entry.metadata.identifier.as_str() == "execution.status")
    .expect("execution status is projected from its row")
    .value
}

/* === Tests === */

#[test]
fn reruns_invalidate_steps() {
    use dokime_runtime::transactions::{BeginStepAttempt, RecordStepAttempt};
    use dokime_runtime::{StepAttemptId, StepId};
    use {
        dokime_domain::model::execution::result::TestStepAttempt, dokime_domain::model::execution::snapshot::ExecutionSnapshot,
        dokime_domain::model::execution::termination::TerminationReason,
    };
    let (mut state, cycle_id) = admitted();
    state.cycles.get_mut(&cycle_id).unwrap().lifecycle = CycleStatus::Running;
    let id = state.cycles[&cycle_id].execution_ids[0].clone();
    let execution = state.executions.get_mut(&id).unwrap();
    let mut template = execution.snapshot.template().unwrap().clone();
    let steps = [StepId::from_seed("prepare"), StepId::from_seed("measure"), StepId::from_seed("seal")];
    template.phases = steps
        .iter()
        .enumerate()
        .map(|(position, step_id)| dokime_runtime::definition::ExecutionPhaseDefinition {
            step_id: step_id.clone(),
            metadata: metadata(&format!("step.{position}")),
            repeatable: position != 2,
        })
        .collect();
    let suite =
        TestSuite { metadata: metadata("snapshot.suite"), tags: vec![], parameter_groups: vec![], test_case_templates: vec![template] };
    execution.snapshot = ExecutionSnapshot::materialize(
        execution.snapshot.case(),
        Some(&suite),
        execution.documents.clone(),
        execution.snapshot.plugin().cloned(),
        Default::default(),
    )
    .unwrap();
    execution.status = ExecutionStatus::Running;
    execution.finished_at = None;
    execution.steps.clear();
    let scope = execution.scope.clone();
    let receipt = |position: usize| RecordStepAttempt {
        scope: scope.clone(),
        attempt: TestStepAttempt {
            attempt_id: dokime_runtime::model::identity::new_id::<StepAttemptId>(),
            step_id: steps[position].clone(),
            started_at: Timestamp::new("2026-10-05T09:00:00Z"),
            finished_at: Some(Timestamp::new("2026-10-05T09:00:01Z")),
            termination: TerminationReason::Completed,
            detail: None,
        },
    };
    for position in 0..2 {
        let completed = receipt(position);
        let begin = BeginStepAttempt { scope: scope.clone(), start: completed.attempt.start() };
        begin.check(&state).unwrap();
        begin.apply(&mut state);
        completed.check(&state).unwrap();
        completed.apply(&mut state);
    }
    let replacement = receipt(0);
    let begin = BeginStepAttempt { scope: scope.clone(), start: replacement.attempt.start() };
    assert_eq!(state.executions[&id].effective_step_attempts().len(), 2);
    begin.check(&state).unwrap();
    begin.apply(&mut state);
    assert!(state.executions[&id].effective_step_attempts().is_empty());
    assert_eq!(state.executions[&id].step_attempts.len(), 2);
    let presented = &state.executions[&id].steps;
    assert_eq!(
        presented.iter().find(|step| step.step_id == steps[0]).unwrap().status,
        dokime_domain::model::execution::result::StepStatus::Running
    );
    let invalidated = presented.iter().find(|step| step.step_id == steps[1]).unwrap();
    assert_eq!(invalidated.status, dokime_domain::model::execution::result::StepStatus::Pending);
    assert!(invalidated.started_at.is_none());
    assert!(invalidated.finished_at.is_none());
    replacement.check(&state).unwrap();
    replacement.apply(&mut state);
    let selected = state.executions[&id].effective_step_attempts();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[&steps[0]].attempt_id, replacement.attempt.attempt_id);
    assert!(!selected.contains_key(&steps[1]));
    assert_eq!(state.executions[&id].status, ExecutionStatus::Running);
    assert!(state.executions[&id].finished_at.is_none());
    for position in 1..3 {
        let completed = receipt(position);
        let begin = BeginStepAttempt { scope: scope.clone(), start: completed.attempt.start() };
        begin.check(&state).unwrap();
        begin.apply(&mut state);
        completed.check(&state).unwrap();
        completed.apply(&mut state);
    }
    let update = dokime_runtime::transactions::UpdateExecutionPhase {
        scope: scope.clone(),
        identifier: Identifier::new("step.2").unwrap(),
        label: "Last Step".into(),
        status: "passed".into(),
        occurred_at: Timestamp::new("2026-10-05T09:00:02Z"),
    };
    update.check(&state).unwrap();
    update.apply(&mut state);
    assert_eq!(state.executions[&id].status, ExecutionStatus::Running);
    assert!(state.executions[&id].finished_at.is_none());
    let before = state.executions[&id].clone();
    for position in 0..3 {
        let completed = receipt(position);
        let begin = BeginStepAttempt { scope: scope.clone(), start: completed.attempt.start() };
        assert!(begin.check(&state).is_err());
        assert_eq!(state.executions[&id], before);
    }
    assert_eq!(before.effective_step_attempts().len(), 3);
    assert_eq!(before.step_attempts.len(), 5);
}

#[test]
fn step_attempt_rewrites() {
    use dokime_runtime::transactions::{BeginStepAttempt, RecordStepAttempt};
    use dokime_runtime::{StepAttemptId, StepId};
    use {
        dokime_domain::model::execution::result::TestStepAttempt, dokime_domain::model::execution::snapshot::ExecutionSnapshot,
        dokime_domain::model::execution::termination::TerminationReason,
    };
    let (mut state, cycle_id) = admitted();
    state.cycles.get_mut(&cycle_id).unwrap().lifecycle = CycleStatus::Running;
    let id = state.cycles[&cycle_id].execution_ids[0].clone();
    let execution = state.executions.get_mut(&id).unwrap();
    let mut template = execution.snapshot.template().unwrap().clone();
    let step_id = StepId::from_seed("repeated-step");
    template.phases.push(dokime_runtime::definition::ExecutionPhaseDefinition {
        repeatable: true,
        step_id: step_id.clone(),
        metadata: metadata("repeated.step"),
    });
    let suite =
        TestSuite { metadata: metadata("snapshot.suite"), tags: vec![], parameter_groups: vec![], test_case_templates: vec![template] };
    execution.snapshot = ExecutionSnapshot::materialize(
        execution.snapshot.case(),
        Some(&suite),
        execution.documents.clone(),
        execution.snapshot.plugin().cloned(),
        Default::default(),
    )
    .unwrap();
    execution.status = ExecutionStatus::Running;
    execution.finished_at = None;
    let scope = execution.scope.clone();
    let mut record = RecordStepAttempt {
        scope,
        attempt: TestStepAttempt {
            attempt_id: StepAttemptId::from_seed("first-step-attempt"),
            step_id,
            started_at: Timestamp::new("2026-10-05T09:00:00Z"),
            finished_at: Some(Timestamp::new("2026-10-05T09:00:01Z")),
            termination: TerminationReason::Completed,
            detail: None,
        },
    };
    let begin = BeginStepAttempt { scope: record.scope.clone(), start: record.attempt.start() };
    begin.check(&state).unwrap();
    begin.apply(&mut state);
    let first_scope = state.executions[&id].capture_scope();
    assert_eq!(first_scope.origin, dokime_runtime::EvidenceOrigin::Step { attempt_id: begin.start.attempt_id.clone() });
    let captured_id = dokime_runtime::RecordId::from_seed("admitted-at-first-attempt");
    let mut admitted = model::records::evidence::Record::new(
        captured_id.clone(),
        dokime_runtime::model::time::now_utc(),
        Identifier::new("fixture.output").unwrap(),
        1,
        "fixture",
        dokime_runtime::records::RecordSeverity::Info,
    );
    admitted.scope = Some(first_scope.clone());
    state.commit_record(admitted);
    assert_eq!(state.executions[&id].records.subscribed_records()[&captured_id], first_scope);

    begin.check(&state).unwrap();
    begin.apply(&mut state);
    let mut other = begin.clone();
    other.start.attempt_id = dokime_runtime::model::identity::new_id::<StepAttemptId>();
    assert!(other.check(&state).is_err());
    let mut mismatched = record.clone();
    mismatched.attempt.started_at = Timestamp::new("2026-10-05T09:00:00.100Z");
    assert!(mismatched.check(&state).is_err());
    assert_eq!(state.executions[&id].active_step_attempt.as_ref(), Some(&begin.start));
    record.check(&state).unwrap();
    record.apply(&mut state);
    assert!(state.executions[&id].active_step_attempt.is_none());
    assert!(begin.check(&state).is_err());
    let first = state.executions[&id].step_attempts[0].clone();
    record.check(&state).unwrap();
    record.apply(&mut state);
    assert_eq!(state.executions[&id].step_attempts.as_slice(), std::slice::from_ref(&first));
    let mut changed = record.clone();
    changed.attempt.finished_at = Some(Timestamp::new("2026-10-05T09:00:02Z"));
    assert!(changed.check(&state).is_err());
    record.attempt.attempt_id = StepAttemptId::from_seed("second-step-attempt");
    let begin = BeginStepAttempt { scope: record.scope.clone(), start: record.attempt.start() };
    begin.check(&state).unwrap();
    begin.apply(&mut state);

    let later_scope = state.executions[&id].capture_scope();
    assert_ne!(first_scope.origin, later_scope.origin);
    state.executions.get_mut(&id).unwrap().records.capture_record(captured_id.clone(), later_scope);
    assert_eq!(state.executions[&id].records.subscribed_records()[&captured_id], first_scope);
    record.check(&state).unwrap();
    record.apply(&mut state);
    assert_eq!(state.executions[&id].step_attempts.len(), 2);
    assert_eq!(state.executions[&id].step_attempts[0], first);
    record.attempt.attempt_id = StepAttemptId::from_seed("lost-step-attempt");
    let pending = BeginStepAttempt { scope: record.scope.clone(), start: record.attempt.start() };
    pending.check(&state).unwrap();
    pending.apply(&mut state);
    let execution = state.executions.get_mut(&id).unwrap();
    // Interruption is idempotent: the second call finds no active attempt.
    let detail = || model::diagnostics::Diagnostic::of("step runner ended without a completion receipt").redacted();
    execution.interrupt_step_attempt(detail());
    execution.interrupt_step_attempt(detail());
    assert!(execution.active_step_attempt.is_none());
    assert_eq!(execution.step_attempts.len(), 3);
    let lost = &execution.step_attempts[2];
    assert_eq!(lost.start(), pending.start);
    assert_eq!(lost.termination, TerminationReason::Interrupted);
    assert!(lost.finished_at.is_none());
    let presented = execution.steps.iter().find(|step| step.step_id == lost.step_id).unwrap();
    assert_eq!(presented.started_at, Some(lost.started_at.clone()));
    assert!(presented.finished_at.is_none());
    assert_eq!(presented.detail, lost.detail);
    execution.validate_step_attempt(lost).unwrap();
    assert!(execution.validate_step_invocation(&lost.start()).is_err());
    record.attempt.attempt_id = StepAttemptId::from_seed("foreign-step-attempt");
    record.scope.test_case_id = Some(CaseId::from_seed("foreign-case"));
    assert!(record.check(&state).is_err());
    record.scope = state.executions[&id].scope.clone();
    state.executions.get_mut(&id).unwrap().finished_at = Some(Timestamp::new("2026-10-05T09:01:00Z"));
    assert!(record.check(&state).is_err());
}

// append checks lifecycle without changing existing execution evidence.

#[test]
fn append_checks_execution() {
    let (mut state, cycle_id) = admitted();
    let original = state.cycle_view(&cycle_id).unwrap().cases().next().unwrap().clone();
    let append = FixtureAppend {
        admission_identifier: dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>(),
        settings: Default::default(),
        test_cycle_id: cycle_id.clone(),
        cases: vec![original.clone()],
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    for lifecycle in [CycleStatus::Running, CycleStatus::Paused, CycleStatus::Stopping] {
        state.cycles.get_mut(&cycle_id).unwrap().lifecycle = lifecycle;
        append.check(&state).unwrap();
        assert_eq!(state.cycle_view(&cycle_id).unwrap().cases().count(), 1);
    }
    for phase in [model::execution::result::CyclePhase::Sealing, model::execution::result::CyclePhase::Sealed] {
        state.cycles.get_mut(&cycle_id).unwrap().phase = phase;
        assert!(append.check(&state).is_err());
    }
    state.cycles.get_mut(&cycle_id).unwrap().phase = model::execution::result::CyclePhase::Active;
    state.cycles.get_mut(&cycle_id).unwrap().lifecycle = CycleStatus::Stopped;
    let existing_count = state.cycle_view(&cycle_id).unwrap().test_executions.len();
    let evidence = serde_json::to_value(&state.cycle_view(&cycle_id).unwrap().test_executions).unwrap();
    let before = serde_json::to_value(&state).unwrap();
    append.check(&state).unwrap();
    assert_eq!(serde_json::to_value(&state).unwrap(), before);
    append.apply(&mut state);
    let cycle = &state.cycle_view(&cycle_id).unwrap();
    assert_eq!(cycle.lifecycle, CycleStatus::Ready);
    assert_eq!(cycle.cases().count(), 1);
    assert_eq!(cycle.cases().next().unwrap().test_case_id, original.test_case_id);
    assert_eq!(serde_json::to_value(&cycle.test_executions[..existing_count]).unwrap(), evidence);
    assert_eq!(cycle.test_executions.len(), existing_count + 1);
    let added = &cycle.test_executions[existing_count];
    assert_eq!(added.test_case_id, original.test_case_id);
    assert_eq!(added.status, ExecutionStatus::Skipped);
    assert!(added.started_at.is_none() && added.finished_at.is_none());
    let removed = state.cycles.remove(&cycle_id).unwrap();
    for id in &removed.execution_ids {
        state.executions.remove(id);
    }
    assert!(append.check(&state).is_err());
}

#[test]
fn append_admissions_identity() {
    let (mut state, cycle_id) = admitted();
    let original = state.cycle_view(&cycle_id).unwrap().cases().next().unwrap().clone();
    state.cycles.get_mut(&cycle_id).unwrap().lifecycle = CycleStatus::Stopped;
    let mut append = FixtureAppend {
        admission_identifier: dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>(),
        settings: Default::default(),
        test_cycle_id: cycle_id.clone(),
        cases: vec![original],
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    append.check(&state).unwrap();
    append.apply(&mut state);
    let removed_id = state.cycles.get_mut(&cycle_id).unwrap().execution_ids.pop().unwrap();
    let removed = state.executions.remove(&removed_id).unwrap();

    append.admission_identifier = dokime_runtime::model::identity::new_id::<dokime_domain::model::identity::CommandId>();
    append.check(&state).unwrap();
    append.apply(&mut state);
    let cycle = state.cycle_view(&cycle_id).unwrap();
    let added = cycle.test_executions.last().unwrap();
    assert_eq!(added.test_case_id, removed.test_case_id);
    assert_ne!(added.test_execution_id, removed.test_execution_id);
    assert_eq!(added.status, ExecutionStatus::Skipped);
}

#[test]
fn repeated_case_order() {
    let (mut state, cycle_id) = planned();
    let case = state.cycle_view(&cycle_id).unwrap().cases().next().unwrap().clone();
    let before = state.cycle_view(&cycle_id).unwrap().test_executions.len();
    let append = FixtureAppend {
        admission_identifier: CommandId::from_seed("repeated-case-admission"),
        settings: BTreeMap::new(),
        test_cycle_id: cycle_id.clone(),
        cases: vec![case.clone(), case.clone()],
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    append.check(&state).unwrap();
    append.apply(&mut state);

    let cycle = &state.cycle_view(&cycle_id).unwrap();
    assert_eq!(cycle.cases().cloned().collect::<Vec<_>>(), vec![case.clone()]);
    assert_eq!(cycle.test_executions.len(), before + 2);
    let added = &cycle.test_executions[before..];
    assert!(added.iter().all(|execution| execution.test_case_id == case.test_case_id));
    assert_ne!(added[0].test_execution_id, added[1].test_execution_id);
    assert_eq!(state.suites.case(&case.test_case_id), Some(&case));

    let mut changed = case;
    changed.parameters.insert("voltage".into(), Value::Integer(240));
    let conflict =
        FixtureAppend { admission_identifier: CommandId::from_seed("conflicting-case-admission"), cases: vec![changed], ..append };
    assert!(conflict.check(&state).is_err());
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions.len(), before + 2);
}

#[test]
fn admission_freezes_them() {
    let (mut state, cycle_id) = admitted();
    let before = state.cycle_view(&cycle_id).unwrap().test_executions[0].snapshot.clone();
    assert_eq!(before.parameters()["voltage"].value, Value::from(12_i64));
    assert!(!before.parameters().contains_key("inherited"));
    assert_eq!(before.parameters()["station"].value, Value::from("A"));
    assert_eq!(before.host_version(), Some(env!("CARGO_PKG_VERSION")));
    let mut serialized = serde_json::to_value(&before).unwrap();
    serialized.as_object_mut().unwrap().remove("host_version");
    let older: dokime_domain::model::execution::snapshot::ExecutionSnapshot = serde_json::from_value(serialized).unwrap();
    assert_eq!(older.host_version(), None, "old snapshots never borrow the reader's version");
    assert_eq!(before.template().unwrap().criteria.len(), 1);
    assert!(!before.documents().is_empty());
    let scope = state.cycle_view(&cycle_id).unwrap().test_executions[0].scope.clone();
    UpsertScopedParameter {
        scope,
        key: "voltage".into(),
        value: Value::from(15_i64),
        source: ParameterSource::Output,
        metadata: None,
        group_id: None,
        unit: None,
    }
    .apply(&mut state);
    state.run_parameters.insert("inherited".into(), parameter(99, ParameterSource::Operator));
    let execution = &state.cycle_view(&cycle_id).unwrap().test_executions[0];
    assert_eq!(execution.snapshot, before);
    assert_eq!(execution.output_parameters["voltage"].value, Value::from(15_i64));
    assert_eq!(execution.effective_parameters()["voltage"].value, Value::from(15_i64));
    assert_eq!(serde_json::from_str::<TestExecution>(&serde_json::to_string(execution).unwrap()).unwrap(), *execution);

    // Re-dispatching the same plan reuses the execution admission materialized,
    // and a request with no materialized plan behind it is refused.
    state.run_requests.clear();
    state.run_queue.clear();
    let cycle = state.cycles.get_mut(&cycle_id).unwrap();
    cycle.lifecycle = CycleStatus::Ready;
    let execution_id = cycle.execution_ids[0].clone();
    let execution = state.executions.get_mut(&execution_id).unwrap();
    execution.status = ExecutionStatus::Skipped;
    execution.started_at = None;
    state.run_parameters.insert("voltage".into(), parameter(99, ParameterSource::Operator));
    let run = RunProvider {
        mode: Default::default(),
        test_cycle_id: cycle_id.clone(),
        test_execution_id: Some(state.cycles[&cycle_id].execution_ids[0].clone()),
        context: Default::default(),
        command_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        accepted_at: dokime_runtime::model::time::now_utc(),
    };
    run.check(&state).unwrap();
    run.apply(&mut state);
    let mut missing = state.clone();
    missing.executions.remove(&missing.cycles[&cycle_id].execution_ids[0]);
    let prepare = dokime_runtime::transactions::PrepareAttempt {
        request_id: run.command_id.clone(),
        ordinal: 0,
        at: dokime_runtime::model::time::now_utc(),
    };
    assert!(prepare.check(&missing).unwrap_err().contains("materialized execution is missing"));
    dispatch_request(&mut state, &run.command_id);
    let cycle = &state.cycle_view(&cycle_id).unwrap();
    assert_eq!(cycle.test_executions.len(), 1);
    assert_eq!(cycle.test_executions[0].test_execution_id, execution_id);
    assert_eq!(cycle.test_executions[0].snapshot, before);

    // The catalog forgetting the suite does not rewrite the admitted inputs.
    let mut forgotten = state.clone();
    forgotten.suites.unregister_source("fixture.plugin");
    assert_eq!(forgotten.cycle_view(&cycle_id).unwrap().test_executions[0].snapshot, before);

    state.run_requests.clear();
    let cycle = state.cycles.get_mut(&cycle_id).unwrap();
    cycle.lifecycle = CycleStatus::Ready;
    let removed = std::mem::take(&mut cycle.execution_ids);
    for id in removed {
        state.executions.remove(&id);
    }
    assert!(run.check(&state).unwrap_err().contains("no test cases"));
}

#[test]
fn settlement_replay_timestamp() {
    let (mut state, cycle_id, id) = queued();
    state.run_requests.get_mut(&id).unwrap().summary.phase = dokime_domain::model::execution::request::RunPhase::Finishing;
    let at = Timestamp::new("2026-10-09T10:00:00Z");
    let later = Timestamp::new("2026-10-09T10:01:00Z");
    for timestamp in [&at, &later] {
        let settle = dokime_runtime::transactions::SettleRun { request_id: id.clone(), at: timestamp.clone() };
        settle.check(&state).unwrap();
        settle.apply(&mut state);
        let bytes = serde_json::to_vec(&state.run_requests[&id]).unwrap();
        state.run_requests.insert(id.clone(), serde_json::from_slice(&bytes).unwrap());
        assert_eq!(state.run_requests[&id].completion_at.as_ref(), Some(&at));
    }
    let execution = &state.cycle_view(&cycle_id).unwrap().test_executions[0];
    let finish = dokime_runtime::transactions::FinishProvider {
        request_id: id.clone(),
        test_cycle_id: cycle_id,
        target: state.run_requests[&id].target.clone(),
        started_execution_ids: vec![execution.test_execution_id.clone()],
        outcome: Ok(provider_outcome(execution)),
        finished_at: later.clone(),
    };
    assert_eq!(finish.check(&state).unwrap_err(), "provider work cannot change after completion is staged");
    let mut complete = dokime_runtime::transactions::CompleteRun { request_id: id, at: later, unsealed: None };
    assert_eq!(complete.check(&state).unwrap_err(), "completion differs from its staged timestamp");
    complete.at = at;
    assert_eq!(complete.check(&state).unwrap_err(), "completed request requires every selected execution seal");
    complete.unsealed = Some(dokime_runtime::transactions::Unsealed {
        cause: "state-only fixture has no report storage".into(),
        execution_ids: state.run_requests[&complete.request_id].selected_execution_ids.clone(),
    });
    complete.check(&state).unwrap();
}

#[test]
fn recovery_completed_outcomes() {
    let (mut state, cycle_id) = admitted();
    let mut cycle = state.cycle_view(&cycle_id).unwrap();
    let mut completed = cycle.test_executions[0].clone();
    completed.test_execution_id = dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>();
    completed.status = ExecutionStatus::Passed;
    completed.finished_at = Some(cycle.updated_at.clone());
    completed.result = Some(ResultSummary { status: ResultStatus::Passed, passed: 1, failed: 0, blocked: 0, skipped: 0 });
    cycle.test_executions.push(completed.clone());
    cycle.test_executions[0].status = ExecutionStatus::Running;
    state.replace_cycle(cycle).unwrap();
    MarkRuntimeStarted {
        record_namespace: dokime_runtime::model::identity::new_id::<dokime_runtime::SessionId>(),
        session_id: dokime_runtime::model::identity::new_id::<dokime_runtime::SessionId>(),
    }
    .apply(&mut state);
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[0].status, ExecutionStatus::Running);
    let invalidation = InvalidateExecutions {
        previous: identity("old"),
        current: Some(identity("new")),
        invalidated_at: dokime_runtime::model::time::now_utc(),
    };
    invalidation.check(&state).unwrap();
    invalidation.apply(&mut state);
    RecoverInterruptedCycles { at: dokime_runtime::model::time::now_utc() }.apply(&mut state);
    let execution = &state.cycle_view(&cycle_id).unwrap().test_executions[0];
    assert_eq!(execution.status, ExecutionStatus::Invalidated);
    assert_eq!(execution.result, None);
    assert_eq!(execution.invalidation.as_ref().unwrap().current, Some(identity("new")));
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[1], completed);
    let before = state.cycle_view(&cycle_id).unwrap().clone();
    invalidation.apply(&mut state);
    assert_eq!(state.cycle_view(&cycle_id).unwrap(), before);
    let execution = before.test_executions[0].clone();
    let target = state.suites.provider_target_for_case(before.cases().next().unwrap()).unwrap();
    dokime_runtime::transactions::FinishProvider {
        request_id: CommandId::from_seed("snapshot.run"),
        test_cycle_id: cycle_id.clone(),
        target,
        started_execution_ids: vec![execution.test_execution_id.clone()],
        outcome: Ok(provider_outcome(&execution)),
        finished_at: dokime_runtime::model::time::now_utc(),
    }
    .apply(&mut state);
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[0], execution);
}

#[test]
fn execution_requires_snapshot() {
    let (state, cycle_id) = admitted();
    let execution = &state.cycle_view(&cycle_id).unwrap().test_executions[0];
    let mut value = serde_json::to_value(execution).unwrap();
    let object = value.as_object_mut().unwrap();
    object.remove("snapshot");
    object.remove("output_parameters");
    object.insert(
        "parameters".into(),
        serde_json::to_value(BTreeMap::from([
            ("input".to_owned(), parameter(12, ParameterSource::Definition)),
            ("output".to_owned(), parameter(15, ParameterSource::Output)),
        ]))
        .unwrap(),
    );
    let error = serde_json::from_value::<TestExecution>(value).unwrap_err();
    assert!(error.to_string().contains("missing field `snapshot`"));
    assert_eq!(
        serde_json::from_value::<TestExecution>(serde_json::to_value(execution).unwrap()).unwrap(),
        *execution,
        "stored executions retain their exact frozen inputs and provenance",
    );
    let unchanged = InvalidateExecutions {
        previous: identity("old"),
        current: Some(identity("old")),
        invalidated_at: dokime_runtime::model::time::now_utc(),
    };
    assert!(unchanged.check(&state).is_err());
}

#[test]
fn admission_validates_parameters() {
    let (mut state, cycle_id) = admitted();
    let mut suite = state.suites.suites().next().unwrap().suite.clone();
    suite.test_case_templates[0].parameters = vec![
        dokime_runtime::Parameter::number("voltage", "Voltage").try_into_definition().unwrap(),
        dokime_runtime::Parameter::number("limit", "Limit").optional().try_into_definition().unwrap(),
        dokime_runtime::Parameter::text("serial", "Serial").optional().try_into_definition().unwrap(),
    ];
    let mut definition = TestSuiteDefinition::new("snapshot.suite", suite).unwrap();
    definition.workflow = Some("fixture.workflow".into());
    state.suites.set("fixture.plugin", SuiteSet::new([definition])).unwrap();
    state.run_parameters.insert("voltage".into(), parameter(48, ParameterSource::Operator));
    state.run_parameters.insert("runtime_only".into(), parameter(99, ParameterSource::Operator));
    let mut cycle = state.cycle_view(&cycle_id).unwrap();
    let prototype = cycle.cases().next().unwrap().clone();
    cycle.test_executions.clear();
    cycle.lifecycle = CycleStatus::Ready;
    state.run_requests.clear();
    state.run_queue.clear();
    state.replace_cycle(cycle).unwrap();
    let mut run = RunProvider {
        mode: Default::default(),
        test_cycle_id: cycle_id.clone(),
        test_execution_id: None,
        context: Default::default(),
        command_id: CommandId::from_seed("validation.run"),
        accepted_at: dokime_runtime::model::time::now_utc(),
    };
    // Each refused write is measured against the same admitted plan: a null or
    // mistyped required value, a missing required value, and a mistyped
    // optional one, and an invalid parameter name. Runtime values must not
    // rescue missing case inputs.
    for (key, value) in [
        ("voltage", Some(Value::Null)),
        ("voltage", Some(Value::from("invalid"))),
        ("voltage", None),
        ("limit", Some(Value::from("invalid"))),
        ("", Some(Value::from(230.0))),
    ] {
        let mut candidate = state.clone();
        let mut case = prototype.clone();
        case.test_case_id = dokime_runtime::model::identity::new_id::<CaseId>();
        let parameters = &mut case.parameters;
        match value {
            Some(value) => parameters.insert(key.to_owned(), value),
            None => parameters.remove(key),
        };
        assert!(materialize_cases(&mut candidate, &cycle_id, vec![case], BTreeMap::new()).is_err());
        assert!(candidate.cycle_view(&cycle_id).unwrap().test_executions.is_empty());
    }
    let mut case = prototype;
    case.test_case_id = dokime_runtime::model::identity::new_id::<CaseId>();
    case.parameters.insert("voltage".into(), Value::from(12.0));
    materialize_cases(&mut state, &cycle_id, vec![case], BTreeMap::new()).unwrap();
    run.test_execution_id = Some(state.cycles[&cycle_id].execution_ids[0].clone());
    run.check(&state).unwrap();
    run.apply(&mut state);
    dispatch_request(&mut state, &run.command_id);
    let snapshot = &state.cycle_view(&cycle_id).unwrap().test_executions[0].snapshot;
    // Only case inputs and captured settings belong to the execution.
    assert_eq!(snapshot.parameter_definitions().len(), 3);
    assert!(!snapshot.parameters().contains_key("runtime_only"));
    assert!(!snapshot.parameters().contains_key("inherited"));
    assert_eq!(snapshot.parameters()["voltage"].value, Value::from(12.0));
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[0].snapshot.parameters()["serial"].value, Value::Null);
}

#[test]
fn terminal_traceability_history() {
    let (mut state, cycle_id) = admitted();
    let first = state.cycle_view(&cycle_id).unwrap().test_executions[0].clone();
    let target = state.suites.provider_target_for_case(state.cycle_view(&cycle_id).unwrap().cases().next().unwrap()).unwrap();
    dokime_runtime::transactions::FinishProvider {
        request_id: CommandId::from_seed("snapshot.run"),
        test_cycle_id: cycle_id.clone(),
        target,
        started_execution_ids: vec![first.test_execution_id.clone()],
        outcome: Ok(provider_outcome(&first)),
        finished_at: dokime_runtime::model::time::now_utc(),
    }
    .apply(&mut state);
    settle_request(&mut state, &CommandId::from_seed("snapshot.run"));
    let completed = state.cycle_view(&cycle_id).unwrap().test_executions[0].clone();
    assert_eq!(completed.status, ExecutionStatus::Passed);
    assert_eq!(state.cycles[&cycle_id].lifecycle, CycleStatus::Completed);
    assert_eq!(projected_status(&state, &cycle_id, &completed), Value::from("Passed"));

    // Aggregate values still arrive after the terminal transaction. Their
    // finalization must preserve the already-terminal traceability fields.
    let aggregates = vec![dokime_runtime::streams::AggregateOutput {
        key: "telemetry.voltage.mean".into(),
        metadata: metadata("voltage.mean"),
        value: Value::from(12_i64),
    }];
    assert!(!completed.output_parameters.contains_key("telemetry.voltage.mean"));
    dokime_runtime::transactions::outputs::FinalizeExecutionOutputs {
        test_execution_id: completed.test_execution_id.clone(),
        outputs: aggregates.clone(),
    }
    .apply(&mut state);
    let completed = state.cycle_view(&cycle_id).unwrap().test_executions[0].clone();
    assert_eq!(projected_status(&state, &cycle_id, &completed), Value::from("Passed"));
    assert_eq!(completed.output_parameters["telemetry.voltage.mean"].value, Value::from(12_i64));

    dokime_runtime::transactions::AppendExecutions {
        cycle_id: cycle_id.clone(),
        executions: vec![(dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>(), completed.test_case_id.clone())],
        settings: Default::default(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    }
    .apply(&mut state);
    // Later cycle metadata must not rewrite the previous execution's evidence.
    state.cycles.get_mut(&cycle_id).unwrap().metadata.label = "Second run".into();
    RunProvider {
        mode: Default::default(),
        test_cycle_id: cycle_id.clone(),
        test_execution_id: Some(state.cycles[&cycle_id].execution_ids.last().unwrap().clone()),
        context: Default::default(),
        command_id: CommandId::from_seed("snapshot.rerun"),
        accepted_at: dokime_runtime::model::time::now_utc(),
    }
    .apply(&mut state);
    dispatch_request(&mut state, &CommandId::from_seed("snapshot.rerun"));
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[0], completed);
    dokime_runtime::transactions::outputs::FinalizeExecutionOutputs {
        test_execution_id: completed.test_execution_id.clone(),
        outputs: aggregates,
    }
    .apply(&mut state);
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[0], completed);
    let repeated = state.cycle_view(&cycle_id).unwrap().test_executions[1].clone();
    assert_eq!(projected_status(&state, &cycle_id, &repeated), Value::from("Pending"));
    let target = state.suites.provider_target_for_case(state.cycle_view(&cycle_id).unwrap().cases().next().unwrap()).unwrap();
    dokime_runtime::transactions::FinishProvider {
        request_id: CommandId::from_seed("snapshot.rerun"),
        test_cycle_id: cycle_id.clone(),
        target,
        started_execution_ids: vec![repeated.test_execution_id.clone()],
        outcome: Err(dokime_runtime::transport::ProviderExecutionFailure {
            stage: "execution".into(),
            template_identifier: None,
            module: None,
            message: "failed".into(),
            detail: None,
        }),
        finished_at: dokime_runtime::model::time::now_utc(),
    }
    .apply(&mut state);
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[0], completed);
    let failed = &state.cycle_view(&cycle_id).unwrap().test_executions[1];
    assert_eq!(failed.status, ExecutionStatus::Failed);
    assert_eq!(projected_status(&state, &cycle_id, failed), Value::from("Failed"));
}

#[test]
fn large_cycle_execution_admission_is_bounded() {
    let (mut state, cycle_id) = planned();
    let mut cycle = state.cycle_view(&cycle_id).unwrap();
    let prototype = cycle.test_executions[0].snapshot.case().clone();
    cycle.test_executions.clear();
    state.replace_cycle(cycle).unwrap();
    let cases = (0..257)
        .map(|index| {
            let mut case = prototype.clone();
            case.test_case_id = CaseId::from_seed(format!("large-{index}"));
            case.generation_index = index;
            case
        })
        .collect();
    materialize_cases(&mut state, &cycle_id, cases, BTreeMap::new()).unwrap();
    let ids = state.cycles[&cycle_id].execution_ids.clone();
    assert_eq!(ids.len(), 257);
    let earlier_admissions = state.run_queue.len();
    let mut receipts = Vec::new();
    for execution_id in &ids {
        let run = RunProvider {
            mode: Default::default(),
            test_cycle_id: cycle_id.clone(),
            test_execution_id: Some(execution_id.clone()),
            context: Default::default(),
            command_id: CommandId::from_seed(format!("large-start:{execution_id}")),
            accepted_at: dokime_runtime::model::time::now_utc(),
        };
        run.check(&state).unwrap();
        let before = state.record_count;
        run.apply(&mut state);
        assert_eq!(state.record_count - before, 1);
        assert_eq!(state.run_requests[&run.command_id].summary.planned, 1);
        assert_eq!(state.run_requests[&run.command_id].selected_execution_ids, vec![execution_id.clone()]);
        receipts.push(run.command_id);
    }
    assert_eq!(state.run_queue.len(), earlier_admissions + 257);
    assert!(ids.iter().all(|id| state.executions[id].started_at.is_none()));
    dokime_runtime::transactions::BeginCycleSeal { test_cycle_id: cycle_id.clone(), at: dokime_runtime::model::time::now_utc() }
        .apply(&mut state);
    for request_id in receipts {
        let prepare = PrepareAttempt { request_id: request_id.clone(), ordinal: 0, at: dokime_runtime::model::time::now_utc() };
        assert!(prepare.check(&state).is_err());
        let stop = dokime_runtime::transactions::StageQueuedStop { request_id, at: dokime_runtime::model::time::now_utc() };
        stop.check(&state).unwrap();
        let before = state.record_count;
        stop.apply(&mut state);
        assert!(state.record_count - before < 256);
    }
    assert_eq!(state.cycles[&cycle_id].execution_ids, ids);
    assert!(ids.iter().all(|id| state.executions[id].status == ExecutionStatus::Cancelled && state.executions[id].started_at.is_none()));
}

// copied cycle preserves authored plan without reusing evidence or defaults.

#[test]
fn copied_cycle_reusing() {
    let (mut state, source_id) = admitted();
    let mut source = state.cycle_view(&source_id).unwrap();
    source.continue_on_failure = true;
    source.test_executions[0].output_parameters.insert(
        "huge".into(),
        MaterializedParameter { value: Value::from("x".repeat(9 * 1024 * 1024)), ..parameter(1, ParameterSource::Output) },
    );
    let before = source.clone();
    state.replace_cycle(source).unwrap();
    let run_parameters = state.run_parameters.clone();
    let copy = dokime_runtime::transactions::CopyCycle {
        source_cycle_id: source_id.clone(),
        test_cycle_id: dokime_runtime::model::identity::new_id::<TestCycleId>(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    copy.check(&state).unwrap();
    copy.apply(&mut state);
    assert_eq!(state.cycle_view(&source_id).unwrap(), before);
    assert_eq!(state.run_parameters, run_parameters);
    let target = &state.cycle_view(&copy.test_cycle_id).unwrap();
    assert_eq!(target.lifecycle, CycleStatus::Ready);
    assert_eq!(target.session_id.as_ref(), Some(&state.session_id));
    assert_eq!(target.metadata, before.metadata);
    assert!(target.cases().eq(before.cases()));
    assert!(serde_json::to_value(target).unwrap().get("parameters").is_none());
    assert_eq!(target.test_executions.len(), target.cases().count());
    for execution in &target.test_executions {
        assert_eq!(execution.status, ExecutionStatus::Skipped);
        assert!(execution.started_at.is_none() && execution.finished_at.is_none());
        assert!(execution.output_parameters.is_empty());
        assert!(execution.traceability.is_empty());
        assert!(execution.steps.iter().all(|step| step.measurements.is_empty()));
        let original = before.test_executions.iter().rev().find(|original| original.test_case_id == execution.test_case_id).unwrap();
        assert_ne!(execution.test_execution_id, original.test_execution_id);
        assert_eq!(execution.snapshot.parameters(), original.snapshot.parameters());
        assert_eq!(execution.snapshot.documents(), original.snapshot.documents());
    }
    assert!(target.result.is_none());
    assert!(target.started_at.is_none());
    assert!(target.continue_on_failure);
    assert!(copy.check(&state).is_err());

    // What the plan carries is checked before anything is cloned: an oversized
    // retained payload, a too-deep value, a plan with no cases, and a source
    // that is gone are each refused by `check`.
    let retry = dokime_runtime::transactions::CopyCycle {
        source_cycle_id: source_id.clone(),
        test_cycle_id: dokime_runtime::model::identity::new_id::<TestCycleId>(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    state.cycles.get_mut(&source_id).unwrap().metadata.label = "x".repeat(8 * 1024 * 1024);
    assert!(retry.check(&state).unwrap_err().contains("source budget"));
    state.cycles.get_mut(&source_id).unwrap().metadata.label = "Source".into();
    // The budget charges what the plan encodes to, not the serializer's
    // write granularity: many small fields well under it are copied.
    state.cycles.get_mut(&source_id).unwrap().metadata.tags = vec!["t".to_owned(); 200_000];
    retry.check(&state).unwrap();
    state.cycles.get_mut(&source_id).unwrap().metadata.tags = Vec::new();
    let mut deep = Value::Null;
    for _ in 0..65 {
        deep = Value::Array(vec![deep]);
    }
    let mut source = state.cycle_view(&source_id).unwrap();
    let mut case = source.test_executions[0].snapshot.case().clone();
    case.parameters.insert("deep".into(), deep);
    source.test_executions[0].snapshot = dokime_domain::model::execution::snapshot::ExecutionSnapshot::materialize(
        &case,
        None,
        Default::default(),
        None,
        Default::default(),
    )
    .unwrap();
    state.replace_cycle(source).unwrap();
    assert!(retry.check(&state).unwrap_err().contains("depth"));
    let removed = std::mem::take(&mut state.cycles.get_mut(&source_id).unwrap().execution_ids);
    for id in removed {
        state.executions.remove(&id);
    }
    assert!(retry.check(&state).unwrap_err().contains("executions"));
    state.cycles.remove(&source_id);
    assert!(retry.check(&state).unwrap_err().contains("not found"));
}

#[test]
fn recovery_requires_request() {
    let (mut state, cycle_id) = admitted();
    let before = state.cycle_view(&cycle_id).unwrap().test_executions[0].clone();
    let at = dokime_runtime::model::time::now_utc();
    RecoverInterruptedCycles { at: at.clone() }.apply(&mut state);
    let sealed = state.cycle_view(&cycle_id).unwrap().test_executions[0].clone();
    assert_eq!(sealed.status, ExecutionStatus::Interrupted);
    assert_eq!(sealed.snapshot, before.snapshot);
    assert_eq!(sealed.test_execution_id, before.test_execution_id);
    assert_eq!(sealed.finished_at, Some(at.clone()));
    assert!(state.run_requests.values().all(|request| {
        request.summary.phase == dokime_runtime::transactions::provider::request::RunPhase::Interrupted
            && request.summary.finished_at == Some(at.clone())
            && request.pending.is_empty()
    }));

    let mut run = RunProvider {
        mode: Default::default(),
        test_cycle_id: cycle_id.clone(),
        test_execution_id: Some(sealed.test_execution_id.clone()),
        context: Default::default(),
        command_id: CommandId::from_seed("recovery.fresh"),
        accepted_at: dokime_runtime::model::time::now_utc(),
    };
    assert!(run.check(&state).is_err());
    let execution_id = dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>();
    let mut append = dokime_runtime::transactions::AppendExecutions {
        cycle_id: cycle_id.clone(),
        executions: vec![(execution_id.clone(), sealed.test_case_id.clone())],
        settings: Default::default(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    assert!(append.check(&state).is_err(), "recovery closes the previous container before new admission");
    let fresh_cycle = dokime_runtime::model::identity::new_id::<TestCycleId>();
    create_cycle(&mut state, &fresh_cycle, "recovery.fresh");
    append.cycle_id = fresh_cycle.clone();
    append.check(&state).unwrap();
    append.apply(&mut state);
    run.test_cycle_id = fresh_cycle.clone();
    run.test_execution_id = Some(execution_id.clone());
    run.check(&state).unwrap();
    run.apply(&mut state);
    dispatch_request(&mut state, &run.command_id);
    assert_eq!(state.cycle_view(&cycle_id).unwrap().test_executions[0], sealed);
    let attempts = &state.cycle_view(&fresh_cycle).unwrap().test_executions;
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].test_execution_id, execution_id);
    assert_ne!(attempts[0].test_execution_id, sealed.test_execution_id);
}

#[test]
fn recovery_keeps_intent() {
    use dokime_runtime::transactions::provider::request::RunPhase;

    let (mut state, cycle_id) = admitted();
    let completed_at = dokime_runtime::model::time::now_utc();
    let request = state.run_requests.values_mut().next().unwrap();
    request.summary.phase = RunPhase::Finishing;
    request.completion_at = Some(completed_at.clone());
    let before = request.clone();
    let mut source = state.cycles[&cycle_id].clone();
    source.lifecycle = CycleStatus::Completed;
    source.updated_at = completed_at;
    let intent = dokime_runtime::transactions::CycleReportFinalization {
        source,
        evidence_at: harmos::storage::StorageCursor {
            history: harmos::storage::StoreHistory {
                store: Identifier::new("fixture.store").unwrap(),
                epoch: Identifier::new("fixture.epoch").unwrap(),
            },
            sequence: 1,
        },
    };
    state.cycle_report_finalizations.insert(cycle_id.clone(), intent.clone());

    RecoverInterruptedCycles { at: dokime_runtime::model::time::now_utc() }.apply(&mut state);
    assert_eq!(state.cycles[&cycle_id].lifecycle, CycleStatus::Running);
    let recovered = state.run_requests.values().next().unwrap();
    assert_eq!(recovered.summary.phase, RunPhase::Interrupted);
    assert_eq!(recovered.completion_at, before.completion_at);
    assert_eq!(recovered.selected_execution_ids, before.selected_execution_ids);
    assert_eq!(recovered.summary.prepared, before.summary.prepared);
    assert!(recovered.pending.is_empty());
    assert_eq!(state.cycle_report_finalizations[&cycle_id], intent);
}

#[test]
fn recovery_seals_receipts() {
    use dokime_runtime::transactions::provider::request::RunPhase;

    for phase in [
        RunPhase::Queued,
        RunPhase::Preparing,
        RunPhase::Running,
        RunPhase::Finishing,
        RunPhase::Completed,
        RunPhase::Failed,
        RunPhase::Stopped,
        RunPhase::Interrupted,
    ] {
        let (mut state, _) = admitted();
        let request = state.run_requests.values_mut().next().unwrap();
        request.summary.phase = phase;
        request.summary.paused = true;
        request.summary.stop_requested = true;
        let before = request.clone();
        let at = dokime_runtime::model::time::now_utc();
        RecoverInterruptedCycles { at: at.clone() }.apply(&mut state);
        let after = state.run_requests.values().next().unwrap().clone();
        if phase.terminal() {
            assert_eq!(after, before);
        } else {
            assert_eq!(after.summary.phase, RunPhase::Interrupted);
            assert_eq!(after.summary.finished_at, Some(at));
            assert!(!after.summary.paused);
            assert_eq!(after.summary.stop_requested, before.summary.stop_requested);
            assert_eq!(after.summary.planned, before.summary.planned);
            assert_eq!(after.summary.prepared, before.summary.prepared);
            assert_eq!(after.summary.finished, before.summary.finished);
            assert_eq!(after.summary.failed, before.summary.failed);
            assert_eq!(after.summary.unstarted, before.summary.unstarted);
            assert_eq!(after.settled_execution_ids, before.settled_execution_ids);
            assert!(after.pending.is_empty());
        }

        RecoverInterruptedCycles { at: dokime_runtime::model::time::now_utc() }.apply(&mut state);
        assert_eq!(state.run_requests.values().next().unwrap(), &after);
    }
}

#[test]
fn copied_cycle_membership() {
    let (mut state, source_id) = admitted();
    let mut source = state.cycle_view(&source_id).unwrap();
    let mut repeated = source.test_executions[0].clone();
    repeated.test_execution_id = dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>();
    repeated.scope.test_execution_id = Some(repeated.test_execution_id.clone());
    source.test_executions.push(repeated);
    let originals = source.test_executions.clone();
    state.replace_cycle(source).unwrap();
    // Cycle-level presentation is not an authority for copied execution inputs.
    let copy = dokime_runtime::transactions::CopyCycle {
        source_cycle_id: source_id,
        test_cycle_id: dokime_runtime::model::identity::new_id::<TestCycleId>(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    copy.check(&state).unwrap();
    copy.apply(&mut state);

    let target = &state.cycle_view(&copy.test_cycle_id).unwrap();
    assert_eq!(target.cases().count(), 1);
    assert_eq!(target.test_executions.len(), 2);
    assert_ne!(target.test_executions[0].test_execution_id, target.test_executions[1].test_execution_id,);
    for (execution, original) in target.test_executions.iter().zip(&originals) {
        assert_eq!(execution.snapshot, original.snapshot);
        assert_eq!(execution.scope.test_cycle_id.as_ref(), Some(&copy.test_cycle_id));
        assert_eq!(execution.status, ExecutionStatus::Skipped);
        assert!(execution.started_at.is_none());
        assert!(execution.output_parameters.is_empty());
    }
}

// A created cycle starts empty even when installed suites have templates.

#[test]
fn cycle_admission_plans() {
    let (mut state, existing) = admitted();
    let id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    create_cycle(&mut state, &id, "explicit.empty");
    let created = state.cycle_view(&id).unwrap();
    assert_eq!(created.lifecycle, CycleStatus::Ready);
    assert!(created.test_executions.is_empty());
    assert_eq!(state.cycle_view(&existing).unwrap().test_executions.len(), 1);
}

// Missing case provenance is an error; a cycle cannot fill it.

#[test]
fn cycle_admission_provenance() {
    let (mut state, existing) = admitted();
    let case = state.cycle_view(&existing).unwrap().cases().next().unwrap().clone();
    let id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    create_cycle(&mut state, &id, "explicit.provenance");
    let mut append = FixtureAppend {
        admission_identifier: dokime_runtime::model::identity::new_id::<CommandId>(),
        test_cycle_id: id,
        cases: vec![case.clone()],
        settings: BTreeMap::new(),
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    append.check(&state).unwrap();
    append.cases[0].suite = None;
    assert!(append.check(&state).unwrap_err().contains("owning suite"));
    assert_eq!(state.suites.case(&case.test_case_id), Some(&case));
}

// Preparation must not restore a queued payload over later canonical evidence.

#[test]
fn request_queues_evidence() {
    let (mut state, cycle_id, request_id) = queued();
    let execution = &mut state.executions.get_mut(&state.cycles[&cycle_id].execution_ids[0]).unwrap();
    let execution_id = execution.test_execution_id.clone();
    assert_eq!(execution.status, ExecutionStatus::Skipped);
    let note = model::execution::OperatorNote {
        note_id: dokime_runtime::model::identity::new_id::<dokime_runtime::NoteId>(),
        scope: execution.scope.clone(),
        author: None,
        message: "Admitted after queuing".into(),
        created_at: dokime_runtime::model::time::now_utc(),
    };
    execution.records.push_note(note.clone());
    execution.output_parameters.insert("voltage".into(), parameter(17, ParameterSource::Output));
    let snapshot = execution.snapshot.clone();
    let pending = serde_json::to_value(&state.run_requests[&request_id]).unwrap();
    assert_eq!(pending["pending"], serde_json::json!([execution_id]));
    let mut obsolete = pending.clone();
    obsolete["pending"] = serde_json::json!([execution]);
    assert!(serde_json::from_value::<dokime_runtime::transactions::provider::request::RunRequest>(obsolete).is_err());

    let prepare = dokime_runtime::transactions::PrepareAttempt {
        request_id: request_id.clone(),
        ordinal: 0,
        at: dokime_runtime::model::time::now_utc(),
    };
    prepare.check(&state).unwrap();
    prepare.apply(&mut state);
    prepare.check(&state).unwrap();
    prepare.apply(&mut state);
    let execution = &state.cycle_view(&cycle_id).unwrap().test_executions[0];
    assert_eq!(execution.test_execution_id, execution_id);
    assert_eq!(execution.status, ExecutionStatus::Pending);
    assert_eq!(execution.snapshot, snapshot);
    assert_eq!(execution.records.notes(), &[note]);
    assert_eq!(execution.output_parameters["voltage"].value, Value::Integer(17));
    assert_eq!(state.run_requests[&request_id].summary.prepared, 1);
    assert!(state.run_requests[&request_id].pending.is_empty());
}

// The queue cannot resurrect an execution that changed or disappeared.

#[test]
fn preparation_refuses_execution() {
    let (mut state, cycle_id, request_id) = queued();
    let prepare = dokime_runtime::transactions::PrepareAttempt {
        request_id: request_id.clone(),
        ordinal: 0,
        at: dokime_runtime::model::time::now_utc(),
    };
    for status in [ExecutionStatus::Invalidated, ExecutionStatus::Running] {
        state.executions.get_mut(&state.cycles[&cycle_id].execution_ids[0]).unwrap().status = status;
        assert!(prepare.check(&state).unwrap_err().contains("no longer unstarted"));
    }
    let execution = &mut state.executions.get_mut(&state.cycles[&cycle_id].execution_ids[0]).unwrap();
    execution.status = ExecutionStatus::Skipped;
    execution.finished_at = Some(dokime_runtime::model::time::now_utc());
    assert!(prepare.check(&state).unwrap_err().contains("no longer unstarted"));
    state.executions.remove(&state.cycles[&cycle_id].execution_ids[0]);
    assert!(prepare.check(&state).unwrap_err().contains("missing from the cycle"));
    assert_eq!(state.run_requests[&request_id].summary.prepared, 0);
    assert_eq!(state.run_requests[&request_id].pending.len(), 1);
}

#[test]
fn execution_cycle_fenced() {
    let (mut state, cycle_id, request_id) = queued();
    let execution = &state.cycle_view(&cycle_id).unwrap().test_executions[0];
    assert_eq!(execution.test_cycle_id, cycle_id);
    let mut encoded = serde_json::to_value(execution).unwrap();
    encoded.as_object_mut().unwrap().remove("test_cycle_id");
    assert!(serde_json::from_value::<TestExecution>(encoded.clone()).is_err());
    encoded["test_cycle_id"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<TestExecution>(encoded).is_err());

    state.executions.get_mut(&state.cycles[&cycle_id].execution_ids[0]).unwrap().test_cycle_id =
        dokime_runtime::model::identity::new_id::<TestCycleId>();
    let prepare = dokime_runtime::transactions::PrepareAttempt {
        request_id: request_id.clone(),
        ordinal: 0,
        at: dokime_runtime::model::time::now_utc(),
    };
    assert!(prepare.check(&state).unwrap_err().contains("another cycle"));
    assert_eq!(state.run_requests[&request_id].summary.prepared, 0);
    assert_eq!(state.run_requests[&request_id].pending.len(), 1);
}

#[test]
fn request_admission_owner() {
    let (mut state, cycle_id) = planned();
    let before = serde_json::to_value(&state.run_requests).unwrap();
    state.executions.get_mut(&state.cycles[&cycle_id].execution_ids[0]).unwrap().test_cycle_id =
        dokime_runtime::model::identity::new_id::<TestCycleId>();
    let run = RunProvider {
        mode: Default::default(),
        test_cycle_id: cycle_id.clone(),
        test_execution_id: Some(state.cycles[&cycle_id].execution_ids[0].clone()),
        context: Default::default(),
        command_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        accepted_at: dokime_runtime::model::time::now_utc(),
    };
    assert!(run.check(&state).unwrap_err().contains("another cycle"));
    assert_eq!(serde_json::to_value(&state.run_requests).unwrap(), before);
}

#[test]
fn execution_selection_attempts() {
    let (mut state, cycle_id) = planned();
    let first = state.cycle_view(&cycle_id).unwrap().test_executions[0].clone();
    let mut second = first.clone();
    second.test_execution_id = dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>();
    second.scope = second.scope.with_test_execution(second.test_execution_id.clone());
    let selected = second.test_execution_id.clone();
    let mut cycle = state.cycle_view(&cycle_id).unwrap();
    cycle.test_executions.push(second);
    state.replace_cycle(cycle).unwrap();
    let mut run = RunProvider {
        mode: dokime_domain::model::execution::ExecutionMode::Interactive,
        test_cycle_id: cycle_id.clone(),
        test_execution_id: Some(selected.clone()),
        context: Default::default(),
        command_id: dokime_runtime::model::identity::new_id::<CommandId>(),
        accepted_at: dokime_runtime::model::time::now_utc(),
    };
    run.check(&state).unwrap();
    run.apply(&mut state);
    assert_eq!(state.run_requests[&run.command_id].summary.planned, 1);
    assert_eq!(state.run_requests[&run.command_id].summary.mode, run.mode);
    let mut conflicting = run.clone();
    conflicting.mode = dokime_domain::model::execution::ExecutionMode::Automatic;
    assert!(conflicting.check(&state).is_err());
    assert_eq!(state.run_requests[&run.command_id].pending.front(), Some(&selected));
    let mut control = dokime_runtime::transactions::ControlExecution {
        test_execution_id: first.test_execution_id.clone(),
        action: dokime_runtime::transactions::CycleControlAction::Pause,
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    assert!(control.check(&state).is_err());
    let unstarted_controls = dokime_runtime::runtime::query::execution_controls(&state, &first.test_execution_id).unwrap();
    assert!(unstarted_controls.can_stop);
    assert!(unstarted_controls.request_id.is_none());
    let projected = dokime_runtime::runtime::query::execution_controls(&state, &selected).unwrap();
    assert_eq!(projected.request_id, Some(run.command_id.clone()));
    assert!(projected.can_pause && projected.can_stop);
    assert!(!projected.can_resume);
    control.test_execution_id = selected.clone();
    control.check(&state).unwrap();
    control.apply(&mut state);
    assert!(state.run_requests[&run.command_id].summary.paused);
    let projected = dokime_runtime::runtime::query::execution_controls(&state, &selected).unwrap();
    assert!(projected.paused && projected.can_resume && projected.can_stop);
    assert!(!projected.can_pause);
    assert_eq!(state.executions[&first.test_execution_id], first);
    control.action = dokime_runtime::transactions::CycleControlAction::Resume;
    control.check(&state).unwrap();
    control.apply(&mut state);
    assert!(!state.run_requests[&run.command_id].summary.paused);
    dispatch_request(&mut state, &run.command_id);
    assert_eq!(state.executions[&selected].mode, run.mode);
    assert_eq!(state.executions[&first.test_execution_id], first);
    let finish = dokime_runtime::transactions::FinishProvider {
        request_id: run.command_id.clone(),
        test_cycle_id: cycle_id.clone(),
        target: state.run_requests[&run.command_id].target.clone(),
        started_execution_ids: vec![selected.clone()],
        outcome: Err(dokime_runtime::transport::ProviderExecutionFailure::new("test", "selected execution failed")),
        finished_at: dokime_runtime::model::time::now_utc(),
    };
    finish.check(&state).unwrap();
    finish.apply(&mut state);
    settle_request(&mut state, &run.command_id);
    assert_eq!(state.executions[&first.test_execution_id], first);
    assert_eq!(state.executions[&selected].status, ExecutionStatus::Failed);
    assert_eq!(state.executions[&selected].mode, run.mode);
    let controls = dokime_runtime::runtime::query::execution_controls(&state, &selected).unwrap();
    assert!(controls.can_stop && !controls.can_pause && !controls.can_resume);
    assert!(controls.request_id.is_none(), "failed publication has no live provider owner");
    assert_eq!(state.cycles[&cycle_id].lifecycle, CycleStatus::Failed);
    assert_eq!(state.run_requests[&run.command_id].summary.finished, 1);
    run.command_id = dokime_runtime::model::identity::new_id::<CommandId>();
    assert!(run.check(&state).is_err(), "finished execution cannot reopen");
    run.test_execution_id = Some(first.test_execution_id.clone());
    run.check(&state).unwrap();
    let failed = state.executions[&selected].clone();
    run.apply(&mut state);
    control.action = dokime_runtime::transactions::CycleControlAction::Stop;
    control.check(&state).unwrap();
    let active_request = state.run_requests[&run.command_id].clone();
    control.apply(&mut state);
    assert_eq!(state.run_requests[&run.command_id], active_request, "stopping an old execution preserves the new request");
    dispatch_request(&mut state, &run.command_id);
    let finish = dokime_runtime::transactions::FinishProvider {
        request_id: run.command_id.clone(),
        test_cycle_id: cycle_id.clone(),
        target: state.run_requests[&run.command_id].target.clone(),
        started_execution_ids: vec![first.test_execution_id.clone()],
        outcome: Ok(provider_outcome(&state.executions[&first.test_execution_id])),
        finished_at: dokime_runtime::model::time::now_utc(),
    };
    finish.check(&state).unwrap();
    finish.apply(&mut state);
    settle_request(&mut state, &run.command_id);
    assert_eq!(state.executions[&selected], failed);
    assert_eq!(state.run_requests[&run.command_id].summary.phase, dokime_runtime::transactions::provider::request::RunPhase::Failed);
    assert!(state.run_requests[&run.command_id].publication_failure.is_some(), "state-only fixture has no durable report seal");
    assert_eq!(state.cycles[&cycle_id].lifecycle, CycleStatus::Failed);
    state.validate_execution_ownership().unwrap();
}

#[test]
fn execution_controls_stop_only_the_selected_attempt() {
    let (mut state, cycle_id, request_id) = queued();
    let id = state.cycles[&cycle_id].execution_ids[0].clone();
    let mut sibling = state.executions[&id].clone();
    sibling.test_execution_id = dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>();
    sibling.scope = sibling.scope.with_test_execution(sibling.test_execution_id.clone());
    let sibling_id = sibling.test_execution_id.clone();
    state.executions.insert(sibling_id.clone(), sibling.clone());
    state.cycles.get_mut(&cycle_id).unwrap().execution_ids.push(sibling_id.clone());
    state.cycles.get_mut(&cycle_id).unwrap().continue_automatically = true;

    let mut control = dokime_runtime::transactions::ControlExecution {
        test_execution_id: id.clone(),
        action: CycleControlAction::Pause,
        occurred_at: dokime_runtime::model::time::now_utc(),
    };
    control.check(&state).unwrap();
    control.apply(&mut state);
    assert!(state.run_requests[&request_id].summary.paused);
    control.action = CycleControlAction::Resume;
    control.check(&state).unwrap();
    control.apply(&mut state);
    assert!(!state.run_requests[&request_id].summary.paused);

    control.action = CycleControlAction::Stop;
    control.check(&state).unwrap();
    control.apply(&mut state);
    let stopped = state.executions[&id].clone();
    assert_eq!(stopped.status, ExecutionStatus::Cancelled);
    assert!(stopped.started_at.is_none() && stopped.finished_at.is_some());
    assert!(state.run_requests[&request_id].summary.stop_requested);
    assert!(!state.cycles[&cycle_id].continue_automatically);
    assert_eq!(state.executions[&sibling_id], sibling);
    control.check(&state).unwrap();
    control.apply(&mut state);
    assert_eq!(state.executions[&id], stopped);
    assert_eq!(state.executions[&sibling_id], sibling);
    let projected = dokime_runtime::runtime::query::execution_controls(&state, &id).unwrap();
    assert!(projected.stop_requested && projected.can_stop);
    assert!(!projected.can_pause && !projected.can_resume);
    let sibling_controls = dokime_runtime::runtime::query::execution_controls(&state, &sibling_id).unwrap();
    assert!(sibling_controls.can_stop && !sibling_controls.stop_requested);
    assert!(sibling_controls.request_id.is_none());
}

#[test]
fn canonical_execution_recovery() {
    let (state, cycle_id) = admitted();
    state.validate_execution_ownership().unwrap();
    let execution_id = state.cycles[&cycle_id].execution_ids[0].clone();
    let restored: RuntimeState = serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();
    restored.validate_execution_ownership().unwrap();
    assert_eq!(restored, state);

    let mut encoded = serde_json::to_value(&restored.cycles[&cycle_id]).unwrap();
    assert!(encoded.get("test_executions").is_none());
    assert_eq!(encoded["execution_ids"].as_array().unwrap().len(), 1);
    encoded.as_object_mut().unwrap().remove("execution_ids");
    assert!(serde_json::from_value::<dokime_domain::model::execution::result::TestCycle>(encoded).is_err());

    let mut missing = restored.clone();
    missing.executions.remove(&execution_id);
    assert!(missing.validate_execution_ownership().unwrap_err().contains("missing"));
    assert_invalid_snapshot(&missing, "missing");

    let mut duplicate = restored.clone();
    duplicate.cycles.get_mut(&cycle_id).unwrap().execution_ids.push(execution_id.clone());
    assert!(duplicate.validate_execution_ownership().unwrap_err().contains("duplicate"));
    assert_invalid_snapshot(&duplicate, "duplicate");

    let mut foreign = restored.clone();
    foreign.executions.get_mut(&execution_id).unwrap().test_cycle_id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    assert!(foreign.validate_execution_ownership().unwrap_err().contains("another cycle"));
    assert_invalid_snapshot(&foreign, "another cycle");

    let mut orphan = restored.clone();
    orphan.cycles.get_mut(&cycle_id).unwrap().execution_ids.clear();
    assert!(orphan.validate_execution_ownership().unwrap_err().contains("absent"));
    assert_invalid_snapshot(&orphan, "absent");

    let mut wrong_key = restored.clone();
    wrong_key.executions.get_mut(&execution_id).unwrap().test_execution_id =
        dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>();
    assert!(wrong_key.validate_execution_ownership().unwrap_err().contains("canonical key"));
    assert_invalid_snapshot(&wrong_key, "canonical key");

    let mut wrong_cycle = restored.clone();
    wrong_cycle.cycles.get_mut(&cycle_id).unwrap().test_cycle_id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    assert_invalid_snapshot(&wrong_cycle, "cycle identity");
}

fn assert_invalid_snapshot(state: &RuntimeState, detail: &str) {
    let bytes = serde_json::to_vec(state).unwrap();
    let error = serde_json::from_slice::<RuntimeState>(&bytes).unwrap_err();
    assert!(error.to_string().contains(detail), "{error}");
}

#[test]
fn cycle_projection_atomic() {
    let (mut state, cycle_id) = admitted();
    let original = state.cycle_view(&cycle_id).unwrap();
    let mut projection = original.clone();
    projection.test_executions[0].status = ExecutionStatus::Failed;
    assert_eq!(state.cycle_view(&cycle_id).unwrap(), original);

    projection.test_executions[0].test_cycle_id = dokime_runtime::model::identity::new_id::<TestCycleId>();
    let before = state.clone();
    assert!(state.replace_cycle(projection).is_err());
    assert_eq!(state, before);

    let mut emptied = original;
    emptied.test_executions.clear();
    assert!(state.replace_cycle(emptied.clone()).is_err(), "admission retains its selected execution");
    state.run_requests.clear();
    state.run_queue.clear();
    state.replace_cycle(emptied).unwrap();
    assert!(state.cycles[&cycle_id].execution_ids.is_empty());
    assert!(state.executions.is_empty());
    state.validate_execution_ownership().unwrap();
}
