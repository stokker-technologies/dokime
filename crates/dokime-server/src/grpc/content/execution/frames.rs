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
use sha2::{Digest, Sha256};

// Local Imports
use super::super::files;
use crate::grpc::value::projection;
use dokime::ExecutionFileContent;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn metadata(
    content: &ExecutionFileContent, maximum: usize,
) -> dokime::harmos::Result<dokime::harmos::operations::Response<dokime::domain::model::content::wire::ExecutionFileDownloadFrame>> {
    let frame = dokime::harmos::storage::ReadBatch::new(()).try_map(|_| {
        let frame = dokime::domain::model::content::wire::ExecutionFileDownloadFrame {
            payload: Some(dokime::domain::model::content::wire::ExecutionFileDownloadFramePayload::Metadata(
                dokime::domain::model::content::wire::ExecutionFileDownloadMetadata {
                    relative_path: projection::text(&content.relative_path)?,
                    content_type: content.content_type.as_deref().map(projection::text).transpose()?,
                    kind: files::kind(&content.content_kind),
                    state: files::state(&content.stream_state),
                    observed_size_bytes: content.size_bytes,
                },
            )),
        };
        checked(frame, maximum)
    })?;
    Ok(dokime::harmos::operations::Response::from_owner(frame.owner()))
}

pub(super) fn chunk(
    content: dokime::harmos::storage::ReadBatch<ExecutionFileContent>, maximum: usize,
) -> dokime::harmos::Result<dokime::harmos::operations::Response<dokime::domain::model::content::wire::ExecutionFileDownloadFrame>> {
    let frame = content.try_map(|content| {
        checked(
            dokime::domain::model::content::wire::ExecutionFileDownloadFrame {
                payload: Some(dokime::domain::model::content::wire::ExecutionFileDownloadFramePayload::Chunk(
                    dokime::domain::model::content::wire::ArtifactChunk {
                        offset: content.offset,
                        data: content.bytes,
                    },
                )),
            },
            maximum,
        )
    })?;
    Ok(dokime::harmos::operations::Response::from_owner(frame.owner()))
}

pub(super) fn complete(
    size: u64, digest: &Sha256, maximum: usize,
) -> dokime::harmos::Result<dokime::harmos::operations::Response<dokime::domain::model::content::wire::ExecutionFileDownloadFrame>> {
    let frame = dokime::harmos::storage::ReadBatch::new(()).try_map(|_| {
        let digest = digest.clone().finalize();
        let mut hex = [0u8; 64];
        let alphabet = b"0123456789abcdef";
        for (index, byte) in digest.iter().enumerate() {
            hex[index * 2] = alphabet[(byte >> 4) as usize];
            hex[index * 2 + 1] = alphabet[(byte & 15) as usize];
        }
        let sha256 = projection::text(std::str::from_utf8(&hex).expect("hexadecimal digest is ASCII"))?;
        checked(
            dokime::domain::model::content::wire::ExecutionFileDownloadFrame {
                payload: Some(dokime::domain::model::content::wire::ExecutionFileDownloadFramePayload::Complete(
                    dokime::domain::model::content::wire::ArtifactCompletion { size_bytes: size, sha256 },
                )),
            },
            maximum,
        )
    })?;
    Ok(dokime::harmos::operations::Response::from_owner(frame.owner()))
}

/* === Private Functions === */

fn checked(
    frame: dokime::domain::model::content::wire::ExecutionFileDownloadFrame, maximum: usize,
) -> dokime::harmos::Result<dokime::domain::model::content::wire::ExecutionFileDownloadFrame> {
    let length = super::super::output_frame_len(&frame).map_err(super::super::encoding_failure)?;
    if length > maximum {
        return Err(exhausted());
    }
    Ok(frame)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "canonical execution download frame",
    }
}

/* === Tests === */
