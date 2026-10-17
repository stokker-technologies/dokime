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
mod saved;

/* === Re-Exports === */
pub(super) use listeners::{read_records, read_samples, sample_interval};

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, OnceLock};

// Third Party Imports

// Local Imports
use super::{Client, Error, ResponseStream, Result, contracts, validate_observed_cursor};
use dokime_domain::model::execution::ExecutionMode;
use dokime_domain::model::execution::StepCommand;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::StepAttemptId;
use dokime_domain::model::identity::StepId;
use dokime_domain::model::identity::TestCycleId;
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

#[derive(Clone)]
pub struct ExecutionRequestHandle {
    client: Client,
    id: CommandId,
    cycle: Arc<OnceLock<TestCycleId>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    pub fn execution_request(&self, id: CommandId) -> ExecutionRequestHandle {
        ExecutionRequestHandle::new(self.clone(), id, None)
    }

    pub fn execution(&self, id: TestExecutionId) -> TestExecutionHandle {
        TestExecutionHandle {
            client: self.clone(),
            id,
        }
    }
}

impl ExecutionRequestHandle {
    pub(super) async fn evidence_ready(&self) -> Result<bool> {
        evidence_complete(&self.status().await?)
    }

    pub(super) async fn wait_evidence(&self) -> Result<()> {
        evidence_complete(&self.wait(None, None).await?).map(|_| ())
    }

    pub(super) fn bind_cycle(&self, client: &Client, cycle: &TestCycleId) -> Result<Self> {
        if !Arc::ptr_eq(&self.client.inner, &client.inner)
            || self.cycle.get_or_init(|| cycle.clone()) != cycle
        {
            return Err(Error::Rpc(tonic::Status::invalid_argument(
                "request belongs to another cycle or connection",
            )));
        }
        Ok(self.clone())
    }

    pub(super) fn new(client: Client, id: CommandId, cycle: Option<TestCycleId>) -> Self {
        let bound = cycle.map(OnceLock::from).unwrap_or_default();
        Self {
            client,
            id,
            cycle: Arc::new(bound),
        }
    }

    pub fn id(&self) -> &CommandId {
        &self.id
    }

    pub fn updates(
        &self,
        interval: Option<std::time::Duration>,
    ) -> Result<ResponseStream<contracts::execution::RunRequestReceipt>> {
        let interval = poll_interval(interval)?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let mut previous = None;
            loop {
                let receipt = match handle.status().await {
                    Ok(receipt) => receipt,
                    Err(error) => {
                        let _ = sender.send(Err(error)).await;
                        return;
                    }
                };
                // Status already validates the original phase before publishing.
                let terminal = request_terminal(receipt.phase).expect("validated receipt phase");
                if previous.as_ref() != Some(&receipt) {
                    previous = Some(receipt.clone());
                    if sender.send(Ok(receipt)).await.is_err() {
                        return;
                    }
                }
                if terminal {
                    return;
                }
                tokio::time::sleep(interval).await;
            }
        })
    }

    pub async fn status(&self) -> Result<contracts::execution::RunRequestReceipt> {
        self.client.verify(false).await?;
        let receipt = self
            .client
            .execution_get_run_request(contracts::execution::GetRunRequest {
                request_id: self.id.to_string(),
            })
            .await?;
        validate_request_receipt(&self.id, &self.cycle, &receipt)?;
        Ok(receipt)
    }

    pub async fn wait(
        &self,
        timeout: Option<std::time::Duration>,
        interval: Option<std::time::Duration>,
    ) -> Result<contracts::execution::RunRequestReceipt> {
        let interval = poll_interval(interval)?;
        let wait = self.client.guard(async {
            loop {
                let receipt = self.status().await?;
                if request_terminal(receipt.phase)? {
                    return Ok(receipt);
                }
                tokio::time::sleep(interval).await;
            }
        });
        match timeout {
            Some(timeout) => tokio::time::timeout(timeout, wait)
                .await
                .map_err(|source| Error::WaitTimedOut { source })??,
            None => wait.await?,
        }
    }
}

