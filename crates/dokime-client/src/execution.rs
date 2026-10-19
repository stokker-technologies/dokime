/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod listeners;
mod prompts;
mod run;
mod saved;

/* === Re-Exports === */

pub(super) use listeners::{read_records, read_samples, sample_interval, validate_filters};
pub use run::{RunCallbacks, RunOptions};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{Client, Error, ResponseStream, Result, contracts, validate_observed_cursor};
use dokime_domain::model::execution::ExecutionMode;
use dokime_domain::model::execution::StepCommand;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::StepAttemptId;
use dokime_domain::model::identity::StepId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

const REQUEST_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct TestExecutionHandle {
    client: Client,
    id: TestExecutionId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub fn execution(&self, id: TestExecutionId) -> TestExecutionHandle {
        TestExecutionHandle { client: self.clone(), id }
    }
}

impl TestExecutionHandle {
    pub async fn export(
        &self, destination: std::path::PathBuf, overwrite: bool,
    ) -> std::result::Result<super::Download<dokime_domain::model::exports::EvidenceExportMetadata>, super::DownloadError> {
        let subject = dokime_domain::model::exports::EvidenceSubject::Execution(self.id.to_string());
        self.client.download_evidence(contracts::content::DownloadEvidence { subject: Some(subject) }, destination, overwrite).await
    }

    pub fn id(&self) -> &TestExecutionId {
        &self.id
    }

    pub async fn snapshot(&self) -> Result<dokime_domain::model::execution::wire::ExecutionDetail> {
        self.client.verify(true).await?;
        let detail = self.client.execution_get_execution(contracts::execution::GetExecution { execution_id: self.id.to_string() }).await?;
        validate_snapshot(&self.id, &detail)?;
        validate_observed_cursor(&self.client.backend().epoch, detail.observed_cursor.as_ref())?;
        Ok(detail)
    }

    pub async fn checkpoint(&self) -> Result<dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint> {
        self.client.verify(false).await?;
        let checkpoint = self
            .client
            .history_get_execution_checkpoint(contracts::history::GetExecutionCheckpoint { execution_identifier: self.id.to_string() })
            .await?;
        validate_checkpoint(&self.id, &checkpoint)?;
        Ok(checkpoint)
    }

    pub async fn document_at(
        &self, checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    ) -> Result<dokime_domain::model::history::wire::CheckpointDocument> {
        validate_checkpoint(&self.id, checkpoint)?;
        self.client.verify(false).await?;
        let document = self
            .client
            .history_get_checkpoint_document(contracts::history::GetCheckpointDocument { checkpoint: Some(checkpoint.clone()) })
            .await?;
        validate_outcome(&self.id, checkpoint, &document)?;
        Ok(document)
    }

    pub async fn wait(
        &self, timeout: Option<std::time::Duration>, interval: Option<std::time::Duration>,
    ) -> Result<dokime_domain::model::history::wire::CheckpointDocument> {
        let interval = poll_interval(interval)?;
        let wait = self.client.guard(async {
            loop {
                let checkpoint = match self.checkpoint().await {
                    Ok(checkpoint) => checkpoint,
                    Err(Error::Rpc(status)) if status.code() == tonic::Code::NotFound => {
                        // An Absent Seal Cannot Certify Work From A Replacement Epoch.
                        self.snapshot().await?;
                        tokio::time::sleep(interval).await;
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                // Keep The Frozen Document RPC Off Nested Listener Future Frames.
                return Box::pin(self.document_at(&checkpoint)).await;
            }
        });
        match timeout {
            Some(timeout) => tokio::time::timeout(timeout, wait).await.map_err(|source| Error::WaitTimedOut { source })??,
            None => wait.await?,
        }
    }

    pub async fn start(&self, mode: ExecutionMode) -> Result<()> {
        self.client.verify(true).await?;
        let mode = match mode {
            ExecutionMode::Automatic => dokime_domain::model::execution::wire::ExecutionMode::Automatic,
            ExecutionMode::Interactive => dokime_domain::model::execution::wire::ExecutionMode::Interactive,
        };
        let reply =
            self.client.execution_start_execution(contracts::execution::StartExecution { execution_id: self.id.to_string(), mode }).await?;
        admission(&self.client.backend().epoch, reply).map(|_| ())
    }

    pub async fn command_receipt(
        &self, request_id: &CommandId, command_id: &CommandId,
    ) -> Result<dokime_domain::model::execution::wire::ExecutionCommandReceipt> {
        self.client.verify(false).await?;
        let receipt = self
            .client
            .execution_get_execution_command(contracts::execution::GetExecutionCommand {
                request_id: request_id.to_string(),
                command_id: command_id.to_string(),
            })
            .await?;
        validate_command_receipt(&self.id, request_id, command_id, &receipt)?;
        Ok(receipt)
    }

    pub async fn command(
        &self, request_id: &CommandId, command_id: &CommandId, command: &StepCommand,
    ) -> Result<dokime_domain::model::execution::wire::ExecutionCommandReceipt> {
        self.client.verify(true).await?;
        let (action, step_id) = match command {
            StepCommand::Next => (dokime_domain::model::execution::wire::ExecutionCommandAction::Next, None),
            StepCommand::RunStep { step_id } => {
                (dokime_domain::model::execution::wire::ExecutionCommandAction::RunStep, Some(step_id.to_string()))
            }
            StepCommand::Finish => (dokime_domain::model::execution::wire::ExecutionCommandAction::Finish, None),
        };
        let request = contracts::execution::SubmitExecutionCommand {
            request_id: request_id.to_string(),
            command_id: command_id.to_string(),
            execution_id: self.id.to_string(),
            action,
            step_id,
        };
        let receipt = self.client.execution_submit_execution_command(request.clone()).await?;
        validate_command_receipt(&self.id, request_id, command_id, &receipt)?;
        if receipt.action != request.action || receipt.step_id != request.step_id {
            return Err(invalid("command receipt differs from submitted action"));
        }
        Ok(receipt)
    }

    pub async fn pause(&self) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_pause_execution(contracts::execution::PauseExecution { execution_id: self.id.to_string(), context: None })
            .await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        Ok(reply)
    }

    pub async fn resume(&self) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_resume_execution(contracts::execution::ResumeExecution { execution_id: self.id.to_string(), context: None })
            .await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        Ok(reply)
    }

