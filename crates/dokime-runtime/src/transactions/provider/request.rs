/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod completion;

/* === Re-Exports === */
pub use completion::{CompleteRun, SettleRun, Unsealed};
pub use dokime_domain::model::execution::request::RunPhase;
pub use dokime_domain::model::execution::request::RunSummary;

/* === Imports === */
// Standard Library Imports
use std::collections::VecDeque;
// Third Party Imports

// Local Imports
use super::*;
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::transactions::records::draft::BuiltinDraft;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunGroup {
    pub target: ExecutionProviderTarget,
    pub pending: VecDeque<TestExecutionId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunRequest {
    pub summary: RunSummary,
    pub target: ExecutionProviderTarget,
    pub test_execution_id: Option<TestExecutionId>,
    pub pending: VecDeque<TestExecutionId>,
    #[serde(default)]
    pub groups: VecDeque<RunGroup>,
    #[serde(default)]
    pub settled_execution_ids: Vec<TestExecutionId>,
    // Members an operator stopped one at a time. The request, its cycle and
    // every sibling keep running; only these attempts are cancelled.
    #[serde(default)]
    pub stopped_execution_ids: Vec<TestExecutionId>,
    pub conclusion: Option<CycleStatus>,
    pub failure: Option<String>,
    #[serde(default)]
    pub publication_failure: Option<String>,
    pub completion_at: Option<Timestamp>,
}

#[harmos::transaction(id = "dokime.execution.request.prepare", version = 2, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrepareAttempt {
    pub request_id: CommandId,
    pub ordinal: u64,
    pub at: Timestamp,
}

#[harmos::transaction(id = "dokime.execution.request.dispatch", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DispatchProvider {
    pub request_id: CommandId,
    pub at: Timestamp,
}

#[harmos::transaction(id = "dokime.execution.request.advance", version = 2, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdvanceRunGroup {
    pub request_id: CommandId,
    pub at: Timestamp,
}

/* === Enums === */
/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for RunGroup {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.target.source.capacity())
            .saturating_add(self.target.binding.workflow.capacity())
            .saturating_add(
                deque_memory(&self.pending).saturating_sub(std::mem::size_of_val(&self.pending)),
            )
    }
}

impl harmos::MemorySize for RunRequest {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.summary.memory_size(),
                std::mem::size_of_val(&self.summary),
            ),
            (
                self.test_execution_id.memory_size(),
                std::mem::size_of_val(&self.test_execution_id),
            ),
            (
                deque_memory(&self.pending),
                std::mem::size_of_val(&self.pending),
            ),
            (
                deque_memory(&self.groups),
                std::mem::size_of_val(&self.groups),
            ),
            (
                self.settled_execution_ids.memory_size(),
                std::mem::size_of_val(&self.settled_execution_ids),
            ),
            (
                self.stopped_execution_ids.memory_size(),
                std::mem::size_of_val(&self.stopped_execution_ids),
            ),
            (
                self.failure.memory_size(),
                std::mem::size_of_val(&self.failure),
            ),
            (
                self.publication_failure.memory_size(),
                std::mem::size_of_val(&self.publication_failure),
            ),
            (
                self.completion_at.memory_size(),
                std::mem::size_of_val(&self.completion_at),
            ),
        ];
        fields.into_iter().fold(
            std::mem::size_of::<Self>()
                .saturating_add(self.target.source.capacity())
                .saturating_add(self.target.binding.workflow.capacity()),
            |bytes, (owned, inline)| bytes.saturating_add(owned.saturating_sub(inline)),
        )
    }
}

fn deque_memory<T: harmos::MemorySize>(values: &VecDeque<T>) -> usize {
    values.iter().fold(
        std::mem::size_of_val(values)
            .saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>())),
        |bytes, value| {
            bytes.saturating_add(value.memory_size().saturating_sub(std::mem::size_of::<T>()))
        },
    )
}

impl RunRequest {
    pub(crate) fn interruption(&self) -> Option<String> {
        match (&self.failure, &self.publication_failure) {
            (Some(provider), Some(publication)) => Some(format!("{provider}; {publication}")),
            (Some(cause), None) | (None, Some(cause)) => Some(cause.clone()),
            (None, None) => None,
        }
    }
}

