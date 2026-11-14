/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Transfer

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::Context;
use crate::{Error, Result};
use dokime_storage::CONTENT_CHUNK_BYTES;
use harmos_protocol::storage::FileReference;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn chunk_bytes(context: &Context) -> usize {
    (4 * 1024 * 1024).min(context.settings.message_bytes.saturating_sub(1024))
}
pub(super) fn identifier(file: Option<&FileReference>) -> Result<&str> {
    file.and_then(|file| file.metadata.as_ref())
        .map(|metadata| metadata.id.as_str())
        .filter(|id| !id.is_empty())
        .ok_or_else(|| Error::Invalid("Access File: Supply Its Identity".into()))
}
pub(super) fn reference(context: &Context, selected: &FileReference) -> Result<FileReference> {
    let id = identifier(Some(selected))?;
    if selected
        .position
        .as_ref()
        .and_then(|position| position.history.as_ref())
        .is_some_and(|history| history.store != context.host.store().identity())
    {
        return Err(Error::Invalid("Get File: Use Its Original Store Reference".into()));
    }
    if let Some((_, reference)) = context.host.temporary_report(id)? {
        return Ok(reference);
    }
    Ok(context.host.store().file_reference(id)?)
}
pub(super) fn read(context: &Context, reference: &FileReference, offset: u64, maximum: usize) -> Result<Vec<u8>> {
    let id = identifier(Some(reference))?;
    if offset > reference.length {
        return Err(Error::Invalid("Read File: Keep the Offset Within Its Selected Length".into()));
    }
    let length = maximum.min((reference.length - offset) as usize);
    if let Some((file, _)) = context.host.temporary_report(id)? {
        let mut bytes = vec![0; length];
        super::content_transfer::read_temporary(&file, offset, &mut bytes)?;
        return Ok(bytes);
    }
    let mut bytes = Vec::with_capacity(length);
    while bytes.len() < length {
        let chunk =
            context.host.store().content_window(id, offset + bytes.len() as u64, (length - bytes.len()).min(CONTENT_CHUNK_BYTES))?;
        if chunk.is_empty() {
            return Err(Error::Precondition("Read File: Content Is Missing; Materialize or Select It Again".into()));
        }
        bytes.extend(chunk);
    }
    Ok(bytes)
}

/* === Private Functions === */

/* === Tests === */
