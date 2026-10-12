/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod codec;
mod execution;
mod files;
mod inventory;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dokime.v1.Content.rs"));
}

/* === Re-Exports === */
pub(super) use generated::content_server::ContentServer;

/* === Imports === */
// Standard Library Imports
use std::pin::Pin;
use std::time::Duration;
// Third Party Imports
use futures_util::Stream;
use prost::Message;
use sha2::{Digest, Sha256};
use tokio::sync::watch;
use tonic::Streaming;
use tonic::{Request, Response, Status};
// Local Imports
use super::{
    chunked::{self, ArtifactFrames},
    error::detailed,
    value,
};
use dokime_app::{
    RuntimeHandle,
    artifacts::{ArtifactRecord, ArtifactStatus},
    capabilities::evidence::ArtifactUploadInput,
    core::{ArtifactId, CaseId, RecordScope, TestCycleId, TestExecutionId},
    runtime::ArtifactReader,
    runtime::ArtifactUpload,
};
use dokime_protocol::v1;
use generated::content_server::Content;

/* === Constants === */
pub(super) const MAX_CONTENT_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_ARTIFACT_SCAN: usize = 4096;
const MAX_TRANSFER_CHUNK_BYTES: usize = 64 * 1024;
const IDLE_TRANSFER_TIMEOUT: Duration = Duration::from_secs(30);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) stopping: watch::Receiver<bool>,
    pub(super) max_message_bytes: usize,
    pub(super) local_only: bool,
}

/* === Enums === */
/* === Traits === */

/* === Implementations === */
impl Service {
    async fn open(&self, id: &str) -> Result<ArtifactReader, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let id = ArtifactId::from_uuid(id.to_owned())
            .map_err(|_| Status::invalid_argument("invalid artifact UUID"))?;
        self.runtime
            .open_artifact_stream(&id)
            .await
            .map_err(|_| Status::internal("cannot open stored artifact"))?
            .ok_or_else(|| Status::not_found("artifact not found"))
    }
}

#[tonic::async_trait]
impl Content for Service {
    type DownloadAppearanceAssetStream =
        Pin<Box<dyn Stream<Item = Result<v1::AppearanceAssetFrame, Status>> + Send + 'static>>;
    async fn download_appearance_asset(
        &self,
        request: Request<v1::DownloadAppearanceAssetRequest>,
    ) -> Result<Response<Self::DownloadAppearanceAssetStream>, Status> {
        super::appearance::download(
            &self.runtime,
            request.into_inner(),
            self.max_message_bytes,
            self.stopping.clone(),
        )
        .await
        .map(Response::new)
    }

    async fn list_execution_files(
        &self,
        request: Request<v1::ListExecutionFilesRequest>,
    ) -> Result<Response<super::charged::Bytes>, Status> {
        files::list(&self.runtime, request.into_inner(), self.max_message_bytes)
            .await
            .map(Response::new)
    }
    async fn read_execution_file(
        &self,
        request: Request<v1::ReadExecutionFileRequest>,
    ) -> Result<Response<super::charged::Bytes>, Status> {
        files::read(&self.runtime, request.into_inner(), self.max_message_bytes)
            .await
            .map(Response::new)
    }
    type DownloadExecutionFileStream = execution::DownloadStream;

    async fn download_execution_file(
        &self,
        request: Request<v1::DownloadExecutionFileRequest>,
    ) -> Result<Response<Self::DownloadExecutionFileStream>, Status> {
        execution::download(
            &self.runtime,
            request.into_inner(),
            self.max_message_bytes,
            self.stopping.clone(),
        )
        .await
        .map(Response::new)
    }
    async fn list_artifacts(
        &self,
        request: Request<v1::ListArtifactsRequest>,
    ) -> Result<Response<v1::ListArtifactsReply>, Status> {
        let request = request.into_inner();
        let after = request.after_artifact_id;
        if request.limit > 100 {
            return Err(Status::invalid_argument("page limit exceeds 100"));
        }
        // Artifact storage has its own publication frontier, so bind the
        // continuation to a metadata fingerprint rather than the journal
        // revision used by runtime state pages.
        value::text(&after)?;
        let records = self
            .runtime
            .artifacts_limited(MAX_ARTIFACT_SCAN)
            .await
            .map_err(|_| Status::resource_exhausted("artifact inventory exceeds limit"))?;
        let artifacts = records
            .iter()
            .map(metadata)
            .collect::<Result<Vec<_>, Status>>()?;
        let mut fingerprint = Sha256::new();
        for artifact in &artifacts {
            fingerprint.update(artifact.encode_length_delimited_to_vec());
        }
        let epoch = self.runtime.process_epoch().to_string();
        let cursor = v1::ArtifactCursor {
            epoch,
            metadata_sha256: fingerprint.finalize().to_vec(),
        };
        if let Some(given) = request.cursor {
            if given.epoch != cursor.epoch {
                return Err(Status::failed_precondition(
                    "artifact cursor epoch mismatch; resynchronize without cursor",
                ));
            }
            if given.metadata_sha256 != cursor.metadata_sha256 {
                return Err(Status::aborted(
                    "artifact listing changed; restart pagination",
                ));
            }
        }
        let start = if after.is_empty() {
            0
        } else {
            artifacts
                .iter()
                .position(|artifact| artifact.artifact_id == after)
                .map(|position| position + 1)
                .ok_or_else(|| Status::invalid_argument("after artifact ID not found"))?
        };
        let limit = if request.limit == 0 {
            50
        } else {
            request.limit as usize
        };
        let more = artifacts.len().saturating_sub(start) > limit;
        let page = artifacts
            .into_iter()
            .skip(start)
            .take(limit)
            .collect::<Vec<_>>();
        let next_after_artifact_id = more.then(|| page.last().unwrap().artifact_id.clone());
        let reply = v1::ListArtifactsReply {
            artifacts: page,
            next_after_artifact_id,
            cursor: Some(cursor),
        };
        if reply.encoded_len() > self.max_message_bytes {
            return Err(Status::resource_exhausted(
                "artifact page exceeds configured message limit",
            ));
        }
        Ok(Response::new(reply))
    }
    type DownloadArtifactStream = chunked::FrameStream<v1::ArtifactFrame>;

