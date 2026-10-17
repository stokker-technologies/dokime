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
use harmos::{
    DefinitionId, Identified, Message,
    schema::Described,
    storage::{ObjectRetention, SchemaKey, StoredObject},
};

// Local Imports
use super::{Error, Result};
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::identity::StepAttemptId;
use dokime_domain::model::identity::StepId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;
use crate::execution::request::{
    AttemptClaim, RunPhase, RunReceipt, RunSummary, StepCommand, StepCommandPhase,
    StepCommandReceipt,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone)]
pub struct Request {
    request_id: String,
    node_id: String,
    cycle_id: String,
    phase: String,
    mode: String,
    commands: Vec<Command>,
    terminal: bool,
    paused: bool,
    stop_requested: bool,
    planned: u64,
    prepared: u64,
    finished: u64,
    unstarted: u64,
    failed: u64,
    accepted_at: String,
    provider_started_at: Option<String>,
    finished_at: Option<String>,
    fingerprint: String,
    submitted_by: Option<String>,
    claimed: u64,
    last_claim: Option<ClaimRow>,
    interruption: Option<String>,
}

#[harmos::message]
#[derive(Clone)]
struct ClaimRow {
    pub ordinal: u64,
    pub case_id: String,
    pub execution_id: String,
}

#[harmos::message]
#[derive(Clone)]
pub struct Command {
    request_id: String,
    item: String,
    command_id: String,
    execution_id: String,
    action: String,
    step_id: Option<String>,
    phase: String,
    attempt_id: Option<String>,
    accepted_at: String,
    updated_at: String,
    detail: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Identified for Request {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("dokime.object.request"), 1)
    }
}

impl StoredObject for Request {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;

    fn key(&self) -> &str {
        &self.request_id
    }
}

impl Request {
    pub fn decoding_memory(bytes: &[u8]) -> Result<usize> {
        let memory = <Self as Message>::decoding_memory(bytes)?;
        reject_embedded_commands(bytes)?;
        Ok(memory)
    }

    pub fn new(receipt: &RunReceipt) -> Result<Self> {
        if Self::encoding_memory(receipt)? == usize::MAX {
            return Err(Error::Invalid("request encoding memory overflows"));
        }
        Ok(Self::from(receipt))
    }

    pub fn encoding_memory(receipt: &RunReceipt) -> Result<usize> {
        let summary = &receipt.summary;
        for time in std::iter::once(&summary.accepted_at)
            .chain(summary.provider_started_at.iter())
            .chain(summary.finished_at.iter())
        {
            time.to_datetime()?;
        }
        let fields = [
            summary.request_id.as_str().len(),
            receipt.node_id.as_str().len(),
            summary.cycle_id.as_str().len(),
            run_phase_text(&summary.phase).len(),
            summary.mode.as_str().len(),
            summary.accepted_at.as_str().len(),
            summary
                .provider_started_at
                .as_ref()
                .map_or(0, |time| time.as_str().len()),
            summary
                .finished_at
                .as_ref()
                .map_or(0, |time| time.as_str().len()),
            receipt.fingerprint.len(),
            receipt.submitted_by.as_deref().map_or(0, str::len),
            receipt.interruption.as_deref().map_or(0, str::len),
        ];
        let mut bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add);
        if let Some(claim) = &receipt.last_claim {
            bytes = bytes
                .saturating_add(claim.case_id.as_str().len())
                .saturating_add(claim.execution_id.as_str().len());
        }
        Ok(bytes)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        if !self.commands.is_empty() {
            return Err(Error::Invalid("parent object embeds command history"));
        }
        let strings = [
            &self.request_id,
            &self.node_id,
            &self.cycle_id,
            &self.accepted_at,
            &self.fingerprint,
        ];
        let mut bytes = strings
            .into_iter()
            .fold(std::mem::size_of::<RunReceipt>(), |bytes, value| {
                bytes.saturating_add(value.capacity())
            });
        for value in [
            &self.provider_started_at,
            &self.finished_at,
            &self.submitted_by,
            &self.interruption,
        ] {
            bytes = bytes.saturating_add(value.as_ref().map_or(0, String::capacity));
        }
        if let Some(claim) = &self.last_claim {
            bytes = bytes
                .saturating_add(claim.case_id.capacity())
                .saturating_add(claim.execution_id.capacity());
        }
        let commands = harmos::btree_map_memory::<CommandId, StepCommandReceipt>(0).saturating_sub(
            std::mem::size_of::<std::collections::BTreeMap<CommandId, StepCommandReceipt>>(),
        );
        Ok(bytes.saturating_add(commands))
    }

    pub fn into_receipt(self) -> Result<RunReceipt> {
        self.try_into()
    }

    pub fn restore(&self) -> Result<RunReceipt> {
        if !self.commands.is_empty() {
            return Err(Error::Invalid("parent object embeds command history"));
        }
        self.clone().try_into()
    }
}

