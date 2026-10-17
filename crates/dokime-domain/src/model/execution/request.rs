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
pub use crate::model::execution::StepCommand;

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::identity::CaseId;
use crate::model::identity::CommandId;
use crate::model::identity::NodeId;
use crate::model::identity::StepAttemptId;
use crate::model::identity::TestCycleId;
use crate::model::identity::TestExecutionId;
use crate::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub mode: super::ExecutionMode,
    pub request_id: CommandId,
    pub cycle_id: TestCycleId,
    pub phase: RunPhase,
    pub paused: bool,
    pub stop_requested: bool,
    pub planned: u64,
    pub prepared: u64,
    pub finished: u64,
    pub unstarted: u64,
    pub failed: u64,
    pub accepted_at: Timestamp,
    pub provider_started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunReceipt {
    pub node_id: NodeId,
    pub summary: RunSummary,
    pub fingerprint: String,
    pub submitted_by: Option<String>,
    pub claimed: u64,
    pub last_claim: Option<AttemptClaim>,
    pub interruption: Option<String>,
    pub commands: BTreeMap<CommandId, StepCommandReceipt>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttemptClaim {
    pub ordinal: u64,
    pub case_id: CaseId,
    pub execution_id: TestExecutionId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepCommandReceipt {
    pub command_id: CommandId,
    pub execution_id: TestExecutionId,
    pub action: StepCommand,
    pub phase: StepCommandPhase,
    pub attempt_id: Option<StepAttemptId>,
    pub accepted_at: Timestamp,
    pub updated_at: Timestamp,
    pub detail: Option<String>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunPhase {
    Queued,
    Preparing,
    Running,
    Finishing,
    Completed,
    Failed,
    Stopped,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepCommandPhase {
    Accepted,
    Dispatching,
    Running,
    Completed,
    Refused,
    Failed,
    Interrupted,
}

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for RunSummary {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.request_id), std::mem::size_of_val(&self.request_id)),
            (harmos::MemorySize::memory_size(&self.cycle_id), std::mem::size_of_val(&self.cycle_id)),
            (harmos::MemorySize::memory_size(&self.accepted_at), std::mem::size_of_val(&self.accepted_at)),
            (harmos::MemorySize::memory_size(&self.provider_started_at), std::mem::size_of_val(&self.provider_started_at)),
            (harmos::MemorySize::memory_size(&self.finished_at), std::mem::size_of_val(&self.finished_at)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for RunReceipt {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.node_id), std::mem::size_of_val(&self.node_id)),
            (harmos::MemorySize::memory_size(&self.summary), std::mem::size_of_val(&self.summary)),
            (harmos::MemorySize::memory_size(&self.fingerprint), std::mem::size_of_val(&self.fingerprint)),
            (harmos::MemorySize::memory_size(&self.submitted_by), std::mem::size_of_val(&self.submitted_by)),
            (harmos::MemorySize::memory_size(&self.last_claim), std::mem::size_of_val(&self.last_claim)),
            (harmos::MemorySize::memory_size(&self.interruption), std::mem::size_of_val(&self.interruption)),
            (harmos::MemorySize::memory_size(&self.commands), std::mem::size_of_val(&self.commands)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for AttemptClaim {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.case_id), std::mem::size_of_val(&self.case_id)),
            (harmos::MemorySize::memory_size(&self.execution_id), std::mem::size_of_val(&self.execution_id)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for StepCommandReceipt {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.command_id), std::mem::size_of_val(&self.command_id)),
            (harmos::MemorySize::memory_size(&self.execution_id), std::mem::size_of_val(&self.execution_id)),
            (harmos::MemorySize::memory_size(&self.action), std::mem::size_of_val(&self.action)),
            (harmos::MemorySize::memory_size(&self.attempt_id), std::mem::size_of_val(&self.attempt_id)),
            (harmos::MemorySize::memory_size(&self.accepted_at), std::mem::size_of_val(&self.accepted_at)),
            (harmos::MemorySize::memory_size(&self.updated_at), std::mem::size_of_val(&self.updated_at)),
            (harmos::MemorySize::memory_size(&self.detail), std::mem::size_of_val(&self.detail)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl RunReceipt {
    pub fn known_step_command(
        &self, command_id: &CommandId, execution_id: &TestExecutionId, action: &StepCommand,
    ) -> Result<Option<&StepCommandReceipt>, &'static str> {
        let Some(known) = self.commands.get(command_id) else {
            return Ok(None);
        };
        if &known.execution_id != execution_id || &known.action != action {
            return Err("step command identity has different content");
        }
        Ok(Some(known))
    }

    pub fn admit_step_command(
        &mut self, command_id: CommandId, execution_id: TestExecutionId, action: StepCommand, at: Timestamp,
    ) -> Result<&StepCommandReceipt, &'static str> {
        if self.commands.contains_key(&command_id) {
            self.known_step_command(&command_id, &execution_id, &action)?;
            return Ok(&self.commands[&command_id]);
        }
        if self.summary.mode != super::ExecutionMode::Interactive
            || self.summary.phase != RunPhase::Running
            || self.summary.stop_requested
            || self.summary.paused
            || self.last_claim.as_ref().is_none_or(|claim| claim.execution_id != execution_id)
        {
            return Err("step command requires the active interactive execution");
        }
        if self.commands.values().any(|command| {
            (command.action == StepCommand::Finish && command.phase != StepCommandPhase::Refused)
                || matches!(command.phase, StepCommandPhase::Failed | StepCommandPhase::Interrupted)
        }) {
            return Err("step command admission is closed");
        }
        if self.commands.values().any(|command| !command.phase.terminal()) {
            return Err("another step command is unsettled");
        }
        if self.commands.len() >= 4096 && (action != StepCommand::Finish || self.commands.len() >= 4097) {
            return Err("step command receipt capacity exceeded");
        }
        let receipt = StepCommandReceipt {
            command_id: command_id.clone(),
            execution_id,
            action,
            phase: StepCommandPhase::Accepted,
            attempt_id: None,
            accepted_at: at.clone(),
            updated_at: at,
            detail: None,
        };
        self.commands.insert(command_id.clone(), receipt);
        Ok(&self.commands[&command_id])
    }

    pub fn reconcile_step_command(
        &mut self, id: &CommandId, phase: StepCommandPhase, attempt_id: Option<StepAttemptId>, detail: Option<String>, at: Timestamp,
    ) -> Result<bool, &'static str> {
        let receipt = self.commands.get_mut(id).ok_or("step command not found")?;
        if receipt.phase == phase && receipt.attempt_id == attempt_id && receipt.detail == detail {
            return Ok(false);
        }
        if receipt.phase == StepCommandPhase::Accepted
            && !matches!(phase, StepCommandPhase::Dispatching | StepCommandPhase::Refused | StepCommandPhase::Interrupted)
        {
            return Err("command dispatch has not been durably marked");
        }
        if receipt.phase.terminal() {
            return Err("terminal step command receipt is immutable");
        }
        if receipt.attempt_id.is_some() && receipt.attempt_id != attempt_id {
            return Err("step command attempt identity cannot change");
        }
        if matches!(phase, StepCommandPhase::Accepted)
            || (receipt.phase == StepCommandPhase::Running && phase == StepCommandPhase::Dispatching)
        {
            return Err("step command receipt cannot regress");
        }
        receipt.phase = phase;
        receipt.attempt_id = attempt_id;
        receipt.detail = detail;
        receipt.updated_at = at;
        Ok(true)
    }

    pub fn interrupt_step_commands(&mut self, reason: &str, at: &Timestamp) {
        for command in self.commands.values_mut().filter(|command| !command.phase.terminal()) {
            command.phase = StepCommandPhase::Interrupted;
            command.updated_at = at.clone();
            command.detail = Some(reason.into());
        }
    }
}

impl StepCommandPhase {
    pub fn terminal(self) -> bool {
        !matches!(self, Self::Accepted | Self::Dispatching | Self::Running)
    }
}

impl RunPhase {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Stopped | Self::Interrupted)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
