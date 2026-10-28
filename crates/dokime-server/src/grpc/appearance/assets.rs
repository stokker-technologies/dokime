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
use std::fmt::Write as _;

// Third Party Imports
use dokime::harmos::operations::{Response, StreamOutput};
use dokime::harmos::storage::{ByteRange, ReadBatch};
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use super::super::chunked;
use super::{failure, source_budget};
use dokime::domain::operations::content;
use dokime::{RuntimeHandle, domains::installed_plugins::RuntimeInstalledPluginAsset};

/* === Constants === */

const SERVER_APPEARANCE_ASSET_BYTES: u64 = 8_388_608;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Asset {
    pub metadata: ReadBatch<dokime::domain::model::appearance::wire::AppearanceAssetMetadata>,
    bytes: ReadBatch<Vec<u8>>,
}

struct AssetFrames {
    metadata: Response<dokime::domain::model::content::wire::AppearanceAssetFrame>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl chunked::Frames for AssetFrames {
    type Frame = dokime::domain::model::content::wire::AppearanceAssetFrame;

    fn metadata(&self) -> Self::Frame {
        (*self.metadata).clone()
    }

    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.metadata.clone())
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        dokime::domain::model::content::wire::AppearanceAssetFrame {
            payload: Some(dokime::domain::model::content::wire::AppearanceAssetFramePayload::Chunk(
                dokime::domain::model::content::wire::ArtifactChunk { offset, data },
            )),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        dokime::domain::model::content::wire::AppearanceAssetFrame {
            payload: Some(dokime::domain::model::content::wire::AppearanceAssetFramePayload::Complete(
                dokime::domain::model::content::wire::ArtifactCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */

pub(super) async fn capture(
    session: &dokime::storage::Manager, source: &RuntimeInstalledPluginAsset, plugin_id: &str, asset_id: &str, maximum_bytes: usize,
) -> Result<Asset, Status> {
    if !source.mime_type.starts_with("image/") {
        return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable));
    }
    // Registered sources are host-owned paths. Reject non-files before opening
    // them, then validate the opened handle again to avoid stale size checks.
    let initial = tokio::fs::metadata(&source.path).await.map_err(io_failure)?;
    if !initial.is_file() {
        return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable));
    }
    if initial.len() > maximum_bytes as u64 {
        return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
    }
    let file = tokio::fs::File::open(&source.path).await.map_err(io_failure)?;
    let metadata = file.metadata().await.map_err(io_failure)?;
    if !metadata.is_file() {
        return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable));
    }
    if metadata.len() > maximum_bytes as u64 {
        return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge));
    }
    let file = file.into_std().await;
    let mut reader = session.read_opened_file(file, ByteRange { offset: 0, limit: None }).await.map_err(native_failure)?;
    let first = reader.next().await.map_err(native_failure)?;
    let mut bytes = match first {
        Some(batch) if batch.bytes().len() <= maximum_bytes => batch.try_map(|batch| Ok(batch.bytes)),
        Some(_) => return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge)),
        None => dokime::harmos::storage::ReadBatch::new(()).try_map(|_| Ok(Vec::new())),
    }
    .map_err(native_failure)?;
    while let Some(batch) = reader.next().await.map_err(native_failure)? {
        let length = bytes
            .len()
            .checked_add(batch.bytes().len())
            .filter(|length| *length <= maximum_bytes)
            .ok_or_else(|| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge))?;
        bytes = bytes
            .try_map(|mut bytes| {
                bytes.try_reserve_exact(length - bytes.len()).map_err(|_| allocation())?;
                bytes.extend_from_slice(batch.bytes());
                Ok(bytes)
            })
            .map_err(native_failure)?;
    }
    reader.completion().map_err(native_failure)?;

    let metadata = dokime::harmos::storage::ReadBatch::new(())
        .try_map(|_| {
            let hash = Sha256::digest(bytes.as_slice());
            let mut sha256 = String::new();
            sha256.try_reserve_exact(64).map_err(|_| allocation())?;
            for byte in hash {
                write!(&mut sha256, "{byte:02x}").map_err(|_| allocation())?;
            }
            Ok(dokime::domain::model::appearance::wire::AppearanceAssetMetadata {
                reference: Some(dokime::domain::model::appearance::wire::AppearanceAssetReference {
                    plugin_id: plugin_id.to_owned(),
                    asset_id: asset_id.to_owned(),
                    sha256,
                }),
                content_type: source.mime_type.clone(),
                size_bytes: bytes.len() as u64,
            })
        })
        .map_err(native_failure)?;
    Ok(Asset { metadata, bytes })
}

