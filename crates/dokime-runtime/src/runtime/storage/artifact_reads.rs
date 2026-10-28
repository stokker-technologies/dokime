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
use harmos::storage::FileRead;
use harmos::storage::{ByteRange, FileReadBatch, FileReader, ReadBatch};

// Local Imports
use super::execution::ExecutionView;
use crate::artifacts::ArtifactRecord;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::Metadata;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ArtifactReader {
    descriptor: ReadBatch<ExecutionOutput>,
    reader: FileReader,
    // The Unconsumed Tail Of The Last Response Batch.
    pending: Option<FileReadBatch>,
    offset: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ArtifactReader {
    pub(super) async fn from_output(session: crate::storage::Manager, descriptor: ReadBatch<ExecutionOutput>) -> Result<Self> {
        let reader = session
            .read_file(FileRead {
                source: harmos::storage::FileSource::Stored(descriptor.file.clone()),
                range: ByteRange { offset: 0, limit: None },
            })
            .await?;
        Ok(Self::new(descriptor, reader))
    }

    pub async fn open(view: &ExecutionView, id: &ArtifactId) -> Result<Self> {
        let output = view
            .outputs()?
            .iter()
            .find(|output| {
                matches!(&output.content, ExecutionOutputContent::Artifact { record, .. }
                if &record.artifact_id == id)
            })
            .ok_or_else(|| Error::Storage(crate::storage::Error::Invalid("artifact descriptor is missing from captured output")))?;
        let output = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, harmos::Error>(output.clone()))?;
        let reader = view.read_file(&output.file, ByteRange { offset: 0, limit: None }).await?;
        Ok(Self::new(output, reader))
    }

    fn new(descriptor: ReadBatch<ExecutionOutput>, reader: FileReader) -> Self {
        Self {
            descriptor,
            reader,
            pending: None,
            offset: 0,
        }
    }

    pub fn record(&self) -> &ArtifactRecord {
        let ExecutionOutputContent::Artifact { record, .. } = &self.descriptor.content else {
            unreachable!("reader construction selects only saved artifact descriptors");
        };
        record
    }

    pub fn metadata(&self) -> &Metadata {
        let ExecutionOutputContent::Artifact { metadata, .. } = &self.descriptor.content else {
            unreachable!("reader construction selects only saved artifact descriptors");
        };
        metadata
    }

    pub async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<ReadBatch<Vec<u8>>>> {
        if self.pending.as_ref().is_none_or(|batch| self.offset == batch.bytes().len()) {
            self.pending = None;
            let Some(batch) = self.reader.next().await? else {
                return Ok(None);
            };
            self.pending = Some(batch);
            self.offset = 0;
        }
        let pending = self.pending.as_ref().expect("read batch retained").bytes();
        if self.offset == 0 && pending.len() <= maximum.get() {
            let batch = self.pending.take().expect("read batch retained");
            let chunk = batch.try_map(|batch| Ok::<_, harmos::Error>(batch.bytes))?;
            return Ok(Some(chunk));
        }
        let end = pending.len().min(self.offset.saturating_add(maximum.get()));
        let length = end - self.offset;
        let chunk = harmos::storage::ReadBatch::new(()).try_map(|_| -> harmos::Result<_> {
            let exhausted = || harmos::Error::ResourceExhausted { resource: "artifact chunk projection" };
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(length).map_err(|_| exhausted())?;
            bytes.extend_from_slice(&pending[self.offset..end]);
            Ok(bytes)
        })?;
        self.offset = end;
        Ok(Some(chunk))
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let Some(maximum) = NonZeroUsize::new(buffer.len()) else {
            return Ok(0);
        };
        if self.pending.as_ref().is_none_or(|batch| self.offset == batch.bytes().len()) {
            self.pending = None;
            let Some(batch) = self.reader.next().await? else {
                return Ok(0);
            };
            self.pending = Some(batch);
            self.offset = 0;
        }
        let pending = self.pending.as_ref().expect("read batch retained").bytes();
        let count = maximum.get().min(pending.len() - self.offset);
        buffer[..count].copy_from_slice(&pending[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
