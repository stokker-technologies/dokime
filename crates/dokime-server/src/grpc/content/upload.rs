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
use dokime::harmos::operations::{Context, Operation, Response, StreamInput};
use tonic::Status;

// Local Imports
use super::{metadata, transfer_chunk_bytes};
use crate::grpc::error::detailed;
use crate::grpc::transcode::Transcode;
use dokime::RuntimeHandle;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::execution::RecordScope;
use dokime::domain::model::identity::CaseId;
use dokime::domain::model::identity::TestCycleId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::content as wire;
use dokime::domains::evidence::ArtifactUploadInput;
use dokime::runtime::ArtifactUpload;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

struct Upload {
    resources: Arc<Resources>,
    upload: Option<ArtifactUpload>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Upload {
    type Declaration = wire::UploadArtifact;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamInput<dokime::domain::model::content::wire::UploadArtifactFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources, upload: None })
    }
    async fn run<'a>(
        &mut self, call_context: Self::Context<'a>, _: &mut wire::UploadArtifact, input: &mut Self::Endpoints,
    ) -> Result<Response<dokime::domain::model::content::wire::ArtifactMetadata>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async {
            let first = input.recv().await.ok_or_else(|| Status::invalid_argument("artifact upload header is required"))?;
            let Some(dokime::domain::model::content::wire::UploadArtifactFramePayload::Header(header)) = &first.payload else {
                return Err(Status::invalid_argument("first artifact upload frame must be a header"));
            };
            let upload_size = header.size_bytes;
            let (scope, filename, content_type, sha256) = upload_header(header)?;
            if scope.test_cycle_id.is_none() {
                return Err(Status::invalid_argument("artifact upload cycle scope is required"));
            }
            if scope.test_execution_id.is_none() {
                return Err(Status::invalid_argument("artifact upload execution scope is required"));
            }
            let upload = self
                .resources
                .runtime
                .begin_artifact_upload(ArtifactUploadInput { scope, filename, content_type, size_bytes: upload_size, sha256 })
                .await
                .map_err(content_failure)?;
            self.upload = Some(upload.clone());
            drop(first);
            let chunk_bytes = transfer_chunk_bytes(&self.resources.runtime, self.resources.maximum)?;
            if chunk_bytes == 0 {
                return Err(Status::resource_exhausted("artifact upload exceeds configured message limit"));
            }
            let receiving = std::time::Instant::now();
            tracing::debug!(phase = "artifact_receive", bytes = upload_size, "artifact upload receiving started");
            let received = receive_artifact(input, upload.clone(), upload_size, chunk_bytes).await;
            tracing::debug!(
                phase = "artifact_receive", bytes = upload_size, elapsed_ms = %receiving.elapsed().as_millis(),
                succeeded = received.is_ok(), "artifact upload receiving finished"
            );
            let trailing = match received {
                Ok(trailing) => trailing,
                Err(source) => {
                    match upload.abandon().await {
                        Ok(_) => self.upload = None,
                        Err(cleanup) => {
                            tracing::warn!(error = %cleanup, "artifact upload abandonment failed")
                        }
                    }
                    return Err(source);
                }
            };
            let sealing = std::time::Instant::now();
            tracing::debug!(phase = "artifact_upload_seal", bytes = upload_size, "original artifact upload sealing started");
            let sealed = upload.finish_verified(trailing).await;
            tracing::debug!(
                phase = "artifact_upload_seal", bytes = upload_size, elapsed_ms = %sealing.elapsed().as_millis(),
                succeeded = sealed.is_ok(), "original artifact upload sealing finished"
            );
            let record = sealed.map_err(content_failure)?;
            self.upload = None;
            let output = metadata(&record)?;
            if output.wire_len()? > self.resources.maximum {
                return Err(Status::resource_exhausted("artifact metadata exceeds response budget"));
            }
            Ok(output.into())
        })
        .await
    }
    async fn stop(&mut self) -> Result<(), Status> {
        self.abandon().await
    }
    async fn cleanup(&mut self) -> Result<(), Status> {
        self.abandon().await
    }
}

