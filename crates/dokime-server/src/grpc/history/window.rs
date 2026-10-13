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
use std::{num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use super::{inspection, wire};
use crate::grpc::{content::window, value};
use dokime_app::{
    ExecutionFileContent, RuntimeHandle, runtime::query::execution_history::ExecutionCheckpoint,
};
use dokime_contract::operations::history;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Read {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for Read {
    type Declaration = history::ReadCheckpointOutput;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut history::ReadCheckpointOutputRequest,
        _: &mut (),
    ) -> Result<Response<history::CheckpointFileWindow>, Status> {
        value::text(&request.relative_path)?;
        if request.relative_path.is_empty() {
            return Err(Status::invalid_argument("output name is required"));
        }
        if request.max_bytes > 1024 * 1024 {
            return Err(Status::invalid_argument("read limit exceeds 1 MiB"));
        }
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let limit = if request.max_bytes == 0 {
            64 * 1024
        } else {
            request.max_bytes as usize
        };
        let limit = limit.min(
            self.resources
                .maximum
                .saturating_sub(request.relative_path.len() + 512),
        );
        if limit == 0 {
            return Err(Status::resource_exhausted(
                "configured message limit cannot contain file window",
            ));
        }
        let view = self
            .resources
            .runtime
            .checkpoint_execution(&checkpoint)
            .await
            .map_err(wire::checkpoint_failure)?;
        let outputs = &view
            .execution()
            .saved
            .as_ref()
            .ok_or_else(|| Status::failed_precondition("execution is not sealed"))?
            .source
            .outputs;
        let output = outputs
            .iter()
            .find(|output| output.relative_path == request.relative_path)
            .ok_or_else(|| Status::not_found("checkpoint output not found"))?;
        value::text(&output.file.metadata.media_type)?;
        let _computation = view
            .view()
            .begin_computation()
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        let bytes = limit
            .saturating_add(request.relative_path.len())
            .saturating_add(output.file.metadata.media_type.len())
            .saturating_add(std::mem::size_of::<dokime_app::ExecutionFileContent>())
            .saturating_mul(2)
            .saturating_add(std::mem::size_of::<history::CheckpointFileWindow>());
        let preparation = self
            .resources
            .runtime
            .storage()
            .prepare_production(std::num::NonZeroUsize::new(bytes).unwrap())
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        let content_type = Some(output.file.metadata.media_type.clone());
        let mut reader = view
            .read_file(&output.file)
            .await
            .map_err(wire::checkpoint_failure)?;
        let mut data = Vec::with_capacity(limit);
        while let Some(chunk) = reader
            .next_chunk(std::num::NonZeroUsize::new(64 * 1024).unwrap())
            .await
            .map_err(|error| wire::checkpoint_failure(error.into()))?
        {
            let end = reader.offset();
            let start = end - chunk.as_bytes().len() as u64;
            if end <= request.offset {
                continue;
            }
            let begin = request.offset.saturating_sub(start) as usize;
            let available = &chunk.as_bytes()[begin..];
            let count = available.len().min(limit - data.len());
            data.extend_from_slice(&available[..count]);
            if data.len() == limit {
                break;
            }
        }
        let state = match output.stream_state {
            Some(dokime::execution::saved::SavedStreamState::Interrupted) => {
                dokime_app::ExecutionFileStreamState::Interrupted
            }
            _ => dokime_app::ExecutionFileStreamState::Finalized,
        };
        let content = preparation
            .finish(dokime_app::ExecutionFileContent {
                relative_path: request.relative_path.clone(),
                content_kind: dokime_app::ExecutionFileContentKind::from_metadata(
                    &request.relative_path,
                    content_type.as_deref(),
                ),
                content_type,
                stream_state: state,
                offset: request.offset.min(output.file.length),
                size_bytes: output.file.length,
                bytes: data,
            })
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        let content = content
            .try_into_value()
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        let response = view
            .view()
            .project_owned(
                content,
                view.view()
                    .decoded_bytes()
                    .map_err(|error| wire::checkpoint_failure(error.into()))?,
                |content, allowance| {
                    project(&checkpoint, content, allowance, self.resources.maximum)
                },
            )
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        response
            .try_into_response()
            .map_err(|source| wire::checkpoint_failure(source.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Read>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn project(
    checkpoint: &ExecutionCheckpoint,
    content: ExecutionFileContent,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<history::CheckpointFileWindow> {
    let required = std::mem::size_of::<history::CheckpointFileWindow>()
        .saturating_add(content.memory_size())
        .saturating_add(checkpoint.memory_size())
        .saturating_add(harmos::Message::encoded_len(&checkpoint.version));
    if required > allowance.get() {
        return Err(exhausted());
    }
    let checkpoint = inspection::project(checkpoint, allowance, maximum)?;
    let content = window::owned(content, allowance, maximum)?;
    let response = history::CheckpointFileWindow {
        checkpoint: Some(checkpoint),
        content: Some(content),
    };
    if response.encoded_len() > maximum {
        return Err(exhausted());
    }
    Ok(response)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical checkpoint file window",
    }
}

/* === Tests === */