    async fn download_artifact(
        &self,
        request: Request<v1::DownloadArtifactRequest>,
    ) -> Result<Response<Self::DownloadArtifactStream>, Status> {
        let reader = self.open(&request.get_ref().artifact_id).await?;
        let metadata = metadata(reader.record())?;
        Ok(Response::new(
            chunked::Download {
                size_bytes: metadata.size_bytes,
                sha256: Some(metadata.sha256.clone()),
                frames: ArtifactFrames(metadata),
                source: chunked::Source::artifact(reader),
                noun: "artifact",
                // The runtime combines storage and integrity failures; do not
                // claim every read failure is corruption.
                read_failure: "artifact content could not be verified",
                chunk_bytes: transfer_chunk_bytes(self.max_message_bytes),
                message_limit: self.max_message_bytes,
                stopping: self.stopping.clone(),
            }
            .stream()?,
        ))
    }

    async fn upload_artifact(
        &self,
        request: Request<Streaming<v1::UploadArtifactFrame>>,
    ) -> Result<Response<v1::ArtifactMetadata>, Status> {
        let mut input = request.into_inner();
        let mut stopping = self.stopping.clone();
        let first = receive_upload_frame(&mut input, &mut stopping)
            .await?
            .ok_or_else(|| Status::invalid_argument("artifact upload header is required"))?;
        let Some(v1::upload_artifact_frame::Payload::Header(header)) = first.payload else {
            return Err(Status::invalid_argument(
                "first artifact upload frame must be a header",
            ));
        };
        let upload_size = header.size_bytes;
        let (scope, filename, content_type, sha256) = upload_header(header)?;
        if scope.test_cycle_id.is_none() {
            return Err(Status::invalid_argument(
                "artifact upload cycle scope is required",
            ));
        }
        if scope.test_execution_id.is_none() {
            return Err(Status::invalid_argument(
                "artifact upload execution scope is required",
            ));
        }
        let upload = self
            .runtime
            .begin_artifact_upload(ArtifactUploadInput {
                scope,
                filename,
                content_type,
                size_bytes: upload_size,
                sha256,
            })
            .await
            .map_err(content_failure)?;
        let chunk_bytes = transfer_chunk_bytes(self.max_message_bytes);
        if chunk_bytes == 0 {
            let _ = upload.abandon().await;
            return Err(Status::resource_exhausted(
                "artifact upload exceeds configured message limit",
            ));
        }
        let result = receive_artifact(
            &mut input,
            &mut stopping,
            upload.clone(),
            upload_size,
            chunk_bytes,
        )
        .await;
        match result {
            Ok(trailing) => upload
                .finish_verified(trailing)
                .await
                .map_err(content_failure)
                .and_then(|record| metadata(&record).map(Response::new)),
            Err(error) => {
                let _ = upload.abandon().await;
                Err(error)
            }
        }
    }

    type DownloadCycleBundleStream = chunked::FrameStream<v1::CycleBundleFrame>;

    async fn download_cycle_bundle(
        &self,
        request: Request<v1::DownloadCycleBundleRequest>,
    ) -> Result<Response<Self::DownloadCycleBundleStream>, Status> {
        let cycle_id = TestCycleId::from_uuid(request.into_inner().cycle_id)
            .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
        Err(Status::unimplemented(format!(
            "cycle bundle export is not supported for {cycle_id}"
        )))
    }

