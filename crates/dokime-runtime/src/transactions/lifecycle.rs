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
use serde::{Deserialize, Serialize};

// Local Imports
use crate::state::RuntimeState;
use crate::transactions::prompts::{PromptSettlement, SettledPrompts, commit_settled_prompts, settle_prompts};
use crate::transactions::provider::request::RunPhase;
use crate::transactions::records::draft::Drafts;
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::StepRecord;
use dokime_domain::model::execution::result::StepStatus;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::DokimeNodeIdentity;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.runtime.started", version = 4, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkRuntimeStarted {
    pub session_id: SessionId,
    pub record_namespace: SessionId,
}

#[harmos::transaction(id = "dokime.runtime.identity.establish", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstablishNodeIdentity {
    pub candidate: DokimeNodeIdentity,
}

#[harmos::transaction(id = "dokime.runtime.shutdown.start", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartShutdown;

#[harmos::transaction(id = "dokime.runtime.interrupted.recover", version = 4, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecoverInterruptedCycles {
    pub at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportFinalization {
    pub evidence_at: harmos::storage::StorageCursor,
    pub requested_at: Timestamp,
    pub lifecycle: Option<ReportLifecycle>,
}

#[harmos::transaction(id = "dokime.report.finalization.request", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestReportFinalization {
    pub executions: Vec<TestExecutionId>,
    pub intent: ReportFinalization,
}

#[harmos::transaction(id = "dokime.report.lifecycle.capture", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaptureReportLifecycle {
    pub lifecycle: ReportLifecycle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportLifecycle {
    pub id: uuid::Uuid,
    pub execution: dokime_domain::model::execution::saved::ExecutionRevision,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CycleReportFinalization {
    pub source: dokime_domain::model::execution::result::TestCycle,
    pub evidence_at: harmos::storage::StorageCursor,
}

#[harmos::transaction(id = "dokime.cycle.report.finalization.request", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestCycleReportFinalization {
    pub intent: CycleReportFinalization,
}

#[harmos::transaction(id = "dokime.cycle.seal", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealCycle {
    pub sealed_cycle: dokime_domain::model::execution::result::TestCycle,
}

#[harmos::transaction(id = "dokime.execution.seal", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealExecution {
    pub seal: dokime_domain::model::execution::saved::ExecutionSeal,
}

#[harmos::transaction(id = "dokime.cycle.sealing.begin", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeginCycleSeal {
    pub test_cycle_id: dokime_domain::model::identity::TestCycleId,
    pub at: Timestamp,
}

#[harmos::transaction(id = "dokime.execution.request.queued.stop", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StageQueuedStop {
    pub request_id: dokime_domain::model::identity::CommandId,
    pub at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for BeginCycleSeal {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        state.cycles.get(&self.test_cycle_id).ok_or("cycle not found")?;
        Ok(())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        if state.cycles[&self.test_cycle_id].phase == dokime_domain::model::execution::result::CyclePhase::Active {
            self.prompts(state).0.collect(out);
        }
    }

    fn apply(&self, state: &mut RuntimeState) {
        use dokime_domain::model::execution::result::CyclePhase;
        if state.cycles[&self.test_cycle_id].phase != CyclePhase::Active {
            return;
        }
        let (records, settled) = self.prompts(state);
        records.apply(state);
        commit_settled_prompts(state, settled);
        let cycle = state.cycles.get_mut(&self.test_cycle_id).expect("checked cycle");
        cycle.phase = CyclePhase::Sealing;
        let idle = !matches!(cycle.lifecycle, CycleStatus::Running | CycleStatus::Paused | CycleStatus::Stopping);
        if !idle {
            cycle.lifecycle = CycleStatus::Stopping;
        }
        cycle.touch_at(self.at.clone());
        let ran = closed_members_ran(cycle, &state.executions);
        for id in &cycle.execution_ids {
            let execution = state.executions.get_mut(id).expect("cycle execution owner exists");
            if execution.status != ExecutionStatus::Skipped || execution.finished_at.is_some() {
                continue;
            }
            execution.status = ExecutionStatus::Cancelled;
            execution.finished_at = Some(self.at.clone());
            execution.result = Some(dokime_domain::model::execution::result::ResultSummary {
                passed: 0,
                failed: 0,
                blocked: 0,
                skipped: 1,
                status: dokime_domain::model::execution::result::ResultStatus::Cancelled,
            });
            for step in &mut execution.steps {
                if step.status == StepStatus::Pending {
                    step.status = StepStatus::Skipped;
                    step.finished_at = Some(self.at.clone());
                }
            }
        }
        for request in state.run_requests.values_mut() {
            if request.summary.cycle_id == self.test_cycle_id && !request.summary.phase.terminal() {
                request.summary.stop_requested = true;
                request.summary.paused = false;
            }
        }
        cycle.result = Some(crate::transactions::provider_finish::summarize(cycle.execution_ids.iter().map(|id| &state.executions[id])));
        if idle {
            cycle.lifecycle = closed_outcome(ran, cycle.result.as_ref());
        }
    }
}

