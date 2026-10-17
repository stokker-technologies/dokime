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
use harmos::{
    Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use super::{inspection, wire};
use crate::grpc::{content::window, value};
use dokime_runtime::{
    ExecutionFileContent, RuntimeHandle, runtime::query::execution_history::ExecutionCheckpoint,
};
use dokime_domain::operations::history;

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
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
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
        let content_type = Some(output.file.metadata.media_type.clone());
        let offset = request.offset.min(output.file.length);
        let mut reader = view
            .read_file(
                &output.file,
                harmos::storage::ByteRange {
                    offset,
                    limit: Some(limit as u64),
                },
            )
            .await
            .map_err(wire::checkpoint_failure)?;
        let mut data = Vec::with_capacity(limit);
        while let Some(batch) = reader
            .next()
            .await
            .map_err(|error| wire::checkpoint_failure(error.into()))?
        {
            data.extend_from_slice(&batch.bytes);
        }
        let state = match output.stream_state {
            Some(dokime_runtime::execution::saved::SavedStreamState::Interrupted) => {
                dokime_runtime::ExecutionFileStreamState::Interrupted
            }
            _ => dokime_runtime::ExecutionFileStreamState::Finalized,
        };
        let content = dokime_runtime::ExecutionFileContent {
            relative_path: request.relative_path.clone(),
            content_kind: dokime_runtime::ExecutionFileContentKind::from_metadata(
                &request.relative_path,
                content_type.as_deref(),
            ),
            content_type,
            stream_state: state,
            offset,
            size_bytes: output.file.length,
            bytes: data,
        };
        project(&checkpoint, content, self.resources.maximum)
            .map(Into::into)
            .map_err(|source| wire::checkpoint_failure(source.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Read>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
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
    maximum: usize,
) -> harmos::Result<history::CheckpointFileWindow> {
    let checkpoint = inspection::project(checkpoint, maximum)?;
    let content = window::owned(content, maximum)?;
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
