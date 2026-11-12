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

use std::{collections::HashSet, sync::Mutex};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use crate::{Error, Host, Result, host::lock};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct State {
    interactive: bool,
    paused: bool,
    stopping: bool,
    busy: bool,
    next: usize,
    completed: HashSet<usize>,
    command: Option<v1::ExecutionCommandReceipt>,
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
    pub(crate) fn interactive() -> Self {
        Self { state: Mutex::new(State { interactive: true, ..Default::default() }), changed: Notify::new() }
    }
    pub(crate) async fn next(&self, case: &v1::TestCase, cancellation: &CancellationToken) -> Result<Option<(usize, Option<v1::ExecutionCommandReceipt>)>> {
        loop {
            let changed = self.changed.notified();
            {
                let mut state = lock(&self.state)?;
                if state.stopping || cancellation.is_cancelled() { return Err(Error::Cancelled); }
                if !state.paused && (!state.interactive || state.command.is_some()) {
                    let command = state.command.clone();
                    let index = match command.as_ref() {
                        Some(command) if command.action == v1::ExecutionCommandAction::Finish as i32 => return Ok(None),
                        Some(command) if command.action == v1::ExecutionCommandAction::RunStep as i32 => case.steps.iter().position(|step| step.metadata.as_ref().is_some_and(|metadata| Some(&metadata.identifier) == command.step_id.as_ref())).ok_or_else(|| Error::Invalid("Run Step: Select a Frozen Step".into()))?,
                        _ => { let index = state.next; state.next += 1; index },
                    };
                    if index >= case.steps.len() { return Ok(None); }
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
        state.busy = false;
        state.command = None;
        Ok(())
    }
    pub(crate) fn take_pending(&self) -> Result<Option<v1::ExecutionCommandReceipt>> {
        Ok(lock(&self.state)?.command.take())
    }
}

impl Host {
    pub fn execution_controls(&self, execution: &str) -> Result<v1::ExecutionControls> {
        let capabilities = self.capabilities(execution)?;
        let active = lock(&self.inner.active)?;
        let control = active.get(execution).map(|active| lock(&active.control.state)).transpose()?;
        let paused = control.as_ref().is_some_and(|state| state.paused);
        let stopping = control.as_ref().is_some_and(|state| state.stopping);
        Ok(v1::ExecutionControls { request_id: control.as_ref().map(|_| execution.into()), paused, stop_requested: stopping, can_pause: capabilities.cancel && !paused && !stopping, can_resume: capabilities.cancel && paused && !stopping, can_stop: capabilities.cancel && !stopping, mode: if control.as_ref().is_some_and(|state| state.interactive) { v1::ExecutionMode::Interactive } else { v1::ExecutionMode::Automatic } as i32, can_start: capabilities.start, can_rerun: capabilities.rerun, can_cancel: capabilities.cancel, can_inspect: capabilities.inspect, can_export: capabilities.export })
    }
    pub fn pause_execution(&self, execution: &str, paused: bool) -> Result<()> {
        let active = lock(&self.inner.active)?;
        let active = active.get(execution).ok_or_else(|| Error::Invalid("Pause or Resume Execution: Select an Active Run".into()))?;
        let mut state = lock(&active.control.state)?;
        if state.stopping { return Err(Error::Invalid("Pause or Resume Execution: Wait for the Stopping Run".into())); }
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
    pub async fn start_execution_mode(&self, execution: &str, mode: v1::ExecutionMode) -> Result<crate::Cycle> {
        crate::execution::start_existing_mode(self.clone(), execution, mode).await
    }
    pub fn submit_execution_command(&self, request: v1::ExecutionSubmitExecutionCommandRequest) -> Result<v1::ExecutionCommandReceipt> {
        if request.request_id != request.execution_id || request.command_id.trim().is_empty() { return Err(Error::Invalid("Submit Execution Command: Use the Run Request ID and a Stable Command ID".into())); }
        if let Some(previous) = self.inner.store.command(&request.request_id, &request.command_id)? {
            if previous.execution_id != request.execution_id || previous.action != request.action || previous.step_id != request.step_id { return Err(Error::Invalid("Submit Execution Command: Use a New Command ID for a Changed Action".into())); }
            return Ok(previous);
        }
        let snapshot = self.inner.store.cut(&request.execution_id)?;
        let case = match snapshot.execution.origin.and_then(|origin| origin.kind) { Some(v1::origin::Kind::PluginRun(run)) => run.test_case.unwrap(), _ => return Err(Error::Invalid("Submit Execution Command: Select an Active Plugin Run".into())) };
        let active = lock(&self.inner.active)?;
        let active = active.get(&request.execution_id).ok_or_else(|| Error::Invalid("Submit Execution Command: Start an Interactive Run First".into()))?;
        let mut state = lock(&active.control.state)?;
        if !state.interactive || state.stopping || state.busy || state.command.is_some() { return Err(Error::Invalid("Submit Execution Command: Wait for an Idle Interactive Run".into())); }
        match (v1::ExecutionCommandAction::try_from(request.action), request.step_id.as_ref()) {
            (Ok(v1::ExecutionCommandAction::Next | v1::ExecutionCommandAction::Finish), None) => {},
            (Ok(v1::ExecutionCommandAction::RunStep), Some(identifier)) => {
                let index = case.steps.iter().position(|step| step.metadata.as_ref().is_some_and(|metadata| &metadata.identifier == identifier)).ok_or_else(|| Error::Invalid("Run Step: Select a Frozen Step Identifier".into()))?;
                if state.completed.contains(&index) && !case.steps[index].repeatable { return Err(Error::Invalid("Run Step: Choose a Repeatable Step or Start a New Execution".into())); }
            },
            _ => return Err(Error::Invalid("Submit Execution Command: Match Next, RunStep or Finish with Its Step Identifier".into())),
        }
        let now = self.now_ns();
        let at = prost_types::Timestamp { seconds: now.div_euclid(1_000_000_000), nanos: now.rem_euclid(1_000_000_000) as i32 };
        let receipt = v1::ExecutionCommandReceipt { request_id: request.request_id, command_id: request.command_id, execution_id: request.execution_id, action: request.action, step_id: request.step_id, phase: v1::ExecutionCommandPhase::Accepted as i32, accepted_at: Some(at), updated_at: Some(at), ..Default::default() };
        self.inner.store.save_command(&receipt)?;
        state.command = Some(receipt.clone());
        active.control.changed.notify_waiters();
        self.signal_change();
        Ok(receipt)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