impl BeginCycleSeal {
    fn prompts(&self, state: &RuntimeState) -> (Drafts, SettledPrompts) {
        let mut records = Drafts::new(state);
        let settled = settle_prompts(state, &mut records, PromptSettlement::Stopped, &self.at, |scope| {
            scope.test_cycle_id.as_ref() == Some(&self.test_cycle_id)
        });
        (records, settled)
    }
}

impl Transaction<RuntimeState> for StageQueuedStop {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let request = state.run_requests.get(&self.request_id).ok_or("queued request not found")?;
        if request.summary.phase != RunPhase::Queued || request.summary.prepared != 0 || request.summary.provider_started_at.is_some() {
            return Err("request already owns provider work".into());
        }
        if !request.summary.stop_requested {
            return Err("queued cancellation requires a stop request".into());
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        let request = state.run_requests.get_mut(&self.request_id).expect("checked queued request");
        request.summary.phase = RunPhase::Finishing;
        request.summary.stop_requested = true;
        request.summary.paused = false;
        request.completion_at = Some(self.at.clone());
    }
}

impl Transaction<RuntimeState> for SealExecution {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let id = &self.seal.source.execution.execution_id;
        let execution = state.executions.get(id).ok_or("execution seal owner is missing")?;
        if execution.saved.as_ref().is_some_and(|saved| saved != &self.seal) {
            return Err("execution already has a different immutable seal".into());
        }
        self.seal.validate(execution).map_err(|error| error.to_string())
    }

    fn apply(&self, state: &mut RuntimeState) {
        let id = &self.seal.source.execution.execution_id;
        state.executions.get_mut(id).expect("checked execution owner").saved = Some(self.seal.clone());
        state.report_finalizations.remove(id);
    }
}

impl harmos::MemorySize for CycleReportFinalization {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(self.source.memory_size()).saturating_add(self.evidence_at.memory_size())
    }
}

impl Transaction<RuntimeState> for RequestCycleReportFinalization {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let source = &self.intent.source;
        if let Some(previous) = state.cycle_report_finalizations.get(&source.test_cycle_id) {
            return if previous == &self.intent { Ok(()) } else { Err("cycle report finalization is already frozen".into()) };
        }
        let current = state.cycles.get(&source.test_cycle_id).ok_or("cycle report owner is missing")?;
        if source.phase != dokime_domain::model::execution::result::CyclePhase::Sealing || source != current {
            return Err("cycle report requires the exact closed container header".into());
        }
        if source.saved.is_some()
            || current.saved.is_some()
            || source.session_id.is_none()
            || !crate::transactions::archive::cycle_terminal(source.lifecycle)
            || source.execution_ids != current.execution_ids
            || source.metadata != current.metadata
            || source.created_at != current.created_at
            || source.started_at != current.started_at
            || source.session_id != current.session_id
        {
            return Err("cycle report source differs from its terminal owner".into());
        }
        Ok(())
    }
    // No Primary Audit Record: The Journal Depends Only On Already-Durable Children.
    fn apply(&self, state: &mut RuntimeState) {
        state.cycle_report_finalizations.entry(self.intent.source.test_cycle_id.clone()).or_insert_with(|| self.intent.clone());
    }
}

