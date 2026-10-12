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
use std::{io::Read, num::NonZeroUsize, path::PathBuf};

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD};
use harmos::{
    MemorySize,
    storage::{QueryValue, SharedQueryValue, StorageProductionComputation},
};

// Local Imports
use super::error;
use crate::{Error, Result, installed_plugins::RuntimeInstalledPluginAsset};
use dokime::storage::Manager;

/* === Constants === */
const MAX_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Source {
    path: PathBuf,
    mime_type: String,
}

struct CapturedLogo {
    result: harmos::Result<QueryValue<String>>,
    _source: SharedQueryValue<Source>,
    _computation: StorageProductionComputation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for Source {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.path.capacity())
            .saturating_add(self.mime_type.capacity())
    }
}

/* === Public Functions === */
pub(in crate::runtime::storage) async fn capture_logo(
    storage: &Manager,
    asset: &RuntimeInstalledPluginAsset,
) -> Result<QueryValue<String>> {
    validate_mime(&asset.mime_type)?;
    let bytes = std::mem::size_of::<Source>()
        .checked_add(asset.path.as_os_str().len())
        .and_then(|bytes| bytes.checked_add(asset.mime_type.len()))
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| error("appearance logo source memory overflows"))?;
    let preparation = storage.prepare_read_production(bytes)?;
    let computation = preparation.computation()?;
    let deadline = computation.deadline();
    let scratch = computation.scratch_bytes();
    let path = asset.path.clone();
    let mut mime_type = String::new();
    mime_type
        .try_reserve_exact(asset.mime_type.len())
        .map_err(error)?;
    if path.capacity() > asset.path.as_os_str().len()
        || mime_type.capacity() > asset.mime_type.len()
    {
        return Err(error("appearance logo source exceeds its original claim"));
    }
    mime_type.push_str(&asset.mime_type);
    let source = preparation
        .finish(Source { path, mime_type })?
        .try_into_value()?
        .try_into_shared()?;
    computation.check()?;
    let read = move || {
        let result = (|| -> harmos::Result<QueryValue<String>> {
            computation.check()?;
            let mut file = std::fs::File::open(&source.path).map_err(file_error)?;
            let before = file.metadata().map_err(file_error)?;
            validate_file(&before).map_err(production_error)?;
            let modified = before.modified().map_err(file_error)?;
            let length = usize::try_from(before.len()).map_err(|_| {
                production_error(error("appearance logo length exceeds address space"))
            })?;
            let encoded = base64::encoded_len(length, true).ok_or_else(|| {
                production_error(error("appearance logo encoding size overflows"))
            })?;
            let url_length = "data:"
                .len()
                .checked_add(source.mime_type.len())
                .and_then(|bytes| bytes.checked_add(";base64,".len()))
                .and_then(|bytes| bytes.checked_add(encoded))
                .ok_or_else(|| production_error(error("appearance logo URL size overflows")))?;
            // The Original Projection Owns Both The Final URL And Conversion Buffer.
            let bytes = std::mem::size_of::<Vec<u8>>()
                .checked_add(length)
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<String>()))
                .and_then(|bytes| bytes.checked_add(url_length))
                .and_then(NonZeroUsize::new)
                .ok_or_else(|| {
                    production_error(error("appearance logo production size overflows"))
                })?;
            computation.project_shared(&source, bytes, |source, _| {
                let mut raw = Vec::new();
                raw.try_reserve_exact(length).map_err(|_| memory_error())?;
                if raw.capacity() > length {
                    return Err(memory_error());
                }
                raw.resize(length, 0);
                file.read_exact(&mut raw).map_err(file_error)?;
                let after = file.metadata().map_err(file_error)?;
                if after.len() != before.len() || after.modified().map_err(file_error)? != modified
                {
                    return Err(production_error(error(
                        "appearance logo changed during capture",
                    )));
                }
                let mut url = String::new();
                url.try_reserve_exact(url_length)
                    .map_err(|_| memory_error())?;
                if url.capacity() > url_length {
                    return Err(memory_error());
                }
                url.push_str("data:");
                url.push_str(&source.mime_type);
                url.push_str(";base64,");
                STANDARD.encode_string(&raw, &mut url);
                if url.len() != url_length || url.capacity() > url_length {
                    return Err(memory_error());
                }
                Ok(url)
            })
        })();
        CapturedLogo {
            result,
            _source: source,
            _computation: computation,
        }
    };
    let working = std::mem::size_of_val(&read)
        .checked_add(std::mem::size_of::<CapturedLogo>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<std::fs::File>()))
        .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<std::fs::Metadata>()))
        .ok_or_else(memory_error)?;
    if working > scratch {
        return Err(memory_error().into());
    }
    let task = tokio::task::spawn_blocking(read);
    let result = tokio::time::timeout_at(deadline, task)
        .await
        .map_err(|_| harmos::Error::ResourceExhausted {
            resource: "query computation time",
        })?
        .map_err(|source| Error::Join { source })?;
    Ok(result.result?)
}

pub(super) fn validate_mime(mime: &str) -> Result<()> {
    if matches!(
        mime,
        "image/png"
            | "image/jpeg"
            | "image/gif"
            | "image/webp"
            | "image/svg+xml"
            | "image/x-icon"
            | "image/vnd.microsoft.icon"
    ) {
        Ok(())
    } else {
        Err(error("unsupported appearance logo image MIME type"))
    }
}

pub(super) fn validate_file(metadata: &std::fs::Metadata) -> Result<()> {
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_BYTES as u64 {
        Err(error(
            "appearance logo must be a regular file containing 1 byte to 1 MiB",
        ))
    } else {
        Ok(())
    }
}

/* === Private Functions === */
fn memory_error() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "appearance logo read memory",
    }
}

fn file_error(source: std::io::Error) -> harmos::Error {
    harmos::Error::backend(
        "capture original appearance logo",
        harmos::BackendFailure::Transient,
        source,
    )
}

fn production_error(source: Error) -> harmos::Error {
    match source {
        Error::Harmos(source) => source,
        source => harmos::Error::backend(
            "capture original appearance logo",
            harmos::BackendFailure::Permanent,
            source,
        ),
    }
}

/* === Tests === */
