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

// Third Party Imports
use harmos::operations::StreamOutput;
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;
use tonic::Status;
// Local Imports
use super::super::chunked;
use super::{failure, source_budget};
use dokime_domain::operations::appearance as wire;
use dokime_domain::operations::content;
use dokime_runtime::{RuntimeHandle, domains::installed_plugins::RuntimeInstalledPluginAsset};

/* === Constants === */
const MAX_ASSET_BYTES: usize = 8 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Asset {
    pub metadata: dokime_domain::model::appearance::wire::AppearanceAssetMetadata,
    bytes: Vec<u8>,
}

struct AssetFrames(dokime_domain::model::appearance::wire::AppearanceAssetMetadata);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl chunked::Frames for AssetFrames {
    type Frame = dokime_domain::model::content::wire::AppearanceAssetFrame;

    fn metadata(&self) -> Self::Frame {
        dokime_domain::model::content::wire::AppearanceAssetFrame {
            payload: Some(dokime_domain::model::content::wire::AppearanceAssetFramePayload::Metadata(self.0.clone())),
        }
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        dokime_domain::model::content::wire::AppearanceAssetFrame {
            payload: Some(dokime_domain::model::content::wire::AppearanceAssetFramePayload::Chunk(
                dokime_domain::model::content::wire::ArtifactChunk { offset, data },
            )),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        dokime_domain::model::content::wire::AppearanceAssetFrame {
            payload: Some(dokime_domain::model::content::wire::AppearanceAssetFramePayload::Complete(
                dokime_domain::model::content::wire::ArtifactCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */
pub(super) async fn capture(source: RuntimeInstalledPluginAsset, plugin_id: String, asset_id: String) -> Result<Asset, Status> {
    if source.mime_type.len() > 128 || !source.mime_type.starts_with("image/") {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable));
    }
    // Registered sources are host-owned paths. Reject non-files before opening
    // them, then validate the opened handle again to avoid stale size checks.
    let initial = tokio::fs::metadata(&source.path).await.map_err(io_failure)?;
    if !initial.is_file() {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable));
    }
    if initial.len() > MAX_ASSET_BYTES as u64 {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
    }
    let file = tokio::fs::File::open(source.path).await.map_err(io_failure)?;
    let metadata = file.metadata().await.map_err(io_failure)?;
    if !metadata.is_file() {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable));
    }
    if metadata.len() > MAX_ASSET_BYTES as u64 {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((MAX_ASSET_BYTES + 1) as u64).read_to_end(&mut bytes).await.map_err(io_failure)?;
    if bytes.len() > MAX_ASSET_BYTES {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
    }
    let sha256 = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    Ok(Asset {
        metadata: dokime_domain::model::appearance::wire::AppearanceAssetMetadata {
            reference: Some(dokime_domain::model::appearance::wire::AppearanceAssetReference { plugin_id, asset_id, sha256 }),
            content_type: source.mime_type,
            size_bytes: bytes.len() as u64,
        },
        bytes,
    })
}

pub(crate) async fn download(
    runtime: &RuntimeHandle, request: content::DownloadAppearanceAsset, max_message_bytes: usize,
    output: &mut StreamOutput<dokime_domain::model::content::wire::AppearanceAssetFrame>,
) -> Result<(), Status> {
    let reference =
        request.reference.ok_or_else(|| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::InvalidReference))?;
    if reference.plugin_id.len() > 256
        || reference.asset_id.len() > 256
        || dokime_domain::model::identity::Identifier::new(&reference.plugin_id).is_err()
        || dokime_domain::model::identity::Identifier::new(&reference.asset_id).is_err()
        || reference.sha256.len() != 64
        || !reference.sha256.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::InvalidReference));
    }
    let captured = runtime
        .capture_appearance(source_budget(max_message_bytes))
        .map_err(|_| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge))?;
    let logo = captured.frame.logo.ok_or_else(|| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetMissing))?;
    if logo.plugin_id.as_str() != reference.plugin_id || logo.asset_id.as_str() != reference.asset_id {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetMissing));
    }
    let source = captured.logo.ok_or_else(|| failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetMissing))?;
    let asset = capture(source, reference.plugin_id.clone(), reference.asset_id.clone()).await?;
    if asset.metadata.reference.as_ref() != Some(&reference) {
        return Err(failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetChanged));
    }
    let sha256 = asset.metadata.reference.as_ref().expect("captured reference").sha256.clone();
    chunked::Download {
        size_bytes: asset.metadata.size_bytes,
        sha256: Some(sha256),
        frames: AssetFrames(asset.metadata),
        source: chunked::Source::bytes(asset.bytes),
        noun: "appearance asset",
        read_failure: "appearance asset content could not be read",
        chunk_bytes: (64 * 1024).min(max_message_bytes.saturating_sub(128)),
        message_limit: max_message_bytes,
    }
    .run(output)
    .await
    .map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            failure(dokime_domain::model::appearance::wire::AppearanceFailureKind::TooLarge)
        } else {
            source
        }
    })
}

/* === Private Functions === */
fn io_failure(error: std::io::Error) -> Status {
    failure(if error.kind() == std::io::ErrorKind::NotFound {
        dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetMissing
    } else {
        dokime_domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable
    })
}

/* === Tests === */
