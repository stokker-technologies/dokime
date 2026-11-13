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
use dokime_sdk::v1;
use prost::Message;
use prost_reflect::{DynamicMessage, Value};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

/* === Constants === */

const CYCLE_REPORT: &str = "cycle.summary";

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

    pub report_origin: Option<v1::ReportOrigin>,
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
            stage: DeliveryStage::Stream,
            failure: DeliveryFailure::Rpc(error.into()),
        }
    }
}

impl VerifiedContent {
    pub(crate) fn report_origin(&self) -> Option<&'static str> {
        match self.report_origin {
            Some(v1::ReportOrigin::Sealed) => Some("sealed"),
            Some(v1::ReportOrigin::Current) => Some("current"),
            _ => None,
        }
    }
}

/* === Public Functions === */

pub(crate) async fn fetch_verified(
    lease: &SessionLease, source: &ArtifactSource, destination: &Path, frame_bytes: usize,
) -> Result<VerifiedContent, DeliveryError> {
    let (path, request, output) = match source {
        ArtifactSource::Artifact { artifact_id } => (
            "/dokime.v1.Content/DownloadArtifact",
            v1::ContentDownloadArtifactRequest { artifact_id: artifact_id.clone() }.encode_to_vec(),
            "ContentDownloadArtifactOutput",
        ),
        ArtifactSource::ExecutionFile { execution_id, relative_path } => (
            "/dokime.v1.Content/DownloadExecutionFile",
            v1::ContentDownloadExecutionFileRequest {
                execution_id: execution_id.clone(),
                relative_path: relative_path.clone(),
            }
            .encode_to_vec(),
            "ContentDownloadExecutionFileOutput",
        ),
        ArtifactSource::ExecutionEvidence { execution_id } => (
            "/dokime.v1.Content/DownloadEvidence",
            v1::ContentDownloadEvidenceRequest {
                subject: Some(v1::EvidenceSubject {
                    kind: Some(v1::evidence_subject::Kind::Execution(execution_id.clone())),
                }),
            }
            .encode_to_vec(),
            "ContentDownloadEvidenceOutput",
        ),
        ArtifactSource::CycleEvidence { cycle_id } => (
            "/dokime.v1.Content/DownloadEvidence",
            v1::ContentDownloadEvidenceRequest {
                subject: Some(v1::EvidenceSubject {
                    kind: Some(v1::evidence_subject::Kind::Cycle(cycle_id.clone())),
                }),
            }
            .encode_to_vec(),
            "ContentDownloadEvidenceOutput",
        ),
        ArtifactSource::CycleReport { cycle_id } => (
            "/dokime.v1.Reports/DownloadCycleReport",
            v1::ReportsDownloadCycleReportRequest {
                cycle_id: cycle_id.clone(),
                identifier: CYCLE_REPORT.into(),
            }
            .encode_to_vec(),
            "ReportsDownloadCycleReportOutput",
        ),
        ArtifactSource::ExecutionReport { execution_id } => (
            "/dokime.v1.Reports/DownloadExecutionReport",
            v1::ReportsDownloadExecutionReportRequest { execution_id: execution_id.clone() }.encode_to_vec(),
            "ReportsDownloadExecutionReportOutput",
        ),
    };
    let session = lease.client();
    let mut watch = session.watch_raw(path, &request).await.map_err(|error| DeliveryError {
        stage: DeliveryStage::Request,
        failure: DeliveryFailure::Rpc(error.into()),
    })?;
    let descriptor =
        dokime_protocol::descriptor_pool().get_message_by_name(&format!("dokime.v1.{output}")).expect("canonical transfer output");
    let directory =
        destination.parent().ok_or_else(|| DeliveryError::local(DeliveryStage::Prepare, "Select a destination with a parent directory"))?;
    tokio::fs::create_dir_all(directory).await.map_err(write_error)?;
    let staged = tempfile::NamedTempFile::new_in(directory).map_err(write_error)?;
    let mut file = tokio::fs::File::from_std(staged.reopen().map_err(write_error)?);
    let mut digest = Sha256::new();
    let mut offset = 0;
    let mut metadata = false;
    let mut complete = false;
    let mut response = false;
    let mut content_type = None;
    let mut report_origin = None;
    let mut declared_size = None;
    let mut declared_digest = None;
    while let Some(bytes) = watch.next().await.transpose()? {
        if bytes.len() > frame_bytes || response {
            return Err(invalid("Transfer returned an oversized frame or data after its final response"));
        }
        let message = DynamicMessage::decode(descriptor.clone(), bytes.as_slice()).map_err(|error| invalid(error.to_string()))?;
        if message.has_field_by_name("response") {
            if !complete {
                return Err(invalid("Transfer completed before verified content"));
            }
            response = true;
            continue;
        }
        if complete {
            return Err(invalid("Transfer returned content after completion"));
        }
        let item = message.get_field_by_name("item").ok_or_else(|| invalid("Transfer omitted its item"))?;
        let Value::Message(item) = item.as_ref() else {
            return Err(invalid("Transfer item is not a message"));
        };
        let payload = item.get_field_by_name("payload").ok_or_else(|| invalid("Transfer omitted its payload"))?;
        let Value::Message(payload) = payload.as_ref() else {
            return Err(invalid("Transfer payload is not a message"));
        };
        if payload.has_field_by_name("metadata") || payload.has_field_by_name("report") {
            if metadata || offset != 0 {
                return Err(invalid("Transfer repeated metadata"));
            }
            metadata = true;
            let value =
                payload.get_field_by_name(if payload.has_field_by_name("report") { "report" } else { "metadata" }).expect("metadata field");
            if let Value::Message(value) = value.as_ref() {
                declared_size = value.get_field_by_name("size_bytes").and_then(|value| value.as_u64());
                declared_digest = value.get_field_by_name("sha256").and_then(|value| value.as_str().map(str::to_owned));
                content_type = value.get_field_by_name("content_type").and_then(|value| value.as_str().map(str::to_owned));
                report_origin = value
                    .get_field_by_name("origin")
                    .and_then(|value| value.as_enum_number())
                    .and_then(|value| v1::ReportOrigin::try_from(value).ok());
                if report_origin.is_some() {
                    content_type = Some("text/html".into());
                }
            }
            continue;
        }
        if !metadata {
            return Err(invalid("Transfer omitted initial metadata"));
        }
        if payload.has_field_by_name("chunk") {
            let value = payload.get_field_by_name("chunk").expect("chunk field");
            let Value::Message(chunk) = value.as_ref() else {
                return Err(invalid("Invalid chunk"));
            };
            let position =
                chunk.get_field_by_name("offset").and_then(|value| value.as_u64()).ok_or_else(|| invalid("Chunk omitted its offset"))?;
            let data = chunk.get_field_by_name("data").ok_or_else(|| invalid("Chunk omitted its bytes"))?;
            let data = data.as_bytes().ok_or_else(|| invalid("Invalid chunk bytes"))?;
            if position != offset {
                return Err(invalid("Transfer chunk offset did not advance contiguously"));
            }
            offset = offset.checked_add(data.len() as u64).ok_or_else(|| invalid("Transfer size overflow"))?;
            digest.update(data);
            file.write_all(data).await.map_err(write_error)?;
            continue;
        }
        if payload.has_field_by_name("complete") {
            let value = payload.get_field_by_name("complete").expect("completion field");
            let Value::Message(value) = value.as_ref() else {
                return Err(invalid("Invalid completion"));
            };
            let size = value.get_field_by_name("size_bytes").and_then(|value| value.as_u64());
            let expected = value.get_field_by_name("sha256").and_then(|value| value.as_str().map(str::to_owned));
            let actual = digest.clone().finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            if size != Some(offset)
                || expected.as_deref() != Some(actual.as_str())
                || declared_size.is_some_and(|size| size != offset)
                || declared_digest.as_deref().is_some_and(|digest| digest != actual)
            {
                return Err(invalid("Transfer size or SHA-256 mismatch"));
            }
            complete = true;
            continue;
        }
        return Err(invalid("Transfer returned an unknown frame"));
    }
    if !response {
        lease.guard(async {}).await.map_err(|error| DeliveryError {
            stage: DeliveryStage::Stream,
            failure: DeliveryFailure::Rpc(error.into()),
        })?;
        return Err(invalid("Transfer ended without a final response"));
    }
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
