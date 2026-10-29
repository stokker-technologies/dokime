/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(in crate::grpc) mod canonical;

use crate::grpc::transcode::Transcode;

use std::sync::Arc;

use dokime::harmos::domain::Portable;
use dokime::harmos::{Context, Response};
use tonic::Status;

use super::{
    execution,
    value::{self, canonical as values},
};
use dokime::app::LiveRuntime;
use dokime::domain::model::execution::wire_cursor::Cursor as NativeCursor;
use dokime::domain::model::execution::wire_cursor::Cursor;
use dokime::domain::model::execution::wire_cursor::Error as CursorError;
use dokime::domain::model::identity::PromptId;
use dokime::domain::model::identity::TestCycleId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::prompts as wire;
use dokime::runtime::query::RuntimeStateSnapshot;

const STREAM: &str = "runtime.state";

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::ListDefinitions),
    Operation1(wire::GetDefinition),
    Operation2(wire::ListPending),
    Operation3(wire::GetPrompt),
    Operation4(wire::Respond),
}

impl Serve<wire::ListDefinitions> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListDefinitions, _: Context<'a, wire::ListDefinitions>,
    ) -> Served<'a, dokime::domain::model::prompts::wire::ListPromptDefinitionsReply> {
        Box::pin(async move {
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            let limit = page(request.limit, &request.after_prompt_type, request.cursor.as_ref(), &snapshot)?;
            let definitions = &snapshot.state().prompt_definitions;
            if !request.after_prompt_type.is_empty() && !definitions.values().any(|d| d.prompt_type.as_str() == request.after_prompt_type) {
                return Err(Status::invalid_argument("after prompt type not found"));
            }
            let mut entries = definitions.values().filter(|d| d.prompt_type.as_str() > request.after_prompt_type.as_str());
            let selected: Vec<_> = entries.by_ref().take(limit).collect();
            self.produce(|| {
                let definitions: Vec<_> = selected.into_iter().map(canonical::identity).collect();
                let next_after_prompt_type = entries.next().is_some().then(|| definitions.last().unwrap().prompt_type.clone());
                Ok(dokime::domain::model::prompts::wire::ListPromptDefinitionsReply {
                    definitions,
                    next_after_prompt_type,
                    cursor: Some(values::cursor(&snapshot)),
                })
            })
        })
    }
}

impl Serve<wire::GetDefinition> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetDefinition, _: Context<'a, wire::GetDefinition>,
    ) -> Served<'a, dokime::domain::model::prompts::wire::PromptDefinition> {
        Box::pin(async move {
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            let definition = snapshot
                .state()
                .prompt_definitions
                .values()
                .find(|d| d.prompt_type.as_str() == request.prompt_type)
                .ok_or_else(|| Status::not_found("registered prompt definition not found"))?;
            self.produce(|| canonical::definition(definition))
        })
    }
}

impl Serve<wire::ListPending> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::ListPending, _: Context<'a, wire::ListPending>,
    ) -> Served<'a, dokime::domain::model::prompts::wire::ListPendingPromptsReply> {
        Box::pin(async move {
            let cycle = request
                .cycle_id
                .take()
                .map(|value| TestCycleId::from_uuid(value).map_err(|_| Status::invalid_argument("invalid cycle UUID")))
                .transpose()?;
            let execution = request
                .execution_id
                .take()
                .map(|value| TestExecutionId::from_uuid(value).map_err(|_| Status::invalid_argument("invalid execution UUID")))
                .transpose()?;
            let selection = dokime::prompts::pending_stream(cycle.as_ref(), execution.as_ref());
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            let mut supplied = request.cursor.take();
            if let Some(cursor) = &mut supplied {
                if cursor.stream != selection {
                    return Err(Status::failed_precondition("prompt cursor selection mismatch; restart pagination"));
                }
                cursor.stream = STREAM.into();
            }
            let limit = page(request.limit, &request.after_prompt_id, supplied.as_ref(), &snapshot)?;
            let selected = |prompt: &&dokime::domains::prompts::PromptInstance| {
                snapshot.state().prompt_open(prompt)
                    && cycle.as_ref().is_none_or(|id| prompt.scope.test_cycle_id.as_ref() == Some(id))
                    && execution.as_ref().is_none_or(|id| prompt.scope.test_execution_id.as_ref() == Some(id))
            };
            if !request.after_prompt_id.is_empty()
                && !snapshot.state().prompt_requests.values().filter(selected).any(|p| p.prompt_id.as_str() == request.after_prompt_id)
            {
                return Err(Status::invalid_argument("after prompt ID not found in selection"));
            }
            let mut entries = snapshot
                .state()
                .prompt_requests
                .values()
                .filter(selected)
                .filter(|p| p.prompt_id.as_str() > request.after_prompt_id.as_str());
            let selected: Vec<_> = entries.by_ref().take(limit).collect();
            self.produce(|| {
                let prompts: Vec<_> = selected
                    .into_iter()
                    .map(|prompt| {
                        Ok(dokime::domain::model::prompts::wire::PendingPromptIdentity {
                            prompt_id: prompt.prompt_id.to_string(),
                            definition: Some(canonical::identity(&prompt.definition)),
                            scope: Some(values::scope(&prompt.scope)?),
                            created_at: Some(values::timestamp(&prompt.created_at)?),
                        })
                    })
                    .collect::<Result<_, Status>>()?;
                let next_after_prompt_id = entries.next().is_some().then(|| prompts.last().unwrap().prompt_id.clone());
                let mut cursor = values::cursor(&snapshot);
                cursor.stream = selection;
                Ok(dokime::domain::model::prompts::wire::ListPendingPromptsReply {
                    prompts,
                    next_after_prompt_id,
                    cursor: Some(cursor),
                })
            })
        })
    }
}

