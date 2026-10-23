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
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::state::RuntimeState;
use crate::transactions::provider::request::RunPhase;
use crate::transactions::records::draft::BuiltinDraft;
use crate::transactions::records::draft::Drafts;
use dokime_domain::model::execution::result::{CyclePhase, CycleStatus, ExecutionStatus, ResultStatus, ResultSummary, StepStatus};
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

const CANCELLED_MESSAGE: &str = "Execution cancelled before it started.";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.cancel", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancelExecution {
    pub test_execution_id: TestExecutionId,
    pub occurred_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CancelRefusal {
    pub code: &'static str,
    pub message: &'static str,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CancelAdmission {
    Cancelled,
    Unqueued,
    Queued(CommandId),
}

/* === Traits === */

/* === Implementations === */

impl CancelExecution {
    pub(crate) fn admit(&self, state: &RuntimeState) -> Result<CancelAdmission, CancelRefusal> {
        let refuse = |code, message| Err(CancelRefusal { code, message });

        // Resolve The Member And Its Open Cycle.
        if state.shutting_down {
            return refuse("runtime_shutting_down", "runtime is shutting down");
        }
        let Some(execution) = state.executions.get(&self.test_execution_id) else {
            return refuse("execution_not_found", "execution not found");
        };
        if execution.status == ExecutionStatus::Skipped && execution.started_at.is_none() && execution.finished_at.is_some() {
            return Ok(CancelAdmission::Cancelled);
        }
        let Some(cycle) = state.cycles.get(&execution.test_cycle_id) else {
            return refuse("cycle_not_found", "execution cycle not found");
        };
        if !cycle.execution_ids.contains(&self.test_execution_id) {
            return refuse("execution_not_planned", "execution is not a member of its cycle");
        }
        if cycle.phase != CyclePhase::Active {
            return refuse("cycle_closed", "cycle is not active; cancellation is closed");
        }

        // Only An Unstarted Member Or Its Untouched Queued Request Cancels.
        if execution.status != ExecutionStatus::Skipped || execution.started_at.is_some() || execution.finished_at.is_some() {
            return refuse("execution_already_started", "execution has already started");
        }
        let Some(request) = crate::transactions::ControlExecution::owning_request(state, &self.test_execution_id) else {
            return Ok(CancelAdmission::Unqueued);
        };
        let untouched =
            request.summary.phase == RunPhase::Queued && request.summary.prepared == 0 && request.summary.provider_started_at.is_none();
        if !untouched {
            return refuse("execution_already_started", "execution has already been prepared");
        }
        let request_id = request.summary.request_id.clone();
        Ok(CancelAdmission::Queued(request_id))
    }

    fn drafts(&self, state: &RuntimeState) -> Drafts {
        let mut records = Drafts::new(state);
        let scope = state.executions[&self.test_execution_id].scope.clone();
        let draft = BuiltinDraft {
            record_type: BuiltinRecordType::TestExecutionStopped,
            source: "runtime".into(),
            severity: RecordSeverity::Info,
            message: Some(CANCELLED_MESSAGE.to_owned()),
            details: Default::default(),
            scope: Some(scope),
        };
        records.draft(draft, self.occurred_at.clone());
        records
    }
}

impl Transaction<RuntimeState> for CancelExecution {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        self.admit(state).map_err(|refusal| refusal.message.to_owned())?;
        Ok(())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        if self.admit(state).is_ok_and(|admission| admission != CancelAdmission::Cancelled) {
            self.drafts(state).collect(out);
        }
    }

    fn apply(&self, state: &mut RuntimeState) {
        let request_id = match self.admit(state).expect("checked execution cancellation") {
            CancelAdmission::Cancelled => return,
            CancelAdmission::Unqueued => None,
            CancelAdmission::Queued(request_id) => Some(request_id),
        };
        let records = self.drafts(state);
        let execution = state.executions.get_mut(&self.test_execution_id).expect("checked execution");
        skip_unstarted(execution, &self.occurred_at);
        let cycle_id = execution.test_cycle_id.clone();

        // Queued Staging Requires The Stop Flag; Continuation Remains The Cycle's.
        if let Some(request_id) = &request_id {
            let request = state.run_requests.get_mut(request_id).expect("checked queued request");
            request.summary.stop_requested = true;
            request.summary.paused = false;
        }
        settle_cycle(state, &cycle_id, request_id.as_ref(), &self.occurred_at);
        records.apply(state);
    }
}

/* === Public Functions === */

/* === Private Functions === */

// A Cancelled Member Is Skipped: It Never Ran, So It Has No Stopped Outcome.
fn skip_unstarted(execution: &mut dokime_domain::model::execution::result::TestExecution, at: &Timestamp) {
    execution.status = ExecutionStatus::Skipped;
    execution.finished_at = Some(at.clone());
    execution.result = Some(ResultSummary { passed: 0, failed: 0, blocked: 0, skipped: 1, status: ResultStatus::Cancelled });
    let pending = execution.steps.iter_mut().filter(|step| step.status == StepStatus::Pending);
    for step in pending {
        step.status = StepStatus::Skipped;
        step.finished_at = Some(at.clone());
    }
}

