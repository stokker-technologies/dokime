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
use std::pin::Pin;
// Third Party Imports
use futures_util::{Stream, stream};
use prost::Message;
use sha2::{Digest as _, Sha256};
use tokio::sync::watch;
use tonic::Status;
// Local Imports
use dokime_app::runtime::ArtifactReader;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(super) type FrameStream<F> = Pin<Box<dyn Stream<Item = Result<F, Status>> + Send + 'static>>;

/* === Structs === */
pub(super) struct ArtifactFrames(pub(super) v1::ArtifactMetadata);

pub(super) struct Download<F> {
    pub(super) frames: F,
    pub(super) source: Source,
    pub(super) size_bytes: u64,
    pub(super) sha256: Option<String>,
    pub(super) noun: &'static str,
    pub(super) read_failure: &'static str,
    pub(super) chunk_bytes: usize,
    pub(super) message_limit: usize,
    pub(super) stopping: watch::Receiver<bool>,
}

struct Transfer<F> {
    download: Download<F>,
    offset: u64,
    digest: Sha256,
    phase: Phase,
}

/* === Enums === */
pub(super) enum Source {
    Bytes { data: Vec<u8>, read: usize },
    Artifact(Box<ArtifactReader>),
    Native(Box<harmos::storage::StorageFileReader>),
}

enum Phase {
    Metadata,
    Bytes,
    Done,
}

/* === Traits === */
pub(super) trait Frames: Send + 'static {
    type Frame: Message + Send + 'static;

    fn metadata(&self) -> Self::Frame;
    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame;
    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame;
}

/* === Implementations === */
impl Frames for ArtifactFrames {
    type Frame = v1::ArtifactFrame;

    fn metadata(&self) -> Self::Frame {
        v1::ArtifactFrame {
            payload: Some(v1::artifact_frame::Payload::Metadata(self.0.clone())),
        }
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        v1::ArtifactFrame {
            payload: Some(v1::artifact_frame::Payload::Chunk(v1::ArtifactChunk {
                offset,
                data,
            })),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        v1::ArtifactFrame {
            payload: Some(v1::artifact_frame::Payload::Complete(
                v1::ArtifactCompletion { size_bytes, sha256 },
            )),
        }
    }
}

impl Source {
    pub(super) fn bytes(data: Vec<u8>) -> Self {
        Self::Bytes { data, read: 0 }
    }

    pub(super) fn artifact(reader: ArtifactReader) -> Self {
        Self::Artifact(Box::new(reader))
    }

    async fn fill(&mut self, buffer: &mut [u8]) -> Result<usize, ()> {
        match self {
            Self::Bytes { data, read } => {
                let end = data.len().min(read.saturating_add(buffer.len()));
                let count = end - *read;
                buffer[..count].copy_from_slice(&data[*read..end]);
                *read = end;
                Ok(count)
            }
            Self::Artifact(reader) => reader.read(buffer).await.map_err(|_| ()),
            Self::Native(reader) => {
                let limit = std::num::NonZeroUsize::new(buffer.len()).ok_or(())?;
                let Some(chunk) = reader.next_chunk(limit).await.map_err(|_| ())? else {
                    return Ok(0);
                };
                let bytes = chunk.as_bytes();
                buffer[..bytes.len()].copy_from_slice(bytes);
                Ok(bytes.len())
            }
        }
    }

    fn self_verifying(&self) -> bool {
        matches!(self, Self::Artifact(_) | Self::Native(_))
    }
}

impl<F: Frames> Download<F> {
    pub(super) fn stream(self) -> Result<FrameStream<F::Frame>, Status> {
        if self.chunk_bytes == 0 || self.frames.metadata().encoded_len() > self.message_limit {
            return Err(Status::resource_exhausted(format!(
                "{} exceeds the configured message limit",
                self.noun
            )));
        }
        let transfer = Transfer {
            download: self,
            offset: 0,
            digest: Sha256::new(),
            phase: Phase::Metadata,
        };
        Ok(Box::pin(stream::try_unfold(transfer, Transfer::next)))
    }
}

impl<F: Frames> Transfer<F> {
    async fn next(mut self) -> Result<Option<(F::Frame, Self)>, Status> {
        let noun = self.download.noun;
        let interrupted =
            || Status::unavailable(format!("{noun} transfer interrupted by listener shutdown"));
        if *self.download.stopping.borrow() {
            return Err(interrupted());
        }
        let frame = match self.phase {
            Phase::Metadata => {
                self.phase = Phase::Bytes;
                self.download.frames.metadata()
            }
            Phase::Bytes => {
                let mut data = vec![0; self.download.chunk_bytes];
                let read_failure = self.download.read_failure;
                let Download {
                    source, stopping, ..
                } = &mut self.download;
                let count = tokio::select! { biased;
                    _ = stopping.changed() => return Err(interrupted()),
                    count = source.fill(&mut data) => {
                        count.map_err(|()| Status::internal(read_failure))?
                    }
                };
                if count == 0 {
                    if self.offset != self.download.size_bytes
                        && !self.download.source.self_verifying()
                    {
                        return Err(Status::data_loss(format!(
                            "{noun} ended before its declared length"
                        )));
                    }
                    self.phase = Phase::Done;
                    let sha256 = self.download.sha256.clone().unwrap_or_else(|| {
                        self.digest
                            .clone()
                            .finalize()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    });
                    self.download.frames.complete(self.offset, sha256)
                } else {
                    data.truncate(count);
                    if self.download.sha256.is_none() {
                        self.digest.update(&data);
                    }
                    let offset = self.offset;
                    self.offset = offset
                        .checked_add(count as u64)
                        .filter(|delivered| *delivered <= self.download.size_bytes)
                        .ok_or_else(|| {
                            Status::data_loss(format!("{noun} exceeds its declared length"))
                        })?;
                    self.download.frames.chunk(offset, data)
                }
            }
            Phase::Done => return Ok(None),
        };
        if frame.encoded_len() > self.download.message_limit {
            return Err(Status::resource_exhausted(format!(
                "{noun} frame exceeds the configured message budget"
            )));
        }
        Ok(Some((frame, self)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
