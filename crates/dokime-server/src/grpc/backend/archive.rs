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
use dokime::harmos::operations::{Response, StreamOutput};
use tonic::Status;

// Local Imports
use crate::grpc::{bundle, chunked, value::canonical};
use dokime::RuntimeHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Frames(Response<dokime::domain::model::backend::wire::DiagnosticsBundleFrame>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl chunked::Frames for Frames {
    type Frame = dokime::domain::model::backend::wire::DiagnosticsBundleFrame;
    fn metadata(&self) -> Self::Frame {
        self.0.as_ref().clone()
    }
    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.0.clone())
    }
    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        dokime::domain::model::backend::wire::DiagnosticsBundleFrame {
            payload: Some(dokime::domain::model::backend::wire::DiagnosticsBundleFramePayload::Chunk(
                dokime::domain::model::backend::wire::DiagnosticsBundleChunk { offset, data },
            )),
        }
    }
    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        dokime::domain::model::backend::wire::DiagnosticsBundleFrame {
            payload: Some(dokime::domain::model::backend::wire::DiagnosticsBundleFramePayload::Complete(
                dokime::domain::model::backend::wire::DiagnosticsBundleCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */

pub(super) async fn deliver(
    runtime: &RuntimeHandle, identity: &dokime::domain::model::backend::wire::BackendIdentity, maximum: usize, tail: Option<u64>,
    output: &mut StreamOutput<dokime::domain::model::backend::wire::DiagnosticsBundleFrame>,
) -> Result<(), Status> {
    let archive = super::collect(runtime, identity, bundle::log_tail_ceiling(tail)).await?;
    let size_bytes = archive.bytes.len() as u64;
    let bytes = archive.bytes;
    let header = super::status::produce_result(maximum, || {
        Ok(dokime::domain::model::backend::wire::DiagnosticsBundleFrame {
            payload: Some(dokime::domain::model::backend::wire::DiagnosticsBundleFramePayload::Header(
                dokime::domain::model::backend::wire::DiagnosticsBundleHeader {
                    filename: archive.filename,
                    content_type: "application/zip".into(),
                    generated_at: Some(canonical::timestamp(&archive.generated_at)?),
                    total_size_bytes: Some(size_bytes),
                    entries: archive
                        .entries
                        .iter()
                        .map(|entry| dokime::domain::model::backend::wire::DiagnosticsBundleEntry {
                            path: entry.path.clone(),
                            size_bytes: entry.size_bytes,
                            present: entry.present,
                            note: entry.note.clone(),
                        })
                        .collect(),
                },
            )),
        })
    })?;
    chunked::Download {
        frames: Frames(header),
        source: chunked::Source::bytes(bytes),
        size_bytes,
        sha256: Some(archive.sha256),
        noun: "diagnostics archive",
        read_failure: "diagnostics archive could not be read",
        chunk_bytes: super::super::content::download_chunk_bytes(maximum),
        message_limit: maximum,
    }
    .run(output)
    .await
}

/* === Private Functions === */

/* === Tests === */
