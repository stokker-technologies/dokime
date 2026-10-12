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
use std::num::NonZeroUsize;

// Third Party Imports
use prost::Message;

// Local Imports
use super::super::enums;
use dokime_app::ExecutionFileContent;
use dokime_protocol::v1;

/* === Constants === */
include!(concat!(env!("OUT_DIR"), "/content-output-fields.rs"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn projection(
    content: &ExecutionFileContent,
    maximum: usize,
) -> harmos::Result<NonZeroUsize> {
    let length = encoded_length(content);
    if length > maximum {
        return Err(exhausted());
    }
    construction_memory(content)
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(|bytes| bytes.checked_add(length))
        .and_then(NonZeroUsize::new)
        .ok_or_else(exhausted)
}

pub(super) fn encode(
    content: &ExecutionFileContent,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let required = projection(content, maximum)?;
    if required > allowance {
        return Err(exhausted());
    }
    let wire = v1::ExecutionFileWindow {
        relative_path: text(&content.relative_path)?,
        content_type: content.content_type.as_deref().map(text).transpose()?,
        kind: enums::execution_file_kind(&content.content_kind) as i32,
        state: enums::execution_file_state(&content.stream_state) as i32,
        offset: content.offset,
        observed_size_bytes: content.size_bytes,
        data: bytes(&content.bytes)?,
    };
    let length = encoded_length(content);
    if wire.encoded_len() != length {
        return Err(invalid());
    }
    let mut target = Vec::new();
    target.try_reserve_exact(length).map_err(|_| exhausted())?;
    if target.capacity() > length {
        return Err(exhausted());
    }
    wire.encode(&mut target).map_err(|_| invalid())?;
    if target.len() != length {
        return Err(invalid());
    }
    Ok(target)
}

/* === Private Functions === */
fn encoded_length(content: &ExecutionFileContent) -> usize {
    let scalars = v1::ExecutionFileWindow {
        kind: enums::execution_file_kind(&content.content_kind) as i32,
        state: enums::execution_file_state(&content.stream_state) as i32,
        offset: content.offset,
        observed_size_bytes: content.size_bytes,
        ..Default::default()
    };
    let path = if content.relative_path.is_empty() {
        0
    } else {
        prost::encoding::string::encoded_len(PATH, &content.relative_path)
    };
    let media_type = content.content_type.as_ref().map_or(0, |value| {
        prost::encoding::string::encoded_len(MEDIA_TYPE, value)
    });
    let data = if content.bytes.is_empty() {
        0
    } else {
        prost::encoding::bytes::encoded_len(DATA, &content.bytes)
    };
    scalars
        .encoded_len()
        .saturating_add(path)
        .saturating_add(media_type)
        .saturating_add(data)
}

fn construction_memory(content: &ExecutionFileContent) -> usize {
    std::mem::size_of::<v1::ExecutionFileWindow>()
        .saturating_add(content.relative_path.len())
        .saturating_add(content.content_type.as_ref().map_or(0, String::len))
        .saturating_add(content.bytes.len())
}

fn text(source: &str) -> harmos::Result<String> {
    let mut target = String::new();
    target
        .try_reserve_exact(source.len())
        .map_err(|_| exhausted())?;
    if target.capacity() > source.len() {
        return Err(exhausted());
    }
    target.push_str(source);
    Ok(target)
}

fn bytes(source: &[u8]) -> harmos::Result<Vec<u8>> {
    let mut target = Vec::new();
    target
        .try_reserve_exact(source.len())
        .map_err(|_| exhausted())?;
    if target.capacity() > source.len() {
        return Err(exhausted());
    }
    target.extend_from_slice(source);
    Ok(target)
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical file window projection",
    }
}

fn invalid() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "canonical file window",
        reason: "encoded length differs",
    }
}

/* === Tests === */
