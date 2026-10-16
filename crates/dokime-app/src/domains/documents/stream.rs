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
use harmos::{MemorySize, storage::QueryValue};

// Local Imports
use super::saved::SavedOutputFile;
use crate::{ExecutionFileContent, Result, runtime::storage::native_outputs::reader::Reader};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct ExecutionFileStream {
    source: Source,
    metadata: QueryValue<ExecutionFileContent>,
    offset: u64,
    pending: Option<Pending>,
}

struct Pending {
    maximum: NonZeroUsize,
    content: QueryValue<ExecutionFileContent>,
}

/* === Enums === */
enum Source {
    Live(Reader),
    Saved(Box<SavedOutputFile>),
}

/* === Traits === */

/* === Implementations === */
impl ExecutionFileStream {
    pub(crate) fn live(reader: Reader) -> Result<Self> {
        let maximum = metadata_memory(reader.metadata())?;
        let metadata = reader.project_metadata(maximum, metadata_copy)?;
        Ok(Self {
            source: Source::Live(reader),
            metadata,
            offset: 0,
            pending: None,
        })
    }

    pub(crate) fn saved(mut file: SavedOutputFile) -> Result<Self> {
        let metadata = file.metadata()?;
        Ok(Self {
            source: Source::Saved(Box::new(file)),
            metadata,
            offset: 0,
            pending: None,
        })
    }

    pub fn metadata(&self) -> &ExecutionFileContent {
        &self.metadata
    }

    pub fn project_metadata<U: MemorySize>(
        &mut self,
        maximum: NonZeroUsize,
        map: impl FnOnce(&ExecutionFileContent, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        match &mut self.source {
            Source::Live(_) => {
                let computation = self.metadata.computation()?;
                Ok(computation.project(&self.metadata, maximum, map)?)
            }
            Source::Saved(file) => file.project(&self.metadata, maximum, map),
        }
    }

    pub async fn next_chunk(
        &mut self,
        maximum: NonZeroUsize,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        if !self.prepare(maximum).await? {
            return Ok(None);
        }
        let pending = self.pending.take().expect("prepared original chunk");
        self.offset = end(&pending.content)?;
        Ok(Some(pending.content))
    }

    pub async fn next_chunk_projected<U: MemorySize>(
        &mut self,
        maximum: NonZeroUsize,
        projection: NonZeroUsize,
        map: impl FnOnce(&ExecutionFileContent, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<Option<QueryValue<U>>> {
        if !self.prepare(maximum).await? {
            return Ok(None);
        }
        let pending = self.pending.as_ref().expect("prepared original chunk");
        let projected = match &mut self.source {
            Source::Live(_) => {
                let computation = pending.content.computation()?;
                computation.project(&pending.content, projection, map)?
            }
            Source::Saved(file) => file.project(&pending.content, projection, map)?,
        };
        self.offset = end(&pending.content)?;
        self.pending = None;
        Ok(Some(projected))
    }

    async fn prepare(&mut self, maximum: NonZeroUsize) -> Result<bool> {
        if let Some(pending) = &self.pending {
            if pending.maximum != maximum {
                return Err(invalid("pending chunk has another maximum").into());
            }
            return Ok(true);
        }
        let content = match &mut self.source {
            Source::Live(reader) => reader.next_chunk(maximum).await?,
            Source::Saved(file) => {
                let projection = file.window_memory(self.offset, maximum)?;
                let content = file.window(self.offset, maximum, projection).await?;
                if content.bytes.is_empty() {
                    None
                } else {
                    Some(content)
                }
            }
        };
        let Some(content) = content else {
            if self.offset != self.metadata.size_bytes {
                return Err(invalid("source ended before observed prefix").into());
            }
            return Ok(false);
        };
        if content.offset != self.offset
            || content.bytes.is_empty()
            || content.size_bytes != self.metadata.size_bytes
            || end(&content)? > self.metadata.size_bytes
        {
            return Err(invalid("original chunk differs from selected prefix").into());
        }
        self.pending = Some(Pending { maximum, content });
        Ok(true)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn end(content: &ExecutionFileContent) -> harmos::Result<u64> {
    content
        .offset
        .checked_add(content.bytes.len() as u64)
        .ok_or_else(|| invalid("chunk offset overflows"))
}
fn metadata_memory(content: &ExecutionFileContent) -> harmos::Result<NonZeroUsize> {
    std::mem::size_of::<ExecutionFileContent>()
        .checked_add(content.relative_path.len())
        .and_then(|bytes| bytes.checked_add(content.content_type.as_ref().map_or(0, String::len)))
        .and_then(NonZeroUsize::new)
        .ok_or_else(exhausted)
}
fn metadata_copy(
    content: &ExecutionFileContent,
    allowance: NonZeroUsize,
) -> harmos::Result<ExecutionFileContent> {
    if metadata_memory(content)? > allowance {
        return Err(exhausted());
    }
    Ok(ExecutionFileContent {
        relative_path: text(&content.relative_path)?,
        content_type: content.content_type.as_deref().map(text).transpose()?,
        content_kind: content.content_kind,
        stream_state: content.stream_state,
        offset: 0,
        size_bytes: content.size_bytes,
        bytes: Vec::new(),
    })
}
fn text(source: &str) -> harmos::Result<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(source.len())
        .map_err(|_| exhausted())?;
    if value.capacity() > source.len() {
        return Err(exhausted());
    }
    value.push_str(source);
    Ok(value)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "execution stream metadata",
    }
}
fn invalid(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "execution stream",
        reason,
    }
}

/* === Tests === */