    async fn get_cycle_folder_path(
        &self,
        request: Request<v1::GetCycleFolderPathRequest>,
    ) -> Result<Response<v1::LocalPathResponse>, Status> {
        if !self.local_only {
            return Err(Status::permission_denied(
                "cycle folder paths are only available on local listeners",
            ));
        }
        let cycle_id = TestCycleId::from_uuid(request.into_inner().cycle_id)
            .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
        let state = self.runtime.watch_state().latest();
        if !state.state().cycles.contains_key(&cycle_id) {
            return Err(Status::not_found("cycle not found"));
        }
        Err(Status::unimplemented(
            "native storage does not expose cycle directories",
        ))
    }
}

/* === Public Functions === */
pub(super) fn transfer_chunk_bytes(max_message_bytes: usize) -> usize {
    MAX_TRANSFER_CHUNK_BYTES.min(max_message_bytes.saturating_sub(32))
}

pub(super) fn metadata_memory(record: &ArtifactRecord) -> Result<usize, Status> {
    artifact_digest(record)?;
    if record.status == ArtifactStatus::Staged {
        return Err(Status::data_loss("invalid stored artifact metadata"));
    }
    let mut bytes = std::mem::size_of::<v1::ArtifactMetadata>()
        .saturating_add(record.artifact_id.as_str().len())
        .saturating_add(record.upload_id.as_str().len())
        .saturating_add(record.filename.len())
        .saturating_add(record.content_type.as_ref().map_or(0, String::len))
        .saturating_add(64);
    for id in [
        record.scope.test_cycle_id.as_ref().map(|id| id.as_str()),
        record.scope.test_case_id.as_ref().map(|id| id.as_str()),
        record
            .scope
            .test_execution_id
            .as_ref()
            .map(|id| id.as_str()),
    ]
    .into_iter()
    .flatten()
    {
        bytes = bytes.saturating_add(id.len());
    }
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted(
            "artifact metadata construction overflows",
        ));
    }
    Ok(bytes)
}

pub(super) fn metadata(record: &ArtifactRecord) -> Result<v1::ArtifactMetadata, Status> {
    metadata_memory(record)?;
    let invalid = || Status::data_loss("invalid stored artifact metadata");
    let sha256 = artifact_digest(record)?.to_owned();
    let created = time::OffsetDateTime::parse(
        record.created_at.as_str(),
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| invalid())?;
    Ok(v1::ArtifactMetadata {
        artifact_id: record.artifact_id.as_str().to_owned(),
        upload_id: record.upload_id.as_str().to_owned(),
        scope: Some(v1::ArtifactScope {
            cycle_id: record
                .scope
                .test_cycle_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            case_id: record
                .scope
                .test_case_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            execution_id: record
                .scope
                .test_execution_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
        }),
        filename: record.filename.clone(),
        content_type: record.content_type.clone(),
        size_bytes: record.size_bytes,
        sha256,
        state: match record.status {
            ArtifactStatus::Complete => v1::ArtifactState::Complete,
            ArtifactStatus::Cancelled => v1::ArtifactState::Cancelled,
            ArtifactStatus::Rejected => v1::ArtifactState::Rejected,
            ArtifactStatus::Staged => return Err(invalid()),
        } as i32,
        created_at: Some(prost_types::Timestamp {
            seconds: created.unix_timestamp(),
            nanos: created.nanosecond() as i32,
        }),
    })
}

/* === Private Functions === */
fn artifact_digest(record: &ArtifactRecord) -> Result<&str, Status> {
    record
        .sha256
        .as_deref()
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        .ok_or_else(|| Status::data_loss("invalid stored artifact metadata"))
}

async fn receive_upload_frame(
    input: &mut Streaming<v1::UploadArtifactFrame>,
    stopping: &mut watch::Receiver<bool>,
) -> Result<Option<v1::UploadArtifactFrame>, Status> {
    if *stopping.borrow() {
        return Err(Status::unavailable("backend is draining"));
    }
    tokio::select! {
        biased;
        _ = stopping.changed() => Err(Status::unavailable("backend is draining")),
        result = tokio::time::timeout(IDLE_TRANSFER_TIMEOUT, input.message()) => {
            result
                .map_err(|_| Status::deadline_exceeded("artifact upload idle deadline exceeded"))?
        },
    }
}

