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

const SERVER_BUNDLE_LOG_TAIL_BYTES: u64 = 2_097_152;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Frames {
    metadata: Response<dokime::domain::model::backend::wire::DiagnosticsBundleFrame>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl chunked::Frames for Frames {
    type Frame = dokime::domain::model::backend::wire::DiagnosticsBundleFrame;
    fn metadata(&self) -> Self::Frame {
        self.metadata.as_ref().clone()
    }
    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.metadata.clone())
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
    let archive = super::collect(runtime, identity, bundle::log_tail_ceiling(tail, SERVER_BUNDLE_LOG_TAIL_BYTES)).await?;
    let size_bytes = archive.metadata.size_bytes;
    let sha256 = archive.metadata.sha256.clone();
    let generated_at = canonical::timestamp(&archive.metadata.generated_at)?;

    let header = archive
        .metadata
        .try_map(|metadata| {
            let mut entries = Vec::new();
            entries
                .try_reserve_exact(metadata.entries.len())
                .map_err(|_| dokime::harmos::Error::ResourceExhausted { resource: "diagnostics header entries" })?;
            for entry in metadata.entries {
                entries.push(dokime::domain::model::backend::wire::DiagnosticsBundleEntry {
                    path: entry.path,
                    size_bytes: entry.size_bytes,
                    present: entry.present,
                    note: entry.note,
                });
            }
            Ok::<_, dokime::harmos::Error>(dokime::domain::model::backend::wire::DiagnosticsBundleFrame {
                payload: Some(dokime::domain::model::backend::wire::DiagnosticsBundleFramePayload::Header(
                    dokime::domain::model::backend::wire::DiagnosticsBundleHeader {
                        filename: metadata.filename,
                        content_type: "application/zip".into(),
                        generated_at: Some(generated_at),
                        total_size_bytes: Some(size_bytes),
                        entries,
                    },
                )),
            })
        })
        .map_err(|error| Status::resource_exhausted(error.to_string()))?;
    chunked::Download {
        frames: Frames { metadata: Response::from_owner(header.owner()) },
        source: chunked::Source::opened(archive.reader),
        size_bytes,
        sha256: Some(sha256),
        noun: "diagnostics archive",
        read_failure: "diagnostics archive could not be read",
        chunk_bytes: super::super::content::download_chunk_bytes(runtime, maximum)?,
        message_limit: maximum,
    }
    .run(output)
    .await
}

/* === Private Functions === */

/* === Tests === */
