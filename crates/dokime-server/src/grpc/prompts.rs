/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod wire;

/* === Re-Exports === */
pub(crate) use wire::{
    instance as historical_instance, instance_memory as historical_instance_memory,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::{Request, Response, Status};
// Local Imports
use super::{execution, observation, value};
use dokime_app::{
    RuntimeHandle,
    core::{PromptId, TestCycleId, TestExecutionId},
};
use dokime_protocol::v1::{self, prompts_server::Prompts};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Prompts for Service {
    async fn list_definitions(
        &self,
        request: Request<v1::ListPromptDefinitionsRequest>,
    ) -> Result<Response<v1::ListPromptDefinitionsReply>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let limit = observation::page(
            request.limit,
            &request.after_prompt_type,
            request.cursor.as_ref(),
            &snapshot,
        )?;
        let definitions = &snapshot.state().prompt_definitions;
        if !request.after_prompt_type.is_empty()
            && !definitions
                .values()
                .any(|d| d.prompt_type.as_str() == request.after_prompt_type)
        {
            return Err(Status::invalid_argument("after prompt type not found"));
        }
        let mut entries = definitions
            .values()
            .filter(|d| d.prompt_type.as_str() > request.after_prompt_type.as_str());
        let definitions: Vec<_> = entries.by_ref().take(limit).map(wire::identity).collect();
        let next_after_prompt_type = entries
            .next()
            .is_some()
            .then(|| definitions.last().unwrap().prompt_type.clone());
        Ok(Response::new(v1::ListPromptDefinitionsReply {
            definitions,
            next_after_prompt_type,
            cursor: Some(observation::cursor(&snapshot)),
        }))
    }
    async fn get_definition(
        &self,
        request: Request<v1::GetPromptDefinitionRequest>,
    ) -> Result<Response<v1::PromptDefinition>, Status> {
        value::text(&request.get_ref().prompt_type)?;
        let snapshot = self.runtime.watch_state().latest();
        let definition = snapshot
            .state()
            .prompt_definitions
            .values()
            .find(|d| d.prompt_type.as_str() == request.get_ref().prompt_type)
            .ok_or_else(|| Status::not_found("registered prompt definition not found"))?;
        Ok(Response::new(wire::definition(definition)?))
    }
    async fn list_pending(
        &self,
        request: Request<v1::ListPendingPromptsRequest>,
    ) -> Result<Response<v1::ListPendingPromptsReply>, Status> {
        let request = request.into_inner();
        let cycle = request
            .cycle_id
            .map(|id| {
                TestCycleId::from_uuid(id)
                    .map_err(|_| Status::invalid_argument("invalid cycle UUID"))
            })
            .transpose()?;
        let execution = request
            .execution_id
            .map(|id| {
                TestExecutionId::from_uuid(id)
                    .map_err(|_| Status::invalid_argument("invalid execution UUID"))
            })
            .transpose()?;
        let selection = dokime::prompts::pending_stream(cycle.as_ref(), execution.as_ref());
        let snapshot = self.runtime.watch_state().latest();
        let mut supplied = request.cursor;
        if let Some(cursor) = &mut supplied {
            if cursor.stream != selection {
                return Err(Status::failed_precondition(
                    "prompt cursor selection mismatch; restart pagination",
                ));
            }
            cursor.stream = observation::STREAM.into();
        }
        let limit = observation::page(
            request.limit,
            &request.after_prompt_id,
            supplied.as_ref(),
            &snapshot,
        )?;
        let selected = |prompt: &&dokime_app::capabilities::prompts::PromptInstance| {
            !snapshot
                .state()
                .prompt_responses
                .contains_key(&prompt.prompt_id)
                && cycle
                    .as_ref()
                    .is_none_or(|id| prompt.scope.test_cycle_id.as_ref() == Some(id))
                && execution
                    .as_ref()
                    .is_none_or(|id| prompt.scope.test_execution_id.as_ref() == Some(id))
        };
        if !request.after_prompt_id.is_empty()
            && !snapshot
                .state()
                .prompt_requests
                .values()
                .filter(selected)
                .any(|p| p.prompt_id.as_str() == request.after_prompt_id)
        {
            return Err(Status::invalid_argument(
                "after prompt ID not found in selection",
            ));
        }
        let mut entries = snapshot
            .state()
            .prompt_requests
            .values()
            .filter(selected)
            .filter(|p| p.prompt_id.as_str() > request.after_prompt_id.as_str());
        let prompts: Vec<_> = entries
            .by_ref()
            .take(limit)
            .map(|prompt| {
                Ok(v1::PendingPromptIdentity {
                    prompt_id: prompt.prompt_id.to_string(),
                    definition: Some(wire::identity(&prompt.definition)),
                    scope: Some(value::scope(&prompt.scope)),
                    created_at: Some(value::timestamp(&prompt.created_at)?),
                })
            })
            .collect::<Result<_, Status>>()?;
        let next_after_prompt_id = entries
            .next()
            .is_some()
            .then(|| prompts.last().unwrap().prompt_id.clone());
        let mut cursor = observation::cursor(&snapshot);
        cursor.stream = selection;
        Ok(Response::new(v1::ListPendingPromptsReply {
            prompts,
            next_after_prompt_id,
            cursor: Some(cursor),
        }))
    }
    async fn get_prompt(
        &self,
        request: Request<v1::GetPromptRequest>,
    ) -> Result<Response<v1::PromptView>, Status> {
        let id = prompt_id(&request.get_ref().prompt_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let prompt = snapshot
            .state()
            .prompt_requests
            .get(&id)
            .ok_or_else(|| Status::not_found("prompt not found"))?;
        let response = snapshot
            .state()
            .prompt_responses
            .get(&id)
            .map(wire::response)
            .transpose()?;
        Ok(Response::new(v1::PromptView {
            prompt: Some(wire::instance(prompt)?),
            response,
            observed_cursor: Some(observation::cursor(&snapshot)),
        }))
    }
    async fn respond(
        &self,
        request: Request<v1::RespondToPromptRequest>,
    ) -> Result<Response<v1::PromptResponseReceipt>, Status> {
        let context = execution::context(&request, request.get_ref().context.as_ref())?;
        let responder = context.submitted_by.clone().expect("authenticated context");
        let request = request.into_inner();
        let id = prompt_id(&request.prompt_id)?;
        value::text(&request.action)?;
        let values = value::decode_object(request.values)?;
        let receipt = self
            .runtime
            .submit_prompt_response_command(
                id.clone(),
                request.action,
                responder,
                values,
                request.accepted,
                context,
            )
            .await
            .map_err(execution::status)?;
        let snapshot = self.runtime.watch_state().latest();
        Ok(Response::new(v1::PromptResponseReceipt {
            command_id: receipt.command_id.to_string(),
            prompt_id: id.to_string(),
            committed_revision: receipt.runtime_revision,
            accepted_at: Some(value::timestamp(&receipt.accepted_at)?),
            observed_cursor: Some(observation::cursor(&snapshot)),
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn prompt_id(id: &str) -> Result<PromptId, Status> {
    PromptId::from_uuid(id.to_owned()).map_err(|_| Status::invalid_argument("invalid prompt UUID"))
}

/* === Tests === */
