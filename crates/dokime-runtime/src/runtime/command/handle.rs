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
use std::time::Instant;
// Third Party Imports
use harmos::{Error as HarmosError, state::Position, state::RequestKey};
// Local Imports
use crate::{
    CommandContext, CommandId, CommandReceipt, runtime::Error, JobRunId, RuntimeHandle,
    commands::accepted_receipt, execution_runtime::provider::admission::RunProvider,
    internal_rejection, rejected_receipt, unavailable_rejection,
};
use crate::core::{CaseId, TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    fn provider_availability_rejection(
        &self,
        test_cycle_id: &TestCycleId,
        test_execution_id: Option<&TestExecutionId>,
        context: &CommandContext,
    ) -> Option<CommandReceipt> {
        let state = self.state_read();
        // A cycle run dispatches one provider run per target, so every target
        // the selection resolves to has to have its executor.
        let targets = crate::execution_runtime::provider::provider_targets_for_cycle(
            &state,
            test_cycle_id,
            test_execution_id,
        );
        if targets.is_empty() {
            return None;
        }
        if targets.iter().all(|target| {
            self.plugin_available(&target.source) && self.providers.get(target).is_some()
        }) {
            None
        } else {
            Some(rejected_receipt(
                CommandId::new(),
                context,
                Timestamp::now_utc(),
                unavailable_rejection("provider executor is unavailable"),
            ))
        }
    }

    pub async fn run_cycle_cases(
        &self,
        test_cycle_id: TestCycleId,
        context: CommandContext,
    ) -> CommandReceipt {
        self.owned_provider(
            test_cycle_id,
            None,
            crate::execution::ExecutionMode::Automatic,
            context,
        )
        .await
    }

    pub async fn run_cycle_case(
        &self,
        test_cycle_id: TestCycleId,
        test_case_id: CaseId,
        context: CommandContext,
    ) -> CommandReceipt {
        let saved_retry = if let Some(id) = self.provider_request_id(&context) {
            match self.request_storage.get(&id).await {
                Ok(Some(receipt)) => receipt
                    .last_claim
                    .as_ref()
                    .filter(|claim| {
                        receipt.summary.cycle_id == test_cycle_id && claim.case_id == test_case_id
                    })
                    .map(|claim| claim.execution_id.clone()),
                Ok(None) => None,
                Err(error) => {
                    return rejected_receipt(
                        id,
                        &context,
                        Timestamp::now_utc(),
                        unavailable_rejection(error.to_string()),
                    );
                }
            }
        } else {
            None
        };
        if let Some(execution_id) = saved_retry {
            return self
                .owned_provider(
                    test_cycle_id,
                    Some(execution_id),
                    crate::execution::ExecutionMode::Automatic,
                    context,
                )
                .await;
        }
        let execution_id = {
            let state = self.state_read();
            let retry = self
                .provider_request_id(&context)
                .and_then(|id| state.run_requests.get(&id))
                .filter(|request| request.summary.cycle_id == test_cycle_id)
                .and_then(|request| request.test_execution_id.as_ref())
                .and_then(|id| state.executions.get(id))
                .filter(|execution| execution.test_case_id == test_case_id);
            retry
                .or_else(|| {
                    state.cycles.get(&test_cycle_id).and_then(|cycle| {
                        state.executions_for_cycle(cycle).find(|execution| {
                            execution.test_case_id == test_case_id
                                && execution.status == crate::execution::ExecutionStatus::Skipped
                                && execution.started_at.is_none()
                                && execution.finished_at.is_none()
                        })
                    })
                })
                .map(|execution| execution.test_execution_id.clone())
        };
        let Some(execution_id) = execution_id else {
            return rejected_receipt(
                CommandId::new(),
                &context,
                Timestamp::now_utc(),
                Error::conflict(
                    "execution_not_planned",
                    crate::transactions::provider::EXECUTION_NOT_PLANNED,
                ),
            );
        };
        self.run_execution(
            execution_id,
            crate::execution::ExecutionMode::Automatic,
            context,
        )
        .await
    }

    pub async fn run_execution(
        &self,
        test_execution_id: TestExecutionId,
        mode: crate::execution::ExecutionMode,
        context: CommandContext,
    ) -> CommandReceipt {
        let cycle_id = self
            .state_read()
            .executions
            .get(&test_execution_id)
            .map(|execution| execution.test_cycle_id.clone());
        let Some(cycle_id) = cycle_id else {
            return rejected_receipt(
                CommandId::new(),
                &context,
                Timestamp::now_utc(),
                Error::not_found("execution_not_found", "execution not found"),
            );
        };
        self.owned_provider(cycle_id, Some(test_execution_id), mode, context)
            .await
    }

    fn provider_request_id(&self, context: &CommandContext) -> Option<CommandId> {
        let key = context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())?;
        use sha2::Digest;
        let node = self.state_read().identity.node.node_id.clone();
        let digest = sha2::Sha256::digest(
            serde_json::to_vec(&("execution-request", &node, &context.submitted_by, key))
                .expect("request identity encodes"),
        );
        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&digest[..16]);
        bytes[6] = (bytes[6] & 0x0f) | 0x80;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Some(
            CommandId::from_uuid(uuid::Uuid::from_bytes(bytes).to_string())
                .expect("derived request UUID"),
        )
    }

    async fn owned_provider(
        &self,
        cycle: TestCycleId,
        execution: Option<TestExecutionId>,
        mode: crate::execution::ExecutionMode,
        context: CommandContext,
    ) -> CommandReceipt {
        if self.is_shutting_down() {
            return rejected_receipt(
                CommandId::new(),
                &context,
                Timestamp::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        }
        let Some(permit) = self.admit_owned_work() else {
            return rejected_receipt(
                CommandId::new(),
                &context,
                Timestamp::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        };
        if self.is_shutting_down() {
            return rejected_receipt(
                CommandId::new(),
                &context,
                Timestamp::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        }
        let runtime = self.clone();
        let fallback = context.clone();
        tokio::spawn(async move {
            // The owned task retains capacity even if its caller disconnects.
            let _permit = permit;
            runtime
                .submit_provider(cycle, execution, mode, context)
                .await
        })
        .await
        .unwrap_or_else(|error| {
            rejected_receipt(
                CommandId::new(),
                &fallback,
                Timestamp::now_utc(),
                internal_rejection(format!("request admission task failed: {error}")),
            )
        })
    }

    async fn submit_provider(
        &self,
        test_cycle_id: TestCycleId,
        test_execution_id: Option<TestExecutionId>,
        mode: crate::execution::ExecutionMode,
        context: CommandContext,
    ) -> CommandReceipt {
        let _plugin_admission = self.plugin_admission.lock().await;
        if self.is_shutting_down() {
            return rejected_receipt(
                CommandId::new(),
                &context,
                Timestamp::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        }
        let started = Instant::now();
        let node = self.state_read().identity.node.node_id.clone();
        let command_id = self.provider_request_id(&context).unwrap_or_default();
        use sha2::Digest;
        let encoded = serde_json::to_vec(&(&test_cycle_id, &test_execution_id, mode))
            .expect("request identity encodes");
        let fingerprint: String = sha2::Sha256::digest(encoded)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        match self.request_storage.get(&command_id).await {
            Ok(Some(receipt)) => {
                if receipt.fingerprint != fingerprint
                    || receipt.submitted_by != context.submitted_by
                {
                    return rejected_receipt(
                        command_id,
                        &context,
                        Timestamp::now_utc(),
                        Error::conflict(
                            "request_identity_conflict",
                            "request identity already belongs to another run selection",
                        ),
                    );
                }
                let mut accepted = accepted_receipt(
                    command_id.clone(),
                    &context,
                    receipt.summary.accepted_at.clone(),
                    Some(JobRunId::from_seed(command_id.as_str())),
                );
                accepted.runtime_revision = self.state_manager.applied().0;
                return accepted;
            }
            Ok(None) => (),
            Err(error) => {
                return rejected_receipt(
                    command_id,
                    &context,
                    Timestamp::now_utc(),
                    unavailable_rejection(error.to_string()),
                );
            }
        }
        let contracts = self.state_read().telemetry_contracts();
        if let Err(error) =
            crate::runtime::storage::telemetry::register_schemas(self.storage(), &contracts).await
        {
            return rejected_receipt(
                command_id.clone(),
                &context,
                Timestamp::now_utc(),
                unavailable_rejection(error.to_string()),
            );
        }
        // Provider admission runs against the published plan, so a cycle whose
        // projection admission deferred has to be published first. Everything
        // after this point assumes its executions are durable.
        if let Err(error) = self
            .execution_storage
            .save_cycle(test_cycle_id.clone())
            .await
        {
            return rejected_receipt(
                CommandId::new(),
                &context,
                Timestamp::now_utc(),
                unavailable_rejection(error.to_string()),
            );
        }
        if let Some(receipt) = self.provider_availability_rejection(
            &test_cycle_id,
            test_execution_id.as_ref(),
            &context,
        ) {
            return receipt;
        }
        let accepted_at = Timestamp::now_utc();
        let job_id = JobRunId::from_seed(command_id.as_str());
        let artifact = crate::execution_runtime::provider::provider_target_for_cycle(
            &self.state_read(),
            &test_cycle_id,
            test_execution_id.as_ref(),
        )
        .and_then(|target| self.installed_plugins().get(&target.source).cloned());
        let transaction = RunProvider {
            mode,
            test_cycle_id,
            test_execution_id,
            context: context.clone(),
            command_id: command_id.clone(),
            accepted_at: accepted_at.clone(),
            plugin_identity: artifact
                .as_ref()
                .and_then(|artifact| self.state_read().execution_identity(&artifact.source)),
        };
        let mut planned = (*self.state_read()).clone();
        let job = match crate::execution_runtime::provider::admission::transition(
            &mut planned,
            &transaction,
        ) {
            Ok(admission) => admission.job,
            Err(receipt) => return *receipt,
        };
        let receipt = crate::runtime::storage::requests::RunReceipt {
            node_id: node,
            summary: planned.run_requests[&command_id].summary.clone(),
            fingerprint,
            submitted_by: context.submitted_by.clone(),
            claimed: 0,
            last_claim: None,
            interruption: None,
            commands: Default::default(),
        };
        drop(planned);
        if let Err(error) = self.request_storage.begin(receipt).await {
            return rejected_receipt(
                command_id,
                &context,
                accepted_at,
                unavailable_rejection(error.to_string()),
            );
        }
        let before = self.state_manager.applied();
        match self
            .commit_keyed(
                RequestKey::new(format!("execution-request:{command_id}")),
                transaction,
            )
            .await
        {
            Ok(committed) => {
                job.submit(self.clone(), before);
                self.accepted_job_receipt(
                    command_id,
                    accepted_at,
                    job_id,
                    &context,
                    committed.frontier,
                    started,
                )
                .await
            }
            Err(error) => {
                let problem = error.to_string();
                let at = Timestamp::now_utc();
                if let Err(storage_error) = self
                    .request_storage
                    .update(&command_id, |receipt| {
                        receipt.summary.phase =
                            crate::transactions::provider::request::RunPhase::Interrupted;
                        receipt.summary.finished_at = Some(at);
                        receipt.interruption =
                            Some(format!("request did not enter runtime state: {problem}"));
                    })
                    .await
                {
                    tracing::error!(%storage_error, "failed to checkpoint rejected request");
                }
                rejected_receipt(
                    command_id,
                    &context,
                    Timestamp::now_utc(),
                    harmos_commit_rejection(error),
                )
            }
        }
    }

    async fn accepted_job_receipt(
        &self,
        command_id: CommandId,
        accepted_at: Timestamp,
        job_id: JobRunId,
        context: &CommandContext,
        position: Position,
        started: Instant,
    ) -> CommandReceipt {
        let mut receipt = accepted_receipt(command_id, context, accepted_at, Some(job_id));
        receipt.runtime_revision = position.0;
        receipt.duration_ms = Some(started.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
        self.reconcile_sampling().await;
        receipt
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn harmos_commit_rejection(error: HarmosError<String>) -> Error {
    match error {
        HarmosError::Rejected(message) => Error::conflict("runtime_conflict", message),
        HarmosError::Preparation(harmos::state::PreparationError::Limit) => Error::overload(
            "admission_resource_limit",
            "command exceeds the atomic admission record or payload budget",
        ),
        HarmosError::Unavailable => unavailable_rejection("Harmos state manager is unavailable"),
        other => internal_rejection(format!("commit command: {other}")),
    }
}

/* === Tests === */
