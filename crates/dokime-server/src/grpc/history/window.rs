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
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::{inspection, wire};
use crate::grpc::content::window;
use crate::grpc::transcode::Transcode;
use dokime::domain::operations::history;
use dokime::{ExecutionFileContent, RuntimeHandle};

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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut history::ReadCheckpointOutput, _: &mut (),
    ) -> Result<Response<dokime::domain::model::history::wire::CheckpointFileWindow>, Status> {
        if request.relative_path.is_empty() {
            return Err(Status::invalid_argument("output name is required"));
        }
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let limit = if request.max_bytes == 0 { self.resources.maximum } else { request.max_bytes as usize };
        let limit = limit.min(self.resources.maximum);
        if limit == 0 {
            return Err(Status::resource_exhausted("configured message limit cannot contain file window"));
        }
        let view = self.resources.runtime.checkpoint_execution(&checkpoint).await.map_err(wire::checkpoint_failure)?;
        let outputs =
            &view.execution().saved.as_ref().ok_or_else(|| Status::failed_precondition("execution is not sealed"))?.source.outputs;
        let output = outputs
            .iter()
            .find(|output| output.relative_path == request.relative_path)
            .ok_or_else(|| Status::not_found("checkpoint output not found"))?;
        let offset = request.offset.min(output.file.length);
        let length = usize::try_from((output.file.length - offset).min(limit as u64))
            .map_err(|_| Status::resource_exhausted("checkpoint file extent overflows"))?;
        let maximum = self.resources.maximum;
        let template_content = ExecutionFileContent {
            relative_path: output.relative_path.clone(),
            content_type: Some(output.file.metadata.media_type.clone()),
            content_kind: dokime::ExecutionFileContentKind::from_metadata(&output.relative_path, Some(&output.file.metadata.media_type)),
            stream_state: match output.stream_state {
                Some(dokime::domain::model::execution::saved::SavedStreamState::Interrupted) => {
                    dokime::ExecutionFileStreamState::Interrupted
                }
                _ => dokime::ExecutionFileStreamState::Finalized,
            },
            offset,
            size_bytes: output.file.length,
            bytes: Vec::new(),
        };
        let template = dokime::domain::model::history::wire::CheckpointFileWindow {
            checkpoint: Some(inspection::project(&checkpoint, maximum).map_err(|error| wire::checkpoint_failure(error.into()))?),
            content: Some(window::project(template_content, maximum).map_err(|error| wire::checkpoint_failure(error.into()))?),
        };
        let allowance = dokime::harmos::grpc::output_payload_limit(&template, &["content", "data"], length, maximum)
            .map_err(|error| Status::internal(error.to_string()))?;
        if length != 0 && allowance == 0 {
            return Err(Status::resource_exhausted("checkpoint file metadata leaves no payload allowance"));
        }
        let length = allowance;

        let mut content = dokime::harmos::storage::ReadBatch::new(())
            .try_map(|_| -> dokime::harmos::Result<_> {
                let checkpoint = inspection::project(&checkpoint, maximum)?;
                let content_type = Some(output.file.metadata.media_type.clone());
                let mut bytes = Vec::new();
                bytes.try_reserve_exact(length).map_err(|_| exhausted())?;
                let state = match output.stream_state {
                    Some(dokime::domain::model::execution::saved::SavedStreamState::Interrupted) => {
                        dokime::ExecutionFileStreamState::Interrupted
                    }
                    _ => dokime::ExecutionFileStreamState::Finalized,
                };
                let content = ExecutionFileContent {
                    relative_path: output.relative_path.clone(),
                    content_kind: dokime::ExecutionFileContentKind::from_metadata(&output.relative_path, content_type.as_deref()),
                    content_type,
                    stream_state: state,
                    offset,
                    size_bytes: output.file.length,
                    bytes,
                };
                Ok((checkpoint, content))
            })
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        let mut reader = view
            .read_file(&output.file, dokime::harmos::storage::ByteRange { offset, limit: Some(length as u64) })
            .await
            .map_err(wire::checkpoint_failure)?;
        while let Some(batch) = reader.next().await.map_err(|error| wire::checkpoint_failure(error.into()))? {
            content = content
                .try_map(|(checkpoint, mut content)| -> dokime::harmos::Result<_> {
                    if batch.bytes().len() > length.saturating_sub(content.bytes.len()) {
                        return Err(exhausted());
                    }
                    content.bytes.extend_from_slice(batch.bytes());
                    Ok((checkpoint, content))
                })
                .map_err(|error| wire::checkpoint_failure(error.into()))?;
        }
        reader.completion().map_err(|error| wire::checkpoint_failure(error.into()))?;
        let response = content
            .try_map(|(checkpoint, content)| -> dokime::harmos::Result<_> {
                if content.bytes.len() != length {
                    return Err(exhausted());
                }
                let content = window::project(content, maximum)?;
                let response =
                    dokime::domain::model::history::wire::CheckpointFileWindow { checkpoint: Some(checkpoint), content: Some(content) };
                if response.wire_len()? > maximum {
                    return Err(exhausted());
                }
                Ok(response)
            })
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        Ok(Response::from_owner(response.owner()))
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Read>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "canonical checkpoint file window" }
}

/* === Tests === */
