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
use dokime_contract::operations::content as v1;

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

/// Where a native delivery failed, carrying the backend's own status when it answered.
///
/// Serialized to the webview as `{ stage, failure }`; `failure` is the same
/// `RpcError` shape generated RPC commands reject with, or a local message
/// when the failure never reached the backend.
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
    /// Session, cache or filename checks before any backend call.
    Prepare,
    /// The backend refused to open the download.
    Request,
    /// The download opened, then a frame failed or did not verify.
    Stream,
    /// Staging the received bytes on this machine failed.
    Write,
    /// Publishing the staged file under its final name failed.
    Rename,
    /// Handing the verified file to the operating system failed.
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

impl From<dokime_client::DownloadError> for DeliveryError {
    fn from(error: dokime_client::DownloadError) -> Self {
        use dokime_client::DownloadError as Download;
        let rpc = |stage: DeliveryStage, error: dokime_client::Error| Self {
            stage,
            failure: DeliveryFailure::Rpc(crate::Error::from(error)),
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
    fn from_download<T>(
        download: dokime_client::Download<T>,
        content_type: impl FnOnce(T) -> Option<String>,
    ) -> Self {
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
    lease: &SessionLease,
    source: &ArtifactSource,
    destination: &Path,
) -> Result<VerifiedContent, DeliveryError> {
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| DeliveryError::local(DeliveryStage::Write, error.to_string()))?;
    }
    let client = lease.client().with_frame_limit(
        std::num::NonZeroUsize::new(crate::rpc::MAX_FRAME_BYTES).expect("desktop frame limit"),
    );
    let destination = destination.to_path_buf();
    let result = match source {
        ArtifactSource::Artifact { artifact_id } => client
            .download_artifact(
                v1::DownloadArtifactRequest {
                    artifact_id: artifact_id.clone(),
                },
                destination,
            )
            .await
            .map(|download| {
                VerifiedContent::from_download(download, |metadata| metadata.content_type)
            }),
        ArtifactSource::ExecutionFile {
            execution_id,
            relative_path,
        } => client
            .download_execution_file(
                v1::DownloadExecutionFileRequest {
                    execution_id: execution_id.clone(),
                    relative_path: relative_path.clone(),
                },
                destination,
            )
            .await
            .map(|download| {
                VerifiedContent::from_download(download, |metadata| metadata.content_type)
            }),
        ArtifactSource::CycleBundle { cycle_id } => client
            .download_cycle_bundle(
                v1::DownloadCycleBundleRequest {
                    cycle_id: cycle_id.clone(),
                },
                destination,
            )
            .await
            .map(|download| {
                VerifiedContent::from_download(download, |metadata| Some(metadata.content_type))
            }),
    };
    result.map_err(DeliveryError::from)
}

pub(crate) async fn push_verified(
    lease: &SessionLease,
    source: &Path,
    header: v1::UploadArtifactHeader,
) -> Result<v1::ArtifactMetadata, String> {
    lease
        .client()
        .with_frame_limit(
            std::num::NonZeroUsize::new(crate::rpc::MAX_FRAME_BYTES).expect("desktop frame limit"),
        )
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
        let error = DeliveryError::from(dokime_client::DownloadError::Request(
            dokime_client::Error::Rpc(refusal),
        ));

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
        let staged = DeliveryError::from(dokime_client::DownloadError::Io(std::io::Error::other(
            "disk full",
        )));
        let invalid = DeliveryError::from(dokime_client::DownloadError::Invalid(
            "size or SHA-256 mismatch",
        ));

        assert_eq!(staged.stage, DeliveryStage::Write);
        assert_eq!(invalid.stage, DeliveryStage::Stream);
    }
}