impl Transaction<RuntimeState> for SealCycle {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let sealed = &self.sealed_cycle;
        let current = state.cycles.get(&sealed.test_cycle_id).ok_or("sealed cycle is missing")?;
        let seal = sealed.saved.as_ref().ok_or("sealed cycle has no seal")?;
        if sealed.phase != dokime_domain::model::execution::result::CyclePhase::Sealed
            || current.phase == dokime_domain::model::execution::result::CyclePhase::Active
        {
            return Err("cycle must close admission before publishing its seal".into());
        }
        if current.saved.as_ref().is_some_and(|saved| saved != seal) {
            return Err("cycle already has a different immutable report seal".into());
        }
        if state.run_requests.values().any(|request| request.summary.cycle_id == sealed.test_cycle_id && !request.summary.phase.terminal())
        {
            return Err("a run request still owns this cycle".into());
        }
        let mut expected = sealed.clone();
        expected.saved = current.saved.clone();
        expected.phase = current.phase;
        if !crate::transactions::archive::cycle_terminal(sealed.lifecycle) || &expected != current {
            return Err("sealed cycle differs from its terminal owner".into());
        }
        seal.validate(sealed).map_err(|source| source.to_string())?;
        for revision in &seal.executions {
            let execution = state.executions.get(&revision.execution_id).ok_or("execution seal owner is missing")?;
            let execution_seal = execution.saved.as_ref().ok_or("cycle member must be sealed before its cycle")?;
            execution_seal.validate(execution).map_err(|error| error.to_string())?;
            let source_matches = execution_seal.source.execution.execution_id == revision.execution_id
                && execution_seal.source.owner_node_id == seal.owner_node_id;
            let dependencies_saved = revision.position.covers(&execution_seal.dependencies_saved).map_err(|error| error.to_string())?;
            if !source_matches || !dependencies_saved {
                return Err("cycle seal differs from its durable execution seals".into());
            }
        }
        Ok(())
    }
    fn apply(&self, state: &mut RuntimeState) {
        let id = &self.sealed_cycle.test_cycle_id;
        state.cycles.insert(id.clone(), self.sealed_cycle.clone());
        state.cycle_report_finalizations.remove(id);
    }
}

impl Transaction<RuntimeState> for RequestReportFinalization {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        if self.intent.lifecycle.is_some() {
            return Err("initial report boundary cannot contain lifecycle metadata".into());
        }
        for id in &self.executions {
            let execution = state.executions.get(id).ok_or("report execution not found")?;
            if execution.saved.is_some() {
                return Err("report execution is already sealed".into());
            }
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        for id in &self.executions {
            // Repeated Stop/Shutdown Never Moves An Existing Evidence Boundary.
            state.report_finalizations.entry(id.clone()).or_insert_with(|| self.intent.clone());
        }
    }
}

impl Transaction<RuntimeState> for CaptureReportLifecycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let id = &self.lifecycle.execution.execution_id;
        let intent = state.report_finalizations.get(id).ok_or("report evidence boundary is missing")?;
        let owner = state.executions.get(id).ok_or("report execution owner is missing")?;
        // The Frozen Revision May Precede The Terminal Commit It Seals; Its
        // Owner Saved It Only After Checking It Terminal.
        if owner.saved.is_some() {
            return Err("report lifecycle must be unsealed".into());
        }
        if intent.lifecycle.as_ref().is_some_and(|lifecycle| lifecycle.execution != self.lifecycle.execution) {
            return Err("report lifecycle is already frozen to another terminal revision".into());
        }
        if self.lifecycle.id.is_nil() {
            return Err("report lifecycle identity is empty".into());
        }
        Ok(())
    }

    // This Transition Deliberately Emits No Primary Audit Record.
    fn apply(&self, state: &mut RuntimeState) {
        let intent =
            state.report_finalizations.get_mut(&self.lifecycle.execution.execution_id).expect("checked report finalization intent exists");
        intent.lifecycle.get_or_insert_with(|| self.lifecycle.clone());
    }
}

impl Transaction<RuntimeState> for MarkRuntimeStarted {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.session_id = self.session_id.clone();
        state.record_namespace = self.record_namespace.clone();
        state.shutting_down = false;
    }
}

