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
use std::path::{Path, PathBuf};

// Third Party Imports

// Local Imports
use super::desktop::ArtifactSource;
use crate::connection::SessionLease;
use dokime_protocol::client;
use dokime_protocol::client::FilesReadResponse;
use dokime_protocol::client::{FilesGetRequest, FilesGetResponse, FilesReadRequest};
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug)]
pub(crate) struct VerifiedContent {
    pub path: PathBuf,

    #[allow(dead_code)]
    pub size_bytes: u64,

    #[allow(dead_code)]
    pub sha256: String,

    pub content_type: Option<String>,

    pub report_origin: Option<client::ReportOrigin>,
}

#[derive(Debug, serde::Serialize, thiserror::Error)]
#[error("{stage:?} failed: {failure}")]
pub struct DeliveryError {
    pub stage: DeliveryStage,
    pub failure: DeliveryFailure,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStage {
    Prepare,
    Request,
    Stream,
    Write,
    Rename,
    Open,
}

#[derive(Debug, serde::Serialize, thiserror::Error)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum DeliveryFailure {
    #[error("{0}")]
    Rpc(crate::Error),

    #[error("{0}")]
    Local(String),
}

/* === Traits === */

/* === Implementations === */

impl DeliveryError {
    pub(crate) fn local(stage: DeliveryStage, message: impl Into<String>) -> Self {
        Self {
            stage,
            failure: DeliveryFailure::Local(message.into()),
        }
    }
}

impl From<crate::connection::host::Error> for DeliveryError {
    fn from(error: crate::connection::host::Error) -> Self {
        Self {
            stage: DeliveryStage::Request,
            failure: DeliveryFailure::Rpc(error.into()),
        }
    }
}

impl VerifiedContent {
    pub(crate) fn report_origin(&self) -> Option<&'static str> {
        match self.report_origin {
            Some(client::ReportOrigin::Sealed) => Some("sealed"),
            Some(client::ReportOrigin::Current) => Some("current"),
            _ => None,
        }
    }
}

/* === Public Functions === */

