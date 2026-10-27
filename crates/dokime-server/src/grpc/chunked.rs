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
use dokime::harmos::domain::Portable;
use dokime::harmos::operations::StreamOutput;
use sha2::{Digest as _, Sha256};
use tonic::Status;

// Local Imports
use dokime::runtime::ArtifactReader;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct ArtifactFrames(pub(super) dokime::harmos::operations::Response<dokime::domain::model::content::wire::ArtifactFrame>);

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
    Bytes { data: dokime::harmos::operations::Response<Vec<u8>>, read: usize },
    Opened { reader: dokime::harmos::storage::FileReader, batch: Option<dokime::harmos::storage::FileReadBatch>, read: usize },
    Artifact(Box<ArtifactReader>),
    Evidence(Box<dokime::domains::exports::evidence::Bundle>),
}

/* === Traits === */

pub(super) trait Frames: Send + 'static {
    type Frame: Portable + Send + Sync + 'static;

    fn metadata(&self) -> Self::Frame;
    fn metadata_owned(&self) -> Option<dokime::harmos::operations::Response<Self::Frame>> {
        None
    }
    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame;
    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame;
}

/* === Implementations === */

impl Frames for ArtifactFrames {
    type Frame = dokime::domain::model::content::wire::ArtifactFrame;

    fn metadata(&self) -> Self::Frame {
        dokime::domain::model::content::wire::ArtifactFrame { payload: self.0.payload.clone() }
    }

    fn metadata_owned(&self) -> Option<dokime::harmos::operations::Response<Self::Frame>> {
        Some(self.0.clone())
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        dokime::domain::model::content::wire::ArtifactFrame {
            payload: Some(dokime::domain::model::content::wire::ArtifactFramePayload::Chunk(
                dokime::domain::model::content::wire::ArtifactChunk { offset, data },
            )),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        dokime::domain::model::content::wire::ArtifactFrame {
            payload: Some(dokime::domain::model::content::wire::ArtifactFramePayload::Complete(
                dokime::domain::model::content::wire::ArtifactCompletion { size_bytes, sha256 },
            )),
        }
    }
}

impl Source {
    pub(super) fn opened(reader: dokime::harmos::storage::FileReader) -> Self {
        Self::Opened { reader, batch: None, read: 0 }
    }

    pub(super) fn bytes(data: dokime::harmos::operations::Response<Vec<u8>>) -> Self {
        Self::Bytes { data, read: 0 }
    }

