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
use futures_util::future::BoxFuture;
use harmos::storage::{ByteRange, FileReader, ReadBatch};

// Local Imports
use super::error;
use crate::storage::Manager;
use crate::{
    execution_runtime::outputs::{ExecutionFileContent, ExecutionFileContentKind, ExecutionFileStreamState},
    runtime::Result,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Reader {
    session: Manager,
    file: std::fs::File,
    metadata: ReadBatch<ExecutionFileContent>,
    offset: u64,
    pending: Option<Pending>,
}

struct Pending {
    maximum: NonZeroUsize,
    future: tokio::sync::Mutex<BoxFuture<'static, Result<ReadBatch<ExecutionFileContent>>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Reader {
    pub(super) fn new(
        session: Manager, file: std::fs::File, path: &str, observed_size: u64, stream_state: ExecutionFileStreamState,
    ) -> Result<Self> {
        let media = super::media_type_for_path(path);
        let metadata = harmos::storage::ReadBatch::new(()).try_map(|_| {
            let content_type = media.map(str::to_owned);
            Ok::<_, crate::runtime::Error>(ExecutionFileContent {
                relative_path: path.to_owned(),
                content_kind: ExecutionFileContentKind::from_metadata(path, content_type.as_deref()),
                content_type,
                stream_state,
                offset: 0,
                size_bytes: observed_size,
                bytes: Vec::new(),
            })
        })?;
        Ok(Self {
            session,
            file,
            metadata,
            offset: 0,
            pending: None,
        })
    }

    pub(crate) fn session(&self) -> &Manager {
        &self.session
    }

    pub(crate) fn metadata(&self) -> &ReadBatch<ExecutionFileContent> {
        &self.metadata
    }

    pub(super) async fn into_window(self, offset: u64, length: usize) -> Result<ReadBatch<ExecutionFileContent>> {
        let range = ByteRange { offset, limit: Some(length as u64) };
        let reader = self.session.read_opened_file(self.file, range).await?;
        collect(self.metadata, reader, offset, length).await
    }

    pub(crate) async fn next_chunk(&mut self, maximum: NonZeroUsize) -> Result<Option<ReadBatch<ExecutionFileContent>>> {
        if let Some(pending) = &self.pending {
            if pending.maximum != maximum {
                return Err(harmos::Error::InvalidInput {
                    field: "native stream chunk",
                    reason: "pending read has another maximum",
                }
                .into());
            }
        } else {
            if self.offset == self.metadata.size_bytes {
                return Ok(None);
            }
            let length = (self.metadata.size_bytes - self.offset).min(maximum.get() as u64) as usize;
            let metadata = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::runtime::Error>((*self.metadata).clone()))?;
            let future = read(self.session.clone(), self.file.try_clone().map_err(error)?, metadata, self.offset, length);
            self.pending = Some(Pending {
                maximum,
                future: tokio::sync::Mutex::new(Box::pin(future)),
            });
        }
        let result = self.pending.as_mut().expect("accepted native read retained").future.get_mut().as_mut().await;
        self.pending = None;
        let content = result?;
        self.offset = content.offset.checked_add(content.bytes.len() as u64).ok_or_else(|| error("native stream offset overflows"))?;
        Ok(Some(content))
    }
}

/* === Public Functions === */

pub(crate) async fn collect(
    metadata: ReadBatch<ExecutionFileContent>, mut reader: FileReader, offset: u64, length: usize,
) -> Result<ReadBatch<ExecutionFileContent>> {
    let first = reader.next().await?;
    if length == 0 {
        if first.is_some() {
            return Err(error("file reader exceeds selected range"));
        }
        reader.completion()?;
        return metadata.try_map(|mut content| {
            content.offset = offset;
            Ok(content)
        });
    }
    let first = first.ok_or_else(|| error("source ended before observed prefix"))?;
    if first.bytes().len() > length {
        return Err(error("file reader exceeds selected range"));
    }
    if first.bytes().len() == length {
        // A Whole Native Batch Moves Its Allocation; No Assembly Buffer Is Needed.
        let batch = first.try_map(Ok::<_, crate::runtime::Error>)?;
        let content = metadata.try_map_with(batch, |mut content, batch| {
            content.offset = offset;
            content.bytes = batch.bytes;
            Ok::<_, crate::runtime::Error>(content)
        })?;
        if reader.next().await?.is_some() {
            return Err(error("file reader exceeds selected range"));
        }
        reader.completion()?;
        return Ok(content);
    }
    let mut content = metadata.try_map(|mut content| {
        content.offset = offset;
        content.bytes.try_reserve_exact(length).map_err(error)?;
        content.bytes.extend_from_slice(first.bytes());
        Ok::<_, crate::runtime::Error>(content)
    })?;
    drop(first);
    // Each Copy Has Its Own Admitted Output Peak Alongside The Native Source Batch.
    while let Some(batch) = reader.next().await? {
        if batch.bytes().len() > length.saturating_sub(content.bytes.len()) {
            return Err(error("file reader exceeds selected range"));
        }
        content = content.try_map(|mut content| {
            content.bytes.extend_from_slice(batch.bytes());
            Ok::<_, crate::runtime::Error>(content)
        })?;
    }
    reader.completion()?;
    if content.bytes.len() != length {
        return Err(error("source ended before observed prefix"));
    }
    Ok(content)
}

/* === Private Functions === */

async fn read(
    session: Manager, file: std::fs::File, metadata: ReadBatch<ExecutionFileContent>, offset: u64, length: usize,
) -> Result<ReadBatch<ExecutionFileContent>> {
    let reader = session.read_opened_file(file, ByteRange { offset, limit: Some(length as u64) }).await?;
    collect(metadata, reader, offset, length).await
}

/* === Tests === */