impl Identified for Command {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("dokime.object.command"), 1)
    }
}

impl StoredObject for Command {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;

    fn key(&self) -> &str {
        &self.item
    }
}

impl Command {
    pub fn decoding_memory(bytes: &[u8]) -> Result<usize> {
        Ok(<Self as Message>::decoding_memory(bytes)?)
    }

    pub fn new(request_id: &CommandId, receipt: &StepCommandReceipt) -> Result<Self> {
        if Self::encoding_memory(request_id, receipt)? == usize::MAX {
            return Err(Error::Invalid("command encoding memory overflows"));
        }
        let length = request_id
            .as_str()
            .len()
            .checked_add(1)
            .and_then(|length| length.checked_add(receipt.command_id.as_str().len()))
            .ok_or(Error::Invalid("command key size overflows"))?;
        let mut item = String::new();
        item.try_reserve_exact(length)
            .map_err(|_| Error::Invalid("command key allocation failed"))?;
        item.push_str(request_id.as_str());
        item.push('/');
        item.push_str(receipt.command_id.as_str());
        let mut row = Self::row(receipt);
        row.request_id = request_id.as_str().to_owned();
        row.item = item;
        Ok(row)
    }

    pub fn encoding_memory(request_id: &CommandId, receipt: &StepCommandReceipt) -> Result<usize> {
        receipt.accepted_at.to_datetime()?;
        receipt.updated_at.to_datetime()?;
        let fields = [
            request_id.as_str().len(),
            request_id
                .as_str()
                .len()
                .saturating_add(1)
                .saturating_add(receipt.command_id.as_str().len()),
            receipt.command_id.as_str().len(),
            receipt.execution_id.as_str().len(),
            command_action_text(&receipt.action).len(),
            match &receipt.action {
                StepCommand::RunStep { step_id } => step_id.as_str().len(),
                _ => 0,
            },
            command_phase_text(&receipt.phase).len(),
            receipt
                .attempt_id
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
            receipt.accepted_at.as_str().len(),
            receipt.updated_at.as_str().len(),
            receipt.detail.as_deref().map_or(0, str::len),
        ];
        Ok(fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    pub fn projection_memory(&self) -> usize {
        let strings = [
            &self.command_id,
            &self.execution_id,
            &self.accepted_at,
            &self.updated_at,
        ];
        let mut bytes = strings
            .into_iter()
            .fold(std::mem::size_of::<StepCommandReceipt>(), |bytes, value| {
                bytes.saturating_add(value.capacity())
            });
        for value in [&self.step_id, &self.attempt_id, &self.detail] {
            bytes = bytes.saturating_add(value.as_ref().map_or(0, String::capacity));
        }
        bytes
    }

    pub fn into_receipt(self, request_id: &CommandId) -> Result<StepCommandReceipt> {
        self.validate_owner(request_id)?;
        self.try_into()
    }

    pub fn restore(&self, request_id: &CommandId) -> Result<StepCommandReceipt> {
        self.validate_owner(request_id)?;
        self.clone().try_into()
    }

    fn validate_owner(&self, request_id: &CommandId) -> Result<()> {
        if self.request_id != request_id.as_str() {
            return Err(Error::Invalid("stored command belongs to another request"));
        }
        let Some((parent, command)) = self.item.split_once('/') else {
            return Err(Error::Invalid("stored command key differs from identity"));
        };
        if parent != request_id.as_str() || command != self.command_id {
            return Err(Error::Invalid("stored command key differs from identity"));
        }
        Ok(())
    }
}

impl Command {
    fn row(value: &StepCommandReceipt) -> Self {
        Self {
            request_id: String::new(),
            item: String::new(),
            command_id: value.command_id.as_str().to_owned(),
            execution_id: value.execution_id.as_str().to_owned(),
            action: command_action_text(&value.action).into(),
            step_id: match &value.action {
                StepCommand::RunStep { step_id } => Some(step_id.as_str().to_owned()),
                _ => None,
            },
            phase: command_phase_text(&value.phase).into(),
            attempt_id: value
                .attempt_id
                .as_ref()
                .map(|value| value.as_str().to_owned()),
            accepted_at: value.accepted_at.as_str().into(),
            updated_at: value.updated_at.as_str().into(),
            detail: value.detail.as_deref().map(str::to_owned),
        }
    }
}

impl TryFrom<Command> for StepCommandReceipt {
    type Error = Error;
    fn try_from(row: Command) -> Result<Self> {
        let action = match (row.action.as_str(), row.step_id) {
            ("next", None) => StepCommand::Next,
            ("finish", None) => StepCommand::Finish,
            ("run_step", Some(id)) => StepCommand::RunStep {
                step_id: StepId::from_uuid(id)?,
            },
            _ => return Err(Error::Invalid("invalid stored command action")),
        };
        let phase = match row.phase.as_str() {
            "accepted" => StepCommandPhase::Accepted,
            "dispatching" => StepCommandPhase::Dispatching,
            "running" => StepCommandPhase::Running,
            "completed" => StepCommandPhase::Completed,
            "refused" => StepCommandPhase::Refused,
            "failed" => StepCommandPhase::Failed,
            "interrupted" => StepCommandPhase::Interrupted,
            _ => return Err(Error::Invalid("invalid stored command phase")),
        };
        Ok(Self {
            command_id: CommandId::from_uuid(row.command_id)?,
            execution_id: TestExecutionId::from_uuid(row.execution_id)?,
            action,
            phase,
            attempt_id: row.attempt_id.map(StepAttemptId::from_uuid).transpose()?,
            accepted_at: timestamp(row.accepted_at)?,
            updated_at: timestamp(row.updated_at)?,
            detail: row.detail,
        })
    }
}

impl From<&RunReceipt> for Request {
    fn from(receipt: &RunReceipt) -> Self {
        let summary = &receipt.summary;
        Self {
            request_id: summary.request_id.as_str().to_owned(),
            node_id: receipt.node_id.as_str().to_owned(),
            cycle_id: summary.cycle_id.as_str().to_owned(),
            mode: summary.mode.as_str().into(),
            commands: Vec::new(),
            phase: run_phase_text(&summary.phase).into(),
            terminal: summary.phase.terminal(),
            paused: summary.paused,
            stop_requested: summary.stop_requested,
            planned: summary.planned,
            prepared: summary.prepared,
            finished: summary.finished,
            unstarted: summary.unstarted,
            failed: summary.failed,
            accepted_at: summary.accepted_at.as_str().to_owned(),
            provider_started_at: summary
                .provider_started_at
                .as_ref()
                .map(|value| value.as_str().to_owned()),
            finished_at: summary
                .finished_at
                .as_ref()
                .map(|value| value.as_str().to_owned()),
            fingerprint: receipt.fingerprint.as_str().to_owned(),
            submitted_by: receipt.submitted_by.as_deref().map(str::to_owned),
            claimed: receipt.claimed,
            last_claim: receipt.last_claim.as_ref().map(|claim| ClaimRow {
                ordinal: claim.ordinal,
                case_id: claim.case_id.as_str().to_owned(),
                execution_id: claim.execution_id.as_str().to_owned(),
            }),
            interruption: receipt.interruption.as_deref().map(str::to_owned),
        }
    }
}

impl TryFrom<Request> for RunReceipt {
    type Error = Error;

    fn try_from(row: Request) -> Result<Self> {
        let phase = match row.phase.as_str() {
            "queued" => RunPhase::Queued,
            "preparing" => RunPhase::Preparing,
            "running" => RunPhase::Running,
            "finishing" => RunPhase::Finishing,
            "completed" => RunPhase::Completed,
            "failed" => RunPhase::Failed,
            "stopped" => RunPhase::Stopped,
            "interrupted" => RunPhase::Interrupted,
            _ => return Err(Error::Invalid("unknown stored request phase")),
        };
        if row.terminal != phase.terminal() {
            return Err(Error::Invalid(
                "stored terminal marker disagrees with phase",
            ));
        }

        Ok(Self {
            node_id: NodeId::from_uuid(row.node_id)?,
            summary: RunSummary {
                mode: match row.mode.as_str() {
                    "automatic" => crate::execution::ExecutionMode::Automatic,
                    "interactive" => crate::execution::ExecutionMode::Interactive,
                    _ => return Err(Error::Invalid("unknown stored execution mode")),
                },
                request_id: CommandId::from_uuid(row.request_id)?,
                cycle_id: TestCycleId::from_uuid(row.cycle_id)?,
                phase,
                paused: row.paused,
                stop_requested: row.stop_requested,
                planned: row.planned,
                prepared: row.prepared,
                finished: row.finished,
                unstarted: row.unstarted,
                failed: row.failed,
                accepted_at: timestamp(row.accepted_at)?,
                provider_started_at: row.provider_started_at.map(timestamp).transpose()?,
                finished_at: row.finished_at.map(timestamp).transpose()?,
            },
            fingerprint: row.fingerprint,
            submitted_by: row.submitted_by,
            claimed: row.claimed,
            last_claim: row
                .last_claim
                .map(|claim| {
                    Ok::<_, Error>(AttemptClaim {
                        ordinal: claim.ordinal,
                        case_id: CaseId::from_wire(claim.case_id)?,
                        execution_id: TestExecutionId::from_uuid(claim.execution_id)?,
                    })
                })
                .transpose()?,
            interruption: row.interruption,
            commands: {
                if row.commands.len() > 4097 {
                    return Err(Error::Invalid("stored command capacity exceeded"));
                }
                let mut commands = std::collections::BTreeMap::new();
                for row in row.commands {
                    let command = StepCommandReceipt::try_from(row)?;
                    if commands
                        .insert(command.command_id.clone(), command)
                        .is_some()
                    {
                        return Err(Error::Invalid("duplicate stored command identity"));
                    }
                }
                commands
            },
        })
    }
}

/* === Public Functions === */
pub fn schema() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: dokime_domain::model::identity::Identifier::new("dokime.execution.request").unwrap(),
        version: 1.try_into().unwrap(),
    }
}

