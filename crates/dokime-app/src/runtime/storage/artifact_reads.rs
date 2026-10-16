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
use harmos::{
    MemorySize,
    storage::{FileChunk, QueryValue, StorageFileReader},
};

// Local Imports
use super::execution::ExecutionView;
use crate::{Error, Result};
use dokime::{
    artifacts::ArtifactRecord,
    core::{ArtifactId, Metadata},
    execution::saved::{ExecutionOutput, ExecutionOutputContent},
};

/* === Constants === */
const CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ArtifactReader {
    descriptor: QueryValue<ExecutionOutput>,
    reader: StorageFileReader,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ArtifactReader {
    pub(super) async fn from_output(
        storage: &dokime::storage::Manager,
        descriptor: QueryValue<ExecutionOutput>,
    ) -> Result<Self> {
        let reader = storage.read_file(&descriptor.file).await?;
        Ok(Self { descriptor, reader })
    }

    pub async fn open(
        view: &ExecutionView,
        id: &ArtifactId,
        projection: NonZeroUsize,
    ) -> Result<Self> {
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
            })?;
        let descriptor =
            view.view()
                .project_shared(view.revision()?, projection, |_, allowance| {
                    let bytes = output.memory_size();
                    if bytes == usize::MAX || bytes > allowance.get() {
                        return Err(harmos::Error::ResourceExhausted {
                            resource: "saved artifact descriptor memory",
                        });
                    }
                    Ok(output.clone())
                })?;
        let reader = view.read_file(&descriptor.file).await?;
        Ok(Self { descriptor, reader })
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

    pub fn project_record<T: MemorySize>(
        &self,
        maximum: NonZeroUsize,
        project: impl FnOnce(&ArtifactRecord, NonZeroUsize) -> harmos::Result<T>,
    ) -> Result<QueryValue<T>> {
        self.descriptor
            .try_project(maximum, |output, allowance| {
                let ExecutionOutputContent::Artifact { record, .. } = &output.content else {
                    unreachable!("reader construction selects only saved artifact descriptors");
                };
                project(record, allowance)
            })
            .map_err(Into::into)
    }

    pub async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<FileChunk>> {
        let maximum = NonZeroUsize::new(maximum.get().min(CHUNK_BYTES))
            .expect("nonempty bounded artifact chunk");
        Ok(self.reader.next_chunk(maximum).await?)
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let Some(maximum) = NonZeroUsize::new(buffer.len().min(CHUNK_BYTES)) else {
            return Ok(0);
        };
        let Some(chunk) = self.next_chunk(maximum).await? else {
            return Ok(0);
        };
        let count = chunk.as_bytes().len();
        buffer[..count].copy_from_slice(chunk.as_bytes());
        Ok(count)
    }
}

/* === Public Functions === */
/* === Private Functions === */

/* === Tests === */
