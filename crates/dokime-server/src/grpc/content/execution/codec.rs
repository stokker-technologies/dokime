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
use sha2::{Digest, Sha256};

// Local Imports
use super::super::super::enums;
use dokime_app::ExecutionFileContent;
use dokime_protocol::v1;

/* === Constants === */
include!(concat!(env!("OUT_DIR"), "/download-output-fields.rs"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn metadata(
    content: &ExecutionFileContent,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let scalars = v1::ExecutionFileDownloadMetadata {
        kind: enums::execution_file_kind(&content.content_kind) as i32,
        state: enums::execution_file_state(&content.stream_state) as i32,
        observed_size_bytes: content.size_bytes,
        ..Default::default()
    };
    let length = scalars
        .encoded_len()
        .checked_add(field_length(
            PATH,
            content.relative_path.len(),
            !content.relative_path.is_empty(),
        )?)
        .and_then(|bytes| {
            bytes.checked_add(
                field_length(
                    MEDIA_TYPE,
                    content.content_type.as_ref().map_or(0, String::len),
                    content.content_type.is_some(),
                )
                .ok()?,
            )
        })
        .ok_or_else(exhausted)?;
    let (mut target, expected) = buffer(METADATA, length, allowance, maximum)?;
    field(
        PATH,
        content.relative_path.as_bytes(),
        !content.relative_path.is_empty(),
        &mut target,
    );
    if let Some(media_type) = &content.content_type {
        field(MEDIA_TYPE, media_type.as_bytes(), true, &mut target);
    }
    scalars.encode(&mut target).map_err(|_| invalid())?;
    finish(target, expected)
}

pub(super) fn chunk(
    content: &ExecutionFileContent,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let scalars = v1::ArtifactChunk {
        offset: content.offset,
        ..Default::default()
    };
    let length = scalars
        .encoded_len()
        .checked_add(field_length(
            DATA,
            content.bytes.len(),
            !content.bytes.is_empty(),
        )?)
        .ok_or_else(exhausted)?;
    let (mut target, expected) = buffer(CHUNK, length, allowance, maximum)?;
    scalars.encode(&mut target).map_err(|_| invalid())?;
    field(DATA, &content.bytes, !content.bytes.is_empty(), &mut target);
    finish(target, expected)
}

pub(super) fn complete(
    size: u64,
    digest: &Sha256,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let scalars = v1::ArtifactCompletion {
        size_bytes: size,
        ..Default::default()
    };
    let length = scalars
        .encoded_len()
        .checked_add(field_length(DIGEST, 64, true)?)
        .ok_or_else(exhausted)?;
    let (mut target, expected) = buffer(COMPLETE, length, allowance, maximum)?;
    let digest = digest.clone().finalize();
    let mut hex = [0u8; 64];
    let alphabet = b"0123456789abcdef";
    for (index, byte) in digest.iter().enumerate() {
        hex[index * 2] = alphabet[(byte >> 4) as usize];
        hex[index * 2 + 1] = alphabet[(byte & 15) as usize];
    }
    scalars.encode(&mut target).map_err(|_| invalid())?;
    field(DIGEST, &hex, true, &mut target);
    finish(target, expected)
}

/* === Private Functions === */
fn field_length(tag: u32, length: usize, present: bool) -> harmos::Result<usize> {
    if !present {
        return Ok(0);
    }
    super::super::super::value::projection::framed(tag, length)
}
fn field(tag: u32, bytes: &[u8], present: bool, target: &mut Vec<u8>) {
    if !present {
        return;
    }
    prost::encoding::encode_key(tag, prost::encoding::WireType::LengthDelimited, target);
    prost::encoding::encode_varint(bytes.len() as u64, target);
    target.extend_from_slice(bytes);
}
fn buffer(
    tag: u32,
    length: usize,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<(Vec<u8>, usize)> {
    let encoded = field_length(tag, length, true)?;
    let memory = std::mem::size_of::<Vec<u8>>()
        .checked_add(encoded)
        .ok_or_else(exhausted)?;
    if memory > allowance.get() || encoded > maximum {
        return Err(exhausted());
    }
    let mut target = Vec::new();
    target.try_reserve_exact(encoded).map_err(|_| exhausted())?;
    if target.capacity() > encoded {
        return Err(exhausted());
    }
    prost::encoding::encode_key(tag, prost::encoding::WireType::LengthDelimited, &mut target);
    prost::encoding::encode_varint(length as u64, &mut target);
    Ok((target, encoded))
}
fn finish(target: Vec<u8>, expected: usize) -> harmos::Result<Vec<u8>> {
    if target.len() != expected || target.capacity() > expected {
        return Err(invalid());
    }
    Ok(target)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical execution download frame",
    }
}
fn invalid() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "execution download frame",
        reason: "canonical encoded length differs",
    }
}

/* === Tests === */
