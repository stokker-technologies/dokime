/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::time::Instant;

use harmos::{Error as HarmosError, state::Position, state::RequestKey};

use crate::{
    CommandContext, CommandId, CommandReceipt, JobRunId, RuntimeHandle, commands::accepted_receipt,
    execution_runtime::provider::admission::RunProvider, internal_rejection, rejected_receipt, runtime::Error, unavailable_rejection,
};
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

impl RuntimeHandle {
    fn provider_availability_rejection(
        &self, test_cycle_id: &TestCycleId, test_execution_id: Option<&TestExecutionId>, context: &CommandContext,
    ) -> Option<CommandReceipt> {
        let state = self.state_read();
        // A cycle run dispatches one provider run per target, so every target
        // the selection resolves to has to have its executor.
        let targets = crate::execution_runtime::provider::provider_targets_for_cycle(&state, test_cycle_id, test_execution_id);
        if targets.is_empty() {
            return None;
        }
        if targets.iter().all(|target| self.plugin_available(&target.source) && self.providers.get(target).is_some()) {
            None
        } else {
            Some(rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                context,
                crate::model::time::now_utc(),
                unavailable_rejection("provider executor is unavailable"),
            ))
        }
    }

    pub async fn start_execution(
        &self, test_execution_id: TestExecutionId, mode: dokime_domain::model::execution::ExecutionMode, mut context: CommandContext,
    ) -> CommandReceipt {
        // Admission Identity Belongs To The Attempt, Independent Of Caller Or Listener.
        context.idempotency_key = None;
        let execution = match self.known_execution(&test_execution_id).await {
            Ok(execution) => execution,
            Err(error) => {
                return rejected_receipt(
                    self.execution_start_id(&test_execution_id),
                    &context,
                    crate::model::time::now_utc(),
                    unavailable_rejection(error.to_string()),
                );
            }
        };
        let Some(execution) = execution else {
            return rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                &context,
                crate::model::time::now_utc(),
                Error::not_found("execution_not_found", "execution not found"),
            );
        };
        self.owned_provider(execution.test_cycle_id, test_execution_id, mode, context).await
    }

    fn execution_start_id(&self, id: &TestExecutionId) -> CommandId {
        let node = self.state_read().identity.node.node_id.clone();
        CommandId::from_seed(format!("execution-start:{node}:{id}"))
    }

    pub(crate) async fn known_execution(
        &self, id: &TestExecutionId,
    ) -> Result<Option<dokime_domain::model::execution::result::TestExecution>, Error> {
        let live = self.state_read().executions.get(id).cloned();
        if live.is_some() {
            return Ok(live);
        }
        let saved =
            crate::runtime::storage::execution::ExecutionView::open(self.storage(), id, harmos::storage::ObjectVersion::Latest).await?;
        Ok(saved.map(|view| view.execution().clone()))
    }

    async fn owned_provider(
        &self, cycle: TestCycleId, execution: TestExecutionId, mode: dokime_domain::model::execution::ExecutionMode,
        context: CommandContext,
    ) -> CommandReceipt {
        if self.is_shutting_down() {
            return rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                &context,
                crate::model::time::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        }
        let Some(permit) = self.admit_owned_work() else {
            return rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                &context,
                crate::model::time::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        };
        if self.is_shutting_down() {
            return rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                &context,
                crate::model::time::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        }
        let runtime = self.clone();
        let fallback = context.clone();
        tokio::spawn(async move {
            // The owned task retains capacity even if its caller disconnects.
            let _permit = permit;
            runtime.submit_provider(cycle, execution, mode, context).await
        })
        .await
        .unwrap_or_else(|error| {
            rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                &fallback,
                crate::model::time::now_utc(),
                internal_rejection(format!("request admission task failed: {error}")),
            )
        })
    }

    async fn submit_provider(
        &self, test_cycle_id: TestCycleId, test_execution_id: TestExecutionId, mode: dokime_domain::model::execution::ExecutionMode,
        context: CommandContext,
    ) -> CommandReceipt {
        let _plugin_admission = self.plugin_admission.lock().await;
        self.submit_provider_admitted(test_cycle_id, test_execution_id, mode, context).await
    }

    pub(crate) async fn submit_provider_admitted(
        &self, test_cycle_id: TestCycleId, test_execution_id: TestExecutionId, mode: dokime_domain::model::execution::ExecutionMode,
        context: CommandContext,
    ) -> CommandReceipt {
        if self.is_shutting_down() {
            return rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                &context,
                crate::model::time::now_utc(),
                unavailable_rejection("runtime is shutting down"),
            );
        }
        let started = Instant::now();
        let node = self.state_read().identity.node.node_id.clone();
        let command_id = self.execution_start_id(&test_execution_id);
        let test_execution_id = Some(test_execution_id);
        use sha2::Digest;
        let encoded = serde_json::to_vec(&(&test_cycle_id, &test_execution_id, mode)).expect("request identity encodes");
        let fingerprint: String = sha2::Sha256::digest(encoded).iter().map(|byte| format!("{byte:02x}")).collect();
        match self.request_storage.get(&command_id).await {
            Ok(Some(receipt)) => {
                if receipt.fingerprint != fingerprint {
                    return rejected_receipt(
                        command_id,
                        &context,
                        crate::model::time::now_utc(),
                        Error::conflict("request_identity_conflict", "request identity already belongs to another run selection"),
                    );
                }
                // A Prepared Receipt Is Not Proof That Runtime Admission Succeeded.
                let interrupted_before_dispatch = receipt.summary.phase == crate::transactions::provider::request::RunPhase::Interrupted
                    && receipt.summary.provider_started_at.is_none();
                let sealed = if let Some(id) = &test_execution_id {
                    self.known_execution(id).await.ok().flatten().is_some_and(|execution| execution.saved.is_some())
                } else {
                    false
                };
                if interrupted_before_dispatch && !sealed {
                    let problem = receipt.interruption.clone().unwrap_or_else(|| "execution admission was interrupted".into());
                    return rejected_receipt(command_id, &context, crate::model::time::now_utc(), unavailable_rejection(problem));
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
                return rejected_receipt(command_id, &context, crate::model::time::now_utc(), unavailable_rejection(error.to_string()));
            }
        }
        if let Some(id) = &test_execution_id {
            match self.known_execution(id).await {
                Ok(Some(execution)) if execution.started_at.is_some() || execution.finished_at.is_some() || execution.saved.is_some() => {
                    if execution.started_at.is_some() && execution.mode != mode {
                        return rejected_receipt(
                            command_id,
                            &context,
                            crate::model::time::now_utc(),
                            Error::conflict("start_options_conflict", "execution was started with different options"),
                        );
                    }
                    let at = execution.started_at.or(execution.finished_at).unwrap_or_else(crate::model::time::now_utc);
                    let mut existing = accepted_receipt(command_id, &context, at, None);
                    existing.runtime_revision = self.state_manager.applied().0;
                    return existing;
                }
                Ok(Some(_)) => (),
                Ok(None) => {
                    return rejected_receipt(
                        command_id,
                        &context,
                        crate::model::time::now_utc(),
                        Error::not_found("execution_not_found", "execution not found"),
                    );
                }
                Err(error) => {
                    return rejected_receipt(command_id, &context, crate::model::time::now_utc(), unavailable_rejection(error.to_string()));
                }
            }
        }
        let contracts = self.state_read().telemetry_contracts();
        if let Err(error) = self.execution_storage.register_schemas(&contracts).await {
            return rejected_receipt(command_id.clone(), &context, crate::model::time::now_utc(), unavailable_rejection(error.to_string()));
        }
        // Provider admission runs against the published plan, so a cycle whose
        // projection admission deferred has to be published first. Everything
        // after this point assumes its executions are durable.
        if let Err(error) = self.execution_storage.save_cycle(test_cycle_id.clone()).await {
            return rejected_receipt(
                crate::model::identity::new_id::<CommandId>(),
                &context,
                crate::model::time::now_utc(),
                unavailable_rejection(error.to_string()),
            );
        }
        if let Some(receipt) = self.provider_availability_rejection(&test_cycle_id, test_execution_id.as_ref(), &context) {
            return receipt;
        }
        let accepted_at = crate::model::time::now_utc();
        let job_id = JobRunId::from_seed(command_id.as_str());
        let transaction = RunProvider {
            mode,
            test_cycle_id,
            test_execution_id,
            context: context.clone(),
            command_id: command_id.clone(),
            accepted_at: accepted_at.clone(),
        };
        let planned = crate::execution_runtime::provider::admission::plan(&self.state_read(), &transaction);
        let (job, summary) = match planned {
            Ok((admission, summary)) => (admission.job, summary),
            Err(receipt) => return *receipt,
        };
        let receipt = crate::runtime::storage::requests::RunReceipt {
            node_id: node,
            summary,
            fingerprint,
            submitted_by: context.submitted_by.clone(),
            claimed: 0,
            last_claim: None,
            interruption: None,
            commands: Default::default(),
        };
        if let Err(error) = self.request_storage.begin(receipt).await {
            return rejected_receipt(command_id, &context, accepted_at, unavailable_rejection(error.to_string()));
        }
        match self.commit_keyed(RequestKey::new(format!("execution-request:{command_id}")), transaction).await {
            Ok(committed) => {
                if self.state_read().run_queue.front() == Some(&command_id) {
                    job.submit(self.clone());
                }
                self.accepted_job_receipt(command_id, accepted_at, job_id, &context, committed.frontier, started).await
            }
            Err(error) => {
                let problem = error.to_string();
                let at = crate::model::time::now_utc();
                let interruption = format!("request did not enter runtime state: {problem}");

                if let Err(storage_error) = self
                    .request_storage
                    .update(&command_id, |receipt| {
                        receipt.summary.phase = crate::transactions::provider::request::RunPhase::Interrupted;
                        receipt.summary.finished_at = Some(at);
                        receipt.interruption = Some(interruption);
                    })
                    .await
                {
                    tracing::error!(%storage_error, "failed to checkpoint rejected request");
                }
                rejected_receipt(command_id, &context, crate::model::time::now_utc(), harmos_commit_rejection(error))
            }
        }
    }

    async fn accepted_job_receipt(
        &self, command_id: CommandId, accepted_at: Timestamp, job_id: JobRunId, context: &CommandContext, position: Position,
        started: Instant,
    ) -> CommandReceipt {
        let mut receipt = accepted_receipt(command_id, context, accepted_at, Some(job_id));
        receipt.runtime_revision = position.0;
        receipt.duration_ms = Some(started.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
        self.reconcile_sampling().await;
        receipt
    }
}

fn harmos_commit_rejection(error: HarmosError<String>) -> Error {
    match error {
        HarmosError::Rejected(message) => Error::conflict("runtime_conflict", message),
        HarmosError::Preparation(harmos::state::PreparationError::Limit) => {
            Error::overload("admission_resource_limit", "command exceeds the atomic admission record or payload budget")
        }
        HarmosError::Unavailable => unavailable_rejection("Harmos state manager is unavailable"),
        other => internal_rejection(format!("commit command: {other}")),
    }
}