fn upload_header(
    header: v1::UploadArtifactHeader,
) -> Result<(RecordScope, String, Option<String>, Option<String>), Status> {
    let scope = artifact_scope(header.scope)?;
    let filename = header.filename.trim().to_owned();
    if filename.is_empty() || filename.len() > 255 {
        return Err(Status::invalid_argument(
            "artifact filename must contain 1–255 bytes",
        ));
    }
    let content_type = header.content_type.filter(|value| !value.is_empty());
    if content_type.as_ref().is_some_and(|value| value.len() > 255) {
        return Err(Status::invalid_argument(
            "artifact content type must not exceed 255 bytes",
        ));
    }
    if header.size_bytes > MAX_CONTENT_BYTES {
        return Err(Status::resource_exhausted(
            "artifact exceeds native content limit",
        ));
    }
    let sha256 = header.sha256;
    if sha256.as_ref().is_some_and(|digest| !valid_digest(digest)) {
        return Err(Status::invalid_argument(
            "artifact sha256 must contain 64 lowercase hexadecimal characters",
        ));
    }
    Ok((scope, filename, content_type, sha256))
}

fn artifact_scope(scope: Option<v1::ArtifactScope>) -> Result<RecordScope, Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("artifact scope is required"))?;
    let cycle_id = scope
        .cycle_id
        .map(|id| {
            TestCycleId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid cycle UUID"))
        })
        .transpose()?;
    let case_id = scope
        .case_id
        .map(|id| CaseId::from_wire(id).map_err(|_| Status::invalid_argument("invalid case ID")))
        .transpose()?;
    let execution_id = scope
        .execution_id
        .map(|id| {
            TestExecutionId::from_uuid(id)
                .map_err(|_| Status::invalid_argument("invalid execution UUID"))
        })
        .transpose()?;
    Ok(RecordScope {
        test_cycle_id: cycle_id,
        test_case_id: case_id,
        test_execution_id: execution_id,
        ..RecordScope::default()
    })
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

async fn receive_artifact(
    input: &mut Streaming<v1::UploadArtifactFrame>,
    stopping: &mut watch::Receiver<bool>,
    upload: ArtifactUpload,
    expected_size: u64,
    max_chunk_bytes: usize,
) -> Result<Option<String>, Status> {
    let mut offset = 0_u64;
    while let Some(frame) = receive_upload_frame(input, stopping).await? {
        match frame.payload {
            Some(v1::upload_artifact_frame::Payload::Chunk(chunk)) => {
                if chunk.data.is_empty()
                    || chunk.data.len() > max_chunk_bytes
                    || chunk.offset != offset
                {
                    return Err(Status::invalid_argument(
                        "invalid artifact chunk offset or size",
                    ));
                }
                offset = offset
                    .checked_add(chunk.data.len() as u64)
                    .ok_or_else(|| Status::invalid_argument("artifact size overflow"))?;
                if offset > expected_size {
                    return Err(Status::invalid_argument("artifact exceeds declared size"));
                }
                upload.send(&chunk.data).await.map_err(content_failure)?;
            }
            Some(v1::upload_artifact_frame::Payload::Complete(completion)) => {
                if completion.size_bytes != offset {
                    return Err(Status::invalid_argument(
                        "artifact completion size does not match the received bytes",
                    ));
                }
                if !valid_digest(&completion.sha256) {
                    return Err(Status::invalid_argument(
                        "artifact sha256 must contain 64 lowercase hexadecimal characters",
                    ));
                }
                if receive_upload_frame(input, stopping).await?.is_some() {
                    return Err(Status::invalid_argument(
                        "no frame may follow the artifact upload completion",
                    ));
                }
                return Ok(Some(completion.sha256));
            }
            _ => {
                return Err(Status::invalid_argument(
                    "only chunks or a completion may follow artifact upload header",
                ));
            }
        }
    }
    Ok(None)
}

fn content_failure(error: dokime_app::Error) -> Status {
    use dokime_app::Error;
    let status = match &error {
        Error::ExportStorage {
            source: dokime_app::capabilities::exports::storage::StorageError::ArchiveTooLarge { .. },
        } => Status::resource_exhausted("cycle bundle exceeds native content limit"),
        Error::NotFound { .. } => Status::not_found("artifact upload owner not found"),
        Error::Validation { message, .. } => Status::invalid_argument(message.clone()),
        Error::Forbidden { .. } => Status::permission_denied("artifact upload failed"),
        Error::Overload { .. } => Status::resource_exhausted("artifact upload failed"),
        Error::Timeout { .. } => Status::deadline_exceeded("artifact upload interrupted"),
        Error::Unavailable { .. } | Error::ShuttingDown | Error::Interrupted => {
            Status::unavailable("artifact upload interrupted")
        }
        Error::Operation(message) => operation_status(message),
        _ => Status::internal("artifact upload failed"),
    };

    detailed(status, &error)
}

fn operation_status(message: &str) -> Status {
    if message.contains("not found") {
        Status::not_found("artifact upload owner not found")
    } else if ["invalid", "requires", "exceeds", "scope", "size/hash"]
        .iter()
        .any(|marker| message.contains(marker))
    {
        Status::invalid_argument(message.to_owned())
    } else {
        Status::internal("artifact upload failed")
    }
}

/* === Tests === */
