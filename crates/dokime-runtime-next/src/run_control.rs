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
use std::{collections::HashSet, sync::Mutex};

// Third Party Imports
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{Error, Result, execution_manager::ExecutionManager, host::lock};
use dokime_protocol::v1;

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
    command: Option<dokime_protocol::client::executions::ExecutionCommandReceipt>,
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
        &self, case: &v1::TestCase, cancellation: &CancellationToken,
    ) -> Result<Option<(usize, Option<dokime_protocol::client::executions::ExecutionCommandReceipt>)>> {
        loop {
            let changed = self.changed.notified();
            {
                let mut state = lock(&self.state)?;
                if state.stopping || cancellation.is_cancelled() {
                    return Err(Error::Cancelled);
                }
                if !state.paused && (!state.interactive || state.command.is_some()) {
                    let command = state.command.clone();
                    let index = match command.as_ref() {
                        Some(command) if command.action == dokime_protocol::client::executions::ExecutionCommandAction::Finish as i32 => {
                            return Ok(None);
                        }
                        Some(command) if command.action == dokime_protocol::client::executions::ExecutionCommandAction::RunStep as i32 => {
                            case.steps
                                .iter()
                                .position(|step| {
                                    step.metadata.as_ref().is_some_and(|metadata| Some(&metadata.identifier) == command.step_id.as_ref())
                                })
                                .ok_or_else(|| Error::Invalid("Run Step: Select a Frozen Step".into()))?
                        }
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
    pub(crate) fn take_pending(&self) -> Result<Option<dokime_protocol::client::executions::ExecutionCommandReceipt>> {
        Ok(lock(&self.state)?.command.take())
    }
}

impl ExecutionManager {
    pub(crate) fn run_phase(&self, execution: &str) -> Result<String> {
        let active = lock(&self.inner.active)?;
        let Some(active) = active.get(execution) else {
            return Ok("finished".into());
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
    pub fn execution_controls(&self, execution: &str) -> Result<dokime_protocol::client::ExecutionControls> {
        let capabilities = self.capabilities(execution)?;
        let active = lock(&self.inner.active)?;
        let control = active.get(execution).map(|active| lock(&active.control.state)).transpose()?;
        let paused = control.as_ref().is_some_and(|state| state.paused);
        let stopping = control.as_ref().is_some_and(|state| state.stopping);
        Ok(dokime_protocol::client::ExecutionControls {
            request_id: control.as_ref().map(|_| execution.into()),
            paused,
            stop_requested: stopping,
            can_pause: control.is_some() && !paused && !stopping,
            can_resume: control.is_some() && paused && !stopping,
            can_stop: control.is_some() && !stopping,
            mode: if control.as_ref().is_some_and(|state| state.interactive) {
                dokime_protocol::client::ExecutionMode::Interactive
            } else {
                dokime_protocol::client::ExecutionMode::Automatic
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
        lock(&active.control.state)?.stopping = true;
        active.control.changed.notify_waiters();
        self.signal_change();
        Ok(())
    }
    pub fn submit_execution_command(
        &self, request: dokime_protocol::client::executions::CommandsCreateRequest,
    ) -> Result<dokime_protocol::client::executions::ExecutionCommandReceipt> {
        if request.request_id.trim().is_empty() || request.parent.trim().is_empty() || request.command_id.trim().is_empty() {
            return Err(Error::Invalid("Submit Execution Command: Use the Run Request ID and a Stable Command ID".into()));
        }
        if let Some(previous) = self.inner.store.command(&request.parent, &request.command_id)? {
            if previous.execution_id != request.parent || previous.action != request.action || previous.step_id != request.step_id {
                return Err(Error::Invalid("Submit Execution Command: Use a New Command ID for a Changed Action".into()));
            }
            return Ok(previous);
        }
        let snapshot = self.inner.store.cut(&request.parent)?;
        let case = match snapshot.execution.origin.and_then(|origin| origin.kind) {
            Some(v1::origin::Kind::PluginRun(run)) => run.test_case.unwrap(),
            _ => return Err(Error::Invalid("Submit Execution Command: Select an Active Plugin Run".into())),
        };
        let active = lock(&self.inner.active)?;
        let active =
            active.get(&request.parent).ok_or_else(|| Error::Invalid("Submit Execution Command: Start an Interactive Run First".into()))?;
        let mut state = lock(&active.control.state)?;
        if !state.interactive || state.stopping || state.busy || state.command.is_some() {
            return Err(Error::Invalid("Submit Execution Command: Wait for an Idle Interactive Run".into()));
        }
        let refusal =
            match (dokime_protocol::client::executions::ExecutionCommandAction::try_from(request.action), request.step_id.as_ref()) {
                (Ok(dokime_protocol::client::executions::ExecutionCommandAction::Next), None) => {
                    if state.completed.len() >= case.steps.len() {
                        Some("Choose Finish; Every Frozen Step Has an Effective Invocation".to_owned())
                    } else {
                        None
                    }
                }
                (Ok(dokime_protocol::client::executions::ExecutionCommandAction::Finish), None) => None,
                (Ok(dokime_protocol::client::executions::ExecutionCommandAction::RunStep), Some(identifier)) => {
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
        let receipt = dokime_protocol::client::executions::ExecutionCommandReceipt {
            request_id: request.request_id,
            command_id: request.command_id,
            execution_id: request.parent,
            action: request.action,
            step_id: request.step_id,
            phase: if refusal.is_some() {
                dokime_protocol::client::executions::ExecutionCommandPhase::Refused
            } else {
                dokime_protocol::client::executions::ExecutionCommandPhase::Accepted
            } as i32,
            detail: refusal.clone(),
            accepted_at: Some(at),
            updated_at: Some(at),
            ..Default::default()
        };
        self.inner.store.save_command(&receipt)?;
        if refusal.is_none() {
            state.command = Some(receipt.clone());
        }
        active.control.changed.notify_waiters();
        self.signal_change();
        Ok(receipt)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