impl Transaction<RuntimeState> for PrepareAttempt {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let request = state
            .run_requests
            .get(&self.request_id)
            .ok_or("run request not found")?;
        if self.ordinal < request.summary.prepared {
            return Ok(());
        }
        active(state, request)?;
        if self.ordinal != request.summary.prepared || request.pending.is_empty() {
            return Err("invalid preparation position".into());
        }
        let execution_id = request.pending.front().expect("checked pending attempt");
        let execution = state
            .executions
            .get(execution_id)
            .ok_or("materialized execution is missing from the cycle")?;
        if execution.test_cycle_id != request.summary.cycle_id {
            return Err("queued execution belongs to another cycle".into());
        }
        if execution.status != ExecutionStatus::Skipped
            || execution.started_at.is_some()
            || execution.finished_at.is_some()
        {
            return Err("queued execution is no longer unstarted".into());
        }
        Ok(())
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.drafts(state).collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        let records = self.drafts(state);
        let request = state
            .run_requests
            .get_mut(&self.request_id)
            .expect("checked request");
        if self.ordinal < request.summary.prepared {
            return;
        }
        let execution_id = request
            .pending
            .pop_front()
            .expect("checked pending attempt");
        request.summary.prepared += 1;
        request.summary.phase = RunPhase::Preparing;
        let cycle = state
            .cycles
            .get_mut(&request.summary.cycle_id)
            .expect("request cycle");
        let existing = state
            .executions
            .get_mut(&execution_id)
            .expect("checked materialized execution");
        existing.mode = request.summary.mode;
        existing.status = ExecutionStatus::Pending;
        cycle.touch_at(self.at.clone());
        records.apply(state);
    }
}

impl PrepareAttempt {
    fn drafts(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        let request = &state.run_requests[&self.request_id];
        if self.ordinal < request.summary.prepared {
            return records;
        }
        let execution_id = request.pending.front().expect("checked pending attempt");
        let execution = state
            .executions
            .get(execution_id)
            .expect("checked materialized execution");
        records.draft(
            BuiltinDraft {
                record_type: BuiltinRecordType::TestExecutionPrepared,
                source: "runtime".into(),
                severity: RecordSeverity::Info,
                message: Some("Test execution prepared for provider dispatch.".into()),
                details: BTreeMap::new(),
                scope: Some(execution.scope.clone()),
            },
            self.at.clone(),
        );
        records
    }
}

impl Transaction<RuntimeState> for DispatchProvider {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let request = state
            .run_requests
            .get(&self.request_id)
            .ok_or("run request not found")?;
        if request.summary.phase == RunPhase::Running {
            return Ok(());
        }
        active(state, request)?;
        if !request.pending.is_empty() {
            return Err("request preparation is incomplete".into());
        }
        Ok(())
    }
    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.drafts(state).collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        let records = self.drafts(state);
        let request = state
            .run_requests
            .get_mut(&self.request_id)
            .expect("checked request");
        if request.summary.phase == RunPhase::Running {
            return;
        }
        request.summary.phase = RunPhase::Running;
        // Later groups of the same request rejoin a run that already started.
        request
            .summary
            .provider_started_at
            .get_or_insert_with(|| self.at.clone());
        let cycle = state
            .cycles
            .get_mut(&request.summary.cycle_id)
            .expect("request cycle");
        cycle.started_at.get_or_insert_with(|| self.at.clone());
        records.apply(state);
    }
}

impl DispatchProvider {
    fn drafts(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        let request = &state.run_requests[&self.request_id];
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        // One run starts once, however many target groups it dispatches.
        if request.summary.phase == RunPhase::Running
            || request.summary.provider_started_at.is_some()
        {
            return records;
        }
        for name in [
            BuiltinRecordType::TestCycleStarted,
            BuiltinRecordType::ProviderExecutionStarted,
        ] {
            records.draft(
                BuiltinDraft {
                    record_type: name,
                    source: "runtime".into(),
                    severity: RecordSeverity::Info,
                    message: Some("Cycle provider dispatch started.".into()),
                    details: BTreeMap::new(),
                    scope: Some(
                        RecordScope::default().with_cycle(request.summary.cycle_id.clone()),
                    ),
                },
                self.at.clone(),
            );
        }
        records
    }
}