impl TestExecutionHandle {
    pub fn id(&self) -> &TestExecutionId {
        &self.id
    }

    pub async fn snapshot(&self) -> Result<contracts::execution::ExecutionDetail> {
        self.client.verify(true).await?;
        let detail = self
            .client
            .execution_get_execution(contracts::execution::GetExecution {
                execution_id: self.id.to_string(),
            })
            .await?;
        validate_snapshot(&self.id, &detail)?;
        validate_observed_cursor(
            &self.client.backend().epoch,
            detail.observed_cursor.as_ref(),
        )?;
        Ok(detail)
    }

    pub async fn checkpoint(&self) -> Result<contracts::checkpoint::ExecutionCheckpoint> {
        self.client.verify(false).await?;
        let checkpoint = self
            .client
            .history_get_execution_checkpoint(contracts::history::GetExecutionCheckpoint {
                execution_identifier: self.id.to_string(),
            })
            .await?;
        validate_checkpoint(&self.id, &checkpoint)?;
        Ok(checkpoint)
    }

    pub async fn document_at(
        &self,
        checkpoint: &contracts::checkpoint::ExecutionCheckpoint,
    ) -> Result<contracts::history::CheckpointDocument> {
        validate_checkpoint(&self.id, checkpoint)?;
        self.client.verify(false).await?;
        let document = self
            .client
            .history_get_checkpoint_document(contracts::history::GetCheckpointDocument {
                checkpoint: Some(checkpoint.clone()),
            })
            .await?;
        validate_outcome(&self.id, checkpoint, &document)?;
        Ok(document)
    }

    pub async fn wait(
        &self,
        timeout: Option<std::time::Duration>,
        interval: Option<std::time::Duration>,
    ) -> Result<contracts::history::CheckpointDocument> {
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
            Some(timeout) => tokio::time::timeout(timeout, wait)
                .await
                .map_err(|source| Error::WaitTimedOut { source })??,
            None => wait.await?,
        }
    }

    pub async fn start(
        &self,
        key: &str,
        mode: ExecutionMode,
    ) -> Result<contracts::execution::Admission> {
        dokime_domain::model::execution::command::validate_idempotency_key(key)
            .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
        self.client.verify(true).await?;
        let mode = match mode {
            ExecutionMode::Automatic => contracts::execution::ExecutionMode::Automatic,
            ExecutionMode::Interactive => contracts::execution::ExecutionMode::Interactive,
        };
        let reply = self
            .client
            .execution_run_execution(contracts::execution::RunExecution {
                execution_id: self.id.to_string(),
                context: None,
                idempotency_key: Some(key.to_owned()),
                mode,
            })
            .await?;
        admission(&self.client.backend().epoch, reply)
    }

    pub async fn command_receipt(
        &self,
        request_id: &CommandId,
        command_id: &CommandId,
    ) -> Result<contracts::execution::ExecutionCommandReceipt> {
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
        &self,
        request_id: &CommandId,
        command_id: &CommandId,
        command: &StepCommand,
    ) -> Result<contracts::execution::ExecutionCommandReceipt> {
        self.client.verify(true).await?;
        let (action, step_id) = match command {
            StepCommand::Next => (contracts::execution::ExecutionCommandAction::Next, None),
            StepCommand::RunStep { step_id } => (
                contracts::execution::ExecutionCommandAction::RunStep,
                Some(step_id.to_string()),
            ),
            StepCommand::Finish => (contracts::execution::ExecutionCommandAction::Finish, None),
        };
        let request = contracts::execution::SubmitExecutionCommand {
            request_id: request_id.to_string(),
            command_id: command_id.to_string(),
            execution_id: self.id.to_string(),
            action,
            step_id,
        };
        let receipt = self
            .client
            .execution_submit_execution_command(request.clone())
            .await?;
        validate_command_receipt(&self.id, request_id, command_id, &receipt)?;
        if receipt.action != request.action || receipt.step_id != request.step_id {
            return Err(invalid("command receipt differs from submitted action"));
        }
        Ok(receipt)
    }

    pub async fn pause(&self) -> Result<contracts::execution::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_pause_execution(contracts::execution::PauseExecution {
                execution_id: self.id.to_string(),
                context: None,
            })
            .await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        Ok(reply)
    }

    pub async fn resume(&self) -> Result<contracts::execution::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_resume_execution(contracts::execution::ResumeExecution {
                execution_id: self.id.to_string(),
                context: None,
            })
            .await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        Ok(reply)
    }

    pub async fn stop(&self) -> Result<contracts::execution::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_stop_execution(contracts::execution::StopExecution {
                execution_id: self.id.to_string(),
                context: None,
            })
            .await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        Ok(reply)
    }
}