fn settle_cycle(state: &mut RuntimeState, cycle_id: &TestCycleId, cancelled_request: Option<&CommandId>, at: &Timestamp) {
    // Another Live Request Owns The Cycle Activity Until It Settles.
    let busy = state.run_requests.values().any(|request| {
        &request.summary.cycle_id == cycle_id && !request.summary.phase.terminal() && Some(&request.summary.request_id) != cancelled_request
    });
    let members = state.cycles[cycle_id].execution_ids.iter().map(|id| &state.executions[id]).collect::<Vec<_>>();
    let result = super::provider_finish::summarize(members);

    // An Active Cycle Is Idle Between Runs; Its Outcome Lives In Its Result.
    let cycle = state.cycles.get_mut(cycle_id).expect("checked cycle");
    if !busy {
        cycle.lifecycle = CycleStatus::Ready;
    }
    cycle.result = Some(result);
    cycle.touch_at(at.clone());
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::definition::suite::TestCase;
    use dokime_domain::model::execution::snapshot::ExecutionSnapshot;
    use dokime_domain::model::identity::{CaseId, Identifier, Metadata, TemplateId};

    fn planned_cycle() -> (RuntimeState, TestCycleId, [TestExecutionId; 2]) {
        let mut state = RuntimeState::new(crate::model::identity::new_id());
        let cycle_id = TestCycleId::from_seed("cancel-cycle");
        let at = crate::model::time::now_utc();
        let ids = [TestExecutionId::from_seed("cancel-first"), TestExecutionId::from_seed("cancel-second")];
        for id in &ids {
            let case = TestCase {
                suite_id: "cancel.suite".into(),
                test_case_id: CaseId::from_seed(id.to_string()),
                test_case_template_id: TemplateId::from_seed("cancel-template"),
                metadata: Metadata::new(Identifier::new("cancel.case").unwrap(), "Cancel Case", None),
                suite: None,
                parameters: Default::default(),
                generation_index: 0,
                label_number: 0,
            };
            let snapshot = ExecutionSnapshot::from_captured(case, None, Vec::new(), None, None, Default::default(), Default::default());
            let mut execution = crate::transactions::provider::scheduled_execution_record(&cycle_id, id.clone(), snapshot);
            execution.status = ExecutionStatus::Skipped;
            execution.result = None;
            state.executions.insert(id.clone(), execution);
        }
        state.cycles.insert(
            cycle_id.clone(),
            dokime_domain::model::execution::result::TestCycle {
                saved: None,
                test_cycle_id: cycle_id.clone(),
                session_id: None,
                metadata: Metadata::new(Identifier::new("cancel.fixture").unwrap(), "Cancel", None),
                lifecycle: CycleStatus::Ready,
                phase: CyclePhase::Active,
                created_at: at.clone(),
                started_at: None,
                updated_at: at,
                execution_ids: ids.to_vec(),
                records: Default::default(),
                continue_automatically: true,
                result: None,
            },
        );
        (state, cycle_id, ids)
    }

    #[test]
    fn cancel_skips_one_member_and_keeps_continuation() {
        let (mut state, cycle_id, [first, second]) = planned_cycle();
        let cancel = CancelExecution { test_execution_id: first.clone(), occurred_at: crate::model::time::now_utc() };
        cancel.check(&state).unwrap();
        cancel.apply(&mut state);

        let cancelled = &state.executions[&first];
        assert_eq!(cancelled.status, ExecutionStatus::Skipped);
        assert!(cancelled.finished_at.is_some(), "a cancelled member is finished, unlike a planned one");
        assert_eq!(cancelled.result.as_ref().map(|result| (result.skipped, &result.status)), Some((1, &ResultStatus::Cancelled)));
        assert!(cancelled.steps.iter().all(|step| step.status != StepStatus::Pending));
        assert_eq!(state.executions[&second].status, ExecutionStatus::Skipped);
        let cycle = &state.cycles[&cycle_id];
        assert!(cycle.continue_automatically);
        assert_eq!(cycle.phase, CyclePhase::Active);
        assert_eq!(cycle.lifecycle, CycleStatus::Ready);
        assert_eq!(cycle.result.as_ref().map(|result| result.skipped), Some(1));

        // A Repeated Cancel Is A No-Op; A Started Member Refuses.
        let records = state.record_count;
        cancel.check(&state).unwrap();
        cancel.apply(&mut state);
        assert_eq!(state.record_count, records);
        state.executions.get_mut(&second).unwrap().started_at = Some(crate::model::time::now_utc());
        let started = CancelExecution { test_execution_id: second, occurred_at: crate::model::time::now_utc() };
        assert_eq!(started.admit(&state).unwrap_err().code, "execution_already_started");
    }

    #[test]
    fn cancel_refuses_a_closed_cycle() {
        let (mut state, cycle_id, [first, _]) = planned_cycle();
        state.cycles.get_mut(&cycle_id).unwrap().phase = CyclePhase::Sealing;
        let cancel = CancelExecution { test_execution_id: first, occurred_at: crate::model::time::now_utc() };
        assert_eq!(cancel.admit(&state).unwrap_err().code, "cycle_closed");
    }
}
