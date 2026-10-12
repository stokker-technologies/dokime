/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod codec;
mod construction;
pub(super) mod document;
mod read;
mod records;
mod telemetry;
mod wire;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dokime.v1.History.rs"));
}

/* === Re-Exports === */
pub(super) use generated::history_server::HistoryServer;
pub(super) use wire::checkpoint;
pub(crate) use wire::traceability;

/* === Imports === */
// Standard Library Imports
use std::pin::Pin;
// Third Party Imports
use futures_util::Stream;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
// Local Imports
use super::{error::detailed, value};
use dokime_app::execution::ExecutionStatus;
use dokime_app::runtime::query::{ExecutionFilter, ExecutionOrder, ExecutionOrderKey};
use dokime_app::{RuntimeHandle, runtime::query::execution_history::ExecutionBrowseQuery};
use dokime_protocol::v1;
use generated::history_server::History;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) stopping: watch::Receiver<bool>,
    pub(super) max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl History for Service {
    type DownloadCheckpointArtifactStream =
        Pin<Box<dyn Stream<Item = Result<v1::ArtifactFrame, Status>> + Send + 'static>>;

    async fn get_execution_checkpoint(
        &self,
        request: Request<v1::GetExecutionCheckpointRequest>,
    ) -> Result<Response<super::charged::Bytes>, Status> {
        let id =
            dokime_app::core::TestExecutionId::from_uuid(request.into_inner().execution_identifier)
                .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        self.runtime
            .execution_saved_checkpoint(id, |checkpoint, allowance| {
                codec::encode_checkpoint(checkpoint, allowance, self.max_message_bytes)
            })
            .await
            .map(Response::new)
            .map_err(|source| match source {
                dokime_app::Error::NotFound { .. } => {
                    detailed(Status::not_found("execution has no saved seal"), &source)
                }
                source => wire::query_failure(source),
            })
    }

    async fn browse_executions(
        &self,
        request: Request<v1::BrowseExecutionsRequest>,
    ) -> Result<Response<codec::Page>, Status> {
        let request = request.into_inner();
        value::text(&request.after_execution_id)?;
        let after = if request.after_execution_id.is_empty() {
            None
        } else {
            Some(
                dokime_app::core::TestExecutionId::from_uuid(request.after_execution_id)
                    .map_err(|_| Status::invalid_argument("invalid execution cursor ID"))?,
            )
        };
        let query = ExecutionBrowseQuery {
            after,
            limit: (request.limit != 0).then_some(request.limit as usize),
            filter: execution_filter(request.filter)?,
            order: execution_order(request.order)?,
        };
        let maximum = self.max_message_bytes.min(4 * 1024 * 1024);
        let page = self
            .runtime
            .browse_executions(query, |page, allowance| {
                codec::encode(page, allowance, maximum)
            })
            .await
            .map_err(wire::query_failure)?;
        Ok(Response::new(page))
    }

    async fn delete_execution(
        &self,
        request: Request<v1::DeleteExecutionRequest>,
    ) -> Result<Response<v1::DeleteExecutionResponse>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }

        let request = request.into_inner();
        wire::checkpoint(request.checkpoint)?;
        wire::deletion_request_identifier(request.request_identifier)?;
        Err(Status::unimplemented(
            "execution deletion is not supported by native storage",
        ))
    }

    async fn get_checkpoint_document(
        &self,
        request: Request<v1::GetCheckpointDocumentRequest>,
    ) -> Result<Response<codec::Page>, Status> {
        let checkpoint = wire::checkpoint(request.into_inner().checkpoint)?;
        let maximum = self.max_message_bytes.min(4 * 1024 * 1024);
        let document = self
            .runtime
            .execution_checkpoint_document(checkpoint, |document, allowance| {
                codec::encode_document(document, allowance, maximum)
            })
            .await
            .map_err(wire::checkpoint_failure)?;
        Ok(Response::new(document))
    }

    async fn get_checkpoint_evidence(
        &self,
        request: Request<v1::GetCheckpointEvidenceRequest>,
    ) -> Result<Response<codec::Page>, Status> {
        let request = request.into_inner();
        let checkpoint = wire::checkpoint(request.checkpoint)?;
        let sections = wire::sections(request.sections)?;
        let maximum = self.max_message_bytes;
        let only_outputs = sections.iter().all(|section| {
            matches!(
                section,
                v1::EvidenceSection::Outputs | v1::EvidenceSection::Artifacts
            )
        });
        let response = if only_outputs {
            self.runtime
                .execution_checkpoint_outputs_projected(
                    checkpoint.clone(),
                    |checkpoint, outputs, allowance| {
                        codec::encode_outputs(checkpoint, outputs, &sections, allowance, maximum)
                    },
                )
                .await
        } else {
            self.runtime
                .execution_checkpoint_evidence(checkpoint, |evidence, allowance| {
                    codec::encode_evidence(evidence, &sections, allowance, maximum)
                })
                .await
        }
        .map_err(wire::checkpoint_failure)?;
        Ok(Response::new(response))
    }

    async fn read_checkpoint_output(
        &self,
        request: Request<v1::ReadCheckpointOutputRequest>,
    ) -> Result<Response<codec::Page>, Status> {
        let request = request.into_inner();
        value::text(&request.relative_path)?;
        if request.relative_path.is_empty() {
            return Err(Status::invalid_argument("output name is required"));
        }
        if request.max_bytes > 1024 * 1024 {
            return Err(Status::invalid_argument("read limit exceeds 1 MiB"));
        }
        let checkpoint = wire::checkpoint(request.checkpoint)?;
        let limit = if request.max_bytes == 0 {
            64 * 1024
        } else {
            request.max_bytes as usize
        };
        let limit = limit.min(
            self.max_message_bytes
                .saturating_sub(request.relative_path.len() + 512),
        );
        if limit == 0 {
            return Err(Status::resource_exhausted(
                "configured message limit cannot contain file window",
            ));
        }
        let view = self
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
        let _computation = view
            .view()
            .begin_computation()
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        let bytes = limit
            .saturating_add(request.relative_path.len())
            .saturating_add(output.file.metadata.media_type.len())
            .saturating_add(std::mem::size_of::<dokime_app::ExecutionFileContent>())
            .saturating_mul(2)
            .saturating_add(std::mem::size_of::<v1::CheckpointFileWindow>());
        let preparation = self
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
                    codec::encode_window(checkpoint, content, allowance, self.max_message_bytes)
                },
            )
            .map_err(|error| wire::checkpoint_failure(error.into()))?;
        Ok(Response::new(response))
    }

    async fn download_checkpoint_artifact(
        &self,
        request: Request<v1::DownloadCheckpointArtifactRequest>,
    ) -> Result<Response<Self::DownloadCheckpointArtifactStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let request = request.into_inner();
        value::text(&request.artifact_id)?;
        let checkpoint = wire::checkpoint(request.checkpoint)?;
        let artifact_id = dokime_app::core::ArtifactId::from_uuid(request.artifact_id)
            .map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
        let reader = self
            .runtime
            .checkpoint_artifact(checkpoint, &artifact_id)
            .await
            .map_err(wire::checkpoint_failure)?;
        let metadata = super::content::metadata(reader.record())?;
        Ok(Response::new(read::checkpoint_artifact(
            reader,
            metadata,
            self.max_message_bytes,
            self.stopping.clone(),
        )?))
    }

    async fn list_checkpoint_records(
        &self,
        request: Request<v1::ListCheckpointRecordsRequest>,
    ) -> Result<Response<super::charged::Bytes>, Status> {
        let request = request.into_inner();
        value::text(&request.after)?;
        let checkpoint = wire::checkpoint(request.checkpoint)?;
        let query = dokime_app::runtime::query::checkpoint_records::ExecutionRecordQuery {
            after: (!request.after.is_empty()).then_some(request.after),
            limit: (request.limit != 0).then_some(request.limit as usize),
        };
        self.runtime
            .execution_checkpoint_records_projected(checkpoint, query, |page, allowance| {
                records::encode(page, allowance, self.max_message_bytes)
            })
            .await
            .map(Response::new)
            .map_err(wire::query_failure)
    }

    async fn list_checkpoint_telemetry(
        &self,
        request: Request<v1::ListCheckpointTelemetryRequest>,
    ) -> Result<Response<super::charged::Bytes>, Status> {
        let request = request.into_inner();
        value::text(&request.table)?;
        value::text(&request.after)?;
        let stream = dokime_app::core::Identifier::new(request.table).map_err(|_| {
            Status::invalid_argument("native telemetry stream identifier is required")
        })?;
        let checkpoint = wire::checkpoint(request.checkpoint)?;
        let query = dokime_app::runtime::query::checkpoint_telemetry::ExecutionTelemetryQuery {
            after: (!request.after.is_empty()).then_some(request.after),
            limit: (request.limit != 0).then_some(request.limit as usize),
        };
        self.runtime
            .execution_checkpoint_telemetry_projected(
                checkpoint,
                &stream,
                query,
                |page, allowance| telemetry::encode(page, allowance, self.max_message_bytes),
            )
            .await
            .map(Response::new)
            .map_err(wire::query_failure)
    }

    async fn get_checkpoint_context(
        &self,
        request: Request<v1::GetCheckpointContextRequest>,
    ) -> Result<Response<codec::Page>, Status> {
        let checkpoint = wire::checkpoint(request.into_inner().checkpoint)?;
        let maximum = self.max_message_bytes.min(4 * 1024 * 1024);
        let context = self
            .runtime
            .execution_checkpoint_context(checkpoint, |context, allowance| {
                codec::encode_context(context, allowance, maximum)
            })
            .await
            .map_err(wire::checkpoint_failure)?;
        Ok(Response::new(context))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn execution_filter(filter: Option<v1::ExecutionFilter>) -> Result<ExecutionFilter, Status> {
    let Some(filter) = filter else {
        return Ok(ExecutionFilter::default());
    };
    value::text(&filter.text)?;
    value::collection(filter.statuses.len())?;
    value::collection(filter.template_ids.len())?;
    value::collection(filter.cycle_ids.len())?;
    for identifier in filter.template_ids.iter().chain(filter.cycle_ids.iter()) {
        value::text(identifier)?;
    }
    Ok(ExecutionFilter {
        text: filter.text,
        statuses: filter
            .statuses
            .into_iter()
            .map(execution_status)
            .collect::<Result<_, Status>>()?,
        template_ids: filter.template_ids,
        cycle_ids: filter.cycle_ids,
        started_after: filter
            .started_after
            .map(value::decoded_timestamp)
            .transpose()?,
        started_before: filter
            .started_before
            .map(value::decoded_timestamp)
            .transpose()?,
    })
}

fn execution_order(order: Option<v1::ExecutionOrder>) -> Result<ExecutionOrder, Status> {
    let Some(order) = order else {
        return Ok(ExecutionOrder::default());
    };
    let key = match v1::ExecutionOrderKey::try_from(order.key)
        .map_err(|_| Status::invalid_argument("unknown execution order key"))?
    {
        v1::ExecutionOrderKey::Unspecified | v1::ExecutionOrderKey::StartedAt => {
            ExecutionOrderKey::StartedAt
        }
        v1::ExecutionOrderKey::FinishedAt => ExecutionOrderKey::FinishedAt,
        v1::ExecutionOrderKey::Title => ExecutionOrderKey::Title,
    };
    Ok(ExecutionOrder {
        key,
        descending: order.descending,
    })
}

fn execution_status(status: i32) -> Result<ExecutionStatus, Status> {
    match v1::ExecutionStatus::try_from(status)
        .map_err(|_| Status::invalid_argument("unknown execution status"))?
    {
        v1::ExecutionStatus::Unspecified => {
            Err(Status::invalid_argument("unspecified execution status"))
        }
        v1::ExecutionStatus::Invalidated => Ok(ExecutionStatus::Invalidated),
        v1::ExecutionStatus::Skipped => Ok(ExecutionStatus::Skipped),
        v1::ExecutionStatus::Pending => Ok(ExecutionStatus::Pending),
        v1::ExecutionStatus::Running => Ok(ExecutionStatus::Running),
        v1::ExecutionStatus::Passed => Ok(ExecutionStatus::Passed),
        v1::ExecutionStatus::Failed => Ok(ExecutionStatus::Failed),
        v1::ExecutionStatus::Blocked => Ok(ExecutionStatus::Blocked),
        v1::ExecutionStatus::Cancelled => Ok(ExecutionStatus::Cancelled),
        v1::ExecutionStatus::Interrupted => Ok(ExecutionStatus::Interrupted),
    }
}

/* === Tests === */
