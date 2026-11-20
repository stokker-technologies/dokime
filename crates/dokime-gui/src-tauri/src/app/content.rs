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
use std::{
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

// Third Party Imports
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

// Local Imports
use dokime_client_internal::{
    ContentArtifactsGetOptions, ContentExecutionFilesGetOptions, EvidenceArchiveOptions, FilesGetOptions, FilesReadOptions,
    ReportsGetOptions,
};

use super::desktop::ArtifactSource;
use crate::connection::{SessionLease, host};
use dokime_client_internal::EvidenceSubject;
use dokime_client_internal::ReportOrigin;
use dokime_client_internal::ReportSubject;
use dokime_client_internal::{evidence_subject, report_subject};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug)]
pub(crate) struct VerifiedContent {
    pub path: PathBuf,

    pub content_type: Option<String>,

    pub report_origin: Option<ReportOrigin>,
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

impl From<host::Error> for DeliveryError {
    fn from(error: host::Error) -> Self {
        Self {
            stage: DeliveryStage::Request,
            failure: DeliveryFailure::Rpc(error.into()),
        }
    }
}

impl VerifiedContent {
    pub(crate) fn report_origin(&self) -> Option<&'static str> {
        match self.report_origin {
            Some(ReportOrigin::Sealed) => Some("sealed"),
            Some(ReportOrigin::Current) => Some("current"),
            _ => None,
        }
    }
}

/* === Public Functions === */

pub(crate) async fn fetch_verified(
    lease: &SessionLease, source: &ArtifactSource, destination: &Path, frame_bytes: usize,
) -> Result<VerifiedContent, DeliveryError> {
    let session = lease.client();
    let client = session.client();
    let started = Instant::now();
    let (reference, report_origin) = match source {
        ArtifactSource::Artifact { artifact_id } => {
            let options = ContentArtifactsGetOptions {
                context: session.call_options()?,
                ..Default::default()
            };
            let reply = session.execute("Content.Artifacts.Get", client.content.artifacts.get_with(artifact_id.clone(), options)).await?;
            (reply.file, None)
        }
        ArtifactSource::ExecutionFile { execution_id, relative_path } => {
            let options = ContentExecutionFilesGetOptions {
                context: session.call_options()?,
                ..Default::default()
            };
            let reply = session
                .execute(
                    "content.execution_files.get",
                    client.content.execution_files.get_with(execution_id.clone(), relative_path.clone(), options),
                )
                .await?;
            (reply.file, None)
        }
        ArtifactSource::ExecutionEvidence { execution_id } => {
            let subject = EvidenceSubject {
                kind: Some(evidence_subject::Kind::Execution(execution_id.clone())),
            };
            let options = EvidenceArchiveOptions {
                context: session.call_options()?,
                ..Default::default()
            };
            let reply = session.execute("Evidence.Archive", client.evidence.archive_with(subject, options)).await?;
            (reply.file, None)
        }
        ArtifactSource::CycleEvidence { cycle_id } => {
            let subject = EvidenceSubject {
                kind: Some(evidence_subject::Kind::Cycle(cycle_id.clone())),
            };
            let options = EvidenceArchiveOptions {
                context: session.call_options()?,
                ..Default::default()
            };
            let reply = session.execute("Evidence.Archive", client.evidence.archive_with(subject, options)).await?;
            (reply.file, None)
        }
        ArtifactSource::CycleReport { cycle_id, name } => {
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Cycle(cycle_id.clone())),
            };
            let options = ReportsGetOptions {
                context: session.call_options()?,
                ..Default::default()
            };
            let reply = session.execute("Reports.Get", client.reports.get_with(subject, name.clone(), options)).await?;
            (reply.file, reply.report.and_then(|report| ReportOrigin::try_from(report.origin).ok()))
        }
        ArtifactSource::ExecutionReport { execution_id, name } => {
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Execution(execution_id.clone())),
            };
            let options = ReportsGetOptions {
                context: session.call_options()?,
                ..Default::default()
            };
            let reply = session.execute("Reports.Get", client.reports.get_with(subject, name.clone(), options)).await?;
            (reply.file, reply.report.and_then(|report| ReportOrigin::try_from(report.origin).ok()))
        }
    };
    let reference = reference.ok_or_else(|| invalid("Read Content: Host Omitted Its File Reference"))?;
    if reference.sha256.len() != 32 {
        return Err(invalid("Read Content: Host Omitted Its SHA-256"));
    }
    let options = FilesGetOptions {
        context: session.call_options()?,
        ..Default::default()
    };
    let selected = session.execute("Files.Get", client.files.get_with(reference.clone(), options)).await?;
    let selected_file = selected.file.as_ref().ok_or_else(|| invalid("Read Content: Host Omitted Its Selected File"))?;
    if selected_file.length != reference.length || selected_file.sha256 != reference.sha256 {
        return Err(invalid("Read Content: Selected File Identity Changed"));
    }
    let frame_bytes = frame_bytes
        .saturating_sub(1024)
        .min(usize::try_from(selected.chunk_bytes).map_err(|_| invalid("Read Content: Invalid Chunk Limit"))?)
        .min(usize::try_from(selected.message_bytes.saturating_sub(1024)).map_err(|_| invalid("Read Content: Invalid Message Limit"))?);
    if frame_bytes == 0 {
        return Err(invalid("Read Content: Host Omitted Its Transfer Limits"));
    }
    let content_type = reference.metadata.as_ref().map(|metadata| metadata.media_type.clone());
    let request_ms = started.elapsed().as_secs_f64() * 1000.0;
    let started = Instant::now();
    let directory =
        destination.parent().ok_or_else(|| DeliveryError::local(DeliveryStage::Prepare, "Select a destination with a parent directory"))?;
    fs::create_dir_all(directory).await.map_err(write_error)?;
    let staged = NamedTempFile::new_in(directory).map_err(write_error)?;
    let mut file = File::from_std(staged.reopen().map_err(write_error)?);
    let mut digest = Sha256::new();
    let mut offset = 0;
    let mut at = selected.resolved_at;
    let mut transfer = Duration::ZERO;
    while offset < reference.length {
        let options = FilesReadOptions {
            at: at.clone(),
            context: session.call_options()?,
        };
        let reading = Instant::now();
        let reply = session.execute("Files.Read", client.files.read_with(reference.clone(), offset, frame_bytes as u64, options)).await?;
        transfer += reading.elapsed();
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
    if digest.finalize().as_slice() != reference.sha256.as_slice() {
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
    tracing::info!(
        file = %reference.metadata.as_ref().map(|metadata| metadata.id.as_str()).unwrap_or_default(),
        request_ms,
        transfer_ms = transfer.as_secs_f64() * 1000.0,
        write_ms = started.elapsed().saturating_sub(transfer).as_secs_f64() * 1000.0,
        bytes = reference.length,
        "Content Delivered"
    );
    Ok(VerifiedContent { path, content_type, report_origin })
}

/* === Private Functions === */

fn invalid(message: impl Into<String>) -> DeliveryError {
    DeliveryError::local(DeliveryStage::Stream, message)
}

fn write_error(error: io::Error) -> DeliveryError {
    DeliveryError::local(DeliveryStage::Write, error.to_string())
}

/* === Tests === */
