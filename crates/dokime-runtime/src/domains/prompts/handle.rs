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
use std::collections::BTreeMap;
use std::time::Instant;
// Third Party Imports

// Local Imports
use crate::records::RecordEntry;
use crate::transactions::prompts::{
    RequestPrompt as Request, SubmitPromptResponse as SubmitResponse, validate_request,
    validate_response,
};
use crate::{
    CommandContext, CommandId, CommandReceipt, runtime::Error, RuntimeHandle, commands::accepted_receipt,
    prompt_runtime::PromptDefinition, unavailable_rejection,
};
use crate::core::{PromptId, TestCycleId, Timestamp, Value};
use crate::prompts::{PromptInstance, PromptRequest, PromptResponse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn prompt_definitions(&self) -> Vec<PromptDefinition> {
        self.state_read().prompt_definitions()
    }

    pub async fn prompt_request_records(&self) -> crate::runtime::Result<Vec<RecordEntry>> {
        Ok(self
            .current_journal_records()
            .await?
            .into_iter()
            .filter(|record| record.record_type.as_str().starts_with("prompt.request."))
            .collect())
    }

    pub async fn prompt_response_records(&self) -> crate::runtime::Result<Vec<RecordEntry>> {
        Ok(self
            .current_journal_records()
            .await?
            .into_iter()
            .filter(|record| record.record_type.as_str().starts_with("prompt.response."))
            .collect())
    }

    pub async fn pending_prompt_projection(&self) -> Vec<PromptInstance> {
        self.state_read().pending_prompt_projection()
    }

    pub async fn request_prompt(
        &self,
        prompt: PromptRequest,
    ) -> std::result::Result<PromptInstance, Error> {
        let prompt_id = prompt.prompt_id.clone();
        let transaction = Request {
            prompt,
            command_id: CommandId::new(),
            accepted_at: Timestamp::now_utc(),
        };
        validate_request(&self.state_read(), &transaction)?;
        self.commit(transaction)
            .await
            .map_err(|error| unavailable_rejection(error.to_string()))?;
        // Read the durable request projection, not the pending set: a fast
        // responder can answer the prompt between
        // the commit above and this read, which removes it from the pending
        // projection — that is a completed round trip, not a missing
        // projection. Consecutive runs against one stack hit exactly that
        // window when an API poller answers immediately.
        self.state_read()
            .projected_prompt_request(&prompt_id)
            .ok_or_else(|| prompt_projection_missing("requested prompt was not projected"))
    }

    async fn precheck_prompt_condition(
        &self,
        prompt_id: &PromptId,
        action: &str,
        accepted: bool,
    ) -> std::result::Result<(), Error> {
        if !accepted {
            return Ok(());
        }
        let snapshot = self
            .capture_state()
            .await
            .map_err(|error| unavailable_rejection(error.to_string()))?;
        let condition = snapshot
            .state()
            .prompt_condition_to_check(prompt_id, action)
            .map_err(|message| {
                Error::validation("prompt_condition_unavailable", message, Vec::new())
            })?;
        let Some(condition) = condition else {
            return Ok(());
        };
        let records = self
            .records_at(&snapshot)
            .await
            .map_err(|error| unavailable_rejection(error.to_string()))?;
        let telemetry = self
            .collect_telemetry(&crate::evidence_runtime::TelemetrySelection::default())
            .await;
        let evaluation =
            crate::definition::evaluate_runtime_condition(&condition, &records, &telemetry);
        if evaluation.satisfied {
            return Ok(());
        }
        Err(Error::conflict(
            "prompt_condition_not_satisfied",
            evaluation
                .reason
                .unwrap_or_else(|| "prompt condition is not satisfied".to_owned()),
        ))
    }

    pub async fn submit_prompt_response(
        &self,
        prompt_id: PromptId,
        action: String,
        responder: String,
        values: BTreeMap<String, Value>,
        accepted: bool,
    ) -> std::result::Result<PromptResponse, Error> {
        self.precheck_prompt_condition(&prompt_id, &action, accepted)
            .await?;
        self.commit_prompt_response(
            prompt_id.clone(),
            action,
            responder,
            values,
            accepted,
            CommandContext::default(),
        )
        .await?;
        self.prompt_response(&prompt_id)
            .await
            .ok_or_else(|| prompt_projection_missing("submitted prompt response was not projected"))
    }

    pub async fn submit_prompt_response_command(
        &self,
        prompt_id: PromptId,
        action: String,
        responder: String,
        values: BTreeMap<String, Value>,
        accepted: bool,
        context: CommandContext,
    ) -> std::result::Result<CommandReceipt, Error> {
        self.precheck_prompt_condition(&prompt_id, &action, accepted)
            .await?;
        self.commit_prompt_response(prompt_id, action, responder, values, accepted, context)
            .await
    }

    pub async fn prompt_response(&self, prompt_id: &PromptId) -> Option<PromptResponse> {
        self.state_read().prompt_response(prompt_id)
    }

    /// When the runtime settled the prompt because its owner ended, if it did.
    pub async fn prompt_settled_at(&self, prompt_id: &PromptId) -> Option<Timestamp> {
        self.state_read().closed_prompts.get(prompt_id).cloned()
    }

    pub async fn wait_for_prompt_response(
        &self,
        prompt_id: PromptId,
    ) -> std::result::Result<PromptResponse, Error> {
        let mut state = self.watch_state();
        let snapshot = state
            .wait_until(|snapshot| {
                snapshot.state().shutting_down
                    || snapshot.state().prompt_response(&prompt_id).is_some()
            })
            .await;

        snapshot
            .and_then(|snapshot| snapshot.state().prompt_response(&prompt_id))
            .ok_or_else(|| {
                prompt_wait_cancelled("runtime cancellation requested while waiting for prompt")
            })
    }

    /// Waits for the prompt's response. A stop settles the prompt in the same
    /// transaction without answering it, so the wait ends as cancelled.
    pub async fn wait_for_prompt_response_for_cycle(
        &self,
        prompt_id: PromptId,
        test_cycle_id: TestCycleId,
    ) -> std::result::Result<PromptResponse, Error> {
        let mut state = self.watch_state();
        let snapshot = state
            .wait_until(|snapshot| {
                let state = snapshot.state();
                state.shutting_down
                    || state.prompt_response(&prompt_id).is_some()
                    || !state.cycles.contains_key(&test_cycle_id)
                    || state
                        .prompt_requests
                        .get(&prompt_id)
                        .is_none_or(|prompt| !state.prompt_open(prompt))
            })
            .await
            .ok_or_else(|| {
                prompt_wait_cancelled("runtime cancellation requested while waiting for prompt")
            })?;

        snapshot
            .state()
            .prompt_response(&prompt_id)
            .ok_or_else(|| prompt_wait_cancelled("prompt closed while waiting for its response"))
    }

    async fn commit_prompt_response(
        &self,
        prompt_id: PromptId,
        action: String,
        responder: String,
        values: BTreeMap<String, Value>,
        accepted: bool,
        context: CommandContext,
    ) -> std::result::Result<CommandReceipt, Error> {
        let started = Instant::now();
        let command_id = context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())
            .map(CommandId::from_seed)
            .unwrap_or_default();
        let accepted_at = Timestamp::now_utc();
        let transaction = SubmitResponse {
            prompt_id,
            action,
            responder,
            values,
            accepted,
            command_id: command_id.clone(),
            accepted_at: accepted_at.clone(),
        };
        let prompt_id = validate_response(&self.state_read(), &transaction)?;
        let committed = match context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())
        {
            Some(key) => {
                self.commit_keyed(harmos::state::RequestKey::new(key), transaction)
                    .await
            }
            None => self.commit(transaction).await,
        }
        .map_err(|error| unavailable_rejection(error.to_string()))?;
        let mut receipt = accepted_receipt(command_id, &context, accepted_at, None);
        receipt.prompt_id = Some(prompt_id);
        receipt.runtime_revision = committed.frontier.0;
        receipt.duration_ms = Some(started.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
        ensure_accepted(&receipt)?;
        Ok(receipt)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn ensure_accepted(receipt: &CommandReceipt) -> std::result::Result<(), Error> {
    match &receipt.rejection {
        Some(rejection) => Err(Error::internal(
            rejection.code.clone(),
            rejection.message.clone(),
        )),
        None => Ok(()),
    }
}

fn prompt_projection_missing(message: impl Into<String>) -> Error {
    Error::unavailable("prompt_projection_unavailable", message)
}

fn prompt_wait_cancelled(message: impl Into<String>) -> Error {
    Error::cancelled("prompt_wait_cancelled", message)
}

/* === Tests === */
