/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Download Published Content Atomically

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write;
use std::path::Path;

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use crate::client::Client;
use dokime::FileReference;
use crate::client::{FilesGetOptions, FilesReadOptions};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Refuse an Existing Destination Before Any Host Call Creates Content.
pub(crate) fn refuse_existing(out: &Path, action: &str) -> crate::Result<()> {
    if out.exists() {
        return Err(crate::error::Error::Invalid(format!("{action}: Choose an Output Path That Does Not Exist")));
    }
    Ok(())
}

/// Write Verified Bytes Beside the Destination, Then Publish Without Replacing Anything.
pub(crate) async fn save(
    client: &Client, settings: &crate::settings::Settings, reference: FileReference, out: &Path, action: &str,
) -> crate::Result<()> {
    let parent = out.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    let mut offset = 0;
    let mut hash = Sha256::new();
    let selected = client
        .files
        .get_with(
            reference.clone(),
            FilesGetOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    let selected_file = selected.file.ok_or_else(|| format!("{action}: Missing Selected File"))?;
    if selected_file.length != reference.length || selected_file.sha256 != reference.sha256 {
        return Err(format!("{action}: Selected File Identity Changed").into());
    }
    let mut at = selected.resolved_at;
    while offset < reference.length {
        let max_bytes = (settings.message_bytes.get().saturating_sub(1024) as u64)
            .min(selected.chunk_bytes)
            .min(selected.message_bytes.saturating_sub(1024))
            .min(reference.length - offset);
        if max_bytes == 0 {
            return Err(crate::error::Error::Capacity(format!("{action}: Increase message_bytes")));
        }
        let page = client
            .files
            .read_with(
                reference.clone(),
                offset,
                max_bytes,
                FilesReadOptions {
                    context: settings.context()?,
                    at: at.clone(),
                },
            )
            .await?;
        if page.total_length != reference.length
            || page.sha256 != reference.sha256
            || page.data.is_empty()
            || page.data.len() as u64 > max_bytes
            || (at.is_some() && page.resolved_at != at)
        {
            return Err(format!("{action}: File Identity, Range or Retained Cut Changed").into());
        }
        file.write_all(&page.data)?;
        offset += page.data.len() as u64;
        hash.update(&page.data);
        at = page.resolved_at;
    }
    if hash.finalize().as_slice() != reference.sha256 {
        return Err(format!("{action}: SHA-256 Does Not Match").into());
    }
    file.as_file().sync_all()?;
    file.persist_noclobber(out)?;
    println!("{}", out.display());
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
