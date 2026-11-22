/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Interactive Run Control

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashSet,
    sync::{Mutex, Weak},
};

// Third Party Imports
use harmos_operations::clock::WindDown;
use harmos_runtime::{operations::OperationHost, plugin::grpc::ScopedBinding};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{Error, Result, execution_manager::ExecutionManager, host::lock};
use dokime_protocol::host::executions::{CommandsCreateRequest, ExecutionCommandAction, ExecutionCommandPhase, ExecutionCommandReceipt};
use dokime_protocol::model::{TestCase, origin};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct State {
    phase: String,
    interactive: bool,
    paused: bool,
    stopping: bool,
    busy: bool,
    next: usize,
    completed: HashSet<usize>,
    invoked: HashSet<usize>,
    command: Option<ExecutionCommandReceipt>,
    wind_down: Vec<WindDown>,
}
#[derive(Default)]
pub(crate) struct RunControl {
    state: Mutex<State>,
    changed: Notify,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RunControl {
    pub(crate) fn bind(&self, binding: &ScopedBinding) -> Result<()> {
        let mut state = lock(&self.state)?;
        if state.stopping {
            binding.request_wind_down();
        }
        let wind_down = OperationHost::context(binding.operations()).get::<WindDown>().expect("bound scope wind-down").clone();
        state.wind_down.push(wind_down);
        Ok(())
    }
    pub(crate) fn stop_requested(&self) -> Result<bool> {
        Ok(lock(&self.state)?.stopping)
    }
    pub(crate) fn is_running(&self) -> Result<bool> {
        let state = lock(&self.state)?;
        Ok(!state.phase.is_empty() && state.phase != "queued")
    }
    pub(crate) fn cancel(&self, cancellation: &CancellationToken) -> Result<()> {
        let state = lock(&self.state)?;
        if state.phase.is_empty() || state.phase == "queued" {
            return Err(Error::Invalid("Cancel Execution: Wait Until the Queued Execution Is Running".into()));
        }
        cancellation.cancel();
        Ok(())
    }
    pub(crate) fn interactive() -> Self {
        Self {
            state: Mutex::new(State { interactive: true, ..Default::default() }),
            changed: Notify::new(),
        }
    }
    pub(crate) async fn next(
        &self, case: &TestCase, cancellation: &CancellationToken,
    ) -> Result<Option<(usize, Option<ExecutionCommandReceipt>)>> {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            {
                let mut state = lock(&self.state)?;
                if cancellation.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                if state.stopping {
                    return Ok(None);
                }
                if !state.paused && (!state.interactive || state.command.is_some()) {
                    let command = state.command.clone();
                    let index = match command.as_ref() {
                        Some(command) if command.action == ExecutionCommandAction::Finish as i32 => {
                            return Ok(None);
                        }
                        Some(command) if command.action == ExecutionCommandAction::RunStep as i32 => case
                            .steps
                            .iter()
                            .position(|step| {
                                step.metadata.as_ref().is_some_and(|metadata| Some(&metadata.identifier) == command.step_id.as_ref())
                            })
                            .ok_or_else(|| Error::Invalid("Run Step: Select a Frozen Step".into()))?,
                        Some(_) => (0..case.steps.len()).find(|index| !state.completed.contains(index)).unwrap_or(case.steps.len()),
                        _ => {
                            while state.completed.contains(&state.next) {
                                state.next += 1;
                            }
                            let index = state.next;
                            state.next += 1;
                            index
                        }
                    };
                    if index >= case.steps.len() {
                        return Ok(None);
                    }
                    state.completed.retain(|completed| *completed < index);
                    state.busy = true;
                    return Ok(Some((index, command)));
                }
            }
            tokio::select! { _ = changed => {}, _ = cancellation.cancelled() => return Err(Error::Cancelled) }
        }
    }
    pub(crate) fn complete_step(&self, index: usize) -> Result<()> {
        let mut state = lock(&self.state)?;
        state.completed.insert(index);
        state.invoked.insert(index);
        state.busy = false;
        state.command = None;
        Ok(())
    }
    pub(crate) fn take_pending(&self) -> Result<Option<ExecutionCommandReceipt>> {
        Ok(lock(&self.state)?.command.take())
    }
}

