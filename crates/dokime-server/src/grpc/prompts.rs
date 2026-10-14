/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(in crate::grpc) mod canonical;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use super::{
    execution,
    value::{self, canonical as values},
};
use dokime::operations::cursor::{Cursor as NativeCursor, Error as CursorError};
use dokime_app::{
    RuntimeHandle,
    core::{PromptId, TestCycleId, TestExecutionId},
    runtime::query::RuntimeStateSnapshot,
};
use dokime_contract::operations::{cursor::Cursor, prompts as wire};

/* === Constants === */
const STREAM: &str = "runtime.state";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct ListDefinitions {
    resources: Arc<Resources>,
}
struct GetDefinition {
    resources: Arc<Resources>,
}
struct ListPending {
    resources: Arc<Resources>,
}
struct GetPrompt {
    resources: Arc<Resources>,
}
struct Respond {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for ListDefinitions {
    type Declaration = wire::ListDefinitions;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _context: Self::Context<'a>,
        request: &mut wire::ListPromptDefinitionsRequest,
        _: &mut (),
    ) -> Result<Response<wire::ListPromptDefinitionsReply>, Status> {
        let snapshot = self.resources.runtime.watch_state().latest();
        let limit = page(
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
        let selected: Vec<_> = entries.by_ref().take(limit).collect();
        let required = selected.iter().try_fold(
            std::mem::size_of::<wire::ListPromptDefinitionsReply>() + 256,
            |bytes, item| {
                value::text(item.prompt_type.as_str())?;
                value::text(&item.title)?;
                Ok::<_, Status>(
                    bytes
                        .saturating_add(4 * std::mem::size_of::<wire::PromptDefinitionIdentity>())
                        .saturating_add(item.prompt_type.as_str().len())
                        .saturating_add(item.title.len())
                        .saturating_add(item.prompt_type.as_str().len()),
                )
            },
        )?;
        self.resources.produce(required, || {
            let definitions: Vec<_> = selected.into_iter().map(canonical::identity).collect();
            let next_after_prompt_type = entries
                .next()
                .is_some()
                .then(|| definitions.last().unwrap().prompt_type.clone());
            Ok(wire::ListPromptDefinitionsReply {
                definitions,
                next_after_prompt_type,
                cursor: Some(values::cursor(&snapshot)),
            })
        })
    }
}
impl Operation for GetDefinition {
    type Declaration = wire::GetDefinition;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _context: Self::Context<'a>,
        request: &mut wire::GetPromptDefinitionRequest,
        _: &mut (),
    ) -> Result<Response<wire::PromptDefinition>, Status> {
        value::text(&request.prompt_type)?;
        let snapshot = self.resources.runtime.watch_state().latest();
        let definition = snapshot
            .state()
            .prompt_definitions
            .values()
            .find(|d| d.prompt_type.as_str() == request.prompt_type)
            .ok_or_else(|| Status::not_found("registered prompt definition not found"))?;
        self.resources
            .produce(canonical::definition_memory(definition)?, || {
                canonical::definition(definition)
            })
    }
}
impl Operation for ListPending {
    type Declaration = wire::ListPending;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _context: Self::Context<'a>,
        request: &mut wire::ListPendingPromptsRequest,
        _: &mut (),
    ) -> Result<Response<wire::ListPendingPromptsReply>, Status> {
        let cycle = request
            .cycle_id
            .take()
            .map(|id| {
                TestCycleId::from_uuid(id)
                    .map_err(|_| Status::invalid_argument("invalid cycle UUID"))
            })
            .transpose()?;
        let execution = request
            .execution_id
            .take()
            .map(|id| {
                TestExecutionId::from_uuid(id)
                    .map_err(|_| Status::invalid_argument("invalid execution UUID"))
            })
            .transpose()?;
        let selection = dokime::prompts::pending_stream(cycle.as_ref(), execution.as_ref());
        let snapshot = self.resources.runtime.watch_state().latest();
        let mut supplied = request.cursor.take();
        if let Some(cursor) = &mut supplied {
            if cursor.stream != selection {
                return Err(Status::failed_precondition(
                    "prompt cursor selection mismatch; restart pagination",
                ));
            }
            cursor.stream = STREAM.into();
        }
        let limit = page(
            request.limit,
            &request.after_prompt_id,
            supplied.as_ref(),
            &snapshot,
        )?;
        let selected = |prompt: &&dokime_app::capabilities::prompts::PromptInstance| {
            snapshot.state().prompt_open(prompt)
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
        let selected: Vec<_> = entries.by_ref().take(limit).collect();
        let required = selected.iter().try_fold(
            std::mem::size_of::<wire::ListPendingPromptsReply>() + 256,
            |bytes, prompt| {
                value::text(prompt.prompt_id.as_str())?;
                value::text(prompt.definition.prompt_type.as_str())?;
                value::text(&prompt.definition.title)?;
                Ok::<_, Status>(
                    bytes
                        .saturating_add(4 * std::mem::size_of::<wire::PendingPromptIdentity>())
                        .saturating_add(2 * prompt.prompt_id.as_str().len())
                        .saturating_add(prompt.definition.prompt_type.as_str().len())
                        .saturating_add(prompt.definition.title.len())
                        .saturating_add(values::scope_memory(&prompt.scope)?),
                )
            },
        )?;
        self.resources.produce(required, || {
            let prompts: Vec<_> = selected
                .into_iter()
                .map(|prompt| {
                    Ok(wire::PendingPromptIdentity {
                        prompt_id: prompt.prompt_id.to_string(),
                        definition: Some(canonical::identity(&prompt.definition)),
                        scope: Some(values::scope(&prompt.scope)?),
                        created_at: Some(values::timestamp(&prompt.created_at)?),
                    })
                })
                .collect::<Result<_, Status>>()?;
            let next_after_prompt_id = entries
                .next()
                .is_some()
                .then(|| prompts.last().unwrap().prompt_id.clone());
            let mut cursor = values::cursor(&snapshot);
            cursor.stream = selection;
            Ok(wire::ListPendingPromptsReply {
                prompts,
                next_after_prompt_id,
                cursor: Some(cursor),
            })
        })
    }
}
impl Operation for GetPrompt {
    type Declaration = wire::GetPrompt;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _context: Self::Context<'a>,
        request: &mut wire::GetPromptRequest,
        _: &mut (),
    ) -> Result<Response<wire::PromptView>, Status> {
        let id = prompt_id(&request.prompt_id)?;
        let snapshot = self.resources.runtime.watch_state().latest();
        let prompt = snapshot
            .state()
            .prompt_requests
            .get(&id)
            .ok_or_else(|| Status::not_found("prompt not found"))?;
        let response = snapshot.state().prompt_responses.get(&id);
        let required = std::mem::size_of::<wire::PromptView>()
            + 256
            + canonical::instance_memory(prompt)?
            + response
                .map(canonical::response_memory)
                .transpose()?
                .unwrap_or(0);
        self.resources.produce(required, || {
            let response = response.map(canonical::response).transpose()?;
            Ok(wire::PromptView {
                prompt: Some(canonical::instance(prompt)?),
                response,
                observed_cursor: Some(values::cursor(&snapshot)),
            })
        })
    }
}
impl Operation for Respond {
    type Declaration = wire::Respond;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut wire::RespondToPromptRequest,
        _: &mut (),
    ) -> Result<Response<wire::PromptResponseReceipt>, Status> {
        let context = execution::command_context(&context, request.context.as_ref())?;
        let responder = context
            .submitted_by
            .clone()
            .ok_or_else(|| Status::unauthenticated("authentication required"))?;

        let id = prompt_id(&request.prompt_id)?;
        value::text(&request.action)?;
        let decoded = request
            .values
            .take()
            .map(|value| values::decode(value, &mut value::Budget::new(), 0))
            .transpose()?;
        let values = match decoded {
            None => Default::default(),
            Some(dokime_app::core::Value::Object(fields)) => fields,
            Some(_) => return Err(Status::invalid_argument("prompt values must be an object")),
        };
        let receipt = self
            .resources
            .runtime
            .submit_prompt_response_command(
                id.clone(),
                std::mem::take(&mut request.action),
                responder,
                values,
                request.accepted,
                context,
            )
            .await
            .map_err(execution::status)?;
        let snapshot = self.resources.runtime.watch_state().latest();
        self.resources.produce(
            std::mem::size_of::<wire::PromptResponseReceipt>() + 512,
            || {
                Ok(wire::PromptResponseReceipt {
                    command_id: receipt.command_id.to_string(),
                    prompt_id: id.to_string(),
                    committed_revision: receipt.runtime_revision,
                    accepted_at: Some(values::timestamp(&receipt.accepted_at)?),
                    observed_cursor: Some(values::cursor(&snapshot)),
                })
            },
        )
    }
}