    pub async fn stop(&self) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_stop_execution(contracts::execution::StopExecution { execution_id: self.id.to_string(), context: None })
            .await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        Ok(reply)
    }
}

/* === Public Functions === */

pub(super) fn admission(
    epoch: &str, reply: dokime_domain::model::execution::wire::RunReply,
) -> Result<dokime_domain::model::execution::wire::Admission> {
    match reply.outcome {
        Some(dokime_domain::model::execution::wire::RunReplyOutcome::Accepted(admission)) => {
            validate_admission(epoch, &admission)?;
            Ok(admission)
        }
        Some(dokime_domain::model::execution::wire::RunReplyOutcome::Rejected(rejection)) => Err(Error::ExecutionRejected { rejection }),
        None => Err(invalid("execution admission has no outcome")),
    }
}

/* === Private Functions === */

fn poll_interval(interval: Option<std::time::Duration>) -> Result<std::time::Duration> {
    let interval = interval.unwrap_or(REQUEST_POLL_INTERVAL);
    if interval.is_zero() {
        return Err(Error::Rpc(tonic::Status::invalid_argument("poll interval must be positive")));
    }
    Ok(interval)
}

fn validate_checkpoint(
    id: &TestExecutionId, checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
) -> Result<dokime_domain::model::execution::ExecutionCheckpoint> {
    use harmos::storage::ObjectVersionToken;
    use std::num::NonZeroUsize;

    if checkpoint.execution_identifier != id.as_str() || checkpoint.version.len() > 4096 {
        return Err(invalid("saved checkpoint identity or version differs"));
    }
    let version = ObjectVersionToken::decode_bounded(
        &checkpoint.version,
        NonZeroUsize::new(4096 + std::mem::size_of::<ObjectVersionToken>()).unwrap(),
    )
    .map_err(|_| invalid("saved checkpoint version is invalid"))?;
    let canonical = version.encode_bounded(NonZeroUsize::new(4096).unwrap()).map_err(|_| invalid("saved checkpoint version is invalid"))?;
    if canonical != checkpoint.version {
        return Err(invalid("saved checkpoint version is noncanonical"));
    }
    let original = dokime_domain::model::execution::ExecutionCheckpoint {
        execution_id: id.clone(),
        position: checkpoint_cursor(checkpoint.position.as_ref())?,
        version,
        evidence_through: checkpoint_cursor(checkpoint.evidence_through.as_ref())?,
    };
    original.validate().map_err(|_| invalid("saved checkpoint cut differs"))?;
    Ok(original)
}

fn checkpoint_cursor(
    cursor: Option<&dokime_domain::model::execution::wire_checkpoint::StorageCursor>,
) -> Result<harmos::storage::StorageCursor> {
    let cursor = cursor.ok_or_else(|| invalid("saved checkpoint cursor is missing"))?;
    let history = cursor.history.as_ref().ok_or_else(|| invalid("saved checkpoint history is missing"))?;
    if history.store.len() > 4096 || history.epoch.len() > 4096 {
        return Err(invalid("saved checkpoint history exceeds its bound"));
    }
    Ok(harmos::storage::StorageCursor {
        history: harmos::storage::StoreHistory {
            store: dokime_domain::model::identity::Identifier::new(&history.store)
                .map_err(|_| invalid("saved checkpoint store is invalid"))?,
            epoch: dokime_domain::model::identity::Identifier::new(&history.epoch)
                .map_err(|_| invalid("saved checkpoint epoch is invalid"))?,
        },
        sequence: cursor.sequence,
    })
}

