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

// Local Imports
use super::*;
use crate::records::BuiltinRecordType;
use crate::transactions::records::draft::BuiltinDraft;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.execution.request.complete")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompleteRun {
    pub request_id: CommandId,
    pub at: Timestamp,

    #[serde(default)]
    pub unsealed: Option<Unsealed>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unsealed {
    pub cause: String,
    pub execution_ids: Vec<TestExecutionId>,
}

#[harmos::transaction(id = "dokime.execution.request.settle")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettleRun {
    pub request_id: CommandId,
    pub at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for CompleteRun {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        outcomes_settled(state, &self.request_id)?;
        let request = &state.run_requests[&self.request_id];
        if request.summary.phase.terminal() {
            return Ok(());
        }
        if request.completion_at.is_none() {
            return Err("terminal evidence has not been staged".into());
        }
        if request.completion_at.as_ref() != Some(&self.at) {
            return Err("completion differs from its staged timestamp".into());
        }
        if let Some(unsealed) = &self.unsealed {
            let members = &request.selected_execution_ids;
            if unsealed.execution_ids.iter().any(|id| !members.contains(id)) {
                return Err("unsealed executions differ from cycle membership".into());
            }
        } else if request.selected_execution_ids.iter().any(|id| {
            state.executions.get(id).is_none_or(|execution| {
                execution.finished_at.is_none()
                    || (execution.saved.is_none() && !state.report_finalizations.get(id).is_some_and(|intent| intent.lifecycle.is_some()))
            })
        }) {
            return Err("completed request requires drained execution evidence and a durable report intent".into());
        }
        Ok(())
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        crate::transactions::history::collect(
            state,
            self,
            state.run_requests.get(&self.request_id).map(|request| &request.summary.cycle_id),
            out,
        );
        if let Some(unsealed) = &self.unsealed
            && !state.run_requests[&self.request_id].summary.phase.terminal()
        {
            self.unsealed_drafts(state, unsealed).collect(out);
        }
    }
    fn apply(&self, state: &mut RuntimeState) {
        let request = state.run_requests.get_mut(&self.request_id).expect("checked request");
        if request.summary.phase.terminal() {
            return;
        }
        let status = conclusion(request);
        let continuation_execution_id = self.continuation(state);
        if let Some(unsealed) = &self.unsealed {
            let records = self.unsealed_drafts(state, unsealed);
            record_publication_failure(state.run_requests.get_mut(&self.request_id).expect("checked request"), &unsealed.cause);
            records.apply(state);
        }
        let request = state.run_requests.get_mut(&self.request_id).expect("checked request");
        request.continuation_execution_id = continuation_execution_id;
        request.summary.phase = if self.unsealed.is_some() {
            RunPhase::Failed
        } else {
            request.publication_failure = None;
            outcome_phase(status)
        };
        request.summary.finished_at = Some(self.at.clone());
        // A Cancelled Admission Retains Its Unstarted Attempt Count.
        request.summary.unstarted += request.pending.len() as u64;
        request.pending.clear();
        let cycle = state.cycles.get_mut(&request.summary.cycle_id).expect("request cycle");
        cycle.result = Some(crate::transactions::provider_finish::summarize(
            cycle.execution_ids.iter().map(|id| state.executions.get(id).expect("cycle execution owner exists")),
        ));
        // An Active Cycle Returns To Idle: A Request's Outcome Belongs To Its Executions And `result`.
        cycle.lifecycle = if cycle.phase == dokime_domain::model::execution::result::CyclePhase::Active {
            CycleStatus::Ready
        } else {
            crate::transactions::lifecycle::closed_lifecycle(cycle, &state.executions)
        };
        cycle.touch_at(self.at.clone());
        let cycle_id = cycle.test_cycle_id.clone();
        state.run_queue.retain(|id| id != &self.request_id);
        sync_cycle_activity(state, &cycle_id);
    }
}

