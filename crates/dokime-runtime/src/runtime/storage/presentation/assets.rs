/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use base64::{Engine, engine::general_purpose::STANDARD};

use super::error;
use crate::{installed_plugins::RuntimeInstalledPluginAsset, runtime::Error, runtime::Result};

pub(in crate::runtime::storage) async fn capture_logo(asset: &RuntimeInstalledPluginAsset, maximum: u64) -> Result<String> {
    validate_mime(&asset.mime_type)?;
    let path = asset.path.clone();
    let mime_type = asset.mime_type.clone();
    tokio::task::spawn_blocking(move || {
        let maximum_bytes = usize::try_from(maximum).map_err(error)?;
        let raw = harmos::filesystem::input::read(&path, maximum_bytes).map_err(error)?;
        let before = std::fs::metadata(&path).map_err(error)?;
        validate_file(&before, maximum)?;
        if before.len() != raw.len() as u64 {
            return Err(error("appearance logo changed during capture"));
        }
        Ok(format!("data:{mime_type};base64,{}", STANDARD.encode(&raw)))
    })
    .await
    .map_err(|source| Error::Join { source })?
}

pub(super) fn validate_mime(mime: &str) -> Result<()> {
    if matches!(
        mime,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml" | "image/x-icon" | "image/vnd.microsoft.icon"
    ) {
        Ok(())
    } else {
        Err(error("unsupported appearance logo image MIME type"))
    }
}

pub(super) fn validate_file(metadata: &std::fs::Metadata, maximum: u64) -> Result<()> {
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum {
        Err(error("appearance logo must be a nonempty regular file within the captured byte limit"))
    } else {
        Ok(())
    }
}
