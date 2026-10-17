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
use std::sync::Arc;
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
use dokime_domain::model::execution::wire_cursor::Cursor as NativeCursor;
use dokime_domain::model::execution::wire_cursor::Cursor;
use dokime_domain::model::execution::wire_cursor::Error as CursorError;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::operations::prompts as wire;
use dokime_runtime::RuntimeHandle;
use dokime_runtime::runtime::query::RuntimeStateSnapshot;

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
        request: &mut wire::ListDefinitions,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::prompts::wire::ListPromptDefinitionsReply>, Status>
    {
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
        self.resources.produce(|| {
            let definitions: Vec<_> = selected.into_iter().map(canonical::identity).collect();
            let next_after_prompt_type = entries
                .next()
                .is_some()
                .then(|| definitions.last().unwrap().prompt_type.clone());
            Ok(
                dokime_domain::model::prompts::wire::ListPromptDefinitionsReply {
                    definitions,
                    next_after_prompt_type,
                    cursor: Some(values::cursor(&snapshot)),
                },
            )
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
        request: &mut wire::GetDefinition,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::prompts::wire::PromptDefinition>, Status> {
        value::text(&request.prompt_type)?;
        let snapshot = self.resources.runtime.watch_state().latest();
        let definition = snapshot
            .state()
            .prompt_definitions
            .values()
            .find(|d| d.prompt_type.as_str() == request.prompt_type)
            .ok_or_else(|| Status::not_found("registered prompt definition not found"))?;
        self.resources.produce(|| canonical::definition(definition))
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
        request: &mut wire::ListPending,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::prompts::wire::ListPendingPromptsReply>, Status>
    {
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
        let selection = dokime_runtime::prompts::pending_stream(cycle.as_ref(), execution.as_ref());
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
        let selected = |prompt: &&dokime_runtime::domains::prompts::PromptInstance| {
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
        self.resources.produce(|| {
            let prompts: Vec<_> = selected
                .into_iter()
                .map(|prompt| {
                    Ok(dokime_domain::model::prompts::wire::PendingPromptIdentity {
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
            Ok(
                dokime_domain::model::prompts::wire::ListPendingPromptsReply {
                    prompts,
                    next_after_prompt_id,
                    cursor: Some(cursor),
                },
            )
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
        request: &mut wire::GetPrompt,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::prompts::wire::PromptView>, Status> {
        let id = prompt_id(&request.prompt_id)?;
        let snapshot = self.resources.runtime.watch_state().latest();
        let prompt = snapshot
            .state()
            .prompt_requests
            .get(&id)
            .ok_or_else(|| Status::not_found("prompt not found"))?;
        let response = snapshot.state().prompt_responses.get(&id);
        self.resources.produce(|| {
            let response = response.map(canonical::response).transpose()?;
            Ok(dokime_domain::model::prompts::wire::PromptView {
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
        request: &mut wire::Respond,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::prompts::wire::PromptResponseReceipt>, Status> {
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
            Some(dokime_domain::model::values::Value::Object(fields)) => fields,
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
        self.resources.produce(|| {
            Ok(dokime_domain::model::prompts::wire::PromptResponseReceipt {
                command_id: receipt.command_id.to_string(),
                prompt_id: id.to_string(),
                committed_revision: receipt.runtime_revision,
                accepted_at: Some(values::timestamp(&receipt.accepted_at)?),
                observed_cursor: Some(values::cursor(&snapshot)),
            })
        })
    }
}

impl Resources {
    fn produce<T: MemorySize + Protobuf + Send + Sync + 'static>(
        &self,
        build: impl FnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.encoded_len() > self.maximum {
            return Err(Status::resource_exhausted(
                "prompt response exceeds wire budget",
            ));
        }
        Ok(output.into())
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
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<GetDefinition>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime
        .operations()
        .register_wire_operation::<ListPending>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            resources.clone(),
        )?;
    runtime.operations().register_wire_operation::<GetPrompt>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        resources.clone(),
    )?;
    runtime.operations().register_wire_operation::<Respond>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
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

/* === Tests === */
