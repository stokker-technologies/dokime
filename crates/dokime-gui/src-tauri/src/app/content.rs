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
use dokime::domain::operations::content as v1;

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

    pub report_origin: Option<dokime::domain::model::reports::wire::ReportOrigin>,
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
        Self { stage, failure: DeliveryFailure::Local(message.into()) }
    }
}

impl From<dokime::client::DownloadError> for DeliveryError {
    fn from(error: dokime::client::DownloadError) -> Self {
        use dokime::client::DownloadError as Download;
        let rpc = |stage: DeliveryStage, error: dokime::client::Error| {
            let error = crate::Error::from(error);
            Self { stage, failure: DeliveryFailure::Rpc(error) }
        };
        match error {
            Download::Request(error) => rpc(DeliveryStage::Request, error),
            Download::Client(error) => rpc(DeliveryStage::Stream, error),
            Download::Invalid(reason) => Self::local(DeliveryStage::Stream, reason),
            Download::Io(error) => Self::local(DeliveryStage::Write, error.to_string()),
            Download::Receipt(error) => Self::local(DeliveryStage::Write, error.to_string()),
            Download::Publication(error) => Self::local(DeliveryStage::Rename, error.to_string()),
            Download::Task(error) => Self::local(DeliveryStage::Rename, error.to_string()),
        }
    }
}

impl VerifiedContent {
    pub(crate) fn report_origin(&self) -> Option<&'static str> {
        use dokime::domain::model::reports::wire::ReportOrigin;
        self.report_origin.as_ref().map(|origin| match origin {
            ReportOrigin::Sealed => "sealed",
            ReportOrigin::Current => "current",
        })
    }

    fn from_report(download: dokime::client::Download<dokime::domain::model::reports::wire::ReportMetadata>) -> Self {
        let origin = download.metadata.origin;
        Self {
            path: download.path,
            size_bytes: download.size_bytes,
            sha256: download.sha256,
            content_type: Some("text/html".into()),
            report_origin: Some(origin),
        }
    }

    fn from_download<T>(download: dokime::client::Download<T>, content_type: impl FnOnce(T) -> Option<String>) -> Self {
        Self {
            path: download.path,
            size_bytes: download.size_bytes,
            sha256: download.sha256,
            content_type: content_type(download.metadata),
            report_origin: None,
        }
    }
}

/* === Public Functions === */

pub(crate) async fn fetch_verified(
    lease: &SessionLease, source: &ArtifactSource, destination: &Path, frame_bytes: usize,
) -> Result<VerifiedContent, DeliveryError> {
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|error| {
            let message = error.to_string();
            DeliveryError::local(DeliveryStage::Write, message)
        })?;
    }
    let frame_limit = std::num::NonZeroUsize::new(frame_bytes).expect("desktop frame limit");
    let client = lease.client().with_frame_limit(frame_limit);
    let client = if matches!(source, ArtifactSource::CycleReport { .. } | ArtifactSource::ExecutionReport { .. }) {
        client.with_timeout(std::time::Duration::from_secs(300))
    } else {
        client
    };
    let destination = destination.to_path_buf();
    let result = match source {
        ArtifactSource::Artifact { artifact_id } => {
            let request = v1::DownloadArtifact { artifact_id: artifact_id.clone() };
            let download = client.download_artifact(request, destination).await?;
            let content = VerifiedContent::from_download(download, |metadata| metadata.content_type);
            Ok(content)
        }
        ArtifactSource::ExecutionFile { execution_id, relative_path } => {
            let request = v1::DownloadExecutionFile { execution_id: execution_id.clone(), relative_path: relative_path.clone() };
            let download = client.download_execution_file(request, destination).await?;
            let content = VerifiedContent::from_download(download, |metadata| metadata.content_type);
            Ok(content)
        }
        ArtifactSource::ExecutionEvidence { execution_id } => {
            let subject = dokime::domain::model::exports::EvidenceSubject::Execution(execution_id.clone());
            let request = v1::DownloadEvidence { subject: Some(subject) };
            let download = client.download_evidence(request, destination, false).await?;
            let content_type = Some("application/zip".into());
            let content = VerifiedContent::from_download(download, |_| content_type);
            Ok(content)
        }
        ArtifactSource::CycleEvidence { cycle_id } => {
            let subject = dokime::domain::model::exports::EvidenceSubject::Cycle(cycle_id.clone());
            let request = v1::DownloadEvidence { subject: Some(subject) };
            let download = client.download_evidence(request, destination, false).await?;
            let content_type = Some("application/zip".into());
            let content = VerifiedContent::from_download(download, |_| content_type);
            Ok(content)
        }
        ArtifactSource::CycleReport { cycle_id } => client
            .download_cycle_report(
                dokime::domain::operations::reports::DownloadCycleReport { cycle_id: cycle_id.clone(), identifier: CYCLE_REPORT.into() },
                destination,
            )
            .await
            .map(VerifiedContent::from_report),
        ArtifactSource::ExecutionReport { execution_id } => client
            .download_execution_report(
                dokime::domain::operations::reports::DownloadExecutionReport { execution_id: execution_id.clone() },
                destination,
            )
            .await
            .map(VerifiedContent::from_report),
    };
    result.map_err(DeliveryError::from)
}

pub(crate) async fn push_verified(
    lease: &SessionLease, source: &Path, header: dokime::domain::model::content::wire::UploadArtifactHeader, frame_bytes: usize,
) -> Result<dokime::domain::model::content::wire::ArtifactMetadata, String> {
    let frame_limit = std::num::NonZeroUsize::new(frame_bytes).expect("desktop frame limit");
    let client = lease.client().with_frame_limit(frame_limit);
    let result = client.upload_artifact_file(source, header).await;
    match result {
        Ok(metadata) => Ok(metadata),
        Err(error) => Err(error.to_string()),
    }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refused_request_keeps_backend_status() {
        let refusal = tonic::Status::failed_precondition(
            "The evidence bundle becomes available when the cycle is terminal; this cycle is still running.",
        );
        let refusal = dokime::client::Error::Rpc(refusal);
        let refusal = dokime::client::DownloadError::Request(refusal);
        let error = DeliveryError::from(refusal);

        let wire = serde_json::to_value(&error).unwrap();

        assert_eq!(
            wire,
            serde_json::json!({
                "stage": "request",
                "failure": {
                    "kind": "rpc",
                    "detail": {
                        "kind": "grpc",
                        "detail": {
                            "code": 9,
                            "message": "The evidence bundle becomes available when the cycle is terminal; this cycle is still running.",
                            "details": [],
                        },
                    },
                },
            })
        );
    }

    #[test]
    fn local_failures_name_their_stage() {
        let write_error = std::io::Error::other("disk full");
        let staged = DeliveryError::from(dokime::client::DownloadError::Io(write_error));
        let invalid = DeliveryError::from(dokime::client::DownloadError::Invalid("size or SHA-256 mismatch"));

        assert_eq!(staged.stage, DeliveryStage::Write);
        assert_eq!(invalid.stage, DeliveryStage::Stream);
    }
}
