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
use harmos::Protobuf;

// Local Imports
use super::files;
use dokime_runtime::ExecutionFileContent;
use dokime_domain::operations::content as wire;

/* === Constants === */
/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn project(
    content: &ExecutionFileContent,
    maximum: usize,
) -> harmos::Result<dokime_domain::model::content::wire::ExecutionFileWindow> {
    let wire = dokime_domain::model::content::wire::ExecutionFileWindow {
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
    maximum: usize,
) -> harmos::Result<dokime_domain::model::content::wire::ExecutionFileWindow> {
    let response = dokime_domain::model::content::wire::ExecutionFileWindow {
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