impl CompleteRun {
    fn continuation(&self, state: &RuntimeState) -> Option<TestExecutionId> {
        let request = &state.run_requests[&self.request_id];
        let cycle = &state.cycles[&request.summary.cycle_id];
        if !cycle.continue_automatically || request.summary.stop_requested || cycle.ensure_active().is_err() {
            return None;
        }
        let previous = request.test_execution_id.as_ref()?;
        let position = cycle.execution_ids.iter().position(|id| id == previous)?;
        for id in cycle.execution_ids.iter().skip(position + 1) {
            let Some(execution) = state.executions.get(id) else { continue };
            if crate::transactions::provider::execution_available(state, execution) {
                return Some(id.clone());
            }
        }
        None
    }

    fn unsealed_drafts(&self, state: &RuntimeState, unsealed: &Unsealed) -> crate::transactions::records::draft::Drafts {
        let request = &state.run_requests[&self.request_id];
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        let mut draft = |record_type: BuiltinRecordType, scope: RecordScope| {
            records.draft(
                BuiltinDraft {
                    record_type,
                    source: "runtime".into(),
                    severity: RecordSeverity::Error,
                    message: Some(unsealed.cause.clone()),
                    details: BTreeMap::new(),
                    scope: Some(scope),
                },
                self.at.clone(),
            );
        };
        for id in &unsealed.execution_ids {
            draft(BuiltinRecordType::TestExecutionFinalizationFailed, state.executions[id].scope.clone());
        }
        draft(BuiltinRecordType::TestCycleFinalizationFailed, RecordScope::default().with_cycle(request.summary.cycle_id.clone()));
        records
    }

    fn drafts(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        let request = &state.run_requests[&self.request_id];
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        if request.summary.phase.terminal() {
            return records;
        }
        let status = conclusion(request);
        let cycle_status = if matches!(status, CycleStatus::Failed | CycleStatus::Stopped) {
            status
        } else {
            crate::transactions::provider_finish::lifecycle_for(Some(&crate::transactions::provider_finish::summarize(
                state.cycles[&request.summary.cycle_id].execution_ids.iter().map(|id| &state.executions[id]),
            )))
        };
        let name = match cycle_status {
            CycleStatus::Completed => BuiltinRecordType::TestCycleCompleted,
            CycleStatus::Stopped => BuiltinRecordType::TestCycleStopped,
            _ => BuiltinRecordType::TestCycleFailed,
        };
        let cycle_complete = state.cycles[&request.summary.cycle_id]
            .execution_ids
            .iter()
            .all(|id| state.executions[id].finished_at.is_some() || request.selected_execution_ids.contains(id));
        if cycle_complete {
            records.draft(
                BuiltinDraft {
                    record_type: name,
                    source: "runtime".into(),
                    severity: if cycle_status == CycleStatus::Failed { RecordSeverity::Error } else { RecordSeverity::Info },
                    message: Some(request.failure.clone().unwrap_or_else(|| "Cycle provider outcome settled.".into())),
                    details: BTreeMap::new(),
                    scope: Some(RecordScope::default().with_cycle(request.summary.cycle_id.clone())),
                },
                self.at.clone(),
            );
        }
        records.draft(
            BuiltinDraft {
                record_type: match status {
                    CycleStatus::Completed => BuiltinRecordType::ProviderExecutionCompleted,
                    CycleStatus::Stopped => BuiltinRecordType::TestExecutionStopped,
                    _ => BuiltinRecordType::ProviderExecutionFailed,
                },
                source: "provider".into(),
                severity: if status == CycleStatus::Failed { RecordSeverity::Error } else { RecordSeverity::Info },
                message: Some(request.failure.clone().unwrap_or_else(|| "Cycle provider work settled.".into())),
                details: BTreeMap::new(),
                scope: Some(RecordScope::default().with_cycle(request.summary.cycle_id.clone())),
            },
            self.at.clone(),
        );
        records
    }
}