fn validate_admission(epoch: &str, admission: &dokime_domain::model::execution::wire::Admission) -> Result<()> {
    CommandId::from_uuid(&admission.command_id).map_err(|_| invalid("invalid admission identity"))?;
    validate_observed_cursor(epoch, admission.cursor.as_ref())
}

fn validate_command_receipt(
    execution_id: &TestExecutionId, request_id: &CommandId, command_id: &CommandId,
    receipt: &dokime_domain::model::execution::wire::ExecutionCommandReceipt,
) -> Result<()> {
    if receipt.execution_id != execution_id.as_str()
        || receipt.request_id != request_id.as_str()
        || receipt.command_id != command_id.as_str()
    {
        return Err(invalid("command receipt identity differs"));
    }
    match (receipt.action, &receipt.step_id) {
        (
            dokime_domain::model::execution::wire::ExecutionCommandAction::Next
            | dokime_domain::model::execution::wire::ExecutionCommandAction::Finish,
            None,
        ) => {}
        (dokime_domain::model::execution::wire::ExecutionCommandAction::RunStep, Some(step_id)) => {
            StepId::from_uuid(step_id).map_err(|_| invalid("command receipt has an invalid step identity"))?;
        }
        _ => return Err(invalid("command receipt has an invalid action")),
    }
    match receipt.phase {
        dokime_domain::model::execution::wire::ExecutionCommandPhase::Accepted
        | dokime_domain::model::execution::wire::ExecutionCommandPhase::Dispatching
        | dokime_domain::model::execution::wire::ExecutionCommandPhase::Running
        | dokime_domain::model::execution::wire::ExecutionCommandPhase::Completed
        | dokime_domain::model::execution::wire::ExecutionCommandPhase::Refused
        | dokime_domain::model::execution::wire::ExecutionCommandPhase::Failed
        | dokime_domain::model::execution::wire::ExecutionCommandPhase::Interrupted => {}
        _ => return Err(invalid("command receipt has an unknown phase")),
    }
    if let Some(attempt_id) = &receipt.attempt_id {
        StepAttemptId::from_uuid(attempt_id).map_err(|_| invalid("command receipt has an invalid attempt identity"))?;
    }
    Ok(())
}

fn validate_snapshot(id: &TestExecutionId, detail: &dokime_domain::model::execution::wire::ExecutionDetail) -> Result<()> {
    let summary = detail.summary.as_ref().ok_or_else(|| invalid("execution snapshot has no identity"))?;
    if summary.execution_id != id.as_str() {
        return Err(invalid("execution snapshot identity differs"));
    }
    Ok(())
}

fn validate_outcome(
    id: &TestExecutionId, checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    outcome: &dokime_domain::model::history::wire::CheckpointDocument,
) -> Result<()> {
    if outcome.checkpoint.as_ref() != Some(checkpoint) || !outcome.acquisition_finished {
        return Err(invalid("execution outcome differs from the persisted seal"));
    }
    let execution = outcome
        .document
        .as_ref()
        .and_then(|document| document.execution.as_ref())
        .ok_or_else(|| invalid("sealed execution outcome is missing"))?;
    if execution.execution_id != id.as_str() || execution.result.is_none() {
        return Err(invalid("sealed execution outcome identity or result is missing"));
    }
    if execution.result.as_ref().expect("checked result").status == dokime_domain::model::execution::wire::ResultStatus::Unspecified {
        return Err(invalid("sealed execution result has an unknown status"));
    }
    match execution.status {
        dokime_domain::model::execution::wire::ExecutionStatus::Invalidated
        | dokime_domain::model::execution::wire::ExecutionStatus::Passed
        | dokime_domain::model::execution::wire::ExecutionStatus::Failed
        | dokime_domain::model::execution::wire::ExecutionStatus::Blocked
        | dokime_domain::model::execution::wire::ExecutionStatus::Cancelled
        | dokime_domain::model::execution::wire::ExecutionStatus::Interrupted
        | dokime_domain::model::execution::wire::ExecutionStatus::Skipped => Ok(()),
        _ => Err(invalid("sealed execution outcome is not terminal")),
    }
}

fn invalid(message: &'static str) -> Error {
    Error::Rpc(tonic::Status::data_loss(message))
}

/* === Tests === */