impl Upload {
    async fn abandon(&mut self) -> Result<(), Status> {
        if let Some(upload) = &self.upload {
            upload.abandon().await.map_err(content_failure)?;
            self.upload = None;
        }
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Upload>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

pub(in crate::grpc) fn artifact_scope(scope: Option<&dokime::domain::model::content::wire::ArtifactScope>) -> Result<RecordScope, Status> {
    let scope = scope.ok_or_else(|| Status::invalid_argument("artifact scope is required"))?;
    let cycle_id = scope
        .cycle_id
        .as_deref()
        .map(|id| TestCycleId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid cycle UUID")))
        .transpose()?;
    let case_id = scope
        .case_id
        .as_deref()
        .map(|id| CaseId::from_wire(id.to_owned()).map_err(|_| Status::invalid_argument("invalid case ID")))
        .transpose()?;
    let execution_id = scope
        .execution_id
        .as_deref()
        .map(|id| TestExecutionId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid execution UUID")))
        .transpose()?;
    Ok(RecordScope { test_cycle_id: cycle_id, test_case_id: case_id, test_execution_id: execution_id, ..RecordScope::default() })
}

pub(in crate::grpc) fn content_failure(error: dokime::runtime::Error) -> Status {
    use dokime::runtime::Error;
    let status = match &error {
        Error::ExportStorage { source: dokime::domains::exports::storage::StorageError::ArchiveTooLarge { .. } } => {
            Status::resource_exhausted("cycle bundle exceeds native content limit")
        }
        Error::Admission(AdmissionFailure::NotFound { .. }) => Status::not_found("artifact upload owner not found"),
        Error::Admission(AdmissionFailure::Validation { message, .. }) => Status::invalid_argument(message.clone()),
        Error::Admission(AdmissionFailure::Forbidden { .. }) => Status::permission_denied("artifact upload failed"),
        Error::Harmos(dokime::harmos::Error::ResourceExhausted { .. }) | Error::Admission(AdmissionFailure::Overload { .. }) => {
            Status::resource_exhausted("artifact upload failed")
        }
        Error::Admission(AdmissionFailure::Timeout { .. }) => Status::deadline_exceeded("artifact upload interrupted"),
        Error::Admission(AdmissionFailure::Unavailable { .. }) | Error::ShuttingDown | Error::Interrupted => {
            Status::unavailable("artifact upload interrupted")
        }
        Error::Operation(message) => operation_status(message),
        _ => Status::internal("artifact upload failed"),
    };

    detailed(status, &error)
}

/* === Private Functions === */

fn upload_header(
    header: &dokime::domain::model::content::wire::UploadArtifactHeader,
) -> Result<(RecordScope, String, Option<String>, Option<String>), Status> {
    let scope = artifact_scope(header.scope.as_ref())?;
    let filename = header.filename.trim();
    dokime::artifacts::name::validate(filename).map_err(|source| Status::invalid_argument(source.to_string()))?;
    let content_type = header.content_type.as_deref().filter(|value| !value.is_empty());
    let sha256 = header.sha256.as_deref();
    if sha256.as_ref().is_some_and(|digest| !valid_digest(digest)) {
        return Err(Status::invalid_argument("artifact sha256 must contain 64 lowercase hexadecimal characters"));
    }
    Ok((scope, filename.to_owned(), content_type.map(str::to_owned), sha256.map(str::to_owned)))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

async fn receive_artifact(
    input: &mut StreamInput<dokime::domain::model::content::wire::UploadArtifactFrame>, upload: ArtifactUpload, expected_size: u64,
    max_chunk_bytes: usize,
) -> Result<Option<String>, Status> {
    let mut offset = 0_u64;
    while let Some(frame) = input.recv().await {
        let (frame, claims) = frame
            .into_owned_request()
            .map_err(|source| super::super::error::detailed_error(Status::internal("artifact upload frame ownership failed"), &source))?;
        match frame.payload {
            Some(dokime::domain::model::content::wire::UploadArtifactFramePayload::Chunk(chunk)) => {
                if chunk.data.is_empty() || chunk.data.len() > max_chunk_bytes || chunk.offset != offset {
                    return Err(Status::invalid_argument("invalid artifact chunk offset or size"));
                }
                offset = offset.checked_add(chunk.data.len() as u64).ok_or_else(|| Status::invalid_argument("artifact size overflow"))?;
                if offset > expected_size {
                    return Err(Status::invalid_argument("artifact exceeds declared size"));
                }
                upload.send(chunk.data).await.map_err(content_failure)?;
            }
            Some(dokime::domain::model::content::wire::UploadArtifactFramePayload::Complete(completion)) => {
                if completion.size_bytes != offset {
                    return Err(Status::invalid_argument("artifact completion size does not match the received bytes"));
                }
                if !valid_digest(&completion.sha256) {
                    return Err(Status::invalid_argument("artifact sha256 must contain 64 lowercase hexadecimal characters"));
                }
                let sha256 = completion.sha256;
                drop(claims);
                if input.recv().await.is_some() {
                    return Err(Status::invalid_argument("no frame may follow the artifact upload completion"));
                }
                return Ok(Some(sha256));
            }
            _ => {
                return Err(Status::invalid_argument("only chunks or a completion may follow artifact upload header"));
            }
        }
    }
    Ok(None)
}

fn operation_status(message: &str) -> Status {
    if message.contains("not found") {
        Status::not_found("artifact upload owner not found")
    } else if ["invalid", "requires", "exceeds", "scope", "size/hash"].iter().any(|marker| message.contains(marker)) {
        Status::invalid_argument(message.to_owned())
    } else {
        Status::internal("artifact upload failed")
    }
}

/* === Tests === */