pub fn encode(receipt: &RunReceipt, max_bytes: usize) -> Result<Vec<u8>> {
    receipt.summary.accepted_at.to_datetime()?;
    if let Some(timestamp) = &receipt.summary.provider_started_at {
        timestamp.to_datetime()?;
    }
    if let Some(timestamp) = &receipt.summary.finished_at {
        timestamp.to_datetime()?;
    }
    if receipt.commands.len() > 4097 {
        return Err(Error::Invalid("command receipt capacity exceeded"));
    }
    for (id, command) in &receipt.commands {
        if id != &command.command_id {
            return Err(Error::Invalid("command receipt key differs from identity"));
        }
        command.accepted_at.to_datetime()?;
        command.updated_at.to_datetime()?;
    }
    let mut row = Request::from(receipt);
    row.commands = receipt.commands.values().map(Command::row).collect();
    if row.encoded_len() > max_bytes {
        return Err(Error::Invalid("request receipt exceeds byte bound"));
    }
    Ok(row.encode_to_vec())
}

pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<RunReceipt> {
    if bytes.len() > max_bytes {
        return Err(Error::Invalid("request receipt exceeds byte bound"));
    }
    let row = Request::decode(bytes)?;
    if row.encode_to_vec() != bytes {
        return Err(Error::Invalid("noncanonical request receipt"));
    }
    row.try_into()
}