impl Serve<wire::GetPrompt> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetPrompt, _: Context<'a, wire::GetPrompt>,
    ) -> Served<'a, dokime::domain::model::prompts::wire::PromptView> {
        Box::pin(async move {
            let id = prompt_id(&request.prompt_id)?;
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            let prompt = snapshot.state().prompt_requests.get(&id).ok_or_else(|| Status::not_found("prompt not found"))?;
            let response = snapshot.state().prompt_responses.get(&id);
            self.produce(|| {
                let response = response.map(canonical::response).transpose()?;
                Ok(dokime::domain::model::prompts::wire::PromptView {
                    prompt: Some(canonical::instance(prompt)?),
                    response,
                    observed_cursor: Some(values::cursor(&snapshot)),
                })
            })
        })
    }
}

impl Serve<wire::Respond> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::Respond, ctx: Context<'a, wire::Respond>,
    ) -> Served<'a, dokime::domain::model::prompts::wire::PromptResponseReceipt> {
        Box::pin(async move {
            let context = ctx.call();
            let context = execution::command_context(context, request.context.as_ref())?;
            let responder = context.submitted_by.clone().ok_or_else(|| Status::unauthenticated("authentication required"))?;

            let id = prompt_id(&request.prompt_id)?;
            let mut budget = value::Budget::new()?;
            let decoded = request.values.take().map(|value| values::decode(value, &mut budget, 0)).transpose()?;
            let values = match decoded {
                None => Default::default(),
                Some(dokime::domain::model::values::Value::Object(fields)) => fields,
                Some(_) => return Err(Status::invalid_argument("prompt values must be an object")),
            };
            let receipt = dokime::app::live_runtime(&self.runtime)
                .await?
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
            let snapshot = dokime::app::live_runtime(&self.runtime).await?.watch_state().latest();
            self.produce(|| {
                Ok(dokime::domain::model::prompts::wire::PromptResponseReceipt {
                    command_id: receipt.command_id.to_string(),
                    prompt_id: id.to_string(),
                    committed_revision: receipt.runtime_revision,
                    accepted_at: Some(values::timestamp(&receipt.accepted_at)?),
                    observed_cursor: Some(values::cursor(&snapshot)),
                })
            })
        })
    }
}

impl Resources {
    fn produce<T: Portable + Send + Sync + 'static>(&self, build: impl FnOnce() -> Result<T, Status>) -> Result<Response<T>, Status> {
        let output = build()?;
        if output.wire_len()? > self.maximum {
            return Err(Status::resource_exhausted("prompt response exceeds wire budget"));
        }
        Ok(output.into())
    }
}

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<wire::ListDefinitions>>(Host::<wire::ListDefinitions>::new(served.clone()));
    setup.resource::<Host<wire::GetDefinition>>(Host::<wire::GetDefinition>::new(served.clone()));
    setup.resource::<Host<wire::ListPending>>(Host::<wire::ListPending>::new(served.clone()));
    setup.resource::<Host<wire::GetPrompt>>(Host::<wire::GetPrompt>::new(served.clone()));
    setup.resource::<Host<wire::Respond>>(Host::<wire::Respond>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

fn page(limit: u32, after: &str, given: Option<&Cursor>, snapshot: &RuntimeStateSnapshot) -> Result<usize, Status> {
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
