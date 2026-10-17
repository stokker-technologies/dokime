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
use harmos::operations::StreamOutput;
use harmos::{MemorySize, Protobuf};
use sha2::{Digest as _, Sha256};
use std::num::NonZeroUsize;
use tonic::Status;
// Local Imports
use dokime_runtime::runtime::ArtifactReader;
use dokime_domain::operations::content as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct ArtifactFrames(pub(super) harmos::operations::Response<dokime_domain::model::content::wire::ArtifactFrame>);

pub(super) struct Download<F> {
    pub(super) frames: F,
    pub(super) source: Source,
    pub(super) size_bytes: u64,
    pub(super) sha256: Option<String>,
    pub(super) noun: &'static str,
    pub(super) read_failure: &'static str,
    pub(super) chunk_bytes: usize,
    pub(super) message_limit: usize,
}

/* === Enums === */
pub(super) enum Source {
    Bytes { data: Vec<u8>, read: usize },
    Artifact(Box<ArtifactReader>),
}

/* === Traits === */
pub(super) trait Frames: Send + 'static {
    type Frame: Protobuf + MemorySize + Send + Sync + 'static;

    fn metadata(&self) -> Self::Frame;
    fn metadata_owned(&self) -> Option<harmos::operations::Response<Self::Frame>> {
        None
    }
    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame;
    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame;
}

/* === Implementations === */
impl Frames for ArtifactFrames {
    type Frame = dokime_domain::model::content::wire::ArtifactFrame;

    fn metadata(&self) -> Self::Frame {
        dokime_domain::model::content::wire::ArtifactFrame {
            payload: self.0.payload.clone(),
        }
    }

    fn metadata_owned(&self) -> Option<harmos::operations::Response<Self::Frame>> {
        Some(self.0.clone())
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        dokime_domain::model::content::wire::ArtifactFrame {
            payload: Some(dokime_domain::model::content::wire::ArtifactFramePayload::Chunk(dokime_domain::model::content::wire::ArtifactChunk {
                offset,
                data,
            })),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        dokime_domain::model::content::wire::ArtifactFrame {
            payload: Some(dokime_domain::model::content::wire::ArtifactFramePayload::Complete(
                dokime_domain::model::content::wire::ArtifactCompletion { size_bytes, sha256 },
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

    async fn next_chunk(
        &mut self,
        maximum: NonZeroUsize,
    ) -> Result<Option<Vec<u8>>, dokime_runtime::runtime::Error> {
        match self {
            Self::Artifact(reader) => reader.next_chunk(maximum).await,
            Self::Bytes { .. } => unreachable!("captured byte sources use direct bounded slices"),
        }
    }
}

impl<F: Frames> Download<F> {
    pub(super) async fn run(mut self, output: &mut StreamOutput<F::Frame>) -> Result<(), Status> {
        let maximum = NonZeroUsize::new(self.chunk_bytes).ok_or_else(|| self.too_large())?;
        if let Some(owner) = self.frames.metadata_owned() {
            self.check(&owner)?;
            output.send_owned(owner).await.map_err(stream_failure)?;
        } else {
            let metadata = self.frames.metadata();
            self.check(&metadata)?;
            output
                .send_with(metadata.memory_size(), || Ok(metadata))
                .await
                .map_err(stream_failure)?;
        }
        let mut offset = 0u64;
        let mut digest = Sha256::new();
        loop {
            match &mut self.source {
                Source::Bytes { data, read } => {
                    if *read == data.len() {
                        break;
                    }
                    let end = data.len().min(read.saturating_add(maximum.get()));
                    let bytes = &data[*read..end];
                    let next = checked_offset(offset, bytes.len(), self.size_bytes, self.noun)?;
                    if self.sha256.is_none() {
                        digest.update(bytes);
                    }
                    let frame_bytes = std::mem::size_of::<F::Frame>()
                        .checked_add(bytes.len())
                        .ok_or_else(|| Status::resource_exhausted("file frame memory overflow"))?;
                    let frames = &self.frames;
                    let limit = self.message_limit;
                    output
                        .send_with(frame_bytes, || {
                            let mut payload = Vec::new();
                            payload.try_reserve_exact(bytes.len()).map_err(|_| {
                                harmos::Error::ResourceExhausted {
                                    resource: "file frame allocation",
                                }
                            })?;
                            payload.extend_from_slice(bytes);
                            let frame = frames.chunk(offset, payload);
                            if frame.encoded_len() > limit {
                                return Err(harmos::Error::ResourceExhausted {
                                    resource: "file frame encoding",
                                });
                            }
                            Ok(frame)
                        })
                        .await
                        .map_err(stream_failure)?;
                    *read = end;
                    offset = next;
                }
                Source::Artifact(_) => {
                    let Some(chunk) = self
                        .source
                        .next_chunk(maximum)
                        .await
                        .map_err(|source| file_failure(source, self.read_failure))?
                    else {
                        break;
                    };
                    let count = chunk.len();
                    let next = checked_offset(offset, count, self.size_bytes, self.noun)?;
                    if self.sha256.is_none() {
                        digest.update(&chunk);
                    }
                    let frame = self.frames.chunk(offset, chunk);
                    self.check(&frame)?;
                    output
                        .send_owned(frame.into())
                        .await
                        .map_err(stream_failure)?;
                    offset = next;
                }
            }
        }
        if offset != self.size_bytes {
            return Err(Status::data_loss(format!(
                "{} ended before its declared length",
                self.noun
            )));
        }
        let sha256 = self.sha256.take().unwrap_or_else(|| {
            digest
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect()
        });
        let complete = self.frames.complete(offset, sha256);
        self.check(&complete)?;
        output
            .send_with(complete.memory_size(), || Ok(complete))
            .await
            .map_err(stream_failure)
    }

    fn check(&self, frame: &F::Frame) -> Result<(), Status> {
        if frame.encoded_len() > self.message_limit {
            return Err(self.too_large());
        }
        Ok(())
    }
    fn too_large(&self) -> Status {
        Status::resource_exhausted(format!(
            "{} exceeds the configured message budget",
            self.noun
        ))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn checked_offset(offset: u64, count: usize, size: u64, noun: &str) -> Result<u64, Status> {
    offset
        .checked_add(count as u64)
        .filter(|delivered| *delivered <= size)
        .ok_or_else(|| Status::data_loss(format!("{noun} exceeds its declared length")))
}
fn file_failure(source: dokime_runtime::runtime::Error, message: &'static str) -> Status {
    let status = match &source {
        dokime_runtime::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. }) => {
            Status::resource_exhausted(message)
        }
        dokime_runtime::runtime::Error::Harmos(harmos::Error::NotReady { .. }) => Status::unavailable(message),
        _ => Status::internal(message),
    };
    super::error::detailed(status, &source)
}
fn stream_failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted(error.to_string()),
        harmos::Error::NotReady { .. } => Status::unavailable(error.to_string()),
        _ => Status::internal(error.to_string()),
    }
}

/* === Tests === */