impl Transaction<RuntimeState> for EstablishNodeIdentity {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.establish_node_identity(self.candidate.clone());
    }
}

impl Transaction<RuntimeState> for StartShutdown {
    type Error = String;

    fn check(&self, _state: &RuntimeState) -> std::result::Result<(), Self::Error> {
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.shutting_down = true;
    }
}

impl RecoverInterruptedCycles {
    fn prompts(&self, state: &RuntimeState) -> (Drafts, SettledPrompts) {
        let mut records = Drafts::new(state);
        let settled = settle_prompts(state, &mut records, PromptSettlement::Interrupted, &self.at, |scope| {
            scope.test_cycle_id.is_some() || scope.test_execution_id.is_some()
        });
        (records, settled)
    }
}

impl Transaction<RuntimeState> for RecoverInterruptedCycles {
    type Error = String;
    fn check(&self, _: &RuntimeState) -> Result<(), String> {
        Ok(())
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.prompts(state).0.collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        let (records, settled) = self.prompts(state);
        records.apply(state);
        commit_settled_prompts(state, settled);

        state.run_queue.clear();
        for request in state.run_requests.values_mut() {
            if request.summary.phase.terminal() {
                continue;
            }

            request.summary.phase = RunPhase::Interrupted;
            request.summary.finished_at = Some(self.at.clone());
            request.summary.paused = false;
            request.pending.clear();
            request.conclusion = Some(CycleStatus::Stopped);
            request.failure = Some("runtime restarted before request completion".into());
        }

        // Sealed Cycles Are Served From Native History After A Restart
        state.retire_sealed_cycles();
        for (id, cycle) in &mut state.cycles {
            // Startup Finishes These Containers Before Exposing New Admission.
            cycle.phase = dokime_domain::model::execution::result::CyclePhase::Sealing;
            if state.cycle_report_finalizations.contains_key(id) {
                // Every Member Was Already Durable Before This Frozen Intent.
                continue;
            }
            let mut interrupted = matches!(cycle.lifecycle, CycleStatus::Running | CycleStatus::Paused | CycleStatus::Stopping);
            for id in &cycle.execution_ids {
                let execution = state.executions.get_mut(id).expect("cycle execution owner exists");
                interrupted |= interrupt_execution(execution, &self.at);
            }
            if interrupted {
                cycle.lifecycle = CycleStatus::Stopped;
                cycle.result =
                    Some(crate::transactions::provider_finish::summarize(cycle.execution_ids.iter().map(|id| &state.executions[id])));
                cycle.touch_at(self.at.clone());
            } else if cycle.lifecycle == CycleStatus::Ready {
                // An Idle Cycle Closes On What Its Members Already Settled.
                cycle.lifecycle = closed_lifecycle(cycle, &state.executions);
                cycle.touch_at(self.at.clone());
            }
        }
    }
}

/* === Public Functions === */

// A step attempt that ended without a completion receipt keeps this redacted detail.
#[track_caller]
pub(crate) fn interrupted_step_detail() -> dokime_domain::model::diagnostics::Diagnostic {
    dokime_domain::model::diagnostics::Diagnostic::of("step runner ended without a completion receipt").redacted()
}

/// The truthful outcome a closing cycle settles on: its members' aggregate once any ran, Stopped otherwise.
pub(crate) fn closed_lifecycle(
    cycle: &dokime_domain::model::execution::result::TestCycle, executions: &BTreeMap<TestExecutionId, TestExecution>,
) -> CycleStatus {
    closed_outcome(closed_members_ran(cycle, executions), cycle.result.as_ref())
}

/* === Private Functions === */

fn closed_members_ran(
    cycle: &dokime_domain::model::execution::result::TestCycle, executions: &BTreeMap<TestExecutionId, TestExecution>,
) -> bool {
    cycle.execution_ids.iter().any(|id| executions.get(id).is_some_and(|execution| execution.started_at.is_some()))
}

fn closed_outcome(ran: bool, result: Option<&dokime_domain::model::execution::result::ResultSummary>) -> CycleStatus {
    match crate::transactions::provider_finish::lifecycle_for(result) {
        CycleStatus::Ready => CycleStatus::Stopped,
        _ if !ran => CycleStatus::Stopped,
        outcome => outcome,
    }
}