pub(crate) async fn download(
    runtime: &RuntimeHandle, request: content::DownloadAppearanceAsset, max_message_bytes: usize,
    output: &mut StreamOutput<dokime::domain::model::content::wire::AppearanceAssetFrame>,
) -> Result<(), Status> {
    let reference =
        request.reference.ok_or_else(|| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::InvalidReference))?;
    if dokime::domain::model::identity::Identifier::new(&reference.plugin_id).is_err()
        || dokime::domain::model::identity::Identifier::new(&reference.asset_id).is_err()
        || reference.sha256.len() != 64
        || !reference.sha256.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::InvalidReference));
    }
    let session = runtime.storage().clone();
    let captured = runtime
        .capture_appearance(source_budget(max_message_bytes))
        .map_err(|_| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge))?;
    let source = captured
        .frame
        .logos
        .iter()
        .zip(captured.logos.iter())
        .find(|(logo, _)| logo.plugin_id.as_str() == reference.plugin_id && logo.asset_id.as_str() == reference.asset_id)
        .and_then(|(_, source)| source.as_ref())
        .ok_or_else(|| failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetMissing))?;
    let asset = capture(
        &session,
        source,
        &reference.plugin_id,
        &reference.asset_id,
        usize::try_from(SERVER_APPEARANCE_ASSET_BYTES).expect("validated appearance asset size"),
    )
    .await?;
    drop(captured);
    if asset.metadata.reference.as_ref() != Some(&reference) {
        return Err(failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetChanged));
    }
    let sha256 = asset.metadata.reference.as_ref().expect("captured reference").sha256.clone();
    let size_bytes = asset.metadata.size_bytes;
    let metadata = asset
        .metadata
        .try_map(|metadata| {
            Ok(dokime::domain::model::content::wire::AppearanceAssetFrame {
                payload: Some(dokime::domain::model::content::wire::AppearanceAssetFramePayload::Metadata(metadata)),
            })
        })
        .map_err(native_failure)?;
    let frames = AssetFrames {
        metadata: Response::from_owner(metadata.owner()),
    };
    chunked::Download {
        size_bytes,
        sha256: Some(sha256),
        frames,
        source: chunked::Source::bytes(Response::from_owner(asset.bytes.owner())),
        noun: "appearance asset",
        read_failure: "appearance asset content could not be read",
        chunk_bytes: super::super::content::download_chunk_bytes(runtime, max_message_bytes)?,
        message_limit: max_message_bytes,
    }
    .run(output)
    .await
    .map_err(|source| {
        if source.code() == tonic::Code::ResourceExhausted {
            failure(dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge)
        } else {
            source
        }
    })
}

/* === Private Functions === */

fn allocation() -> dokime::harmos::storage::Error {
    dokime::harmos::storage::Error::ResourceExhausted {
        resource: "appearance asset construction",
    }
}

pub(super) fn native_failure(error: dokime::harmos::storage::Error) -> Status {
    let kind = if matches!(error, dokime::harmos::storage::Error::ResourceExhausted { .. }) {
        dokime::domain::model::appearance::wire::AppearanceFailureKind::TooLarge
    } else {
        dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable
    };
    failure(kind)
}

fn io_failure(error: std::io::Error) -> Status {
    failure(if error.kind() == std::io::ErrorKind::NotFound {
        dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetMissing
    } else {
        dokime::domain::model::appearance::wire::AppearanceFailureKind::AssetUnavailable
    })
}

/* === Tests === */
