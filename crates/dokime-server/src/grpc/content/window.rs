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
use harmos::Protobuf;

// Local Imports
use super::files;
use dokime_app::ExecutionFileContent;
use dokime_contract::operations::content as wire;

/* === Constants === */
/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn projection(content: &ExecutionFileContent) -> harmos::Result<NonZeroUsize> {
    NonZeroUsize::new(construction_memory(content))
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(exhausted)
}

pub(super) fn project(
    content: &ExecutionFileContent,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<wire::ExecutionFileWindow> {
    let required = projection(content)?;
    if required > allowance {
        return Err(exhausted());
    }
    let wire = wire::ExecutionFileWindow {
        relative_path: text(&content.relative_path)?,
        content_type: content.content_type.as_deref().map(text).transpose()?,
        kind: files::kind(&content.content_kind),
        state: files::state(&content.stream_state),
        offset: content.offset,
        observed_size_bytes: content.size_bytes,
        data: bytes(&content.bytes)?,
    };
    if wire.encoded_len() > maximum {
        return Err(exhausted());
    }
    Ok(wire)
}

pub(in crate::grpc) fn owned(
    content: ExecutionFileContent,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<wire::ExecutionFileWindow> {
    let required = std::mem::size_of::<wire::ExecutionFileWindow>()
        .saturating_add(content.relative_path.capacity())
        .saturating_add(content.content_type.as_ref().map_or(0, String::capacity))
        .saturating_add(content.bytes.capacity());
    if required > allowance.get() {
        return Err(exhausted());
    }
    let response = wire::ExecutionFileWindow {
        kind: files::kind(&content.content_kind),
        state: files::state(&content.stream_state),
        relative_path: content.relative_path,
        content_type: content.content_type,
        offset: content.offset,
        observed_size_bytes: content.size_bytes,
        data: content.bytes,
    };
    if response.encoded_len() > maximum {
        return Err(exhausted());
    }
    Ok(response)
}

/* === Private Functions === */
fn construction_memory(content: &ExecutionFileContent) -> usize {
    std::mem::size_of::<wire::ExecutionFileWindow>()
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

/* === Tests === */
