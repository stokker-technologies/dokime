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
use base64::{Engine, engine::general_purpose::STANDARD};

// Local Imports
use super::error;
use crate::{installed_plugins::RuntimeInstalledPluginAsset, runtime::Error, runtime::Result};

/* === Constants === */
const MAX_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Reads an appearance logo into a data URL, refusing a file that changes meanwhile.
pub(in crate::runtime::storage) async fn capture_logo(asset: &RuntimeInstalledPluginAsset) -> Result<String> {
    validate_mime(&asset.mime_type)?;
    let path = asset.path.clone();
    let mime_type = asset.mime_type.clone();
    tokio::task::spawn_blocking(move || {
        let raw = std::fs::read(&path).map_err(error)?;
        let before = std::fs::metadata(&path).map_err(error)?;
        validate_file(&before)?;
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

pub(super) fn validate_file(metadata: &std::fs::Metadata) -> Result<()> {
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_BYTES as u64 {
        Err(error("appearance logo must be a regular file containing 1 byte to 1 MiB"))
    } else {
        Ok(())
    }
}

/* === Private Functions === */

/* === Tests === */
