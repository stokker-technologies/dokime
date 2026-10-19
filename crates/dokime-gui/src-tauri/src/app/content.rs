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
use dokime_domain::operations::content as v1;

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

impl From<dokime_client::DownloadError> for DeliveryError {
    fn from(error: dokime_client::DownloadError) -> Self {
        use dokime_client::DownloadError as Download;
        let rpc =
            |stage: DeliveryStage, error: dokime_client::Error| Self { stage, failure: DeliveryFailure::Rpc(crate::Error::from(error)) };
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
    fn from_download<T>(download: dokime_client::Download<T>, content_type: impl FnOnce(T) -> Option<String>) -> Self {
        Self {
            path: download.path,
            size_bytes: download.size_bytes,
            sha256: download.sha256,
            content_type: content_type(download.metadata),
        }
    }
}

/* === Public Functions === */

pub(crate) async fn fetch_verified(
    lease: &SessionLease, source: &ArtifactSource, destination: &Path,
) -> Result<VerifiedContent, DeliveryError> {
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|error| DeliveryError::local(DeliveryStage::Write, error.to_string()))?;
    }
    let client = lease.client().with_frame_limit(std::num::NonZeroUsize::new(crate::rpc::MAX_FRAME_BYTES).expect("desktop frame limit"));
    let destination = destination.to_path_buf();
    let result = match source {
        ArtifactSource::Artifact { artifact_id } => client
            .download_artifact(v1::DownloadArtifact { artifact_id: artifact_id.clone() }, destination)
            .await
            .map(|download| VerifiedContent::from_download(download, |metadata| metadata.content_type)),
        ArtifactSource::ExecutionFile { execution_id, relative_path } => client
            .download_execution_file(
                v1::DownloadExecutionFile { execution_id: execution_id.clone(), relative_path: relative_path.clone() },
                destination,
            )
            .await
            .map(|download| VerifiedContent::from_download(download, |metadata| metadata.content_type)),
        ArtifactSource::ExecutionEvidence { execution_id } => client
            .download_evidence(
                v1::DownloadEvidence { subject: Some(dokime_domain::model::exports::EvidenceSubject::Execution(execution_id.clone())) },
                destination,
                false,
            )
            .await
            .map(|download| VerifiedContent::from_download(download, |_| Some("application/zip".into()))),
        ArtifactSource::CycleEvidence { cycle_id } => client
            .download_evidence(
                v1::DownloadEvidence { subject: Some(dokime_domain::model::exports::EvidenceSubject::Cycle(cycle_id.clone())) },
                destination,
                false,
            )
            .await
            .map(|download| VerifiedContent::from_download(download, |_| Some("application/zip".into()))),
    };
    result.map_err(DeliveryError::from)
}

pub(crate) async fn push_verified(
    lease: &SessionLease, source: &Path, header: dokime_domain::model::content::wire::UploadArtifactHeader,
) -> Result<dokime_domain::model::content::wire::ArtifactMetadata, String> {
    lease
        .client()
        .with_frame_limit(std::num::NonZeroUsize::new(crate::rpc::MAX_FRAME_BYTES).expect("desktop frame limit"))
        .upload_artifact_file(source, header)
        .await
        .map_err(|error| error.to_string())
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
        let error = DeliveryError::from(dokime_client::DownloadError::Request(dokime_client::Error::Rpc(refusal)));

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
        let staged = DeliveryError::from(dokime_client::DownloadError::Io(std::io::Error::other("disk full")));
        let invalid = DeliveryError::from(dokime_client::DownloadError::Invalid("size or SHA-256 mismatch"));

        assert_eq!(staged.stage, DeliveryStage::Write);
        assert_eq!(invalid.stage, DeliveryStage::Stream);
    }
}