impl ExecutionManager {
    pub(crate) fn run_phase(&self, execution: &str) -> Result<String> {
        let active = lock(&self.inner.active)?;
        let Some(active) = active.get(execution) else {
            return Ok("sealed".into());
        };
        let state = lock(&active.control.state)?;
        Ok(if state.paused {
            "paused".into()
        } else if state.phase.is_empty() {
            "queued".into()
        } else {
            state.phase.clone()
        })
    }
    pub(crate) fn set_run_phase(&self, execution: &str, phase: &str) -> Result<()> {
        if let Some(active) = lock(&self.inner.active)?.get(execution) {
            lock(&active.control.state)?.phase = phase.into();
        }
        self.signal_change();
        Ok(())
    }
    pub fn execution_controls(&self, execution: &str) -> Result<dokime_protocol::host::ExecutionControls> {
        let capabilities = self.capabilities(execution)?;
        let active = lock(&self.inner.active)?;
        let control = active.get(execution).map(|active| lock(&active.control.state)).transpose()?;
        let paused = control.as_ref().is_some_and(|state| state.paused);
        let stopping = control.as_ref().is_some_and(|state| state.stopping);
        Ok(dokime_protocol::host::ExecutionControls {
            request_id: control.as_ref().map(|_| execution.into()),
            paused,
            stop_requested: stopping,
            can_pause: control.is_some() && !paused && !stopping,
            can_resume: control.is_some() && paused && !stopping,
            can_stop: control.is_some() && !stopping,
            mode: if control.as_ref().is_some_and(|state| state.interactive) {
                dokime_protocol::host::ExecutionMode::Interactive
            } else {
                dokime_protocol::host::ExecutionMode::Automatic
            } as i32,
            can_start: capabilities.start,
            can_rerun: capabilities.rerun,
            can_cancel: capabilities.cancel,
            can_inspect: capabilities.inspect,
            can_export: capabilities.export,
        })
    }
    pub fn pause_execution(&self, execution: &str, paused: bool) -> Result<()> {
        let active = lock(&self.inner.active)?;
        let active = active.get(execution).ok_or_else(|| Error::Invalid("Pause or Resume Execution: Select an Active Run".into()))?;
        let mut state = lock(&active.control.state)?;
        if state.stopping {
            return Err(Error::Invalid("Pause or Resume Execution: Wait for the Stopping Run".into()));
        }
        state.paused = paused;
        active.control.changed.notify_waiters();
        self.signal_change();
        Ok(())
    }
    pub fn stop_execution(&self, execution: &str) -> Result<()> {
        let active = lock(&self.inner.active)?;
        let active = active.get(execution).ok_or_else(|| Error::Invalid("Stop Execution: Select an Active Run".into()))?;
        let mut state = lock(&active.control.state)?;
        state.stopping = true;
        for wind_down in &state.wind_down {
            wind_down.request();
        }
        active.control.changed.notify_waiters();
        self.signal_change();
        Ok(())
    }
    pub(crate) async fn admit_execution_command(&self, request: CommandsCreateRequest) -> Result<ExecutionCommandReceipt> {
        if request.request_id.trim().is_empty() || request.parent.trim().is_empty() || request.command_id.trim().is_empty() {
            return Err(Error::Invalid("Submit Execution Command: Use the Run Request ID and a Stable Command ID".into()));
        }
        let context = self
            .inner
            .context
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Invalid("Submit Execution Command: Assemble the Host Before Submitting Commands".into()))?;
        let _admission = context.administration.lock().await;
        if let Some(previous) =
            dokime_operations::host::executions::commands::read(&context, &request.parent, &request.command_id, None).await?.0
        {
            if previous.execution_id != request.parent || previous.action != request.action || previous.step_id != request.step_id {
                return Err(Error::Invalid("Submit Execution Command: Use a New Command ID for a Changed Action".into()));
            }
            return Ok(previous);
        }
        let snapshot = self.inner.store.cut(&request.parent)?;
        let case = match snapshot.execution.origin.and_then(|origin| origin.kind) {
            Some(origin::Kind::PluginRun(run)) => run.test_case.unwrap(),
            _ => return Err(Error::Invalid("Submit Execution Command: Select an Active Plugin Run".into())),
        };
        let mut receipt = {
            let active = lock(&self.inner.active)?;
            let active = active
                .get(&request.parent)
                .ok_or_else(|| Error::Invalid("Submit Execution Command: Start an Interactive Run First".into()))?;
            let state = lock(&active.control.state)?;
            if !state.interactive || state.stopping || state.busy || state.command.is_some() {
                return Err(Error::Invalid("Submit Execution Command: Wait for an Idle Interactive Run".into()));
            }
            let refusal = match (ExecutionCommandAction::try_from(request.action), request.step_id.as_ref()) {
                (Ok(ExecutionCommandAction::Next), None) => {
                    if state.completed.len() >= case.steps.len() {
                        Some("Choose Finish; Every Frozen Step Has an Effective Invocation".to_owned())
                    } else {
                        None
                    }
                }
                (Ok(ExecutionCommandAction::Finish), None) => None,
                (Ok(ExecutionCommandAction::RunStep), Some(identifier)) => {
                    let index = case
                        .steps
                        .iter()
                        .position(|step| step.metadata.as_ref().is_some_and(|metadata| &metadata.identifier == identifier))
                        .ok_or_else(|| Error::Invalid("Run Step: Select a Frozen Step Identifier".into()))?;
                    if state.invoked.contains(&index) && !case.steps[index].repeatable {
                        Some("Choose a Repeatable Step or Start a New Execution".into())
                    } else if case
                        .steps
                        .iter()
                        .enumerate()
                        .skip(index + 1)
                        .any(|(position, step)| state.invoked.contains(&position) && !step.repeatable)
                    {
                        Some("Start a New Execution; Repeating This Step Would Invalidate an Invoked Nonrepeatable Downstream Step".into())
                    } else {
                        None
                    }
                }
                _ => return Err(Error::Invalid("Submit Execution Command: Match Next, RunStep or Finish with Its Step Identifier".into())),
            };
            let now = self.now_ns();
            let at = prost_types::Timestamp {
                seconds: now.div_euclid(1_000_000_000),
                nanos: now.rem_euclid(1_000_000_000) as i32,
            };
            ExecutionCommandReceipt {
                request_id: request.request_id,
                command_id: request.command_id,
                execution_id: request.parent,
                action: request.action,
                step_id: request.step_id,
                phase: if refusal.is_some() { ExecutionCommandPhase::Refused } else { ExecutionCommandPhase::Accepted } as i32,
                detail: refusal,
                accepted_at: Some(at),
                updated_at: Some(at),
                ..Default::default()
            }
        };
        dokime_operations::host::executions::commands::save(&context, &receipt).await?;
        if receipt.phase == ExecutionCommandPhase::Refused as i32 {
            self.signal_change();
            return Ok(receipt);
        }

        let queued = {
            let active = lock(&self.inner.active)?;
            if let Some(active) = active.get(&receipt.execution_id) {
                let mut state = lock(&active.control.state)?;
                if !state.stopping && !active.cancellation.is_cancelled() {
                    state.command = Some(receipt.clone());
                    active.control.changed.notify_waiters();
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        if !queued {
            receipt.phase = ExecutionCommandPhase::Interrupted as i32;
            receipt.detail = Some("Submit Execution Command: Start a New Run; This Run Sealed During Admission".into());
            dokime_operations::host::executions::commands::save(&context, &receipt).await?;
        }
        self.signal_change();
        Ok(receipt)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
