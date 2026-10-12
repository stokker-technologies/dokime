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
use dokime_protocol::v1;

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl VerifiedContent {
    fn from_download<T>(
        download: dokime::client::Download<T>,
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
) -> Result<VerifiedContent, String> {
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
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
    result.map_err(|error| error.to_string())
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