impl Resources {
    fn produce<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        required: usize,
        build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        if required > self.maximum {
            return Err(Status::resource_exhausted(
                "prompt response exceeds original budget",
            ));
        }
        let preparation = self
            .runtime
            .storage()
            .prepare_production(
                NonZeroUsize::new(required)
                    .ok_or_else(|| Status::resource_exhausted("invalid prompt budget"))?,
            )
            .map_err(failure)?;
        let output = build()?;
        if output.encoded_len() > self.maximum {
            return Err(Status::resource_exhausted(
                "prompt response exceeds wire budget",
            ));
        }
        preparation
            .finish(output)
            .map_err(failure)?
            .try_into_value()
            .map_err(failure)?
            .try_into_response()
            .map_err(failure)
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let resources = Arc::new(Resources {
        runtime: runtime.clone(),
        maximum,
    });
    runtime
        .operations()
        .register_wire_operation::<ListDefinitions>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<GetDefinition>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<ListPending>(
            "dokime",
            dokime_app::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime.operations().register_wire_operation::<GetPrompt>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<Respond>(
        "dokime",
        dokime_app::app::Lanes::Open,
        resources.clone(),
    )?;
    Ok(())
}

/* === Private Functions === */
fn page(
    limit: u32,
    after: &str,
    given: Option<&Cursor>,
    snapshot: &RuntimeStateSnapshot,
) -> Result<usize, Status> {
    let current = NativeCursor {
        epoch: snapshot.state().process_epoch().to_string(),
        stream: STREAM.into(),
        position: snapshot.revision(),
    };
    let given = given.map(|value| NativeCursor {
        epoch: value.epoch.clone(),
        stream: value.stream.clone(),
        position: value.position,
    });
    current.page(limit, after, given.as_ref()).map_err(|error| {
        let code = match error {
            CursorError::Selection => tonic::Code::FailedPrecondition,
            CursorError::Ahead => tonic::Code::OutOfRange,
            CursorError::Changed => tonic::Code::Aborted,
            CursorError::AfterTooLarge => tonic::Code::ResourceExhausted,
            CursorError::Limit | CursorError::Missing => tonic::Code::InvalidArgument,
        };
        Status::new(code, error.to_string())
    })
}
fn prompt_id(id: &str) -> Result<PromptId, Status> {
    PromptId::from_uuid(id.to_owned()).map_err(|_| Status::invalid_argument("invalid prompt UUID"))
}
fn failure(error: harmos::Error) -> Status {
    let status = if matches!(error, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("prompt admission failed")
    } else {
        Status::internal("prompt admission failed")
    };
    super::error::detailed(status, &error.into())
}

/* === Tests === */
