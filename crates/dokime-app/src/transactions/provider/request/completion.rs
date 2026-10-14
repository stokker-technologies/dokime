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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.execution.request.complete", version = 3, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompleteRun {
    pub request_id: CommandId,
    pub at: Timestamp,
    pub sealed_cycle: Option<dokime::execution::TestCycle>,
    pub sealed_executions: Vec<dokime::execution::saved::ExecutionSeal>,
    /// A final report that could not be produced. The run settles Failed with
    /// its cause instead of a seal; nothing claims a report that does not exist.
    #[serde(default)]
    pub unsealed: Option<Unsealed>,
}

/// The permanent cause a run's final reports could not be sealed, and the
/// executions whose own reports were never produced. A cycle seal failure
/// names no executions: their seals already exist and stay as they are.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unsealed {
    pub cause: String,
    pub execution_ids: Vec<TestExecutionId>,
}

#[harmos::transaction(id = "dokime.execution.request.settle", version = 2, irreversible)]
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
        if !request.summary.phase.terminal() && request.completion_at.is_none() {
            return Err("terminal evidence has not been staged".into());
        }
        if !request.summary.phase.terminal() && request.completion_at.as_ref() != Some(&self.at) {
            return Err("completion differs from its staged timestamp".into());
        }
        if self.sealed_cycle.is_none() && !self.sealed_executions.is_empty() {
            return Err("sealed executions require a sealed cycle".into());
        }
        if let Some(unsealed) = &self.unsealed {
            if self.sealed_cycle.is_some() {
                return Err("an unsealed completion cannot carry a seal".into());
            }
            let members = &state.cycles[&request.summary.cycle_id].execution_ids;
            if unsealed
                .execution_ids
                .iter()
                .any(|id| !members.contains(id))
            {
                return Err("unsealed executions differ from cycle membership".into());
            }
        }
        if let Some(sealed) = &self.sealed_cycle {
            let current = &state.cycles[&request.summary.cycle_id];
            let seal = sealed
                .saved
                .as_ref()
                .ok_or("completed cycle has no saved seal")?;
            seal.validate(sealed).map_err(|source| source.to_string())?;
            if self.sealed_executions.len() != sealed.execution_ids.len()
                || self
                    .sealed_executions
                    .iter()
                    .zip(&sealed.execution_ids)
                    .any(|(seal, id)| &seal.source.execution.execution_id != id)
            {
                return Err("sealed executions differ from cycle membership".into());
            }
            for (member, seal) in seal.executions.iter().zip(&self.sealed_executions) {
                if !member
                    .position
                    .covers(&seal.dependencies_saved)
                    .map_err(|source| source.to_string())?
                {
                    return Err("cycle member is not saved after its dependencies".into());
                }
                seal.source
                    .validate(&member.execution_id)
                    .map_err(|source| source.to_string())?;
            }
            if sealed.test_cycle_id != current.test_cycle_id
                || sealed.execution_ids != current.execution_ids
                || sealed.updated_at != self.at
                || sealed.metadata != current.metadata
                || sealed.session_id != current.session_id
                || sealed.created_at != current.created_at
                || sealed.started_at != current.started_at
                || sealed.records != current.records
                || sealed.continue_on_failure != current.continue_on_failure
                || (request.test_execution_id.is_none() && sealed.lifecycle != conclusion(request))
            {
                return Err("cycle changed after saved finalization".into());
            }
        }
        Ok(())
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        if let Some(unsealed) = &self.unsealed
            && !state.run_requests[&self.request_id]
                .summary
                .phase
                .terminal()
        {
            self.unsealed_drafts(state, unsealed).collect(out);
        }
    }
    fn apply(&self, state: &mut RuntimeState) {
        let request = state
            .run_requests
            .get_mut(&self.request_id)
            .expect("checked request");
        if request.summary.phase.terminal() {
            return;
        }
        if let Some(unsealed) = &self.unsealed {
            let records = self.unsealed_drafts(state, unsealed);
            fail_unsealed(state, &self.request_id, unsealed);
            records.apply(state);
        }
        let request = state
            .run_requests
            .get_mut(&self.request_id)
            .expect("checked request");
        let status = conclusion(request);
        request.summary.phase = match status {
            CycleStatus::Completed => RunPhase::Completed,
            CycleStatus::Stopped => RunPhase::Stopped,
            _ => RunPhase::Failed,
        };
        request.summary.finished_at = Some(self.at.clone());
        // A target group the policy or a Stop never let start still owes the
        // receipt its planned attempts, as unstarted.
        request.summary.unstarted += request
            .groups
            .iter()
            .map(|group| group.pending.len() as u64)
            .sum::<u64>();
        request.summary.unstarted += request.pending.len() as u64;
        request.pending.clear();
        request.groups.clear();
        let cycle = state
            .cycles
            .get_mut(&request.summary.cycle_id)
            .expect("request cycle");
        for id in &cycle.execution_ids {
            if request
                .test_execution_id
                .as_ref()
                .is_some_and(|selected| selected != id)
            {
                continue;
            }
            let execution = state
                .executions
                .get_mut(id)
                .expect("cycle execution owner exists");
            if execution.status == ExecutionStatus::Skipped && execution.finished_at.is_none() {
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
        cycle.result = Some(crate::transactions::provider_finish::summarize(
            cycle.execution_ids.iter().map(|id| {
                state
                    .executions
                    .get(id)
                    .expect("cycle execution owner exists")
            }),
        ));
        cycle.lifecycle = if request.test_execution_id.is_some()
            && cycle.execution_ids.iter().any(|id| {
                let execution = &state.executions[id];
                execution.status == ExecutionStatus::Skipped && execution.finished_at.is_none()
            }) {
            CycleStatus::Ready
        } else if request.test_execution_id.is_some() {
            crate::transactions::provider_finish::lifecycle_for(cycle.result.as_ref())
        } else {
            status
        };
        if self.unsealed.is_some() && cycle.lifecycle != CycleStatus::Ready {
            cycle.lifecycle = CycleStatus::Failed;
        }
        cycle.touch_at(self.at.clone());
        if let Some(sealed) = &self.sealed_cycle {
            *cycle = sealed.clone();
            for seal in &self.sealed_executions {
                state
                    .executions
                    .get_mut(&seal.source.execution.execution_id)
                    .expect("checked member")
                    .saved = Some(seal.clone());
            }
        }
    }
}

impl CompleteRun {
    /// The permanent finalization cause, recorded once against every execution
    /// it left unsealed and once against the cycle as its terminal diagnosis.
    fn unsealed_drafts(
        &self,
        state: &RuntimeState,
        unsealed: &Unsealed,
    ) -> crate::transactions::records::draft::Drafts {
        let request = &state.run_requests[&self.request_id];
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        let mut draft = |record_type: &str, scope: RecordScope| {
            records.draft(
                ProductRecordDraft {
                    record_type: identifier(record_type),
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
            draft(
                "test.execution.finalization.failed",
                state.executions[id].scope.clone(),
            );
        }
        draft(
            "test.cycle.finalization.failed",
            RecordScope::default().with_cycle(request.summary.cycle_id.clone()),
        );
        records
    }

    fn drafts(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        let request = &state.run_requests[&self.request_id];
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        if request.summary.phase.terminal() {
            return records;
        }
        let status = conclusion(request);
        let cycle_status = if request.test_execution_id.is_some() {
            crate::transactions::provider_finish::lifecycle_for(Some(
                &crate::transactions::provider_finish::summarize(
                    state.cycles[&request.summary.cycle_id]
                        .execution_ids
                        .iter()
                        .map(|id| &state.executions[id]),
                ),
            ))
        } else {
            status
        };
        let name = match cycle_status {
            CycleStatus::Completed => "test.cycle.completed",
            CycleStatus::Stopped => "test.cycle.stopped",
            _ => "test.cycle.failed",
        };
        let cycle_complete = request.test_execution_id.as_ref().is_none_or(|selected| {
            !state.cycles[&request.summary.cycle_id]
                .execution_ids
                .iter()
                .any(|id| {
                    let execution = &state.executions[id];
                    id != selected
                        && execution.status == ExecutionStatus::Skipped
                        && execution.finished_at.is_none()
                })
        });
        if cycle_complete {
            records.draft(
                ProductRecordDraft {
                    record_type: identifier(name),
                    source: "runtime".into(),
                    severity: if cycle_status == CycleStatus::Failed {
                        RecordSeverity::Error
                    } else {
                        RecordSeverity::Info
                    },
                    message: Some(
                        request
                            .failure
                            .clone()
                            .unwrap_or_else(|| "Cycle provider outcome settled.".into()),
                    ),
                    details: BTreeMap::new(),
                    scope: Some(
                        RecordScope::default().with_cycle(request.summary.cycle_id.clone()),
                    ),
                },
                self.at.clone(),
            );
        }
        records.draft(
            ProductRecordDraft {
                record_type: identifier(match status {
                    CycleStatus::Completed => "provider.execution.completed",
                    CycleStatus::Stopped => "test.execution.stopped",
                    _ => "provider.execution.failed",
                }),
                source: "provider".into(),
                severity: if status == CycleStatus::Failed {
                    RecordSeverity::Error
                } else {
                    RecordSeverity::Info
                },
                message: Some(
                    request
                        .failure
                        .clone()
                        .unwrap_or_else(|| "Cycle provider work settled.".into()),
                ),
                details: BTreeMap::new(),
                scope: Some(
                    request
                        .test_execution_id
                        .as_ref()
                        .and_then(|id| state.executions.get(id))
                        .map(|execution| execution.scope.clone())
                        .unwrap_or_else(|| {
                            RecordScope::default().with_cycle(request.summary.cycle_id.clone())
                        }),
                ),
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
        if state.run_requests[&self.request_id].completion_at.is_none() {
            CompleteRun {
                request_id: self.request_id.clone(),
                at: self.at.clone(),
                sealed_cycle: None,
                sealed_executions: Vec::new(),
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
            sealed_cycle: None,
            sealed_executions: Vec::new(),
            unsealed: None,
        }
        .drafts(state);
        state
            .run_requests
            .get_mut(&self.request_id)
            .expect("checked request")
            .completion_at = Some(self.at.clone());
        records.apply(state);
    }
}

/* === Public Functions === */

/* === Private Functions === */
/// Settles every execution whose final report was never produced as Failed,
/// and the request's diagnosis with the cause.
fn fail_unsealed(state: &mut RuntimeState, request_id: &CommandId, unsealed: &Unsealed) {
    let mut failed = 0;
    for id in &unsealed.execution_ids {
        let execution = state.executions.get_mut(id).expect("checked cycle member");
        if execution.status != ExecutionStatus::Failed {
            failed += 1;
        }
        execution.status = ExecutionStatus::Failed;
        let result = execution
            .result
            .get_or_insert_with(|| summary(ResultStatus::Failed));
        result.status = ResultStatus::Failed;
        result.failed = result.failed.max(1);
    }
    let request = state
        .run_requests
        .get_mut(request_id)
        .expect("checked request");
    request.summary.failed += failed;
    request.failure = Some(match request.failure.take() {
        Some(failure) => format!("{failure}; {}", unsealed.cause),
        None => unsealed.cause.clone(),
    });
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
    let request = state
        .run_requests
        .get(request_id)
        .ok_or("request not found")?;
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
