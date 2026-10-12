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
use sha2::{Digest, Sha256};
use tokio::{io::AsyncReadExt, sync::watch};
use tonic::Status;
// Local Imports
use super::super::chunked;
use super::{failure, source_budget};
use dokime_app::{RuntimeHandle, capabilities::installed_plugins::RuntimeInstalledPluginAsset};
use dokime_protocol::v1;

/* === Constants === */
const MAX_ASSET_BYTES: usize = 8 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */
pub(crate) type DownloadStream = chunked::FrameStream<v1::AppearanceAssetFrame>;

/* === Structs === */
pub(super) struct Asset {
    pub metadata: v1::AppearanceAssetMetadata,
    bytes: Vec<u8>,
}

struct AssetFrames(v1::AppearanceAssetMetadata);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl chunked::Frames for AssetFrames {
    type Frame = v1::AppearanceAssetFrame;

    fn metadata(&self) -> Self::Frame {
        v1::AppearanceAssetFrame {
            payload: Some(v1::appearance_asset_frame::Payload::Metadata(
                self.0.clone(),
            )),
        }
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        v1::AppearanceAssetFrame {
            payload: Some(v1::appearance_asset_frame::Payload::Chunk(
                v1::ArtifactChunk { offset, data },
            )),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        v1::AppearanceAssetFrame {
            payload: Some(v1::appearance_asset_frame::Payload::Complete(
                v1::ArtifactCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */
pub(super) async fn capture(
    source: RuntimeInstalledPluginAsset,
    plugin_id: String,
    asset_id: String,
) -> Result<Asset, Status> {
    if source.mime_type.len() > 128 || !source.mime_type.starts_with("image/") {
        return Err(failure(v1::AppearanceFailureKind::AssetUnavailable));
    }
    // Registered sources are host-owned paths. Reject non-files before opening
    // them, then validate the opened handle again to avoid stale size checks.
    let initial = tokio::fs::metadata(&source.path)
        .await
        .map_err(io_failure)?;
    if !initial.is_file() {
        return Err(failure(v1::AppearanceFailureKind::AssetUnavailable));
    }
    if initial.len() > MAX_ASSET_BYTES as u64 {
        return Err(failure(v1::AppearanceFailureKind::TooLarge));
    }
    let file = tokio::fs::File::open(source.path)
        .await
        .map_err(io_failure)?;
    let metadata = file.metadata().await.map_err(io_failure)?;
    if !metadata.is_file() {
        return Err(failure(v1::AppearanceFailureKind::AssetUnavailable));
    }
    if metadata.len() > MAX_ASSET_BYTES as u64 {
        return Err(failure(v1::AppearanceFailureKind::TooLarge));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((MAX_ASSET_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(io_failure)?;
    if bytes.len() > MAX_ASSET_BYTES {
        return Err(failure(v1::AppearanceFailureKind::TooLarge));
    }
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(Asset {
        metadata: v1::AppearanceAssetMetadata {
            reference: Some(v1::AppearanceAssetReference {
                plugin_id,
                asset_id,
                sha256,
            }),
            content_type: source.mime_type,
            size_bytes: bytes.len() as u64,
        },
        bytes,
    })
}

pub(crate) async fn download(
    runtime: &RuntimeHandle,
    request: v1::DownloadAppearanceAssetRequest,
    max_message_bytes: usize,
    stopping: watch::Receiver<bool>,
) -> Result<DownloadStream, Status> {
    if *stopping.borrow() {
        return Err(Status::unavailable("listener is stopping"));
    }
    let reference = request
        .reference
        .ok_or_else(|| failure(v1::AppearanceFailureKind::InvalidReference))?;
    if reference.plugin_id.len() > 256
        || reference.asset_id.len() > 256
        || dokime_app::core::Identifier::new(&reference.plugin_id).is_err()
        || dokime_app::core::Identifier::new(&reference.asset_id).is_err()
        || reference.sha256.len() != 64
        || !reference
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(failure(v1::AppearanceFailureKind::InvalidReference));
    }
    let captured = runtime
        .capture_appearance(source_budget(max_message_bytes))
        .map_err(|_| failure(v1::AppearanceFailureKind::TooLarge))?;
    let logo = captured
        .frame
        .logo
        .ok_or_else(|| failure(v1::AppearanceFailureKind::AssetMissing))?;
    if logo.plugin_id.as_str() != reference.plugin_id
        || logo.asset_id.as_str() != reference.asset_id
    {
        return Err(failure(v1::AppearanceFailureKind::AssetMissing));
    }
    let source = captured
        .logo
        .ok_or_else(|| failure(v1::AppearanceFailureKind::AssetMissing))?;
    let asset = capture(
        source,
        reference.plugin_id.clone(),
        reference.asset_id.clone(),
    )
    .await?;
    if asset.metadata.reference.as_ref() != Some(&reference) {
        return Err(failure(v1::AppearanceFailureKind::AssetChanged));
    }
    let sha256 = asset
        .metadata
        .reference
        .as_ref()
        .expect("captured reference")
        .sha256
        .clone();
    chunked::Download {
        size_bytes: asset.metadata.size_bytes,
        sha256: Some(sha256),
        frames: AssetFrames(asset.metadata),
        source: chunked::Source::bytes(asset.bytes),
        noun: "appearance asset",
        read_failure: "appearance asset content could not be read",
        chunk_bytes: (64 * 1024).min(max_message_bytes.saturating_sub(128)),
        message_limit: max_message_bytes,
        stopping,
    }
    .stream()
    // A listener that cannot carry the asset is an appearance failure, reported
    // with the typed detail every Appearance refusal carries.
    .map_err(|_| failure(v1::AppearanceFailureKind::TooLarge))
}

/* === Private Functions === */
fn io_failure(error: std::io::Error) -> Status {
    failure(if error.kind() == std::io::ErrorKind::NotFound {
        v1::AppearanceFailureKind::AssetMissing
    } else {
        v1::AppearanceFailureKind::AssetUnavailable
    })
}

/* === Tests === */