/* === Private Functions === */
fn reject_embedded_commands(bytes: &[u8]) -> Result<()> {
    // The Generated Schema Owns Field Order; Inspect Only The Parent Prefix.
    let prefix = [
        "request_id",
        "node_id",
        "cycle_id",
        "phase",
        "mode",
        "commands",
    ];
    if !Request::FIELDS
        .iter()
        .map(|field| field.name.as_ref())
        .take(prefix.len())
        .eq(prefix)
    {
        return Err(Error::Invalid(
            "request command prefix differs from its schema",
        ));
    }
    let mut input = harmos::message::postcard::Reader::new(bytes);
    for _ in 0..5 {
        let _: &str = input.value()?;
    }
    let commands: usize = input.value()?;
    if commands != 0 {
        return Err(Error::Invalid("parent object embeds command history"));
    }
    Ok(())
}

fn run_phase_text(value: &RunPhase) -> &'static str {
    match value {
        RunPhase::Queued => "queued",
        RunPhase::Preparing => "preparing",
        RunPhase::Running => "running",
        RunPhase::Finishing => "finishing",
        RunPhase::Completed => "completed",
        RunPhase::Failed => "failed",
        RunPhase::Stopped => "stopped",
        RunPhase::Interrupted => "interrupted",
    }
}

fn command_phase_text(value: &StepCommandPhase) -> &'static str {
    match value {
        StepCommandPhase::Accepted => "accepted",
        StepCommandPhase::Dispatching => "dispatching",
        StepCommandPhase::Running => "running",
        StepCommandPhase::Completed => "completed",
        StepCommandPhase::Refused => "refused",
        StepCommandPhase::Failed => "failed",
        StepCommandPhase::Interrupted => "interrupted",
    }
}

fn command_action_text(value: &StepCommand) -> &'static str {
    match value {
        StepCommand::Next => "next",
        StepCommand::RunStep { .. } => "run_step",
        StepCommand::Finish => "finish",
    }
}

fn timestamp(value: String) -> Result<Timestamp> {
    let timestamp = Timestamp::new(value);
    timestamp.to_datetime()?;
    Ok(timestamp)
}

/* === Tests === */