impl Transaction<RuntimeState> for AdvanceRunGroup {
    type Error = String;
    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        let request = state
            .run_requests
            .get(&self.request_id)
            .ok_or("run request not found")?;
        if request.completion_at.is_some() {
            return Err("run request completion has already been staged".into());
        }
        if !request.pending.is_empty() {
            // The group is already current: a replayed advance changes nothing.
            return Ok(());
        }
        if state.shutting_down || request.summary.stop_requested {
            return Err("run request is stopping".into());
        }
        if request.summary.phase.terminal() {
            return Err("run request is terminal".into());
        }
        if request.groups.is_empty() {
            return Err("run request has no further provider target".into());
        }
        Ok(())
    }
    fn apply(&self, state: &mut RuntimeState) {
        let request = state
            .run_requests
            .get_mut(&self.request_id)
            .expect("checked request");
        if !request.pending.is_empty() {
            return;
        }
        let group = request.groups.pop_front().expect("checked group");
        request.target = group.target;
        request.pending = group.pending;
        request.summary.phase = RunPhase::Queued;
        request.conclusion = None;
        let cycle_id = request.summary.cycle_id.clone();
        let cycle = state.cycles.get_mut(&cycle_id).expect("request cycle");
        cycle.touch_at(self.at.clone());
    }
}

/* === Public Functions === */
pub(super) fn queue(
    state: &mut RuntimeState,
    run: &RunProvider,
) -> Result<ExecutionProviderTarget, Error> {
    if let Some(request) = state.run_requests.get(&run.command_id) {
        if request.summary.cycle_id != run.test_cycle_id
            || request.test_execution_id != run.test_execution_id
            || request.summary.mode != run.mode
        {
            return Err(Error::conflict(
                "request_identity_conflict",
                "request identity belongs to another selection",
            ));
        }
        return Ok(request.target.clone());
    }
    let plan = prepare_run_provider(state, run)?;
    let mut groups = plan
        .into_iter()
        .map(|group| RunGroup {
            target: group.target,
            pending: group.execution_ids.into(),
        })
        .collect::<VecDeque<_>>();
    let current = groups.pop_front().expect("admission planned one group");
    let planned = current.pending.len()
        + groups
            .iter()
            .map(|group| group.pending.len())
            .sum::<usize>();
    // Queuing controls the cycle; its canonical executions remain unstarted.
    let summary = RunSummary {
        mode: run.mode,
        request_id: run.command_id.clone(),
        cycle_id: run.test_cycle_id.clone(),
        phase: RunPhase::Queued,
        paused: false,
        stop_requested: false,
        planned: planned as u64,
        prepared: 0,
        finished: 0,
        unstarted: 0,
        failed: 0,
        accepted_at: run.accepted_at.clone(),
        provider_started_at: None,
        finished_at: None,
    };
    let records = queue_records(state, run);
    let target = current.target.clone();
    let cycle = state
        .cycles
        .get_mut(&run.test_cycle_id)
        .expect("checked cycle");
    cycle.lifecycle = cycle
        .lifecycle
        .try_transition(CycleStatus::Running)
        .unwrap_or(CycleStatus::Running);
    cycle.touch_at(run.accepted_at.clone());
    state.run_requests.insert(
        run.command_id.clone(),
        RunRequest {
            summary,
            target: current.target,
            test_execution_id: run.test_execution_id.clone(),
            pending: current.pending,
            groups,
            settled_execution_ids: Vec::new(),
            stopped_execution_ids: Vec::new(),
            conclusion: None,
            failure: None,
            publication_failure: None,
            completion_at: None,
        },
    );
    records.apply(state);
    Ok(target)
}

pub(super) fn queue_records(
    state: &RuntimeState,
    run: &RunProvider,
) -> crate::transactions::records::draft::Drafts {
    let mut records = crate::transactions::records::draft::Drafts::new(state);
    if state.run_requests.contains_key(&run.command_id) {
        return records;
    }
    records.draft(
        BuiltinDraft {
            record_type: BuiltinRecordType::TestCycleQueued,
            source: "runtime".into(),
            severity: RecordSeverity::Info,
            message: Some("Cycle execution request queued.".into()),
            details: BTreeMap::new(),
            scope: Some(RecordScope::default().with_cycle(run.test_cycle_id.clone())),
        },
        run.accepted_at.clone(),
    );
    records
}

/* === Private Functions === */
fn active(state: &RuntimeState, request: &RunRequest) -> Result<(), String> {
    if state.shutting_down || request.summary.stop_requested || request.summary.phase.terminal() {
        return Err("run request is stopping or terminal".into());
    }
    if request.summary.paused {
        return Err("run request is paused".into());
    }
    if !matches!(
        request.summary.phase,
        RunPhase::Queued | RunPhase::Preparing
    ) {
        return Err("run request is already dispatched".into());
    }
    Ok(())
}

/* === Tests === */
