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
use super::super::{enums, value::projection};
use dokime_app::{ExecutionFileEntry, core::TestExecutionId};
use dokime_protocol::v1;

/* === Constants === */
include!(concat!(env!("OUT_DIR"), "/inventory-output-fields.rs"));
const INVENTORY_LIMIT: usize = 512;
const DIGEST_BYTES: usize = std::mem::size_of::<[u8; 32]>();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Count {
    memory: usize,
    encoded: usize,
    metadata_buffer: usize,
    after: usize,
    returned: usize,
    more: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn encode(
    entries: &[ExecutionFileEntry],
    id: &TestExecutionId,
    epoch: &str,
    request: &v1::ListExecutionFilesRequest,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<Vec<u8>> {
    let counted = count(entries, id, epoch, request)?;
    if counted.memory > allowance.get() || counted.encoded > maximum {
        return Err(exhausted());
    }

    let mut files = projection::slots(entries.len())?;
    for entry in entries {
        files.push(metadata(entry)?);
    }
    let mut scratch = projection::slots(counted.metadata_buffer)?;
    let mut hash = Sha256::new();
    for file in &files {
        scratch.clear();
        file.encode_length_delimited(&mut scratch)
            .map_err(|_| invalid_wire())?;
        if scratch.capacity() > counted.metadata_buffer {
            return Err(exhausted());
        }
        hash.update(&scratch);
    }
    let digest = hash.finalize();
    if request
        .cursor
        .as_ref()
        .is_some_and(|cursor| cursor.metadata_sha256.as_slice() != digest.as_slice())
    {
        return Err(harmos::Error::InvalidInput {
            field: "execution files cursor",
            reason: "metadata fingerprint differs",
        });
    }
    let mut metadata_sha256 = projection::slots(digest.len())?;
    metadata_sha256.extend_from_slice(&digest);
    let cursor = v1::ExecutionFilesCursor {
        epoch: projection::text(epoch)?,
        execution_id: projection::text(id.as_str())?,
        metadata_sha256,
    };
    let mut selected = projection::slots(counted.returned)?;
    for file in files.into_iter().skip(counted.after).take(counted.returned) {
        selected.push(file);
    }
    let next_after_path = if counted.more {
        Some(projection::text(
            &selected
                .last()
                .expect("nonempty continuation page")
                .relative_path,
        )?)
    } else {
        None
    };
    let wire = v1::ListExecutionFilesReply {
        files: selected,
        next_after_path,
        cursor: Some(cursor),
    };
    if wire.encoded_len() != counted.encoded {
        return Err(invalid_wire());
    }
    let mut target = projection::slots(counted.encoded)?;
    wire.encode(&mut target).map_err(|_| invalid_wire())?;
    if target.len() != counted.encoded {
        return Err(invalid_wire());
    }
    Ok(target)
}

/* === Private Functions === */
fn count(
    entries: &[ExecutionFileEntry],
    id: &TestExecutionId,
    epoch: &str,
    request: &v1::ListExecutionFilesRequest,
) -> harmos::Result<Count> {
    if entries.len() > INVENTORY_LIMIT {
        return Err(exhausted());
    }
    if entries
        .windows(2)
        .any(|pair| pair[0].relative_path >= pair[1].relative_path)
    {
        return Err(invalid_wire());
    }
    let after = if request.after_path.is_empty() {
        0
    } else {
        entries
            .iter()
            .position(|entry| entry.relative_path == request.after_path)
            .map(|index| index + 1)
            .ok_or(harmos::Error::InvalidInput {
                field: "execution files after path",
                reason: "after path not found",
            })?
    };
    let limit = if request.limit == 0 {
        50
    } else {
        request.limit as usize
    };
    let returned = entries.len().saturating_sub(after).min(limit);
    let more = entries.len().saturating_sub(after) > returned;
    let mut memory = std::mem::size_of::<Vec<v1::ExecutionFileMetadata>>()
        .checked_add(
            entries
                .len()
                .checked_mul(std::mem::size_of::<v1::ExecutionFileMetadata>())
                .ok_or_else(exhausted)?,
        )
        .ok_or_else(exhausted)?;
    let mut metadata_buffer = 0usize;
    let mut encoded = 0usize;
    for (index, entry) in entries.iter().enumerate() {
        super::super::value::text(&entry.relative_path).map_err(|_| exhausted())?;
        if let Some(media_type) = &entry.content_type {
            super::super::value::text(media_type).map_err(|_| exhausted())?;
        }
        memory = memory
            .checked_add(entry.relative_path.len())
            .and_then(|bytes| bytes.checked_add(entry.content_type.as_ref().map_or(0, String::len)))
            .ok_or_else(exhausted)?;
        let length = metadata_length(entry)?;
        let delimited = prost::encoding::encoded_len_varint(length as u64)
            .checked_add(length)
            .ok_or_else(exhausted)?;
        metadata_buffer = metadata_buffer.max(delimited);
        if index >= after && index < after + returned {
            encoded = encoded
                .checked_add(projection::framed(FILES, length)?)
                .ok_or_else(exhausted)?;
        }
    }
    let next = if more {
        entries[after + returned - 1].relative_path.len()
    } else {
        0
    };
    if more {
        encoded = encoded
            .checked_add(projection::framed(NEXT, next)?)
            .ok_or_else(exhausted)?;
    }
    let digest_length = projection::framed(DIGEST, DIGEST_BYTES)?;
    let cursor_length = string_length(EPOCH, epoch)?
        .checked_add(string_length(EXECUTION, id.as_str())?)
        .and_then(|bytes| bytes.checked_add(digest_length))
        .ok_or_else(exhausted)?;
    encoded = encoded
        .checked_add(projection::framed(CURSOR, cursor_length)?)
        .ok_or_else(exhausted)?;
    for bytes in [
        std::mem::size_of::<Vec<u8>>(),
        metadata_buffer,
        std::mem::size_of::<v1::ListExecutionFilesReply>(),
        returned
            .checked_mul(std::mem::size_of::<v1::ExecutionFileMetadata>())
            .ok_or_else(exhausted)?,
        next,
        epoch.len(),
        id.as_str().len(),
        DIGEST_BYTES,
        std::mem::size_of::<Vec<u8>>(),
        encoded,
    ] {
        memory = memory.checked_add(bytes).ok_or_else(exhausted)?;
    }
    Ok(Count {
        memory,
        encoded,
        metadata_buffer,
        after,
        returned,
        more,
    })
}

fn metadata_length(entry: &ExecutionFileEntry) -> harmos::Result<usize> {
    let scalars = v1::ExecutionFileMetadata {
        kind: enums::execution_file_kind(&entry.content_kind) as i32,
        state: enums::execution_file_state(&entry.stream_state) as i32,
        size_bytes: entry.size_bytes,
        created_at: Some(timestamp(entry)?),
        ..Default::default()
    };
    let media_type = entry
        .content_type
        .as_ref()
        .map_or(Ok(0), |value| projection::framed(MEDIA_TYPE, value.len()))?;
    scalars
        .encoded_len()
        .checked_add(string_length(PATH, &entry.relative_path)?)
        .and_then(|bytes| bytes.checked_add(media_type))
        .ok_or_else(exhausted)
}

fn string_length(tag: u32, value: &str) -> harmos::Result<usize> {
    if value.is_empty() {
        Ok(0)
    } else {
        projection::framed(tag, value.len())
    }
}

fn metadata(entry: &ExecutionFileEntry) -> harmos::Result<v1::ExecutionFileMetadata> {
    Ok(v1::ExecutionFileMetadata {
        relative_path: projection::text(&entry.relative_path)?,
        content_type: entry
            .content_type
            .as_deref()
            .map(projection::text)
            .transpose()?,
        kind: enums::execution_file_kind(&entry.content_kind) as i32,
        state: enums::execution_file_state(&entry.stream_state) as i32,
        size_bytes: entry.size_bytes,
        created_at: Some(timestamp(entry)?),
    })
}

fn timestamp(entry: &ExecutionFileEntry) -> harmos::Result<prost_types::Timestamp> {
    let at = time::OffsetDateTime::parse(
        entry.updated_at.as_str(),
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|source| {
        harmos::Error::backend(
            "parse original output creation time",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    Ok(prost_types::Timestamp {
        seconds: at.unix_timestamp(),
        nanos: at.nanosecond() as i32,
    })
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical execution inventory",
    }
}

fn invalid_wire() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "canonical execution inventory",
        reason: "original order or encoded length differs",
    }
}

/* === Tests === */