/* === Public Functions === */
pub(super) fn admission(
    epoch: &str,
    reply: contracts::execution::RunReply,
) -> Result<contracts::execution::Admission> {
    match reply.outcome {
        Some(contracts::execution::RunReplyOutcome::Accepted(admission)) => {
            validate_admission(epoch, &admission)?;
            Ok(admission)
        }
        Some(contracts::execution::RunReplyOutcome::Rejected(rejection)) => {
            Err(Error::ExecutionRejected { rejection })
        }
        None => Err(invalid("execution admission has no outcome")),
    }
}

/* === Private Functions === */
fn evidence_complete(receipt: &contracts::execution::RunRequestReceipt) -> Result<bool> {
    if receipt.phase == contracts::execution::RunRequestPhase::Interrupted {
        return Err(invalid(
            "interrupted acquisition does not certify complete live evidence",
        ));
    }
    request_terminal(receipt.phase)
}

fn validate_request_receipt(
    id: &CommandId,
    cycle: &OnceLock<TestCycleId>,
    receipt: &contracts::execution::RunRequestReceipt,
) -> Result<()> {
    if receipt.request_id != id.as_str() {
        return Err(invalid("request receipt identity differs"));
    }
    request_terminal(receipt.phase)?;
    let observed = TestCycleId::from_uuid(&receipt.cycle_id)
        .map_err(|_| invalid("request receipt has an invalid cycle identity"))?;
    if cycle.get_or_init(|| observed.clone()) != &observed {
        return Err(invalid("request receipt belongs to another cycle"));
    }
    Ok(())
}

fn poll_interval(interval: Option<std::time::Duration>) -> Result<std::time::Duration> {
    let interval = interval.unwrap_or(REQUEST_POLL_INTERVAL);
    if interval.is_zero() {
        return Err(Error::Rpc(tonic::Status::invalid_argument(
            "poll interval must be positive",
        )));
    }
    Ok(interval)
}

fn validate_checkpoint(
    id: &TestExecutionId,
    checkpoint: &contracts::checkpoint::ExecutionCheckpoint,
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
    let canonical = version
        .encode_bounded(NonZeroUsize::new(4096).unwrap())
        .map_err(|_| invalid("saved checkpoint version is invalid"))?;
    if canonical != checkpoint.version {
        return Err(invalid("saved checkpoint version is noncanonical"));
    }
    let original = dokime_domain::model::execution::ExecutionCheckpoint {
        execution_id: id.clone(),
        position: checkpoint_cursor(checkpoint.position.as_ref())?,
        version,
        evidence_through: checkpoint_cursor(checkpoint.evidence_through.as_ref())?,
    };
    original
        .validate()
        .map_err(|_| invalid("saved checkpoint cut differs"))?;
    Ok(original)
}