impl Transaction<RuntimeState> for SettleRun {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        outcomes_settled(state, &self.request_id)
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        crate::transactions::history::collect(
            state,
            self,
            state.run_requests.get(&self.request_id).map(|request| &request.summary.cycle_id),
            out,
        );
        if state.run_requests[&self.request_id].completion_at.is_none() {
            CompleteRun {
                request_id: self.request_id.clone(),
                at: self.at.clone(),
                unsealed: None,
            }
            .drafts(state)
            .collect(out);
        }
    }
    fn apply(&self, state: &mut RuntimeState) {
        if state.run_requests[&self.request_id].completion_at.is_some() {
            return;
        }
        let records = CompleteRun {
            request_id: self.request_id.clone(),
            at: self.at.clone(),
            unsealed: None,
        }
        .drafts(state);
        state.run_requests.get_mut(&self.request_id).expect("checked request").completion_at = Some(self.at.clone());
        records.apply(state);
        // Unstarted Selected Members Settle Before Their Evidence Is Sealed.
        let request = &state.run_requests[&self.request_id];
        for id in &request.selected_execution_ids {
            let execution = state.executions.get_mut(id).expect("selected execution owner exists");
            if execution.status != ExecutionStatus::Skipped || execution.finished_at.is_some() {
                continue;
            }
            execution.status = ExecutionStatus::Cancelled;
            execution.finished_at = Some(self.at.clone());
            execution.result = Some(ResultSummary {
                skipped: 1,
                ..summary(ResultStatus::Cancelled)
            });
            for step in &mut execution.steps {
                if step.status == StepStatus::Pending {
                    step.status = StepStatus::Skipped;
                    step.finished_at = Some(self.at.clone());
                }
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn record_publication_failure(request: &mut RunRequest, cause: &str) {
    request.publication_failure = Some(cause.to_owned());
}

fn outcome_phase(status: CycleStatus) -> RunPhase {
    match status {
        CycleStatus::Completed => RunPhase::Completed,
        CycleStatus::Stopped => RunPhase::Stopped,
        _ => RunPhase::Failed,
    }
}

fn conclusion(request: &RunRequest) -> CycleStatus {
    if request.summary.stop_requested {
        CycleStatus::Stopped
    } else if request.summary.failed > 0 || request.failure.is_some() {
        CycleStatus::Failed
    } else {
        request.conclusion.unwrap_or(CycleStatus::Failed)
    }
}

fn outcomes_settled(state: &RuntimeState, request_id: &CommandId) -> Result<(), String> {
    let request = state.run_requests.get(request_id).ok_or("request not found")?;
    if request.summary.phase.terminal() {
        return Ok(());
    }
    if request.summary.phase != RunPhase::Finishing {
        return Err("provider completion has not settled".into());
    }
    if request.summary.finished != request.summary.prepared {
        return Err("prepared attempts have not all finished".into());
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn settled_request(stopped: bool) -> (RuntimeState, CompleteRun) {
        let mut state = RuntimeState::new(crate::model::identity::new_id());
        let cycle_id = TestCycleId::from_seed("publication-failure-cycle");
        let request_id = CommandId::from_seed("publication-failure-request");
        let at = crate::model::time::now_utc();
        state.cycles.insert(
            cycle_id.clone(),
            dokime_domain::model::execution::result::TestCycle {
                saved: None,
                test_cycle_id: cycle_id.clone(),
                session_id: None,
                metadata: Metadata::new(Identifier::new("publication.fixture").unwrap(), "Publication", None),
                lifecycle: CycleStatus::Running,
                phase: Default::default(),
                created_at: at.clone(),
                started_at: Some(at.clone()),
                updated_at: at.clone(),
                execution_ids: Vec::new(),
                records: Default::default(),
                continue_automatically: false,
                result: None,
            },
        );
        state.run_requests.insert(
            request_id.clone(),
            RunRequest {
                summary: RunSummary {
                    mode: Default::default(),
                    request_id: request_id.clone(),
                    cycle_id,
                    phase: RunPhase::Finishing,
                    paused: false,
                    stop_requested: stopped,
                    planned: 0,
                    prepared: 0,
                    finished: 0,
                    unstarted: 0,
                    failed: 0,
                    accepted_at: at.clone(),
                    provider_started_at: Some(at.clone()),
                    finished_at: None,
                },
                target: ExecutionProviderTarget {
                    source: "fixture".into(),
                    binding: crate::domains::SidecarExecutionBinding { workflow: "fixture".into() },
                },
                test_execution_id: None,
                selected_execution_ids: Vec::new(),
                pending: Default::default(),
                settled_execution_ids: Vec::new(),
                conclusion: Some(CycleStatus::Completed),
                failure: None,
                publication_failure: None,
                completion_at: Some(at.clone()),
                continuation_execution_id: None,
            },
        );
        (
            state,
            CompleteRun {
                request_id,
                at,
                unsealed: Some(Unsealed {
                    cause: "report memory exhausted".into(),
                    execution_ids: Vec::new(),
                }),
            },
        )
    }

    #[test]
    fn continuation_is_frozen_before_later_appends() {
        for existing_next in [false, true] {
            let (mut state, mut complete) = settled_request(false);
            complete.unsealed = None;
            let cycle_id = state.run_requests[&complete.request_id].summary.cycle_id.clone();
            let first = append_case(&mut state, &cycle_id, "first");
            let expected = existing_next.then(|| append_case(&mut state, &cycle_id, "second"));
            state.cycles.get_mut(&cycle_id).unwrap().continue_automatically = true;
            let request = state.run_requests.get_mut(&complete.request_id).unwrap();
            request.test_execution_id = Some(first);
            request.conclusion = Some(CycleStatus::Failed);

            complete.check(&state).unwrap();
            complete.apply(&mut state);
            assert_eq!(state.run_requests[&complete.request_id].continuation_execution_id, expected);
            append_case(&mut state, &cycle_id, "later");
            complete.check(&state).unwrap();
            complete.apply(&mut state);
            assert_eq!(state.run_requests[&complete.request_id].continuation_execution_id, expected);
        }
    }

    fn append_case(state: &mut RuntimeState, cycle_id: &TestCycleId, name: &str) -> TestExecutionId {
        use dokime_domain::model::definition::suite::TestCase;
        use dokime_domain::model::execution::snapshot::ExecutionSnapshot;
        use dokime_domain::model::identity::{CaseId, TemplateId};

        let id = TestExecutionId::from_seed(name);
        let case = TestCase {
            suite_id: "continuation.suite".into(),
            test_case_id: CaseId::from_seed(name),
            test_case_template_id: TemplateId::from_seed("continuation-template"),
            metadata: Metadata::new(Identifier::new("continuation.case").unwrap(), name, None),
            suite: None,
            parameters: Default::default(),
            generation_index: 0,
            label_number: 0,
        };
        let snapshot = ExecutionSnapshot::from_captured(case, None, Vec::new(), None, None, Default::default(), Default::default());
        let mut execution = crate::transactions::provider::scheduled_execution_record(cycle_id, id.clone(), snapshot);
        execution.status = dokime_domain::model::execution::result::ExecutionStatus::Skipped;
        execution.result = None;
        state.executions.insert(id.clone(), execution);
        state.cycles.get_mut(cycle_id).unwrap().execution_ids.push(id.clone());
        id
    }

    #[test]
    fn publication_refusal_preserves_actual_cycle_outcomes_and_counts() {
        for stopped in [false, true] {
            let (mut state, complete) = settled_request(stopped);
            complete.check(&state).unwrap();
            complete.apply(&mut state);
            let request = &state.run_requests[&complete.request_id];
            assert_eq!(request.summary.phase, RunPhase::Failed);
            assert_eq!(request.summary.failed, 0);
            assert!(request.failure.is_none());
            assert_eq!(request.publication_failure.as_deref(), Some("report memory exhausted"));
            let cycle = &state.cycles[&request.summary.cycle_id];
            assert_eq!(cycle.lifecycle, CycleStatus::Ready, "a request outcome leaves its Active cycle idle");
            assert!(cycle.saved.is_none());
            let record_count = state.record_count;
            let summary = request.summary.clone();
            complete.check(&state).unwrap();
            complete.apply(&mut state);
            assert_eq!(state.record_count, record_count, "retrying a failed completion adds no duplicate records");
            assert_eq!(state.run_requests[&complete.request_id].summary, summary);
        }
    }

    #[test]
    fn publication_diagnosis_does_not_replace_a_provider_failure() {
        let (mut state, complete) = settled_request(false);
        let request = state.run_requests.get_mut(&complete.request_id).unwrap();
        request.failure = Some("device disconnected".into());
        record_publication_failure(request, "report memory exhausted");
        assert_eq!(request.interruption().as_deref(), Some("device disconnected; report memory exhausted"));
        request.publication_failure = None;
        assert_eq!(request.interruption().as_deref(), Some("device disconnected"));
        assert_eq!(conclusion(request), CycleStatus::Failed);
    }
}