    pub(super) fn artifact(reader: ArtifactReader) -> Self {
        Self::Artifact(Box::new(reader))
    }
}

impl<F: Frames> Download<F> {
    pub(super) async fn run(mut self, output: &mut StreamOutput<F::Frame>) -> Result<(), Status> {
        let template = self.frames.chunk(self.size_bytes, Vec::new());
        let bytes = super::content::output_chunk_bytes(&template, self.chunk_bytes, self.message_limit)?;
        let maximum = NonZeroUsize::new(bytes).ok_or_else(|| self.too_large())?;
        if let Some(owner) = self.frames.metadata_owned() {
            self.check(&owner)?;
            output.send_owned(owner).await.map_err(stream_failure)?;
        } else {
            let metadata = self.frames.metadata();
            self.check(&metadata)?;
            output.send_owned((metadata).into()).await.map_err(stream_failure)?;
        }
        let mut offset = 0u64;
        let mut digest = Sha256::new();
        loop {
            match &mut self.source {
                Source::Opened { reader, batch, read } => {
                    if batch.is_none() {
                        *batch = reader.next().await.map_err(super::bundle::native_failure)?;
                        *read = 0;
                    }
                    let Some(page) = batch.as_ref() else {
                        reader.completion().map_err(super::bundle::native_failure)?;
                        break;
                    };
                    if page.offset().checked_add(*read as u64) != Some(offset) || *read >= page.bytes().len() {
                        return Err(Status::data_loss("opened file page differs from delivered offset"));
                    }
                    let length = maximum.get().min(page.bytes().len() - *read);
                    let next = checked_offset(offset, length, self.size_bytes, self.noun)?;
                    if self.sha256.is_none() {
                        digest.update(&page.bytes()[*read..*read + length]);
                    }
                    let end = *read + length;
                    let done = end == page.bytes().len();
                    let frame = if *read == 0 && done {
                        batch.take().expect("selected native page").try_map(|page| Ok(self.frames.chunk(offset, page.bytes)))
                    } else {
                        let bytes = &page.bytes()[*read..end];
                        dokime::harmos::storage::ReadBatch::new(()).try_map(|_| {
                            let mut payload = Vec::new();
                            payload
                                .try_reserve_exact(length)
                                .map_err(|_| dokime::harmos::storage::Error::ResourceExhausted { resource: "opened file frame" })?;
                            payload.extend_from_slice(bytes);
                            Ok(self.frames.chunk(offset, payload))
                        })
                    }
                    .map_err(super::bundle::native_failure)?;
                    if super::content::output_frame_len(&*frame)? > self.message_limit {
                        return Err(Status::resource_exhausted("opened file frame exceeds message budget"));
                    }
                    let response = dokime::harmos::operations::Response::from_owner(frame.owner());
                    output.send_owned(response).await.map_err(stream_failure)?;
                    if done {
                        *batch = None;
                        *read = 0;
                    } else {
                        *read = end;
                    }
                    offset = next;
                }
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
                    let mut payload = Vec::new();
                    payload.try_reserve_exact(bytes.len()).map_err(|_| Status::resource_exhausted("file frame allocation refused"))?;
                    payload.extend_from_slice(bytes);
                    let frame = self.frames.chunk(offset, payload);
                    if super::content::output_frame_len(&frame)? > self.message_limit {
                        return Err(Status::resource_exhausted("file frame exceeds wire limit"));
                    }
                    output.send_owned(frame.into()).await.map_err(stream_failure)?;
                    *read = end;
                    offset = next;
                }
                Source::Artifact(reader) => {
                    let Some(chunk) = reader.next_chunk(maximum).await.map_err(|error| file_failure(error, self.read_failure))? else {
                        break;
                    };
                    let next = checked_offset(offset, chunk.len(), self.size_bytes, self.noun)?;
                    if self.sha256.is_none() {
                        digest.update(chunk.as_slice());
                    }
                    let frame =
                        chunk.try_map(|bytes| Ok::<_, dokime::harmos::Error>(self.frames.chunk(offset, bytes))).map_err(stream_failure)?;
                    self.check(&frame)?;
                    let response = dokime::harmos::operations::Response::from_owner(frame.owner());
                    output.send_owned(response).await.map_err(stream_failure)?;
                    offset = next;
                }
                Source::Evidence(reader) => {
                    let Some(chunk) = reader.next_chunk(maximum).await.map_err(|error| super::content::evidence::failure(&error))? else {
                        break;
                    };
                    if chunk.offset != offset {
                        return Err(Status::data_loss("evidence chunk differs from delivered offset"));
                    }
                    let next = checked_offset(offset, chunk.data.len(), self.size_bytes, self.noun)?;
                    if self.sha256.is_none() {
                        digest.update(&chunk.data);
                    }
                    let frame = chunk
                        .try_map(|chunk| Ok::<_, dokime::harmos::Error>(self.frames.chunk(chunk.offset, chunk.data)))
                        .map_err(stream_failure)?;
                    self.check(&frame)?;
                    let response = dokime::harmos::operations::Response::from_owner(frame.owner());
                    output.send_owned(response).await.map_err(stream_failure)?;
                    offset = next;
                }
            }
        }
        if offset != self.size_bytes {
            return Err(Status::data_loss(format!("{} ended before its declared length", self.noun)));
        }
        let sha256 = self.sha256.take().unwrap_or_else(|| digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect());
        let complete = self.frames.complete(offset, sha256);
        self.check(&complete)?;
        output.send_owned((complete).into()).await.map_err(stream_failure)
    }

    fn check(&self, frame: &F::Frame) -> Result<(), Status> {
        if super::content::output_frame_len(frame)? > self.message_limit {
            return Err(self.too_large());
        }
        Ok(())
    }
    fn too_large(&self) -> Status {
        Status::resource_exhausted(format!("{} exceeds the configured message budget", self.noun))
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

fn file_failure(source: dokime::runtime::Error, message: &'static str) -> Status {
    let status = match &source {
        dokime::runtime::Error::Harmos(dokime::harmos::Error::ResourceExhausted { .. }) => Status::resource_exhausted(message),
        dokime::runtime::Error::Harmos(dokime::harmos::Error::NotReady { .. }) => Status::unavailable(message),
        _ => Status::internal(message),
    };
    super::error::detailed(status, &source)
}

fn stream_failure(error: dokime::harmos::Error) -> Status {
    match error {
        dokime::harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted(error.to_string()),
        dokime::harmos::Error::NotReady { .. } => Status::unavailable(error.to_string()),
        _ => Status::internal(error.to_string()),
    }
}

/* === Tests === */
