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
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::execution::{CycleStatus, ExecutionStatus, StepRecord, StepStatus, TestExecution};
use crate::state::RuntimeState;
use crate::transactions::prompts::{
    PromptSettlement, SettledPrompts, commit_settled_prompts, settle_prompts,
};
use crate::transactions::provider::request::RunPhase;
use crate::transactions::records::draft::Drafts;
use crate::core::{DokimeNodeIdentity, SessionId, TestExecutionId, Timestamp};

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

/// The Immutable Measurement Boundary Of A Pending Final Report.
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

/// The Terminal Execution A Final Report Captures, By Its Durable Revision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportLifecycle {
    pub id: uuid::Uuid,
    pub execution: crate::execution::saved::ExecutionRevision,
}

/// The Frozen Cycle Header And Durable Member Boundary Used By Every Retry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CycleReportFinalization {
    pub source: crate::execution::TestCycle,
    pub evidence_at: harmos::storage::StorageCursor,
}

#[harmos::transaction(
    id = "dokime.cycle.report.finalization.request",
    version = 1,
    irreversible
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestCycleReportFinalization {
    pub intent: CycleReportFinalization,
}

/// Seals a terminal cycle no run request will complete, with its published report.
#[harmos::transaction(id = "dokime.cycle.seal", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SealCycle {
    pub sealed_cycle: crate::execution::TestCycle,
    pub sealed_executions: Vec<crate::execution::saved::ExecutionSeal>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for CycleReportFinalization {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.source.memory_size())
            .saturating_add(self.evidence_at.memory_size())
    }
}

impl Transaction<RuntimeState> for RequestCycleReportFinalization {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let source = &self.intent.source;
        let current = state
            .cycles
            .get(&source.test_cycle_id)
            .ok_or("cycle report owner is missing")?;
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
        if state
            .cycle_report_finalizations
            .get(&source.test_cycle_id)
            .is_some_and(|previous| previous != &self.intent)
        {
            return Err("cycle report finalization is already frozen".into());
        }
        Ok(())
    }
    // No Primary Audit Record: The Journal Depends Only On Already-Durable Children.
    fn apply(&self, state: &mut RuntimeState) {
        state
            .cycle_report_finalizations
            .entry(self.intent.source.test_cycle_id.clone())
            .or_insert_with(|| self.intent.clone());
    }
}

impl Transaction<RuntimeState> for SealCycle {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let sealed = &self.sealed_cycle;
        let current = state
            .cycles
            .get(&sealed.test_cycle_id)
            .ok_or("sealed cycle is missing")?;
        let seal = sealed.saved.as_ref().ok_or("sealed cycle has no seal")?;
        if current.saved.as_ref().is_some_and(|saved| saved != seal) {
            return Err("cycle already has a different immutable report seal".into());
        }
        if state.run_requests.values().any(|request| {
            request.summary.cycle_id == sealed.test_cycle_id && !request.summary.phase.terminal()
        }) {
            return Err("a run request still owns this cycle".into());
        }
        if !crate::transactions::archive::cycle_terminal(sealed.lifecycle)
            || sealed.execution_ids != current.execution_ids
            || sealed.metadata != current.metadata
            || sealed.created_at != current.created_at
            || sealed.started_at != current.started_at
        {
            return Err("sealed cycle differs from its terminal owner".into());
        }
        seal.validate(sealed).map_err(|source| source.to_string())?;
        if self.sealed_executions.len() != sealed.execution_ids.len() {
            return Err("sealed executions differ from cycle membership".into());
        }
        for (id, execution_seal) in sealed.execution_ids.iter().zip(&self.sealed_executions) {
            if &execution_seal.source.execution.execution_id != id {
                return Err("sealed executions differ from cycle membership".into());
            }
            if state.executions[id]
                .saved
                .as_ref()
                .is_some_and(|saved| saved != execution_seal)
            {
                return Err("execution already has a different immutable report seal".into());
            }
        }
        Ok(())
    }
    fn apply(&self, state: &mut RuntimeState) {
        let id = &self.sealed_cycle.test_cycle_id;
        state.cycles.insert(id.clone(), self.sealed_cycle.clone());
        state.cycle_report_finalizations.remove(id);
        for seal in &self.sealed_executions {
            let member = &seal.source.execution.execution_id;
            state
                .executions
                .get_mut(member)
                .expect("checked member")
                .saved = Some(seal.clone());
            state.report_finalizations.remove(member);
        }
    }
}

impl Transaction<RuntimeState> for RequestReportFinalization {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        if self.intent.lifecycle.is_some() {
            return Err("initial report boundary cannot contain lifecycle metadata".into());
        }
        for id in &self.executions {
            let execution = state
                .executions
                .get(id)
                .ok_or("report execution not found")?;
            if execution.saved.is_some() {
                return Err("report execution is already sealed".into());
            }
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        for id in &self.executions {
            // Repeated Stop/Shutdown Never Moves An Existing Evidence Boundary.
            state
                .report_finalizations
                .entry(id.clone())
                .or_insert_with(|| self.intent.clone());
        }
    }
}

