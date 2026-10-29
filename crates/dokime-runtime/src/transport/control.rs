/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use harmos::sidecar::Refusal;
use tokio::sync::Notify;

use super::{CommandExecutionReply, CommandExecutionRequest};
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::StepAttemptId;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Command {
    pub id: CommandId,
    pub action: Action,
}

pub(crate) struct Controls {
    maximum: u64,
    state: Mutex<State>,
    pub changed: Notify,
}

struct State {
    interactive: bool,
    closed: bool,
    pending: Option<Command>,
    active: Option<CommandId>,
    receipts: BTreeMap<CommandId, (Action, CommandExecutionReply)>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Action {
    Next,
    Step(Identifier),
    Finish,
}

impl Command {
    pub fn parse(request: &CommandExecutionRequest) -> Result<Self, Refusal> {
        let id = CommandId::from_uuid(&request.command_id).map_err(|error| Refusal::of(error.to_string()))?;
        let action = match (request.action.as_str(), request.step.as_str()) {
            ("next", "") => Action::Next,
            ("finish", "") => Action::Finish,
            ("run_step", step) => Action::Step(Identifier::new(step).map_err(|error| Refusal::of(error.to_string()))?),
            _ => {
                return Err(Refusal::of("command must be next, run_step with a step, or finish"));
            }
        };
        Ok(Self { id, action })
    }
}

impl Controls {
    pub fn new(interactive: bool, maximum: u64) -> Self {
        Self {
            maximum,
            state: Mutex::new(State {
                interactive,
                closed: false,
                pending: None,
                active: None,
                receipts: BTreeMap::new(),
            }),
            changed: Notify::new(),
        }
    }

    pub fn interactive(&self) -> bool {
        self.state().interactive
    }

    pub fn admit(&self, command: Command) -> Result<CommandExecutionReply, Refusal> {
        let mut state = self.state();
        if let Some((action, receipt)) = state.receipts.get(&command.id) {
            return if action == &command.action {
                Ok(receipt.clone())
            } else {
                Err(Refusal::of("command identity was reused with different content"))
            };
        }
        if !state.interactive {
            return Err(Refusal::of("commands require interactive execution"));
        }
        if state.closed {
            return Err(Refusal::of("execution command input is closed"));
        }
        if state.pending.is_some() || state.active.is_some() {
            return Err(Refusal::of("another execution command is still settling"));
        }
        if state.receipts.len() as u64 >= self.maximum && command.action != Action::Finish {
            return Err(Refusal::of("execution command receipts are at capacity; finish this execution"));
        }
        let receipt = CommandExecutionReply {
            state: "accepted".into(),
            attempt_id: String::new(),
            message: String::new(),
        };
        state.receipts.insert(command.id.clone(), (command.action.clone(), receipt.clone()));
        state.pending = Some(command);
        drop(state);
        self.changed.notify_one();
        Ok(receipt)
    }

    pub fn take(&self) -> Option<Command> {
        let mut state = self.state();
        let command = state.pending.take()?;
        state.active = Some(command.id.clone());
        state.receipts.get_mut(&command.id)?.1.state = "running".into();
        Some(command)
    }

    pub fn complete(&self, id: &CommandId, attempt: Option<&StepAttemptId>, error: Option<String>) {
        let mut state = self.state();
        if state.active.as_ref() != Some(id) {
            return;
        }
        let Some((_, receipt)) = state.receipts.get_mut(id) else {
            return;
        };
        receipt.state = if error.is_some() { if attempt.is_some() { "failed" } else { "refused" } } else { "completed" }.into();
        receipt.attempt_id = attempt.map(ToString::to_string).unwrap_or_default();
        receipt.message = error.unwrap_or_default();
        state.active = None;
    }

    pub fn stop_accepting(&self) {
        self.state().closed = true;
    }

    pub fn close(&self) {
        let mut state = self.state();
        state.closed = true;
        let pending = state.pending.take().map(|command| command.id);
        let active = state.active.take();
        for id in pending.into_iter().chain(active) {
            if let Some((action, receipt)) = state.receipts.get_mut(&id) {
                if *action == Action::Finish && receipt.state == "running" {
                    receipt.state = "completed".into();
                } else {
                    receipt.state = "refused".into();
                    receipt.message = "execution ended before command completion".into();
                }
            }
        }
        drop(state);
        self.changed.notify_one();
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