pub(crate) async fn fetch_verified(
    lease: &SessionLease, source: &ArtifactSource, destination: &Path, frame_bytes: usize,
) -> Result<VerifiedContent, DeliveryError> {
    let session = lease.client();
    let (reference, report_origin) = match source {
        ArtifactSource::Artifact { artifact_id } => {
            let reply: client::content::ArtifactsGetResponse = session
                .unary(
                    "/dokime.client.v1.content.Artifacts/Get",
                    client::content::ArtifactsGetRequest {
                        name: artifact_id.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            (reply.file, None)
        }
        ArtifactSource::ExecutionFile { execution_id, relative_path } => {
            let reply: client::content::ExecutionFilesGetResponse = session
                .unary(
                    "/dokime.client.v1.content.ExecutionFiles/Get",
                    client::content::ExecutionFilesGetRequest {
                        parent: execution_id.clone(),
                        name: relative_path.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            (reply.file, None)
        }
        ArtifactSource::ExecutionEvidence { execution_id } => {
            let reply: client::EvidenceArchiveResponse = session
                .unary(
                    "/dokime.client.v1.Evidence/Archive",
                    client::EvidenceArchiveRequest {
                        subject: Some(client::EvidenceSubject {
                            kind: Some(client::evidence_subject::Kind::Execution(execution_id.clone())),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            (reply.file, None)
        }
        ArtifactSource::CycleEvidence { cycle_id } => {
            let reply: client::EvidenceArchiveResponse = session
                .unary(
                    "/dokime.client.v1.Evidence/Archive",
                    client::EvidenceArchiveRequest {
                        subject: Some(client::EvidenceSubject {
                            kind: Some(client::evidence_subject::Kind::Cycle(cycle_id.clone())),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            (reply.file, None)
        }
        ArtifactSource::CycleReport { cycle_id } => {
            let reply: client::ReportsCreateResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Create",
                    client::ReportsCreateRequest {
                        subject: Some(client::ReportSubject {
                            kind: Some(client::report_subject::Kind::Cycle(cycle_id.clone())),
                        }),
                        request_id: format!("desktop-report-{:032x}", rand::random::<u128>()),
                        ..Default::default()
                    },
                )
                .await?;
            (reply.file, reply.report.and_then(|report| client::ReportOrigin::try_from(report.origin).ok()))
        }
        ArtifactSource::ExecutionReport { execution_id } => {
            let reply: client::ReportsGetResponse = session
                .unary(
                    "/dokime.client.v1.Reports/Get",
                    client::ReportsGetRequest {
                        subject: Some(client::ReportSubject {
                            kind: Some(client::report_subject::Kind::Execution(execution_id.clone())),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            (reply.file, reply.report.and_then(|report| client::ReportOrigin::try_from(report.origin).ok()))
        }
    };
    let reference = reference.ok_or_else(|| invalid("Read Content: Host Omitted Its File Reference"))?;
    if reference.sha256.len() != 32 {
        return Err(invalid("Read Content: Host Omitted Its SHA-256"));
    }
    let selected: FilesGetResponse =
        session.unary("/dokime.client.v1.Files/Get", FilesGetRequest { file: Some(reference.clone()), at: None }).await?;
    let selected_file = selected.file.as_ref().ok_or_else(|| invalid("Read Content: Host Omitted Its Selected File"))?;
    if selected_file.length != reference.length || selected_file.sha256 != reference.sha256 {
        return Err(invalid("Read Content: Selected File Identity Changed"));
    }
    let frame_bytes = frame_bytes
        .min(usize::try_from(selected.chunk_bytes).map_err(|_| invalid("Read Content: Invalid Chunk Limit"))?)
        .min(usize::try_from(selected.message_bytes.saturating_sub(1024)).map_err(|_| invalid("Read Content: Invalid Message Limit"))?);
    if frame_bytes == 0 {
        return Err(invalid("Read Content: Host Omitted Its Transfer Limits"));
    }
    let content_type = reference.metadata.as_ref().map(|metadata| metadata.media_type.clone());
    let directory =
        destination.parent().ok_or_else(|| DeliveryError::local(DeliveryStage::Prepare, "Select a destination with a parent directory"))?;
    tokio::fs::create_dir_all(directory).await.map_err(write_error)?;
    let staged = tempfile::NamedTempFile::new_in(directory).map_err(write_error)?;
    let mut file = File::from_std(staged.reopen().map_err(write_error)?);
    let mut digest = Sha256::new();
    let mut offset = 0;
    let mut at = selected.resolved_at;
    while offset < reference.length {
        let reply: FilesReadResponse = session
            .unary(
                "/dokime.client.v1.Files/Read",
                FilesReadRequest {
                    file: Some(reference.clone()),
                    offset,
                    max_bytes: frame_bytes as u64,
                    at: at.clone(),
                },
            )
            .await?;
        if reply.total_length != reference.length
            || reply.sha256 != reference.sha256
            || reply.data.is_empty()
            || reply.data.len() > frame_bytes
            || offset.saturating_add(reply.data.len() as u64) > reference.length
            || reply.resolved_at.is_none()
            || (at.is_some() && at != reply.resolved_at)
        {
            return Err(invalid("Read Content: Invalid Range, Digest or Retained Cut"));
        }
        at = reply.resolved_at;
        offset += reply.data.len() as u64;
        digest.update(&reply.data);
        file.write_all(&reply.data).await.map_err(write_error)?;
    }
    if digest.clone().finalize().as_slice() != reference.sha256.as_slice() {
        return Err(invalid("Read Content: SHA-256 Mismatch"));
    }
    lease.guard(async {}).await.map_err(|error| DeliveryError {
        stage: DeliveryStage::Stream,
        failure: DeliveryFailure::Rpc(error.into()),
    })?;
    file.flush().await.map_err(write_error)?;
    file.sync_all().await.map_err(write_error)?;
    drop(file);
    let path = destination.to_path_buf();
    staged.persist_noclobber(&path).map_err(|error| DeliveryError::local(DeliveryStage::Rename, error.to_string()))?;
    Ok(VerifiedContent {
        path,
        size_bytes: offset,
        sha256: digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        content_type,
        report_origin,
    })
}

/* === Private Functions === */

fn invalid(message: impl Into<String>) -> DeliveryError {
    DeliveryError::local(DeliveryStage::Stream, message)
}

fn write_error(error: std::io::Error) -> DeliveryError {
    DeliveryError::local(DeliveryStage::Write, error.to_string())
}

/* === Tests === */