impl Transaction<RuntimeState> for CaptureReportLifecycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let id = &self.lifecycle.execution.execution_id;
        let intent = state
            .report_finalizations
            .get(id)
            .ok_or("report evidence boundary is missing")?;
        let owner = state
            .executions
            .get(id)
            .ok_or("report execution owner is missing")?;
        // The Frozen Revision May Precede The Terminal Commit It Seals; Its
        // Owner Saved It Only After Checking It Terminal.
        if owner.saved.is_some() {
            return Err("report lifecycle must be unsealed".into());
        }
        if intent
            .lifecycle
            .as_ref()
            .is_some_and(|lifecycle| lifecycle.execution != self.lifecycle.execution)
        {
            return Err("report lifecycle is already frozen to another terminal revision".into());
        }
        if self.lifecycle.id.is_nil() {
            return Err("report lifecycle identity is empty".into());
        }
        Ok(())
    }

    // This Transition Deliberately Emits No Primary Audit Record.
    fn apply(&self, state: &mut RuntimeState) {
        let intent = state
            .report_finalizations
            .get_mut(&self.lifecycle.execution.execution_id)
            .expect("checked report finalization intent exists");
        intent
            .lifecycle
            .get_or_insert_with(|| self.lifecycle.clone());
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
    /// The requester no longer exists after restart: every prompt a cycle or
    /// execution raised expires as a runtime response, never an operator one.
    fn prompts(&self, state: &RuntimeState) -> (Drafts, SettledPrompts) {
        let mut records = Drafts::new(state);
        let settled = settle_prompts(
            state,
            &mut records,
            PromptSettlement::Interrupted,
            &self.at,
            |scope| scope.test_cycle_id.is_some() || scope.test_execution_id.is_some(),
        );
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

        for request in state.run_requests.values_mut() {
            // A Request Whose Cycle Report Intent Is Durable Has Settled Its
            // Outcome; Only Its Seal Is Pending, And A Later Stop Completes It.
            if request.summary.phase.terminal()
                || state
                    .cycle_report_finalizations
                    .contains_key(&request.summary.cycle_id)
            {
                continue;
            }

            request.summary.phase = RunPhase::Interrupted;
            request.summary.finished_at = Some(self.at.clone());
            request.summary.paused = false;
            request.pending.clear();
            request.groups.clear();
            request.conclusion = Some(CycleStatus::Stopped);
            request.failure = Some("runtime restarted before request completion".into());
        }

        // Sealed Cycles Are Served From Native History After A Restart
        state.retire_sealed_cycles();
        for (id, cycle) in &mut state.cycles {
            // An Unsealed Cycle Report Is Not Resumed: The Live Cycle Stops, And
            // A Later Stop Seals It From The Persisted Intent's Frozen Source.
            let mut changed = state.cycle_report_finalizations.contains_key(id);
            changed |= matches!(
                cycle.lifecycle,
                CycleStatus::Ready
                    | CycleStatus::Running
                    | CycleStatus::Paused
                    | CycleStatus::Stopping
            );
            for id in &cycle.execution_ids {
                let execution = state
                    .executions
                    .get_mut(id)
                    .expect("cycle execution owner exists");
                changed |= interrupt_execution(execution, &self.at);
            }
            if changed {
                cycle.lifecycle = CycleStatus::Stopped;
                cycle.result = None;
                cycle.touch_at(self.at.clone());
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn interrupt_execution(execution: &mut TestExecution, at: &Timestamp) -> bool {
    if !matches!(
        execution.status,
        ExecutionStatus::Pending | ExecutionStatus::Running
    ) && !(execution.status == ExecutionStatus::Skipped && execution.finished_at.is_none())
    {
        return false;
    }

    execution.interrupt_step_attempt();
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
    use crate::core::{Metadata, TestCycleId};

    #[test]
    fn cycle_intent_survives_state_encoding_and_refuses_a_new_cursor() {
        let mut state = RuntimeState::new(crate::model::identity::new_id());
        let id = crate::model::identity::new_id::<TestCycleId>();
        let at = crate::model::time::now_utc();
        let source = crate::execution::TestCycle {
            saved: None,
            test_cycle_id: id.clone(),
            session_id: Some(state.session_id.clone()),
            metadata: Metadata {
                identifier: crate::core::Identifier::new("cycle.restart").unwrap(),
                label: "Cycle Restart".into(),
                description: None,
                tags: vec![],
                attributes: Default::default(),
                created_at: None,
                updated_at: None,
            },
            lifecycle: CycleStatus::Stopped,
            created_at: at.clone(),
            started_at: Some(at.clone()),
            updated_at: at,
            execution_ids: vec![],
            records: Default::default(),
            continue_on_failure: false,
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
        let request = RequestCycleReportFinalization {
            intent: CycleReportFinalization {
                source,
                evidence_at: cursor.clone(),
            },
        };
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
        assert_eq!(
            recovered.cycle_report_finalizations[&id].evidence_at,
            cursor
        );
        assert_eq!(
            recovered.cycle_report_finalizations[&id].source,
            request.intent.source
        );
    }
}
