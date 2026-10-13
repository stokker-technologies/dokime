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
use std::{mem::size_of, num::NonZeroUsize};
// Third Party Imports
use harmos::operations::{Response, StreamOutput};
use tonic::Status;
// Local Imports
use crate::grpc::{bundle, chunked, value::canonical};
use dokime_app::RuntimeHandle;
use dokime_contract::operations::backend as wire;
/* === Constants === */
const CHUNK_BYTES: usize = 64 * 1024;
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
struct Frames(Response<wire::DiagnosticsBundleFrame>);
/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl chunked::Frames for Frames {
    type Frame = wire::DiagnosticsBundleFrame;
    fn metadata(&self) -> Self::Frame {
        self.0.as_ref().clone()
    }
    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.0.clone())
    }
    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        wire::DiagnosticsBundleFrame {
            payload: Some(wire::DiagnosticsBundleFramePayload::Chunk(
                wire::DiagnosticsBundleChunk { offset, data },
            )),
        }
    }
    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        wire::DiagnosticsBundleFrame {
            payload: Some(wire::DiagnosticsBundleFramePayload::Complete(
                wire::DiagnosticsBundleCompletion { size_bytes, sha256 },
            )),
        }
    }
}
/* === Public Functions === */
pub(super) async fn deliver(
    runtime: &RuntimeHandle,
    identity: &wire::BackendIdentity,
    maximum: usize,
    tail: Option<u64>,
    output: &mut StreamOutput<wire::DiagnosticsBundleFrame>,
) -> Result<(), Status> {
    let allowance =
        NonZeroUsize::new(size_of::<Vec<u8>>().saturating_add(2 * bundle::MAX_BUNDLE_BYTES))
            .ok_or_else(|| Status::resource_exhausted("invalid diagnostics archive allowance"))?;
    let preparation = runtime
        .storage()
        .prepare_production(allowance)
        .map_err(memory_failure)?;
    let archive = super::collect(runtime, identity, bundle::log_tail_ceiling(tail)).await?;
    let size_bytes = archive.bytes.len() as u64;
    let bytes = preparation
        .finish(archive.bytes)
        .map_err(memory_failure)?
        .try_into_value()
        .map_err(memory_failure)?;
    let required = archive.entries.iter().fold(
        size_of::<wire::DiagnosticsBundleFrame>()
            .saturating_add(archive.filename.capacity())
            .saturating_add("application/zip".len())
            .saturating_add(
                archive
                    .entries
                    .len()
                    .saturating_mul(4)
                    .saturating_mul(size_of::<wire::DiagnosticsBundleEntry>()),
            ),
        |bytes, entry| {
            bytes
                .saturating_add(entry.path.len())
                .saturating_add(entry.note.as_ref().map_or(0, String::len))
        },
    );
    let header = super::status::produce_result(runtime, maximum, required, || {
        Ok(wire::DiagnosticsBundleFrame {
            payload: Some(wire::DiagnosticsBundleFramePayload::Header(
                wire::DiagnosticsBundleHeader {
                    filename: archive.filename,
                    content_type: "application/zip".into(),
                    generated_at: Some(canonical::timestamp(&archive.generated_at)?),
                    total_size_bytes: Some(size_bytes),
                    entries: archive
                        .entries
                        .iter()
                        .map(|entry| wire::DiagnosticsBundleEntry {
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
        source: chunked::Source::owned_bytes(bytes),
        size_bytes,
        sha256: Some(archive.sha256),
        noun: "diagnostics archive",
        read_failure: "diagnostics archive could not be read",
        chunk_bytes: CHUNK_BYTES.min(maximum.saturating_sub(256)),
        message_limit: maximum,
    }
    .run(output)
    .await
}
/* === Private Functions === */
fn memory_failure(source: harmos::Error) -> Status {
    let status = if matches!(source, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("diagnostics archive admission failed")
    } else {
        Status::internal("diagnostics archive admission failed")
    };
    crate::grpc::error::detailed(status, &source.into())
}
/* === Tests === */