fn interrupt_execution(execution: &mut TestExecution, at: &Timestamp) -> bool {
    if !matches!(execution.status, ExecutionStatus::Pending | ExecutionStatus::Running)
        && !(execution.status == ExecutionStatus::Skipped && execution.finished_at.is_none())
    {
        return false;
    }

    if execution.status == ExecutionStatus::Skipped && execution.started_at.is_none() {
        execution.status = ExecutionStatus::Cancelled;
        execution.finished_at = Some(at.clone());
        execution.result = Some(dokime_domain::model::execution::result::ResultSummary {
            status: dokime_domain::model::execution::result::ResultStatus::Cancelled,
            passed: 0,
            failed: 0,
            blocked: 0,
            skipped: 1,
        });
        for step in &mut execution.steps {
            if step.status == StepStatus::Pending {
                step.status = StepStatus::Skipped;
                step.finished_at = Some(at.clone());
            }
        }
        return true;
    }
    execution.interrupt_step_attempt(interrupted_step_detail());
    execution.status = ExecutionStatus::Interrupted;
    execution.result = None;
    execution.finished_at = Some(at.clone());
    for step in &mut execution.steps {
        interrupt_step(step, at);
    }
    execution.refresh_step_records();
    true
}

fn interrupt_step(step: &mut StepRecord, at: &Timestamp) {
    if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
        step.status = StepStatus::Cancelled;
        step.finished_at = Some(at.clone());
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::identity::Metadata;
    use dokime_domain::model::identity::TestCycleId;

    #[test]
    fn cycle_intent_survives_state_encoding_and_refuses_a_new_cursor() {
        let mut state = RuntimeState::new(crate::model::identity::new_id());
        let id = crate::model::identity::new_id::<TestCycleId>();
        let at = crate::model::time::now_utc();
        let source = dokime_domain::model::execution::result::TestCycle {
            saved: None,
            test_cycle_id: id.clone(),
            session_id: Some(state.session_id.clone()),
            metadata: Metadata {
                identifier: dokime_domain::model::identity::Identifier::new("cycle.restart").unwrap(),
                label: "Cycle Restart".into(),
                description: None,
                tags: vec![],
                attributes: Default::default(),
                created_at: None,
                updated_at: None,
            },
            lifecycle: CycleStatus::Stopped,
            phase: dokime_domain::model::execution::result::CyclePhase::Sealing,
            created_at: at.clone(),
            started_at: Some(at.clone()),
            updated_at: at,
            execution_ids: vec![],
            records: Default::default(),
            continue_automatically: false,
            result: None,
        };
        state.cycles.insert(id.clone(), source.clone());
        let cursor = harmos::storage::StorageCursor {
            history: harmos::storage::StoreHistory {
                store: harmos::Identifier::new("restart-store").unwrap(),
                epoch: harmos::Identifier::new("restart-epoch").unwrap(),
            },
            sequence: 17,
        };
        let request = RequestCycleReportFinalization { intent: CycleReportFinalization { source, evidence_at: cursor.clone() } };
        request.check(&state).unwrap();
        request.apply(&mut state);
        let bytes = serde_json::to_vec(&state).unwrap();
        let mut recovered: RuntimeState = serde_json::from_slice(&bytes).unwrap();
        // Startup And Later Lifecycle Metadata Cannot Move The Frozen Header.
        recovered.session_id = crate::model::identity::new_id::<SessionId>();
        recovered.cycles.get_mut(&id).unwrap().updated_at = crate::model::time::now_utc();
        assert_eq!(recovered.cycle_report_finalizations[&id], request.intent);
        request.check(&recovered).unwrap();
        request.apply(&mut recovered);
        let mut changed = request.clone();
        changed.intent.evidence_at.sequence += 1;
        assert!(changed.check(&recovered).is_err());
        assert_eq!(recovered.cycle_report_finalizations[&id].evidence_at, cursor);
        assert_eq!(recovered.cycle_report_finalizations[&id].source, request.intent.source);
    }
}
