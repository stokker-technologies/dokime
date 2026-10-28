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

// Local Imports
use super::files;
use crate::grpc::transcode::Transcode;
use dokime::ExecutionFileContent;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::grpc) fn owned(
    content: dokime::harmos::storage::ReadBatch<ExecutionFileContent>, maximum: usize,
) -> dokime::harmos::Result<dokime::harmos::operations::Response<dokime::domain::model::content::wire::ExecutionFileWindow>> {
    let response = content.try_map(|content| project(content, maximum))?;
    Ok(dokime::harmos::operations::Response::from_owner(response.owner()))
}

pub(in crate::grpc) fn project(
    content: ExecutionFileContent, maximum: usize,
) -> dokime::harmos::Result<dokime::domain::model::content::wire::ExecutionFileWindow> {
    let mut response = dokime::domain::model::content::wire::ExecutionFileWindow {
        kind: files::kind(&content.content_kind),
        state: files::state(&content.stream_state),
        relative_path: content.relative_path,
        content_type: content.content_type,
        offset: content.offset,
        observed_size_bytes: content.size_bytes,
        data: content.bytes,
    };
    loop {
        let encoded = response.wire_len()?;
        if encoded <= maximum {
            break;
        }
        let retained = response.data.len().checked_sub(encoded - maximum).filter(|length| *length > 0).ok_or_else(exhausted)?;
        response.data.truncate(retained);
    }
    Ok(response)
}

/* === Private Functions === */

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "canonical file window projection",
    }
}

/* === Tests === */
