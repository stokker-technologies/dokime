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
use harmos::storage::{ByteRange, FileReader, ReadFile};

// Local Imports
use super::execution::ExecutionView;
use crate::{Error, Result};
use dokime::{
    artifacts::ArtifactRecord,
    core::{ArtifactId, Metadata},
    execution::saved::{ExecutionOutput, ExecutionOutputContent},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// One saved artifact's descriptor and its verified bytes.
pub struct ArtifactReader {
    descriptor: ExecutionOutput,
    reader: FileReader,
    // The Unconsumed Tail Of The Last Response Batch.
    pending: Vec<u8>,
    offset: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ArtifactReader {
    pub(super) async fn from_output(
        storage: &dokime::storage::Manager,
        descriptor: ExecutionOutput,
    ) -> Result<Self> {
        let reader = storage
            .read_file(ReadFile {
                file: descriptor.file.clone(),
                range: ByteRange {
                    offset: 0,
                    limit: None,
                },
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
            .ok_or_else(|| {
                Error::Storage(dokime::storage::Error::Invalid(
                    "artifact descriptor is missing from captured output",
                ))
            })?
            .clone();
        let reader = view
            .read_file(
                &output.file,
                harmos::storage::ByteRange {
                    offset: 0,
                    limit: None,
                },
            )
            .await?;
        Ok(Self::new(output, reader))
    }

    fn new(descriptor: ExecutionOutput, reader: FileReader) -> Self {
        Self {
            descriptor,
            reader,
            pending: Vec::new(),
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

    /// The next verified bytes, at most `maximum`, or `None` at the end.
    pub async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<Vec<u8>>> {
        if self.offset == self.pending.len() {
            let Some(batch) = self.reader.next().await? else {
                return Ok(None);
            };
            self.pending = batch.bytes;
            self.offset = 0;
        }
        let end = self.pending.len().min(self.offset + maximum.get());
        let chunk = self.pending[self.offset..end].to_vec();
        self.offset = end;
        Ok(Some(chunk))
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let Some(maximum) = NonZeroUsize::new(buffer.len()) else {
            return Ok(0);
        };
        let Some(chunk) = self.next_chunk(maximum).await? else {
            return Ok(0);
        };
        buffer[..chunk.len()].copy_from_slice(&chunk);
        Ok(chunk.len())
    }
}

/* === Public Functions === */
/* === Private Functions === */

/* === Tests === */