fn checkpoint_cursor(
    cursor: Option<&contracts::checkpoint::StorageCursor>,
) -> Result<harmos::storage::StorageCursor> {
    let cursor = cursor.ok_or_else(|| invalid("saved checkpoint cursor is missing"))?;
    let history = cursor
        .history
        .as_ref()
        .ok_or_else(|| invalid("saved checkpoint history is missing"))?;
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

fn request_terminal(phase: contracts::execution::RunRequestPhase) -> Result<bool> {
    match phase {
        contracts::execution::RunRequestPhase::Completed
        | contracts::execution::RunRequestPhase::Failed
        | contracts::execution::RunRequestPhase::Stopped
        | contracts::execution::RunRequestPhase::Interrupted => Ok(true),
        contracts::execution::RunRequestPhase::Queued
        | contracts::execution::RunRequestPhase::Preparing
        | contracts::execution::RunRequestPhase::Running
        | contracts::execution::RunRequestPhase::Finishing => Ok(false),
        _ => Err(invalid("request receipt has an unknown phase")),
    }
}

fn validate_admission(epoch: &str, admission: &contracts::execution::Admission) -> Result<()> {
    CommandId::from_uuid(&admission.command_id)
        .map_err(|_| invalid("invalid admission identity"))?;
    validate_observed_cursor(epoch, admission.cursor.as_ref())
}

fn validate_command_receipt(
    execution_id: &TestExecutionId,
    request_id: &CommandId,
    command_id: &CommandId,
    receipt: &contracts::execution::ExecutionCommandReceipt,
) -> Result<()> {
    if receipt.execution_id != execution_id.as_str()
        || receipt.request_id != request_id.as_str()
        || receipt.command_id != command_id.as_str()
    {
        return Err(invalid("command receipt identity differs"));
    }
    match (receipt.action, &receipt.step_id) {
        (
            contracts::execution::ExecutionCommandAction::Next
            | contracts::execution::ExecutionCommandAction::Finish,
            None,
        ) => {}
        (contracts::execution::ExecutionCommandAction::RunStep, Some(step_id)) => {
            StepId::from_uuid(step_id)
                .map_err(|_| invalid("command receipt has an invalid step identity"))?;
        }
        _ => return Err(invalid("command receipt has an invalid action")),
    }
    match receipt.phase {
        contracts::execution::ExecutionCommandPhase::Accepted
        | contracts::execution::ExecutionCommandPhase::Dispatching
        | contracts::execution::ExecutionCommandPhase::Running
        | contracts::execution::ExecutionCommandPhase::Completed
        | contracts::execution::ExecutionCommandPhase::Refused
        | contracts::execution::ExecutionCommandPhase::Failed
        | contracts::execution::ExecutionCommandPhase::Interrupted => {}
        _ => return Err(invalid("command receipt has an unknown phase")),
    }
    if let Some(attempt_id) = &receipt.attempt_id {
        StepAttemptId::from_uuid(attempt_id)
            .map_err(|_| invalid("command receipt has an invalid attempt identity"))?;
    }
    Ok(())
}

fn validate_snapshot(
    id: &TestExecutionId,
    detail: &contracts::execution::ExecutionDetail,
) -> Result<()> {
    let summary = detail
        .summary
        .as_ref()
        .ok_or_else(|| invalid("execution snapshot has no identity"))?;
    if summary.execution_id != id.as_str() {
        return Err(invalid("execution snapshot identity differs"));
    }
    Ok(())
}

fn validate_outcome(
    id: &TestExecutionId,
    checkpoint: &contracts::checkpoint::ExecutionCheckpoint,
    outcome: &contracts::history::CheckpointDocument,
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
        return Err(invalid(
            "sealed execution outcome identity or result is missing",
        ));
    }
    if execution.result.as_ref().expect("checked result").status
        == contracts::execution::ResultStatus::Unspecified
    {
        return Err(invalid("sealed execution result has an unknown status"));
    }
    match execution.status {
        contracts::execution::ExecutionStatus::Invalidated
        | contracts::execution::ExecutionStatus::Passed
        | contracts::execution::ExecutionStatus::Failed
        | contracts::execution::ExecutionStatus::Blocked
        | contracts::execution::ExecutionStatus::Cancelled
        | contracts::execution::ExecutionStatus::Interrupted
        | contracts::execution::ExecutionStatus::Skipped => Ok(()),
        _ => Err(invalid("sealed execution outcome is not terminal")),
    }
}

fn invalid(message: &'static str) -> Error {
    Error::Rpc(tonic::Status::data_loss(message))
}

/* === Tests === */
