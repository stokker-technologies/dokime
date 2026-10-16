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
use dokime::core::{DokimeNodeIdentity, SessionId, TestExecutionId, Timestamp};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportLifecycle {
    pub id: uuid::Uuid,
    pub execution: TestExecution,
}

/// The Frozen Cycle Header And Durable Member Boundary Used By Every Retry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CycleReportFinalization {
    pub source: dokime::execution::TestCycle,
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
        let execution = &self.lifecycle.execution;
        let id = &execution.test_execution_id;
        let intent = state
            .report_finalizations
            .get(id)
            .ok_or("report evidence boundary is missing")?;
        let owner = state
            .executions
            .get(id)
            .ok_or("report execution owner is missing")?;
        if owner.scope != execution.scope || owner.test_cycle_id != execution.test_cycle_id {
            return Err("report lifecycle owner differs from the managed execution".into());
        }
        if matches!(
            execution.status,
            ExecutionStatus::Pending | ExecutionStatus::Running
        ) || execution.saved.is_some()
        {
            return Err("report lifecycle must be terminal and unsealed".into());
        }
        if intent
            .lifecycle
            .as_ref()
            .is_some_and(|lifecycle| lifecycle.execution != *execution)
        {
            return Err("report lifecycle is already frozen to another terminal snapshot".into());
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
            .get_mut(&self.lifecycle.execution.test_execution_id)
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
            // All Children Were Sealed Before This Intent Became Durable.
            // Restart Must Resume Publication Without Changing Its Outcome.
            if state.cycle_report_finalizations.contains_key(id) {
                continue;
            }
            let mut changed = matches!(
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
    use dokime::core::{Metadata, TestCycleId};

    #[test]
    fn cycle_intent_survives_state_encoding_and_refuses_a_new_cursor() {
        let mut state = RuntimeState::default();
        let id = TestCycleId::new();
        let at = Timestamp::now_utc();
        let source = dokime::execution::TestCycle {
            saved: None,
            test_cycle_id: id.clone(),
            session_id: Some(state.session_id.clone()),
            metadata: Metadata {
                identifier: dokime::core::Identifier::new("cycle.restart").unwrap(),
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
        recovered.session_id = SessionId::new();
        recovered.cycles.get_mut(&id).unwrap().updated_at = Timestamp::now_utc();
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
